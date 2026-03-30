pub mod email;
pub mod slack;
pub mod sms;
pub mod teams;
pub mod telegram;
pub mod webhook;

pub use email::{EmailChannel, EmailMessage};
pub use slack::{SlackChannel, SlackMessage};
pub use sms::SmsChannel;
pub use teams::{TeamsChannel, TeamsMessage};
pub use telegram::{TelegramChannel, TelegramMessage};
pub use webhook::{WebhookChannel, WebhookMessage};

/// Unified notification channel trait.
/// Each implementation maps to a delivery method name (e.g. "EMAIL", "SLACK").
#[async_trait::async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Delivery method name (e.g., "EMAIL", "SLACK", "SMS", "TEAMS", "WEBHOOK", "TELEGRAM")
    fn method(&self) -> &'static str;

    /// Send a notification. Returns Ok(()) on success, Err with message on failure.
    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), String>;
}
