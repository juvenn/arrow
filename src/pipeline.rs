use anyhow::Context as _;
use serde::Deserialize;
use serde_yaml as yaml;
use std::fs::File;
use std::path::PathBuf;

use crate::action::{Action, IAction};
use crate::context::Context;
use crate::decode;

pub struct Pipelines {
    pipelines: Vec<Pipeline>,
    context: Context,
}

impl Pipelines {
    pub fn new(context: Context) -> Self {
        Pipelines {
            pipelines: Vec::new(),
            context,
        }
    }

    pub fn parse_pipelines(dir: &str) -> anyhow::Result<Vec<Pipeline>> {
        let mut pipelines = Vec::new();
        let pipeline_dir = PathBuf::from(dir);
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
            .with_context(|| format!("Failed to parse definition file {}", name))?;
        Ok(pipeline)
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        // TODO: allow to customize pipeline dir
        let pipelines = Self::parse_pipelines(".arrow")?;
        self.pipelines = pipelines;
        for pipeline in &self.pipelines {
            pipeline.run(&self.context)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    name: String,
    #[serde(default = "WhenSpec::on_main_branch")]
    when: WhenSpec,
    actions: Vec<Action>,
}

#[derive(Debug, Deserialize)]
pub struct WhenSpec {
    #[serde(deserialize_with = "decode::string_or_seq")]
    branch: Vec<String>, // list of branch to trigger on
    #[serde(default)]
    changes: Vec<String>, // list of glob patterns, relative to repo root
}

impl WhenSpec {
    pub fn on_main_branch() -> WhenSpec {
        WhenSpec {
            branch: vec!["master".to_string(), "main".to_string()],
            changes: Vec::new(),
        }
    }

    pub fn match_changes(&self, branch: &String, fileset: Option<Vec<String>>) -> bool {
        if self.branch.is_empty() {
            return false;
        }
        if !(self.branch[0] == "*" || self.branch.contains(branch)) {
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
