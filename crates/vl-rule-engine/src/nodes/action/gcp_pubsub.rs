use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Publish message to Google Cloud Pub/Sub via REST API.
/// Java: TbPubSubNode
/// Config:
/// ```json
/// {
///   "projectId": "my-gcp-project",
///   "topicName": "my-topic",
///   "accessToken": "ya29...."
/// }
/// ```
pub struct GcpPubSubNode {
    project_id: String,
    topic_name: String,
    access_token: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "projectId", default)]
    project_id: String,
    #[serde(rename = "topicName", default)]
    topic_name: String,
    #[serde(rename = "accessToken", default)]
    access_token: String,
}

impl GcpPubSubNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GcpPubSubNode: {}", e)))?;
        Ok(Self {
            project_id: cfg.project_id,
            topic_name: cfg.topic_name,
            access_token: cfg.access_token,
        })
    }
}

#[async_trait]
impl RuleNode for GcpPubSubNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if self.project_id.is_empty() {
            tracing::info!("GcpPubSubNode (log-only): topic={} body={}", self.topic_name, msg.data);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        use std::collections::HashMap;
        let encoded = base64_encode(msg.data.as_bytes());
        let payload = serde_json::json!({
            "messages": [{ "data": encoded, "attributes": HashMap::<String, String>::new() }]
        });

        let url = format!(
            "https://pubsub.googleapis.com/v1/projects/{}/topics/{}:publish",
            self.project_id, self.topic_name
        );
        let client = reqwest::Client::new();
        let res = client.post(&url)
            .bearer_auth(&self.access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| RuleEngineError::Processing(format!("PubSub publish: {}", e)))?;

        if res.status().is_success() {
            Ok(vec![(RelationType::Success, msg)])
        } else {
            Ok(vec![(RelationType::Failure, msg)])
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b = match chunk.len() {
            3 => [chunk[0], chunk[1], chunk[2]],
            2 => [chunk[0], chunk[1], 0],
            _ => [chunk[0], 0, 0],
        };
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 { out.push(CHARS[((n >> 6) & 63) as usize] as char); } else { out.push('='); }
        if chunk.len() > 2 { out.push(CHARS[(n & 63) as usize] as char); } else { out.push('='); }
    }
    out
}
