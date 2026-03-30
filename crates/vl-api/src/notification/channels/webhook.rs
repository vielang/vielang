use std::collections::HashMap;
use std::time::Duration;

use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use tracing::{info, warn};
use chrono::Utc;
use super::NotificationChannel;

const REQUEST_TIMEOUT_SECS: u64 = 10;

type HmacSha256 = Hmac<Sha256>;

pub struct WebhookChannel {
    client: Client,
}

pub struct WebhookMessage {
    pub url:        String,
    pub method:     String,
    pub headers:    HashMap<String, String>,
    pub body:       serde_json::Value,
    /// Optional HMAC secret — when set, adds X-ThingsBoard-Signature header
    pub hmac_secret: Option<String>,
}

impl WebhookChannel {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn send_message(&self, msg: &WebhookMessage) -> Result<(), String> {
        let method = reqwest::Method::from_bytes(msg.method.to_uppercase().as_bytes())
            .unwrap_or(reqwest::Method::POST);

        let body_bytes = serde_json::to_vec(&msg.body)
            .map_err(|e| format!("Webhook serialize error: {}", e))?;

        let mut request = self.client
            .request(method, &msg.url)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .header("Content-Type", "application/json");

        for (key, value) in &msg.headers {
            request = request.header(key, value);
        }

        // HMAC-SHA256 signature
        if let Some(secret) = &msg.hmac_secret {
            let signature = compute_hmac_sha256(secret, &body_bytes);
            request = request.header("X-ThingsBoard-Signature", format!("sha256={}", signature));
        }

        let resp = request
            .body(body_bytes)
            .send()
            .await
            .map_err(|e| {
                warn!(error = %e, url = %msg.url, "Webhook send failed");
                if e.is_timeout() {
                    format!("Webhook timeout after {}s: {}", REQUEST_TIMEOUT_SECS, e)
                } else {
                    format!("Webhook HTTP error: {}", e)
                }
            })?;

        if resp.status().is_success() {
            info!(url = %msg.url, "Webhook notification sent");
            Ok(())
        } else {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Webhook returned error");
            Err(format!("Webhook error {}: {}", status, body))
        }
    }
}

fn compute_hmac_sha256(secret: &str, payload: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(payload);
    let result = mac.finalize().into_bytes();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

#[async_trait::async_trait]
impl NotificationChannel for WebhookChannel {
    fn method(&self) -> &'static str { "WEBHOOK" }

    /// `recipient` is the webhook URL; `body` is sent as JSON payload.
    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), String> {
        let payload = serde_json::json!({
            "subject": subject,
            "body":    body,
            "timestamp": Utc::now().timestamp_millis(),
        });
        let msg = WebhookMessage {
            url:         recipient.to_owned(),
            method:      "POST".to_owned(),
            headers:     HashMap::new(),
            body:        payload,
            hmac_secret: None,
        };
        self.send_message(&msg).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_hmac_signature() {
        let secret  = "my-secret-key";
        let payload = b"{\"body\":\"test\"}";
        let sig     = compute_hmac_sha256(secret, payload);
        // Verify it's a 64-char hex string (SHA256 = 32 bytes)
        assert_eq!(sig.len(), 64);
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
        // Deterministic
        assert_eq!(sig, compute_hmac_sha256(secret, payload));
    }
}
