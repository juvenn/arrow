mod shell;
mod ssh;
mod webhook;

use crate::envs::Envs;
use crate::repo::Context;
use anyhow;
use serde::Deserialize;

use shell::ShellAction;
use ssh::SshAction;
use webhook::WebHookAction;

pub trait IAction {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()>;
}

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

impl IAction for Action {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()> {
        println!("");
        match self {
            Action::Ssh(action) => action.run(ctx, parent_env),
            Action::Shell(action) => {
                let action = action.set_shell("sh".to_string());
                action.run(ctx, parent_env)
            }
            Action::Bash(action) => {
                let action = action.set_shell("bash".to_string());
                action.run(ctx, parent_env)
            }
            Action::WebHook(action) => action.run(ctx, parent_env),
        }
    }
}
