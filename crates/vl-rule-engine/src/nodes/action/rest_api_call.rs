use std::collections::HashMap;
use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Call an external REST API and attach the response to the message.
/// Config JSON:
/// ```json
/// {
///   "restEndpointUrlPattern": "https://api.example.com/${deviceName}",
///   "requestMethod": "POST",
///   "headers": { "Content-Type": "application/json" },
///   "readTimeoutMs": 10000
/// }
/// ```
pub struct RestApiCallNode {
    url_pattern:    String,
    method:         String,
    headers:        HashMap<String, String>,
    read_timeout_ms: u64,
}

#[derive(Deserialize)]
struct RestApiConfig {
    #[serde(rename = "restEndpointUrlPattern")]
    rest_endpoint_url_pattern: String,
    #[serde(rename = "requestMethod", default = "default_method")]
    request_method: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(rename = "readTimeoutMs", default = "default_timeout")]
    read_timeout_ms: u64,
}

fn default_method() -> String { "POST".to_string() }
fn default_timeout() -> u64 { 10_000 }

impl RestApiCallNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: RestApiConfig = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("RestApiCallNode config: {}", e)))?;
        Ok(Self {
            url_pattern:     cfg.rest_endpoint_url_pattern,
            method:          cfg.request_method,
            headers:         cfg.headers,
            read_timeout_ms: cfg.read_timeout_ms,
        })
    }
}

#[async_trait]
impl RuleNode for RestApiCallNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Substitute ${key} placeholders from metadata
        let url = substitute_placeholders(&self.url_pattern, &msg.metadata);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(self.read_timeout_ms))
            .build()
            .map_err(|e| RuleEngineError::Script(format!("HTTP client error: {}", e)))?;

        let mut req = match self.method.to_uppercase().as_str() {
            "GET"    => client.get(&url),
            "DELETE" => client.delete(&url),
            "PUT"    => client.put(&url).body(msg.data.clone()),
            _        => client.post(&url).body(msg.data.clone()),
        };

        for (k, v) in &self.headers {
            let val = substitute_placeholders(v, &msg.metadata);
            req = req.header(k, val);
        }

        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                let mut out = msg;
                out.metadata.insert("status".into(), status.to_string());
                if status < 400 {
                    out.data = body;
                    Ok(vec![(RelationType::Success, out)])
                } else {
                    out.metadata.insert("error".into(), body);
                    Ok(vec![(RelationType::Failure, out)])
                }
            }
            Err(e) => {
                let mut out = msg;
                out.metadata.insert("error".into(), e.to_string());
                Ok(vec![(RelationType::Failure, out)])
            }
        }
    }
}

fn substitute_placeholders(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (k, v) in vars {
        result = result.replace(&format!("${{{}}}", k), v);
    }
    result
}
