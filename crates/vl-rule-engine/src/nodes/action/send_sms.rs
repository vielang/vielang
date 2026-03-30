use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Send SMS via HTTP provider (Twilio-compatible).
/// Java: TbSendSmsNode
/// Config:
/// ```json
/// {
///   "providerUrl": "https://api.twilio.com/2010-04-01/Accounts/{sid}/Messages",
///   "accountSid": "...",
///   "authToken": "...",
///   "from": "+15551234567",
///   "toMetadataKey": "phone"
/// }
/// ```
pub struct SendSmsNode {
    provider_url: String,
    account_sid: String,
    auth_token: String,
    from: String,
    to_metadata_key: String,
    body_metadata_key: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "providerUrl", default)]
    provider_url: String,
    #[serde(rename = "accountSid", default)]
    account_sid: String,
    #[serde(rename = "authToken", default)]
    auth_token: String,
    #[serde(default)]
    from: String,
    #[serde(rename = "toMetadataKey", default = "default_to_key")]
    to_metadata_key: String,
    #[serde(rename = "bodyMetadataKey")]
    body_metadata_key: Option<String>,
}

fn default_to_key() -> String { "phone".into() }

impl SendSmsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("SendSmsNode: {}", e)))?;
        Ok(Self {
            provider_url: cfg.provider_url,
            account_sid: cfg.account_sid,
            auth_token: cfg.auth_token,
            from: cfg.from,
            to_metadata_key: cfg.to_metadata_key,
            body_metadata_key: cfg.body_metadata_key,
        })
    }
}

#[async_trait]
impl RuleNode for SendSmsNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let to = match msg.metadata.get(&self.to_metadata_key) {
            Some(v) => v.clone(),
            None => return Ok(vec![(RelationType::Failure, msg)]),
        };
        let body = self.body_metadata_key.as_ref()
            .and_then(|k| msg.metadata.get(k))
            .cloned()
            .unwrap_or_else(|| msg.data.clone());

        if self.provider_url.is_empty() {
            tracing::info!("SendSmsNode (log-only): to={} body={}", to, body);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let client = reqwest::Client::new();
        let res = client.post(&self.provider_url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&[("From", self.from.as_str()), ("To", to.as_str()), ("Body", body.as_str())])
            .send()
            .await
            .map_err(|e| RuleEngineError::Processing(format!("SMS send: {}", e)))?;

        if res.status().is_success() {
            Ok(vec![(RelationType::Success, msg)])
        } else {
            Ok(vec![(RelationType::Failure, msg)])
        }
    }
}
