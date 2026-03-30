use std::collections::HashMap;
use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Publish message to an external MQTT broker.
/// Config JSON:
/// ```json
/// {
///   "topicPattern": "devices/${deviceName}/telemetry",
///   "host": "broker.example.com",
///   "port": 1883,
///   "clientId": "vielang-rule-engine",
///   "username": null,
///   "password": null,
///   "cleanSession": true,
///   "ssl": false,
///   "qos": 1,
///   "retain": false
/// }
/// ```
pub struct MqttPublishNode {
    topic_pattern: String,
    host:          String,
    port:          u16,
    client_id:     String,
    username:      Option<String>,
    password:      Option<String>,
    qos:           u8,
    retain:        bool,
}

#[derive(Deserialize)]
struct MqttConfig {
    #[serde(rename = "topicPattern")]
    topic_pattern: String,
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(rename = "clientId", default = "default_client_id")]
    client_id: String,
    username: Option<String>,
    password: Option<String>,
    #[serde(default)]
    qos: u8,
    #[serde(default)]
    retain: bool,
}

fn default_port() -> u16 { 1883 }
fn default_client_id() -> String { "vl-rule-engine".to_string() }

impl MqttPublishNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: MqttConfig = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("MqttPublishNode config: {}", e)))?;
        Ok(Self {
            topic_pattern: cfg.topic_pattern,
            host:          cfg.host,
            port:          cfg.port,
            client_id:     cfg.client_id,
            username:      cfg.username,
            password:      cfg.password,
            qos:           cfg.qos,
            retain:        cfg.retain,
        })
    }
}

#[async_trait]
impl RuleNode for MqttPublishNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        use rumqttc::{AsyncClient, MqttOptions, QoS};

        let topic = substitute_placeholders(&self.topic_pattern, &msg.metadata);
        let qos = match self.qos {
            2 => QoS::ExactlyOnce,
            1 => QoS::AtLeastOnce,
            _ => QoS::AtMostOnce,
        };

        let mut opts = MqttOptions::new(&self.client_id, &self.host, self.port);
        opts.set_keep_alive(std::time::Duration::from_secs(30));
        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            opts.set_credentials(u, p);
        }

        let (client, mut eventloop) = AsyncClient::new(opts, 16);

        // Publish and wait for ack
        let payload = msg.data.as_bytes().to_vec();
        client.publish(topic, qos, self.retain, payload).await
            .map_err(|e| RuleEngineError::Script(format!("MQTT publish error: {}", e)))?;

        // Poll until published/acked or error
        let result = loop {
            match eventloop.poll().await {
                Ok(event) => {
                    use rumqttc::Event;
                    if let Event::Outgoing(rumqttc::Outgoing::Publish(_)) = event {
                        break Ok(());
                    }
                    // For QoS 0 we won't get Puback; break after first outgoing
                    if self.qos == 0 { break Ok(()); }
                    if let Event::Incoming(rumqttc::Packet::PubAck(_)) = event {
                        break Ok(());
                    }
                }
                Err(e) => break Err(e),
            }
        };

        client.disconnect().await.ok();

        match result {
            Ok(_) => Ok(vec![(RelationType::Success, msg)]),
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
