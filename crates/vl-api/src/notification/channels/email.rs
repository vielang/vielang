use vl_config::SmtpConfig;
use tracing::{info, warn};
use super::NotificationChannel;

pub struct EmailChannel {
    config: SmtpConfig,
}

pub struct EmailMessage {
    pub from:    String,
    pub to:      String,
    pub subject: String,
    pub body:    String,
    pub is_html: bool,
}

impl EmailChannel {
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    pub async fn send_message(&self, msg: &EmailMessage) -> Result<(), String> {
        if self.config.host.is_empty() {
            // SMTP not configured — log only (safe for tests and dev environments)
            info!(
                to    = %msg.to,
                subj  = %msg.subject,
                "Email channel: SMTP not configured, logging notification only"
            );
            return Ok(());
        }

        self.send_smtp(msg).await
    }

    async fn send_smtp(&self, msg: &EmailMessage) -> Result<(), String> {
        use lettre::{
            AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
            transport::smtp::authentication::Credentials,
            message::header::ContentType,
        };

        let email = Message::builder()
            .from(msg.from.parse().map_err(|e| format!("Invalid from: {}", e))?)
            .to(msg.to.parse().map_err(|e| format!("Invalid to: {}", e))?)
            .subject(&msg.subject)
            .header(if msg.is_html { ContentType::TEXT_HTML } else { ContentType::TEXT_PLAIN })
            .body(msg.body.clone())
            .map_err(|e| format!("Build email error: {}", e))?;

        let creds = Credentials::new(
            self.config.username.clone(),
            self.config.password.clone(),
        );

        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.host)
            .port(self.config.port)
            .credentials(creds)
            .build();

        transport
            .send(email)
            .await
            .map_err(|e| {
                warn!(error = %e, to = %msg.to, "Failed to send email");
                format!("SMTP error: {}", e)
            })?;

        info!(to = %msg.to, subject = %msg.subject, "Email sent");
        Ok(())
    }
}

#[async_trait::async_trait]
impl NotificationChannel for EmailChannel {
    fn method(&self) -> &'static str { "EMAIL" }

    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), String> {
        let msg = EmailMessage {
            from:    "noreply@vielang.local".to_owned(),
            to:      recipient.to_owned(),
            subject: subject.to_owned(),
            body:    body.to_owned(),
            is_html: false,
        };
        self.send_message(&msg).await
    }
}
