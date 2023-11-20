use crate::decode;
use serde::Deserialize;
use std::collections::HashMap;
use tempfile::{Builder, TempPath};

use env_file_reader::read_file;

/// Env variables, that can be defined in variables, or sourced from env files.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Envs {
    #[serde(default, deserialize_with = "decode::string_or_seq")]
    env_file: Vec<String>,
    #[serde(default)]
    variables: HashMap<String, String>,
}

/// Env output key name
const OUTPUT_ENV: &str = "ARROW_ENV";

impl Envs {
    /// Create new Envs from hashmap
    pub fn from_vars(vars: HashMap<String, String>) -> Self {
        let mut envs = Envs::default();
        envs.variables = vars;
        envs
    }

    // Setup output env file, and export it as $ARROW_ENV.
    pub fn setup_output_env(&self) -> anyhow::Result<Self> {
        let mut envs = self.clone();
        let env_path = Self::create_output_env_file()?
            .to_path_buf()
            .to_string_lossy()
            .to_string();
        envs.env_file.push(env_path.clone());
        envs.variables.insert(OUTPUT_ENV.to_string(), env_path);
        Ok(envs)
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
            let path = std::path::Path::new(file);
            if path.exists() {
                let variables = read_file(file)?;
                vars.extend(variables);
            }
        }
        vars.extend(self.variables.clone());
        Ok(vars)
    }

    /// Create temporary file for output envs
    fn create_output_env_file() -> anyhow::Result<TempPath> {
        let file = Builder::new().prefix("arrow-").suffix(".env").tempfile()?;
        return Ok(file.into_temp_path());
    }
}
