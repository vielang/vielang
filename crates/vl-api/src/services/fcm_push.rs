use reqwest::Client;
use serde_json::{json, Value};
use tracing::{info, warn};

pub struct FcmPushService {
    server_key:  String,
    http_client: Client,
}

impl FcmPushService {
    pub fn new(server_key: String) -> Self {
        Self {
            server_key,
            http_client: Client::new(),
        }
    }

    /// Send to a single FCM token. Returns true if sent successfully.
    pub async fn send_to_token(
        &self,
        token: &str,
        title: &str,
        body:  &str,
        data:  Option<Value>,
    ) -> bool {
        let mut msg = json!({
            "to": token,
            "notification": {
                "title": title,
                "body":  body,
            },
            "priority": "high",
        });

        if let Some(d) = data {
            msg["data"] = d;
        }

        match self.http_client
            .post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={}", self.server_key))
            .header("Content-Type", "application/json")
            .json(&msg)
            .send()
            .await
        {
            Err(e) => {
                warn!("FCM send error: {e}");
                false
            }
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    info!("FCM sent to token ...{}", &token[token.len().saturating_sub(8)..]);
                    true
                } else {
                    let body_text = resp.text().await.unwrap_or_default();
                    warn!("FCM error {status}: {body_text}");
                    false
                }
            }
        }
    }

    /// Send to multiple tokens. Returns count of successes.
    pub async fn send_to_tokens(
        &self,
        tokens: &[String],
        title:  &str,
        body:   &str,
        data:   Option<Value>,
    ) -> usize {
        let mut success = 0usize;
        for token in tokens {
            if self.send_to_token(token, title, body, data.clone()).await {
                success += 1;
            }
        }
        success
    }
}
