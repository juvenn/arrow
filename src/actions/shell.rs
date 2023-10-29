use crate::actions::IAction;
use crate::envs::Envs;
use crate::repo::Context;
use serde::Deserialize;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

#[derive(Debug, Deserialize, Clone)]
pub struct ShellAction {
    name: String,
    script: String,
    #[serde(skip)]
    shell: String,

    #[serde(flatten)]
    envs: Envs,
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

impl ShellAction {
    pub fn set_shell(&self, name: String) -> Self {
        let mut action = self.clone();
        action.shell = name;
        return action;
    }
}
