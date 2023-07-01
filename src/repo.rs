use anyhow::{anyhow, Context as _};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Default)]
pub struct Context {
    pub refname: String, // refs/heads/master
    pub old_rev: String, // old revision
    pub new_rev: String, // new revision
    pub branch: String,
    pub repo_dir: PathBuf, // absolute path to .git dir
    pub repo_name: String,
    pub workspace: PathBuf, // where to checkout the repo
    /// whether capable of worktree or not
    cap_worktree: bool,
    /// files that have changed
    fileset: Option<Vec<String>>,
}

/// Worktree represents a checkout of repo, which will be cleaned upon drop
pub struct Worktree<'a> {
    ctx: &'a Context,
}

impl<'a> Drop for Worktree<'a> {
    fn drop(&mut self) {
        self.ctx.cleanup_workspace().unwrap();
    }
}

// Repo context
impl Context {
    // Resolve context on hook invokation
    pub fn resolve_on_hook(
        refname: String,
        old_rev: String,
        new_rev: String,
    ) -> anyhow::Result<Self> {
        let repo_dir = Self::resolve_repo_dir()?;
        let branch = Self::resolve_branch(&refname)?;
        let repo_name = Self::resolve_reponame(&repo_dir);
        let workspace = PathBuf::from("/tmp/arrow-workspace"); // TODO: allow to customize
        let fileset = Self::resolve_fileset(&old_rev, &new_rev)?;
        let cap_worktree = Self::resolve_worktree_capable();
        let ctx = Context {
            refname,
            old_rev,
            new_rev,
            branch,
            repo_name,
            workspace,
            repo_dir,
            cap_worktree,
            fileset: Some(fileset),
        };
        Ok(ctx)
    }

    /// Checkout or init work dir with latest changes. It will try to use
    /// worktree if possible, or fallback to clone.
    ///
    /// It also change current working dir for the process.
    pub fn checkout_workspace(&self) -> anyhow::Result<Worktree> {
        if self.cap_worktree {
            self.checkout_worktree(&self.branch)?;
        } else {
            self.checkout_clone(&self.branch)?;
        }
        Result::Ok(Worktree { ctx: &self })
    }

    /// Cleanup work dir after all actions are done
    pub fn cleanup_workspace(&self) -> anyhow::Result<()> {
        if self.cap_worktree {
            self.cleanup_worktree(&self.branch)
        } else {
            self.cleanup_clone(&self.branch)
        }
    }

    fn print_git_ref(&self) {
        println!("GIT_DIR: {}", self.repo_dir.display());
        println!(
            "On {}: {}..{}",
            self.branch,
            &self.old_rev[..8],
            &self.new_rev[..8]
        );
    }

    /// Checkout by clone the repo to {workspace}/{repo-name}
    fn checkout_clone(&self, branch: &String) -> anyhow::Result<()> {
        self.print_git_ref();
        let workdir = self.workspace.join(&self.repo_name);
        let _ = std::fs::create_dir_all(&workdir)?;
        let script = format!(
            "
            if [ ! -d .git ]; then
                echo '.git not present, clone it'
                git clone {origin} .
            fi
            git clean -fdx
            git remote update
            git checkout {branch}
            git reset --hard {new_rev}
            ",
            origin = self.repo_dir.display(),
            branch = branch,
            new_rev = self.new_rev
        );
        let _ = Command::new("sh")
            .current_dir(&workdir)
            .env_remove("GIT_DIR") // working in new repo now
            .arg("-ex")
            .arg("-c")
            .arg(&script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .with_context(|| {
                format!(
                    "Failed to checkout and update work dir at {}",
                    workdir.display()
                )
            })?;
        env::set_current_dir(&workdir)?;
        println!("Work dir: {}", workdir.display());
        Ok(())
    }

    fn cleanup_clone(&self, _: &String) -> anyhow::Result<()> {
        // change back to repo dir
        env::set_current_dir(&self.repo_dir)?;
        // probably should remove the workdir?
        Ok(())
    }

    /// Use git worktree to checkout a working copy at {workspace}/app-{branch}
    fn checkout_worktree(&self, branch: &String) -> anyhow::Result<()> {
        self.print_git_ref();
        let workdir = Self::build_worktree_dir(&self, branch);
        let script = format!("git worktree add {} {}", workdir.to_string_lossy(), branch);

        let _ = Command::new("sh")
            .current_dir(&self.repo_dir)
            .arg("-ex")
            .arg("-c")
            .arg(&script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .with_context(|| {
                format!(
                    "Failed to checkout and update work tree at {}",
                    workdir.display()
                )
            })?;
        env::set_current_dir(&workdir)?;
        println!("Work dir: {}", env::current_dir()?.display());
        Ok(())
    }

    fn cleanup_worktree(&self, branch: &String) -> anyhow::Result<()> {
        // change back to repo dir
        env::set_current_dir(&self.repo_dir)?;
        let workdir = Self::build_worktree_dir(&self, branch);
        let script = format!("git worktree remove --force {}", workdir.to_string_lossy());

        let _ = Command::new("sh")
            .current_dir(&self.repo_dir)
            .arg("-ex")
            .arg("-c")
            .arg(&script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .with_context(|| format!("Failed to remove worktree {}", workdir.display()))?;
        Ok(())
    }

    fn build_worktree_dir(&self, branch: &String) -> PathBuf {
        let mut worktree = self.workspace.clone();
        let name = format!("{}-{}", self.repo_name, branch);
        worktree.push(name);
        worktree
    }

    fn resolve_branch(refname: &String) -> anyhow::Result<String> {
        match refname.split("/").last() {
            Some(branch) => return Ok(branch.to_string()),
            None => return Err(anyhow!("No branch resolved from refname '{}'", refname)),
        };
    }

    fn resolve_repo_dir() -> anyhow::Result<PathBuf> {
        match env::var("GIT_DIR") {
            Ok(dir) => return Ok(std::fs::canonicalize(PathBuf::from(dir))?),
            Err(_) => {
                return Err(anyhow!(
                    "env GIT_DIR not found, it should be run from bare repo"
                ))
            }
        };
    }

    fn resolve_reponame(repodir: &PathBuf) -> String {
        let name = match repodir.file_stem() {
            Some(name) => name.to_str().unwrap().to_string(),
            None => return String::from("Unamed-repo"),
        };
        if name == ".git" {
            let parentdir = repodir.parent().unwrap().to_path_buf();
            return Self::resolve_reponame(&parentdir);
        } else {
            return name;
        }
    }

    pub fn get_fileset(&self) -> Option<Vec<String>> {
        return self.fileset.clone();
    }

    fn resolve_fileset(old_rev: &String, new_rev: &String) -> anyhow::Result<Vec<String>> {
        let mut fileset: Vec<String> = Vec::new();
        let diff_cmd = format!("git diff --name-only {}..{}", old_rev, new_rev);
        let output = Command::new("sh")
            .arg("-c")
            .arg(&diff_cmd)
            .stderr(Stdio::inherit())
            .output()
            .with_context(|| format!("Command error: {}", &diff_cmd))?;
        let output = String::from_utf8_lossy(&output.stdout);
        for line in output.lines() {
            fileset.push(line.to_string());
        }
        Ok(fileset)
    }

    /// resole if git is capable of worktree
    fn resolve_worktree_capable() -> bool {
        let ret = Command::new("git").arg("worktree").arg("list").output();
        if let Ok(output) = ret {
            return output.status.success();
        }
        return false;
    }
}
