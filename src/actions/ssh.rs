use crate::actions::IAction;
use crate::envs::Envs;
use crate::repo::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SshAction {
    name: String,
    user: String,
    hosts: Vec<String>,
    script: String,
}

impl IAction for SshAction {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()> {
        println!("### {}", self.name);
        println!("ssh action is yet to be implemented");
        Ok(())
    }
}
