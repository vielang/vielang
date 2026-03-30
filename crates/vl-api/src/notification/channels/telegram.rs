use reqwest::Client;
use serde_json::json;
use tracing::{info, warn};
use super::NotificationChannel;

pub struct TelegramChannel {
    client: Client,
}

pub struct TelegramMessage {
    pub bot_token: String,
    pub chat_id:   String,
    pub text:      String,
}

impl TelegramChannel {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// POST https://api.telegram.org/bot{token}/sendMessage
    /// Hỗ trợ HTML parse mode: <b>bold</b>, <i>italic</i>, <code>code</code>
    pub async fn send_message(&self, msg: &TelegramMessage) -> Result<(), String> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            msg.bot_token
        );

        let payload = json!({
            "chat_id":    msg.chat_id,
            "text":       msg.text,
            "parse_mode": "HTML",
        });

        let resp = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                warn!(error = %e, chat_id = %msg.chat_id, "Telegram send failed");
                format!("Telegram HTTP error: {}", e)
            })?;

        if resp.status().is_success() {
            info!(chat_id = %msg.chat_id, "Telegram notification sent");
            Ok(())
        } else {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Telegram returned error");
            Err(format!("Telegram error {}: {}", status, body))
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for TelegramChannel {
    fn method(&self) -> &'static str { "TELEGRAM" }

    /// `recipient` is formatted as "bot_token:chat_id"; `body` is the message text.
    async fn send(&self, recipient: &str, _subject: &str, body: &str) -> Result<(), String> {
        let (bot_token, chat_id) = recipient.split_once(':')
            .ok_or_else(|| "Telegram recipient must be 'bot_token:chat_id'".to_owned())?;
        let msg = TelegramMessage {
            bot_token: bot_token.to_owned(),
            chat_id:   chat_id.to_owned(),
            text:      body.to_owned(),
        };
        self.send_message(&msg).await
    }
}
