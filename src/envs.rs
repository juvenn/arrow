use crate::decode;
use serde::Deserialize;
use std::collections::HashMap;

use env_file_reader::read_file;

/// Env variables, that can be defined in variables, or sourced from env files.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Envs {
    #[serde(default, deserialize_with = "decode::string_or_seq")]
    env_file: Vec<String>,
    #[serde(default)]
    variables: HashMap<String, String>,
}

impl Envs {
    /// Setup ouput env file
    pub fn with_output_env(&self, key: String, path: String) -> Self {
        let mut envs = Envs::default();
        envs.env_file.push(path.clone());
        envs.env_file.extend(self.env_file.clone());
        envs.variables.insert(key, path);
        envs.variables.extend(self.variables.clone());
        envs
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
        let vars = read_file(file)?;
        Ok(vars)
    }
}
