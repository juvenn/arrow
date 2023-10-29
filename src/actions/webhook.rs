use crate::action::IAction;
use crate::envs::Envs;
use crate::repo::Context;
use core::time;
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
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
    body: Option<BodyData>,
    timeout: Option<time::Duration>,
}

fn default_method() -> String {
    "POST".to_string()
}

const USER_AGENT: &str = "git-arrow/0.1.0";

impl IAction for WebHookAction {
    fn run(&self, ctx: &Context, parent_env: &Envs) -> anyhow::Result<()> {
        println!("### {}\n", self.name);
        let mut envs = self.envs.inherit(parent_env);

        let method = self.http.method.to_uppercase();
        let url = self.http.url.as_ref();
        println!("  {} {}", method, url);
        let mut req = ureq::request(method.as_str(), url);
        let timeout = self.http.timeout.unwrap_or(time::Duration::from_secs(10));
        req = req.timeout(timeout);
        req = req.set("User-Agent", USER_AGENT);
        for (k, v) in self.http.headers.clone() {
            req = req.set(&k, &v);
        }
        let start_time = Instant::now();

        let resp: Response;
        // send request depend on body type
        match &self.http.body {
            None => {
                resp = req.call()?;
            }
            Some(body) => match body {
                BodyData::FormData(form_data) => {
                    let pairs = form_data
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect::<Vec<_>>();
                    resp = req.send_form(&pairs)?;
                }
                BodyData::JsonData(json_data) => {
                    resp = req.send_json(json_data)?;
                }
            },
        }
        let status = resp.status();
        let duration = format_duration(start_time.elapsed());
        let resp_body = resp.into_string()?;
        println!("  ({}) {}: {}", duration, status, resp_body);
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

fn format_duration(du: Duration) -> String {
    let ms = du.as_millis();
    if ms < 1000 {
        return format!("{}ms", ms);
    }
    let s = du.as_secs();
    if s < 60 {
        return format!("{}s", s);
    }
    let m = s / 60;
    let s = s % 60;
    if m < 60 {
        return format!("{}m{}s", m, s);
    }
    let h = m / 60;
    let m = m % 60;
    return format!("{}h{}m{}s", h, m, s);
}
