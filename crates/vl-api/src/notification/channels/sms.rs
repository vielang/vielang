use reqwest::Client;
use tracing::{info, warn};
use vl_config::SmsConfig;
use super::NotificationChannel;

pub struct SmsChannel {
    inner: SmsInner,
}

enum SmsInner {
    Twilio {
        account_sid:  String,
        auth_token:   String,
        from_number:  String,
        client:       Client,
    },
    Disabled,
}

impl SmsChannel {
    pub fn new(config: &SmsConfig, client: Client) -> Self {
        let inner = match config.provider.as_str() {
            "twilio" if !config.twilio_account_sid.is_empty() => SmsInner::Twilio {
                account_sid:  config.twilio_account_sid.clone(),
                auth_token:   config.twilio_auth_token.clone(),
                from_number:  config.twilio_from_number.clone(),
                client,
            },
            "twilio" => {
                warn!("SMS provider=twilio but twilio_account_sid is empty — disabling SMS");
                SmsInner::Disabled
            }
            other => {
                if other != "disabled" {
                    warn!(provider = other, "Unknown SMS provider — disabling SMS");
                }
                SmsInner::Disabled
            }
        };
        Self { inner }
    }

    /// Send an SMS. Returns Ok(()) even when disabled (provider="disabled").
    pub async fn send_message(&self, to: &str, body: &str) -> Result<(), String> {
        match &self.inner {
            SmsInner::Disabled => {
                info!(to, "SMS suppressed (provider=disabled)");
                Ok(())
            }
            SmsInner::Twilio { account_sid, auth_token, from_number, client } => {
                let url = format!(
                    "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
                    account_sid
                );
                let resp = client
                    .post(&url)
                    .basic_auth(account_sid, Some(auth_token))
                    .form(&[("To", to), ("From", from_number.as_str()), ("Body", body)])
                    .send()
                    .await
                    .map_err(|e| format!("Twilio request failed: {e}"))?;

                if resp.status().is_success() {
                    info!(to, "SMS sent via Twilio");
                    Ok(())
                } else {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    warn!(to, %status, "Twilio error: {text}");
                    Err(format!("Twilio error {status}: {text}"))
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for SmsChannel {
    fn method(&self) -> &'static str { "SMS" }

    /// `recipient` is the phone number; `body` is the message text. `subject` is ignored.
    async fn send(&self, recipient: &str, _subject: &str, body: &str) -> Result<(), String> {
        self.send_message(recipient, body).await
    }
}
