use anyhow::Context as _;
use serde::Deserialize;
use serde_yaml as yaml;
use std::fs::File;
use std::path::PathBuf;

use crate::action::{Action, IAction};
use crate::decode;
use crate::repo::Context;

#[derive(Debug, Default)]
pub struct Pipelines {
    pipelines: Vec<Pipeline>,
}

impl Pipelines {
    pub fn new() -> Self {
        Pipelines::default()
    }

    pub fn parse_pipelines(dir: &str) -> anyhow::Result<Vec<Pipeline>> {
        let mut pipelines = Vec::new();
        let pipeline_dir = PathBuf::from(dir);
        if !pipeline_dir.exists() {
            return Ok(pipelines);
        }
        for entry in std::fs::read_dir(pipeline_dir)
            .with_context(|| format!("Failed to read pipeline definitions from {}", dir))?
        {
            let path = entry?.path();
            if path.is_file() {
                let pipeline = Self::parse_path(&path)?;
                pipelines.push(pipeline);
            }
        }
        Ok(pipelines)
    }

    fn parse_path(path: &PathBuf) -> anyhow::Result<Pipeline> {
        let name = path.display();
        let file = File::open(path).with_context(|| format!("Failed to open file {}", name))?;
        let pipeline: Pipeline = yaml::from_reader(file)
            .with_context(|| format!("Failed to parse pipeline file {}", name))?;
        Ok(pipeline)
    }

    pub fn run(&mut self, ctx: Context) -> anyhow::Result<()> {
        ctx.checkout_workspace()?;
        self.pipelines = Self::parse_pipelines(".arrow")?;
        if self.pipelines.is_empty() {
            ctx.cleanup_workspace()?;
            return Ok(());
        }
        println!("GIT_DIR: {}", ctx.repo_dir.display());
        println!("On {}: {}..{}", ctx.branch, ctx.old_rev, ctx.new_rev);
        for pipeline in &self.pipelines {
            pipeline.run(&ctx)?;
        }
        ctx.cleanup_workspace()?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    name: String,
    #[serde(default = "WhenSpec::always")]
    when: WhenSpec,
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize, Default)]
pub struct WhenSpec {
    #[serde(
        default = "WhenSpec::any_branch",
        deserialize_with = "decode::string_or_seq"
    )]
    branch: Vec<String>, // list of branch to trigger on
    #[serde(default)]
    changes: Vec<String>, // list of glob patterns, relative to repo root
}

/// A special branch name that matches all branches
const STAR_BRANCH: &str = "*";

impl WhenSpec {
    /// A spec runs on all branches and all changes
    pub fn always() -> WhenSpec {
        WhenSpec {
            branch: vec![STAR_BRANCH.to_string()],
            changes: Vec::new(),
        }
    }

    pub fn any_branch() -> Vec<String> {
        vec!["*".to_string()]
    }

    pub fn match_changes(&self, branch: &String, fileset: Option<Vec<String>>) -> bool {
        if self.branch.is_empty() {
            return false;
        }
        if !(self.branch[0] == STAR_BRANCH || self.branch.contains(branch)) {
            return false;
        }
        if self.changes.is_empty() {
            return true;
        }

        // TODO: precompile glob patterns
        self.changes.iter().any(|pat| {
            let pattern = glob::Pattern::new(pat).unwrap();
            match &fileset {
                Some(fileset) => fileset.iter().any(|f| pattern.matches(f)),
                None => false,
            }
        })
    }
}

impl Pipeline {
    pub fn run(&self, ctx: &Context) -> anyhow::Result<()> {
        if !self.should_run(ctx) {
            return Ok(());
        }
        println!();
        println!("{}", self.name);
        println!("----");
        for action in &self.actions {
            action.run(ctx)?;
        }
        Ok(())
    }

    fn should_run(&self, ctx: &Context) -> bool {
        self.when.match_changes(&ctx.branch, ctx.get_fileset())
    }
}
