use reqwest::Client;
use serde_json::json;
use tokio::time::sleep;
use std::time::Duration;
use tracing::{info, warn};
use super::NotificationChannel;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 500;

pub struct SlackChannel {
    client: Client,
}

pub struct SlackMessage {
    pub webhook_url: String,
    pub channel:     String,
    pub text:        String,
}

impl SlackChannel {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn send_message(&self, msg: &SlackMessage) -> Result<(), String> {
        let payload = json!({
            "text":       msg.text,
            "channel":    msg.channel,
            "username":   "VieLang",
            "icon_emoji": ":bell:",
            "mrkdwn":     true
        });

        let mut attempt = 0u32;
        loop {
            let resp = self.client
                .post(&msg.webhook_url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| {
                    warn!(error = %e, channel = %msg.channel, "Slack send failed");
                    format!("Slack HTTP error: {}", e)
                })?;

            let status = resp.status();

            // Rate limited — respect Retry-After or use exponential backoff
            if status.as_u16() == 429 && attempt < MAX_RETRIES {
                let retry_after_ms = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|s| s * 1000)  // seconds → ms
                    .unwrap_or(INITIAL_BACKOFF_MS << attempt);

                warn!(
                    channel = %msg.channel,
                    attempt = attempt + 1,
                    retry_after_ms = retry_after_ms,
                    "Slack rate limited — retrying"
                );
                sleep(Duration::from_millis(retry_after_ms)).await;
                attempt += 1;
                continue;
            }

            if status.is_success() {
                info!(channel = %msg.channel, "Slack notification sent");
                return Ok(());
            }

            let body = resp.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Slack returned error");
            return Err(format!("Slack error {}: {}", status, body));
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for SlackChannel {
    fn method(&self) -> &'static str { "SLACK" }

    /// `recipient` is treated as the webhook URL; `body` is the message text.
    async fn send(&self, recipient: &str, _subject: &str, body: &str) -> Result<(), String> {
        let msg = SlackMessage {
            webhook_url: recipient.to_owned(),
            channel:     "#general".to_owned(),
            text:        body.to_owned(),
        };
        self.send_message(&msg).await
    }
}
