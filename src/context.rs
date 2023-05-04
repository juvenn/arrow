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
    pub workspace: PathBuf,       // where to checkout the repo
    fileset: Option<Vec<String>>, // files that have changed
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
        println!("GIT_DIR: {}", repo_dir.display());
        let branch = Self::resolve_branch(&refname)?;
        let repo_name = Self::resolve_reponame(&repo_dir);
        let workspace = PathBuf::from("/tmp/arrow-workspace"); // TODO: allow to customize
        println!("On {}: {}..{}", branch, old_rev, new_rev);
        let ctx = Context {
            refname,
            old_rev,
            new_rev,
            branch,
            repo_name,
            workspace,
            repo_dir,
            fileset: None,
        };
        Ok(ctx)
    }

    /// Checkout or init work dir with latest changes. It also changes the
    /// current working dir for the process.
    pub fn checkout_workspace(&self) -> anyhow::Result<()> {
        self.checkout_worktree(&self.branch)
    }

    pub fn cleanup_workspace(&self) -> anyhow::Result<()> {
        self.cleanup_worktree(&self.branch)
    }

    /// Checkout or init work dir with clone.
    fn checkout_v1(&self, branch: &String) -> anyhow::Result<()> {
        let workdir = self.workspace.join(&self.repo_name);
        let script = format!(
            "
            if [ ! -d {workspace} ]; then
                mkdir -p {workspace}
                git clone {origin} {workspace}
            fi
        cd {workspace}
        git fetch origin {branch}
        git checkout {branch}
        git reset --hard {new_rev}
        git clean -fdx
        ",
            origin = self.repo_dir.display(),
            workspace = workdir.to_string_lossy(),
            branch = branch,
            new_rev = self.new_rev
        );
        let _ = Command::new("sh")
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
        env::set_current_dir(&self.workspace)?;
        println!("Workspace: {}", workdir.display());
        Ok(())
    }

    /// Use git worktree to checkout a working copy at {workspace}/app-{branch}
    fn checkout_worktree(&self, branch: &String) -> anyhow::Result<()> {
        let workdir = Self::build_worktree_dir(&self, branch);
        let script = format!("git worktree add {} {}", workdir.to_string_lossy(), branch);

        let _ = Command::new("sh")
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
        env::set_current_dir(&self.workspace)?;
        println!("Workspace: {}", workdir.display());
        Ok(())
    }

    fn cleanup_worktree(&self, branch: &String) -> anyhow::Result<()> {
        let workdir = Self::build_worktree_dir(&self, branch);
        let script = format!("git worktree remove {}", workdir.to_string_lossy());

        let _ = Command::new("sh")
            .arg("-ex")
            .arg("-c")
            .arg(&script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .with_context(|| format!("Failed to git worktree remove {}", workdir.display()))?;
        // change back to repo dir
        env::set_current_dir(&self.repo_dir)?;
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
        if self.old_rev == "" || self.new_rev == "" {
            return None;
        } else {
            let fileset = Self::resolve_fileset(&self.old_rev, &self.new_rev);
            return Some(fileset);
        }
    }

    fn resolve_fileset(old_rev: &String, new_rev: &String) -> Vec<String> {
        let mut fileset: Vec<String> = Vec::new();
        let diff_cmd = format!("git diff --name-only {}..{}", old_rev, new_rev);
        let output = Command::new("sh")
            .arg("-c")
            .arg(diff_cmd)
            .output()
            .expect("failed to execute process");
        let output = String::from_utf8_lossy(&output.stdout);
        for line in output.lines() {
            fileset.push(line.to_string());
        }
        fileset
    }
}
