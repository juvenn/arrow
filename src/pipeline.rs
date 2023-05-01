use serde::Deserialize;
use serde_yaml as yaml;
use std::fs::File;
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

    pub fn parse_pipelines(dir: String) -> Result<Vec<Pipeline>, yaml::Error> {
        let mut pipelines = Vec::new();
        let pipeline_dir = std::path::PathBuf::from(dir);
        for entry in std::fs::read_dir(pipeline_dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let pipeline = Pipelines::parse_file(path.to_str().unwrap())?;
                pipelines.push(pipeline);
            }
        }
        Ok(pipelines)
    }

    pub fn parse_file(filename: &str) -> Result<Pipeline, yaml::Error> {
        let file = File::open(filename).unwrap();
        let pipeline: Pipeline = yaml::from_reader(file)?;
        Ok(pipeline)
    }

    pub fn run(&mut self) {
        // TODO: allow to customize pipeline dir
        let pipelines = Self::parse_pipelines(".arrow".to_string()).unwrap();
        self.pipelines = pipelines;
        for pipeline in &self.pipelines {
            pipeline.run(&self.context);
        }
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
    script: String,
}

impl Pipeline {
    pub fn run(&self, ctx: &Context) -> Result<(), yaml::Error> {
        println!("{}", self.name);
        println!("----");
        for action in &self.actions {
            action.run(ctx)?;
        }
        Ok(())
    }
}

impl Action {
    pub fn run(&self, ctx: &Context) -> Result<(), yaml::Error> {
        println!("### {}", self.name);
        let mut script = "set -ex\n".to_string();
        script.push_str(&self.script);
        let output = Command::new("sh")
            .arg("-x")
            .arg("-c")
            .arg(script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute process");
        Ok(())
    }
}
