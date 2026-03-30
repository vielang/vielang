use reqwest::Client;
use serde_json::{json, Value};
use tracing::{info, warn};
use super::NotificationChannel;

pub struct TeamsChannel {
    client: Client,
}

pub struct TeamsMessage {
    pub webhook_url: String,
    pub title:       String,
    pub body:        String,
}

impl TeamsChannel {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn send_message(&self, msg: &TeamsMessage) -> Result<(), String> {
        let card = build_message_card(&msg.title, &msg.body);

        let resp = self.client
            .post(&msg.webhook_url)
            .json(&card)
            .send()
            .await
            .map_err(|e| {
                warn!(error = %e, "Teams send failed");
                format!("Teams HTTP error: {}", e)
            })?;

        if resp.status().is_success() {
            info!("Microsoft Teams notification sent");
            Ok(())
        } else {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            warn!(status = %status, body = %body, "Teams returned error");
            Err(format!("Teams error {}: {}", status, body))
        }
    }
}

/// Build Outlook Actionable Message Card (legacy Teams webhook format)
/// Java ref: MicrosoftTeamsNotificationChannel.java
fn build_message_card(title: &str, body: &str) -> Value {
    json!({
        "@type":       "MessageCard",
        "@context":    "http://schema.org/extensions",
        "themeColor":  "0076D7",
        "summary":     title,
        "sections": [{
            "activityTitle": title,
            "activityText":  body,
            "markdown":      true
        }]
    })
}

#[async_trait::async_trait]
impl NotificationChannel for TeamsChannel {
    fn method(&self) -> &'static str { "TEAMS" }

    /// `recipient` is the webhook URL; `subject` becomes the card title.
    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), String> {
        let title = if subject.is_empty() { "VieLang Notification" } else { subject };
        let msg = TeamsMessage {
            webhook_url: recipient.to_owned(),
            title:       title.to_owned(),
            body:        body.to_owned(),
        };
        self.send_message(&msg).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teams_card_format() {
        let card = build_message_card("Alarm: High Temperature", "Device temp exceeded 80°C");
        assert_eq!(card["@type"].as_str().unwrap(), "MessageCard");
        assert_eq!(card["themeColor"].as_str().unwrap(), "0076D7");
        let section = &card["sections"][0];
        assert_eq!(section["activityTitle"].as_str().unwrap(), "Alarm: High Temperature");
        assert_eq!(section["activityText"].as_str().unwrap(), "Device temp exceeded 80°C");
    }
}
