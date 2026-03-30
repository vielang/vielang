use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Publish message to AWS SNS topic via HTTP endpoint.
/// Java: TbSnsNode
/// Config:
/// ```json
/// { "topicArn": "arn:aws:sns:us-east-1:123:MyTopic", "endpointUrl": "https://sns.us-east-1.amazonaws.com" }
/// ```
pub struct AwsSnsNode {
    endpoint_url: String,
    topic_arn: String,
    subject_metadata_key: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "endpointUrl", default)]
    endpoint_url: String,
    #[serde(rename = "topicArn", default)]
    topic_arn: String,
    #[serde(rename = "subjectMetadataKey")]
    subject_metadata_key: Option<String>,
}

impl AwsSnsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("AwsSnsNode: {}", e)))?;
        Ok(Self {
            endpoint_url: cfg.endpoint_url,
            topic_arn: cfg.topic_arn,
            subject_metadata_key: cfg.subject_metadata_key,
        })
    }
}

#[async_trait]
impl RuleNode for AwsSnsNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if self.endpoint_url.is_empty() {
            tracing::info!("AwsSnsNode (log-only): topic={} body={}", self.topic_arn, msg.data);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let subject = self.subject_metadata_key.as_ref()
            .and_then(|k| msg.metadata.get(k))
            .cloned()
            .unwrap_or_else(|| "ThingsBoard".into());

        let params = format!(
            "Action=Publish&TopicArn={}&Message={}&Subject={}",
            urlencoding(&self.topic_arn),
            urlencoding(&msg.data),
            urlencoding(&subject)
        );

        let client = reqwest::Client::new();
        let res = client.post(&self.endpoint_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(params)
            .send()
            .await
            .map_err(|e| RuleEngineError::Processing(format!("SNS publish: {}", e)))?;

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
