use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Publish message to a RabbitMQ exchange via HTTP Management API.
/// Java: TbRabbitMqNode
/// Config:
/// ```json
/// {
///   "host": "http://rabbitmq:15672",
///   "virtualHost": "/",
///   "username": "guest",
///   "password": "guest",
///   "exchangeName": "amq.topic",
///   "routingKeyMetadataKey": "deviceType"
/// }
/// ```
pub struct RabbitMqNode {
    host: String,
    virtual_host: String,
    username: String,
    password: String,
    exchange_name: String,
    routing_key_metadata_key: String,
    default_routing_key: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    host: String,
    #[serde(rename = "virtualHost", default = "default_vhost")]
    virtual_host: String,
    #[serde(default = "default_guest")]
    username: String,
    #[serde(default = "default_guest")]
    password: String,
    #[serde(rename = "exchangeName", default = "default_exchange")]
    exchange_name: String,
    #[serde(rename = "routingKeyMetadataKey", default = "default_routing_key_key")]
    routing_key_metadata_key: String,
    #[serde(rename = "defaultRoutingKey", default)]
    default_routing_key: String,
}

fn default_vhost() -> String { "%2F".into() }
fn default_guest() -> String { "guest".into() }
fn default_exchange() -> String { "amq.topic".into() }
fn default_routing_key_key() -> String { "deviceType".into() }

impl RabbitMqNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("RabbitMqNode: {}", e)))?;
        Ok(Self {
            host: cfg.host,
            virtual_host: cfg.virtual_host,
            username: cfg.username,
            password: cfg.password,
            exchange_name: cfg.exchange_name,
            routing_key_metadata_key: cfg.routing_key_metadata_key,
            default_routing_key: cfg.default_routing_key,
        })
    }
}

#[async_trait]
impl RuleNode for RabbitMqNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        if self.host.is_empty() {
            tracing::info!("RabbitMqNode (log-only): exchange={} body={}", self.exchange_name, msg.data);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let routing_key = msg.metadata.get(&self.routing_key_metadata_key)
            .cloned()
            .unwrap_or_else(|| self.default_routing_key.clone());

        let url = format!(
            "{}/api/exchanges/{}/{}/publish",
            self.host.trim_end_matches('/'),
            self.virtual_host,
            self.exchange_name
        );
        let payload = serde_json::json!({
            "routing_key": routing_key,
            "payload": msg.data,
            "payload_encoding": "string",
            "properties": {}
        });

        let client = reqwest::Client::new();
        let res = client.post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| RuleEngineError::Processing(format!("RabbitMQ publish: {}", e)))?;

        if res.status().is_success() {
            Ok(vec![(RelationType::Success, msg)])
        } else {
            Ok(vec![(RelationType::Failure, msg)])
        }
    }
}
