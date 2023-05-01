use dirs;
use std::env;
use std::path::PathBuf;

pub struct Context {
    pub refname: String, // branch name
    pub old_rev: String, // old revision
    pub new_rev: String, // new revision
    workspace: PathBuf,  // where to checkout the repo
    repo_dir: PathBuf,
    fileset: Option<Vec<String>>, // files that have changed
}

// Repo context
impl Context {
    // Resolve context on hook invokation
    pub fn resolve_on_hook(refname: String, old_rev: String, new_rev: String) -> Context {
        // TODO: should be independent on OS
        let mut workspace = dirs::home_dir().unwrap_or(PathBuf::from("/tmp"));
        workspace.push("arrow");
        let repo_dir = PathBuf::from(env::var("GIT_DIR").unwrap_or_default());
        let ctx = Context {
            refname,
            old_rev,
            new_rev,
            workspace,
            repo_dir,
            fileset: None,
        };
        ctx
    }

    pub fn resolve_fileset(&mut self) -> Vec<String> {
        match &self.fileset {
            Some(fileset) => return fileset.clone(),
            None => {
                let fileset = Self::get_fileset_from_git(&self.old_rev, &self.new_rev);
                self.fileset = Some(fileset.clone());
                return fileset;
            }
        }
    }

    fn get_fileset_from_git(old_rev: &String, new_rev: &String) -> Vec<String> {
        let mut fileset: Vec<String> = Vec::new();
        let diff_cmd = format!("git diff --name-only {}..{}", old_rev, new_rev);
        let output = std::process::Command::new("sh")
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
