use crate::actions::IAction;
use crate::decode;
use crate::envs::Envs;
use crate::repo::Context;
use serde::Deserialize;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

#[derive(Debug, Deserialize)]
pub struct SshAction {
    name: String,
    user: String,
    identity_file: Option<String>,
    #[serde(deserialize_with = "decode::string_or_seq")]
    hosts: Vec<String>,
    args: Vec<String>, // ssh args
    script: String,
    #[serde(flatten)]
    envs: Envs,
}

impl IAction for SshAction {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()> {
        println!("### {}\n", self.name);
        let vars = self.envs.inherit(parent_env).build_env()?;
        let env_lines: Vec<String> = vars.iter().map(|(k, v)| format!("{}='{}'", k, v)).collect();
        let env_sh = env_lines.join("\n");
        for host in &self.hosts {
            self.run_on_host(host, &env_sh)?;
        }
        Ok(())
    }
}

impl SshAction {
    fn run_on_host(&self, host_port: &String, env_sh: &String) -> anyhow::Result<()> {
        let (host, port) = host_port.split_once(":").unwrap_or((host_port, "22"));
        let user_host = format!("{}@{}", self.user, host);
        println!("ssh -p {} {} 'sh -s'\n", port, user_host);
        let mut cmd = Command::new("ssh");
        if let Some(ref identity_file) = self.identity_file {
            cmd.arg("-i").arg(identity_file);
        }
        for arg in &self.args {
            cmd.arg(arg);
        }
        cmd.arg("-p").arg(&port);
        let mut child = cmd
            .arg(&user_host)
            .arg("sh -s") // read commands from stdin
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        // source env vars first
        // then exec user script
        let childinput = child.stdin.as_mut().unwrap();
        childinput.write_all(env_sh.as_bytes())?;
        childinput.write_all(b"\n")?;
        childinput.flush()?;
        childinput.write_all(self.script.as_bytes())?;
        childinput.write_all(b"exit\n")?; // why it dose not exit automatically?
        childinput.flush()?;
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
