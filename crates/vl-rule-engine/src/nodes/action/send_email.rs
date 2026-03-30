use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Send an email using SMTP (via lettre).
/// Java: TbSendEmailNode
/// Config:
/// ```json
/// {
///   "smtpHost": "smtp.example.com",
///   "smtpPort": 587,
///   "username": "user",
///   "password": "pass",
///   "from": "noreply@example.com",
///   "toMetadataKey": "email",
///   "subjectMetadataKey": "subject"
/// }
/// ```
pub struct SendEmailNode {
    smtp_host: String,
    smtp_port: u16,
    username: String,
    password: String,
    from: String,
    to_metadata_key: String,
    subject_metadata_key: String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "smtpHost", default)]
    smtp_host: String,
    #[serde(rename = "smtpPort", default = "default_smtp_port")]
    smtp_port: u16,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    from: String,
    #[serde(rename = "toMetadataKey", default = "default_to_key")]
    to_metadata_key: String,
    #[serde(rename = "subjectMetadataKey", default = "default_subject_key")]
    subject_metadata_key: String,
}

fn default_smtp_port() -> u16 { 587 }
fn default_to_key() -> String { "email".into() }
fn default_subject_key() -> String { "subject".into() }

impl SendEmailNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("SendEmailNode: {}", e)))?;
        Ok(Self {
            smtp_host: cfg.smtp_host,
            smtp_port: cfg.smtp_port,
            username: cfg.username,
            password: cfg.password,
            from: cfg.from,
            to_metadata_key: cfg.to_metadata_key,
            subject_metadata_key: cfg.subject_metadata_key,
        })
    }
}

#[async_trait]
impl RuleNode for SendEmailNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        use lettre::{Message, SmtpTransport, Transport, transport::smtp::authentication::Credentials};

        let to_addr = match msg.metadata.get(&self.to_metadata_key) {
            Some(v) => v.clone(),
            None => return Ok(vec![(RelationType::Failure, msg)]),
        };
        let subject = msg.metadata.get(&self.subject_metadata_key)
            .cloned()
            .unwrap_or_else(|| "ThingsBoard Notification".into());

        if self.smtp_host.is_empty() {
            // Log-only mode (no SMTP configured)
            tracing::info!("SendEmailNode (log-only): to={} subject={} body={}", to_addr, subject, msg.data);
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let email = Message::builder()
            .from(self.from.parse().map_err(|e| RuleEngineError::Config(format!("invalid from: {}", e)))?)
            .to(to_addr.parse().map_err(|e| RuleEngineError::Config(format!("invalid to: {}", e)))?)
            .subject(&subject)
            .body(msg.data.clone())
            .map_err(|e| RuleEngineError::Processing(format!("email build: {}", e)))?;

        let creds = Credentials::new(self.username.clone(), self.password.clone());
        let mailer = SmtpTransport::starttls_relay(&self.smtp_host)
            .map_err(|e| RuleEngineError::Processing(format!("SMTP relay: {}", e)))?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        tokio::task::spawn_blocking(move || mailer.send(&email))
            .await
            .map_err(|e| RuleEngineError::Processing(format!("smtp spawn: {}", e)))?
            .map_err(|e| RuleEngineError::Processing(format!("smtp send: {}", e)))?;

        Ok(vec![(RelationType::Success, msg)])
    }
}
