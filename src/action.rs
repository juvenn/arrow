use crate::envs::Envs;
use crate::repo::Context;
use anyhow;
use serde::Deserialize;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use crate::actions::webhook::WebHookAction;

#[derive(Debug, Deserialize)]
#[serde(tag = "runner")]
pub enum Action {
    #[serde(rename = "shell")]
    Shell(ShellAction),
    #[serde(rename = "bash")]
    Bash(ShellAction),
    #[serde(rename = "webhook")]
    WebHook(WebHookAction),
    #[serde(rename = "ssh")]
    Ssh(SshAction),
}

#[derive(Debug, Deserialize, Clone)]
pub struct ShellAction {
    name: String,
    script: String,
    #[serde(skip)]
    shell: String,

    #[serde(flatten)]
    envs: Envs,
}

#[derive(Debug, Deserialize)]
pub struct SshAction {
    name: String,
    user: String,
    hosts: Vec<String>,
    script: String,
}

pub trait IAction {
    fn run(&self, ctx: &Context, envs: &Envs) -> anyhow::Result<()>;
}

impl IAction for Action {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()> {
        println!("");
        match self {
            Action::Ssh(action) => action.run(ctx, parent_env),
            Action::Shell(action) => {
                let mut action = action.clone();
                action.shell = "sh".to_string();
                action.run(ctx, parent_env)
            }
            Action::Bash(action) => {
                let mut action = action.clone();
                action.shell = "bash".to_string();
                action.run(ctx, parent_env)
            }
            Action::WebHook(action) => action.run(ctx, parent_env),
        }
    }
}

impl IAction for ShellAction {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()> {
        println!("### {}\n", self.name);
        let envs = self.envs.inherit(parent_env);
        let vars = envs.build_env()?;
        let mut child = Command::new(self.shell.clone())
            .arg("-c")
            .envs(&vars)
            .arg(&self.script)
            .stdout(Stdio::piped())
            .spawn()?;
        if let Some(ref mut stdout) = child.stdout {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => println!("  {}", line),
                    Err(err) => eprintln!("Error: {}", err),
                }
            }
        }
        let _ = child.wait()?;

        Ok(())
    }
}

impl IAction for SshAction {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()> {
        println!("### {}", self.name);
        println!("ssh action is yet to be implemented");
        Ok(())
    }
}
