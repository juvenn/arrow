use crate::decode;
use serde::Deserialize;
use std::collections::HashMap;

use std::fs::File;
use std::io::{BufRead, BufReader};

/// Env variables, that can be defined in variables, or sourced from env files.
#[derive(Debug, Deserialize, Default)]
pub struct Envs {
    #[serde(deserialize_with = "decode::string_or_seq")]
    env_file: Vec<String>,
    variables: HashMap<String, String>,
}

impl Envs {
    pub fn from_map(vars: HashMap<String, String>) -> Self {
        let mut envs = Envs::default();
        envs.variables = vars;
        return envs;
    }

    /// Inherit env variables from parent, returns new merged one.
    pub fn inherit(&self, parent: &Envs) -> Self {
        let mut envs = Envs::default();
        envs.env_file.extend(parent.env_file.clone());
        envs.env_file.extend(self.env_file.clone());
        envs.variables.extend(parent.variables.clone());
        envs.variables.extend(parent.variables.clone());
        envs
    }

    /// Render template string in currrent env
    pub fn render(&self, template: &String) -> anyhow::Result<String> {
        let vars = self.build_env()?;
        let mut template = template.clone();
        for (key, value) in vars.iter() {
            template = template.replace(&format!("${}", key), value);
        }
        Ok(template)
    }

    /// Build and return environment variables from env_file and environment
    pub fn build_env(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut vars = HashMap::new();
        for file in &self.env_file {
            let file_envs = Self::parse_env_file(file)?;
            vars.extend(file_envs);
        }
        vars.extend(self.variables.clone());
        Ok(vars)
    }

    fn parse_env_file(file: &String) -> anyhow::Result<HashMap<String, String>> {
        let mut envs = HashMap::new();
        let file = File::open(file)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap();
            let value = parts.next().unwrap();
            envs.insert(key.to_string(), value.to_string());
        }
        Ok(envs)
    }
}
