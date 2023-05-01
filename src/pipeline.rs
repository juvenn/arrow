use anyhow::Context as _;
use serde::Deserialize;
use serde_yaml as yaml;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::context::Context;

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
    pub name: String,
    pub actions: Vec<Action>,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    name: String,
    xxx: String,
    script: String,
}

impl Pipeline {
    pub fn run(&self, ctx: &Context) -> anyhow::Result<()> {
        println!("{}", self.name);
        println!("----");
        for action in &self.actions {
            action.run(ctx)?;
        }
        Ok(())
    }
}

impl Action {
    pub fn run(&self, ctx: &Context) -> anyhow::Result<()> {
        println!("### {}", self.name);
        let output = Command::new("sh")
            .arg("-x")
            .arg("-c")
            .arg(&self.script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        println!("### Done: {}", self.name);
        Ok(())
    }
}
