use crate::actions::IAction;
use crate::envs::Envs;
use crate::helper::format_duration;
use crate::repo::Context;
use core::time;
use handlebars::Handlebars;
use serde::Deserialize;
use serde_urlencoded;
use serde_yaml::Value;
use std::collections::HashMap;
use std::time::Instant;
use ureq::{self, Response};

#[derive(Debug, Deserialize, Clone)]
pub struct WebHookAction {
    name: String,
    http: HookSpec,

    #[serde(flatten)]
    envs: Envs,
}

#[derive(Debug, Deserialize, Clone)]
struct HookSpec {
    #[serde(default = "default_method")]
    method: String,
    url: String,
    headers: HashMap<String, String>,
    timeout: Option<time::Duration>,
    body: Option<BodyData>,

    // env rendered body string
    #[serde(skip_deserializing)]
    body_string: String,
}

fn default_method() -> String {
    "POST".to_string()
}

const USER_AGENT: &str = "git-arrow/0.1.0";

impl IAction for WebHookAction {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()> {
        println!("### {}\n", self.name);
        let envs = self.envs.inherit(parent_env);
        let hook = self.http.render_env(envs)?;

        let method = hook.method.to_uppercase();
        let url = hook.url.as_ref();
        println!("  {} {}", method, url);
        let mut req = ureq::request(method.as_str(), url);
        let timeout = hook.timeout.unwrap_or(time::Duration::from_secs(10));
        req = req.timeout(timeout);
        req = req.set("User-Agent", USER_AGENT);
        for (k, v) in hook.headers.clone() {
            req = req.set(&k, &v);
        }
        let start_time = Instant::now();

        let resp: Response;
        // send request depend on body type
        match &hook.body {
            None => {
                resp = req.call()?;
            }
            Some(body) => match body {
                BodyData::FormData(_) => {
                    req = req.set("Content-Type", "application/x-www-form-urlencoded");
                    resp = req.send_bytes(&hook.body_string.as_bytes())?;
                }
                BodyData::JsonData(_) => {
                    req = req.set("Content-Type", "application/json");
                    resp = req.send_bytes(&hook.body_string.as_bytes())?;
                }
            },
        }
        let status = resp.status();
        let duration = format_duration(start_time.elapsed());
        let resp_body = resp.into_string()?;
        println!("  {} ({}): {}", status, duration, resp_body);
        if status >= 400 {
            return Err(anyhow::anyhow!("{}: {}", status, resp_body));
        }
        return Ok(());
    }
}

type FormData = HashMap<String, String>;
type JsonData = HashMap<String, Value>;

#[derive(Debug, Deserialize, Clone)]
enum BodyData {
    #[serde(rename = "formData")]
    FormData(FormData),
    #[serde(rename = "jsonData")]
    JsonData(JsonData),
}

impl HookSpec {
    fn render_env(&self, envs: Envs) -> anyhow::Result<Self> {
        let mut out = self.clone();
        let hbs = Handlebars::new();
        let variables = envs.build_env()?;
        out.method = hbs.render_template(&self.method, &variables)?;
        out.url = hbs.render_template(&self.url, &variables)?;
        for (k, v) in self.headers.clone() {
            let v = hbs.render_template(&v, &variables)?;
            out.headers.insert(k, v);
        }
        match &self.body {
            None => {}
            Some(body) => match body {
                BodyData::FormData(form_data) => {
                    let body = serde_urlencoded::to_string(form_data)?;
                    let body = hbs.render_template(&body, &variables)?;
                    out.body_string = body;
                }
                BodyData::JsonData(json_data) => {
                    let body = serde_json::to_string(json_data)?;
                    let body = hbs.render_template(&body, &variables)?;
                    out.body_string = body;
                }
            },
        }
        Ok(out)
    }
}
