use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Publish message to a Kafka topic via Kafka REST Proxy.
/// Java: TbKafkaNode
/// Config:
/// ```json
/// {
///   "bootstrapServers": "http://kafka-rest:8082",
///   "topicPattern": "my-topic",
///   "keyMetadataKey": "deviceId"
/// }
/// ```
pub struct KafkaNode {
    rest_url: String,
    topic: String,
    key_metadata_key: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "bootstrapServers", default)]
    bootstrap_servers: String,
    #[serde(rename = "topicPattern", default = "default_topic")]
    topic_pattern: String,
    #[serde(rename = "keyMetadataKey")]
    key_metadata_key: Option<String>,
}

fn default_topic() -> String { "vl-rule-engine".into() }

impl KafkaNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("KafkaNode: {}", e)))?;
        Ok(Self {
            rest_url: cfg.bootstrap_servers,
            topic: cfg.topic_pattern,
            key_metadata_key: cfg.key_metadata_key,
        })
    }
}

#[async_trait]
impl RuleNode for KafkaNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if self.rest_url.is_empty() {
            tracing::info!("KafkaNode (log-only): topic={} body={}", self.topic, msg.data);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let key = self.key_metadata_key.as_ref()
            .and_then(|k| msg.metadata.get(k))
            .map(|v| serde_json::Value::String(v.clone()))
            .unwrap_or(serde_json::Value::Null);

        let payload = serde_json::json!({
            "records": [{
                "key": key,
                "value": msg.data
            }]
        });

        let url = format!("{}/topics/{}", self.rest_url.trim_end_matches('/'), self.topic);
        let client = reqwest::Client::new();
        let res = client.post(&url)
            .header("Content-Type", "application/vnd.kafka.json.v2+json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| RuleEngineError::Processing(format!("Kafka REST: {}", e)))?;

        if res.status().is_success() {
            Ok(vec![(RelationType::Success, msg)])
        } else {
            Ok(vec![(RelationType::Failure, msg)])
        }
    }
}
