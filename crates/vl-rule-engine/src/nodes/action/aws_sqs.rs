use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Send message to AWS SQS queue via HTTP endpoint.
/// Java: TbSqsNode
/// Config:
/// ```json
/// { "queueUrl": "https://sqs.us-east-1.amazonaws.com/123/MyQueue" }
/// ```
pub struct AwsSqsNode {
    queue_url: String,
    delay_seconds: u32,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "queueUrl", default)]
    queue_url: String,
    #[serde(rename = "delaySeconds", default)]
    delay_seconds: u32,
}

impl AwsSqsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("AwsSqsNode: {}", e)))?;
        Ok(Self { queue_url: cfg.queue_url, delay_seconds: cfg.delay_seconds })
    }
}

#[async_trait]
impl RuleNode for AwsSqsNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if self.queue_url.is_empty() {
            tracing::info!("AwsSqsNode (log-only): queue={} body={}", self.queue_url, msg.data);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let params = format!(
            "Action=SendMessage&MessageBody={}&DelaySeconds={}",
            urlencoding(&msg.data),
            self.delay_seconds
        );

        let client = reqwest::Client::new();
        let res = client.post(&self.queue_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(params)
            .send()
            .await
            .map_err(|e| RuleEngineError::Processing(format!("SQS send: {}", e)))?;

        if res.status().is_success() {
            Ok(vec![(RelationType::Success, msg)])
        } else {
            Ok(vec![(RelationType::Failure, msg)])
        }
    }
}

fn urlencoding(s: &str) -> String {
    s.bytes().flat_map(|b| {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            vec![b as char]
        } else {
            format!("%{:02X}", b).chars().collect()
        }
    }).collect()
}
