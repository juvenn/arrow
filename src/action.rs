use crate::context::Context;
use anyhow;
use serde::Deserialize;
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize)]
#[serde(tag = "runner")]
pub enum Action {
    #[serde(rename = "shell")]
    Shell(ShellAction),

    #[serde(rename = "ssh")]
    Ssh(SshAction),
}

#[derive(Debug, Deserialize)]
pub struct ShellAction {
    name: String,
    script: String,
}

#[derive(Debug, Deserialize)]
pub struct SshAction {
    name: String,
    user: String,
    hosts: Vec<String>,
    script: String,
}

pub trait IAction {
    fn run(&self, ctx: &Context) -> anyhow::Result<()>;
}

impl IAction for Action {
    fn run(&self, ctx: &Context) -> anyhow::Result<()> {
        match self {
            Action::Shell(action) => action.run(ctx),
            Action::Ssh(action) => action.run(ctx),
        }
    }
}

impl IAction for ShellAction {
    fn run(&self, ctx: &Context) -> anyhow::Result<()> {
        println!("### {}", self.name);
        let output = Command::new("sh")
            .arg("-x")
            .arg("-c")
            .arg(&self.script)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()?;
        println!("Done: {}", self.name);
        Ok(())
    }
}

impl IAction for SshAction {
    fn run(&self, ctx: &Context) -> anyhow::Result<()> {
        println!("### {}", self.name);
        println!("ssh action is yet to be implemented");
        Ok(())
    }
}
