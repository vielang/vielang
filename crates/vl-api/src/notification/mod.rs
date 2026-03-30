pub mod channels;
pub mod template;

use std::sync::Arc;
use std::collections::HashMap;

use vl_config::{SmtpConfig, SmsConfig};
use vl_core::entities::{
    NotificationRequest, NotificationStatus, NotificationTarget, NotificationTemplate,
    NotificationType,
};
use vl_dao::postgres::{
    notification_delivery::NotificationDeliveryDao,
    notification_request::NotificationRequestDao,
    notification_target::NotificationTargetDao,
    notification_template::NotificationTemplateDao,
};
use tracing::{info, warn};

use channels::{
    EmailChannel, EmailMessage,
    SlackChannel, SlackMessage,
    SmsChannel,
    TeamsChannel, TeamsMessage,
    TelegramChannel, TelegramMessage,
    WebhookChannel, WebhookMessage,
};
use template::render_template;

use crate::services::fcm_push::FcmPushService;

pub struct NotificationService {
    pub template_dao:  Arc<NotificationTemplateDao>,
    pub target_dao:    Arc<NotificationTargetDao>,
    pub request_dao:   Arc<NotificationRequestDao>,
    delivery_dao:      Arc<NotificationDeliveryDao>,
    email:    EmailChannel,
    slack:    SlackChannel,
    sms:      SmsChannel,
    teams:    TeamsChannel,
    telegram: TelegramChannel,
    webhook:  WebhookChannel,
    fcm:      Option<Arc<FcmPushService>>,
}

impl NotificationService {
    pub fn new(
        smtp:         SmtpConfig,
        sms_cfg:      &SmsConfig,
        template_dao: Arc<NotificationTemplateDao>,
        target_dao:   Arc<NotificationTargetDao>,
        request_dao:  Arc<NotificationRequestDao>,
        delivery_dao: Arc<NotificationDeliveryDao>,
    ) -> Self {
        let client = reqwest::Client::new();
        Self {
            template_dao,
            target_dao,
            request_dao,
            delivery_dao,
            email:    EmailChannel::new(smtp),
            slack:    SlackChannel::new(client.clone()),
            sms:      SmsChannel::new(sms_cfg, client.clone()),
            teams:    TeamsChannel::new(client.clone()),
            telegram: TelegramChannel::new(client.clone()),
            webhook:  WebhookChannel::new(client),
            fcm:      None,
        }
    }

    /// Optionally attach an FCM service for MobilePush channel
    pub fn with_fcm(mut self, fcm: Arc<FcmPushService>) -> Self {
        self.fcm = Some(fcm);
        self
    }

    /// Dispatch a notification request:
    /// 1. Load template + targets
    /// 2. Render templates with context
    /// 3. Deliver per channel + record delivery status
    /// 4. Update request status
    pub async fn dispatch(
        &self,
        request:    NotificationRequest,
        target_ids: &[uuid::Uuid],
    ) -> Result<(), String> {
        let now = chrono::Utc::now().timestamp_millis();

        let template = match self.template_dao.find_by_id(request.template_id).await {
            Ok(Some(t)) => t,
            Ok(None)    => return Err(format!("Template {} not found", request.template_id)),
            Err(e)      => return Err(format!("Template load error: {}", e)),
        };

        let targets = match self.target_dao.find_by_ids(target_ids).await {
            Ok(t)  => t,
            Err(e) => return Err(format!("Targets load error: {}", e)),
        };

        if targets.is_empty() {
            warn!(request_id = %request.id, "No targets for notification request");
        }

        let _ = self.request_dao
            .update_status(request.id, NotificationStatus::Processing, None, None)
            .await;

        let mut errors = Vec::new();
        for target in &targets {
            match self.dispatch_to_target(&template, &request.info, target, now).await {
                Ok(()) => {}
                Err(e) => {
                    warn!(
                        target_id = %target.id,
                        target    = %target.name,
                        error     = %e,
                        "Notification delivery failed"
                    );
                    errors.push(format!("{}: {}", target.name, e));
                }
            }
        }

        let (status, error) = if errors.is_empty() {
            info!(request_id = %request.id, targets = targets.len(), "Notification dispatched");
            (NotificationStatus::Sent, None)
        } else {
            (NotificationStatus::Failed, Some(errors.join("; ")))
        };

        let _ = self.request_dao
            .update_status(request.id, status, error, Some(now))
            .await;

        Ok(())
    }

    async fn dispatch_to_target(
        &self,
        template:    &NotificationTemplate,
        context:     &serde_json::Value,
        target:      &NotificationTarget,
        created_time: i64,
    ) -> Result<(), String> {
        let body    = render_template(&template.body_template, context);
        let subject = template.subject_template.as_deref()
            .map(|s| render_template(s, context))
            .unwrap_or_default();

        let channel_type = format!("{:?}", template.notification_type).to_uppercase();
        let recipient    = self.extract_recipient(template.notification_type.clone(), target);

        let result = match template.notification_type {
            NotificationType::Email => {
                self.dispatch_email(template, &subject, &body, context, target).await
            }
            NotificationType::Slack => {
                self.dispatch_slack(&body, target).await
            }
            NotificationType::MicrosoftTeams => {
                self.dispatch_teams(&subject, &body, target).await
            }
            NotificationType::Webhook => {
                self.dispatch_webhook(&body, context, target).await
            }
            NotificationType::Telegram => {
                self.dispatch_telegram(&body, target).await
            }
            NotificationType::MobilePush => {
                self.dispatch_mobile_push(&subject, &body, target).await
            }
            NotificationType::Sms => {
                self.dispatch_sms(&body, target).await
            }
        };

        let (status, error, sent_at) = match &result {
            Ok(())  => ("SENT", None, Some(chrono::Utc::now().timestamp_millis())),
            Err(e)  => ("FAILED", Some(e.as_str()), None),
        };

        let _ = self.delivery_dao.record(
            target.id,
            &channel_type,
            &recipient,
            status,
            error,
            sent_at,
            created_time,
        ).await;

        result
    }

    fn extract_recipient(
        &self,
        notification_type: NotificationType,
        target: &NotificationTarget,
    ) -> String {
        match notification_type {
            NotificationType::Email =>
                target.target_config["emails"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            NotificationType::Sms =>
                target.target_config["phoneNumber"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
            _ => target.name.clone(),
        }
    }

    async fn dispatch_sms(
        &self,
        body:   &str,
        target: &NotificationTarget,
    ) -> Result<(), String> {
        let phone = target.target_config["phoneNumber"]
            .as_str()
            .ok_or_else(|| "Missing phoneNumber in SMS target config".to_string())?;

        self.sms.send_message(phone, body).await
    }

    async fn dispatch_email(
        &self,
        template: &NotificationTemplate,
        subject:  &str,
        body:     &str,
        _context: &serde_json::Value,
        target:   &NotificationTarget,
    ) -> Result<(), String> {
        let emails: Vec<String> = target.target_config["emails"]
            .as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();

        if emails.is_empty() {
            return Err("No email addresses in target config".into());
        }

        let is_html = template.additional_config.as_ref()
            .and_then(|c| c["isHtml"].as_bool())
            .unwrap_or(false);

        let from = template.additional_config.as_ref()
            .and_then(|c| c["from"].as_str().map(String::from))
            .unwrap_or_else(|| "noreply@vielang.local".into());

        for email_addr in emails {
            let msg = EmailMessage {
                from:    from.clone(),
                to:      email_addr,
                subject: subject.to_string(),
                body:    body.to_string(),
                is_html,
            };
            self.email.send_message(&msg).await?;
        }
        Ok(())
    }

    async fn dispatch_slack(&self, body: &str, target: &NotificationTarget) -> Result<(), String> {
        let webhook_url = target.target_config["webhookUrl"]
            .as_str()
            .ok_or_else(|| "Missing webhookUrl in Slack target config".to_string())?
            .to_string();

        let channel = target.target_config["channel"]
            .as_str()
            .unwrap_or("#general")
            .to_string();

        let msg = SlackMessage { webhook_url, channel, text: body.to_string() };
        self.slack.send_message(&msg).await
    }

    async fn dispatch_teams(
        &self,
        subject: &str,
        body:    &str,
        target:  &NotificationTarget,
    ) -> Result<(), String> {
        let webhook_url = target.target_config["webhookUrl"]
            .as_str()
            .ok_or_else(|| "Missing webhookUrl in Teams target config".to_string())?
            .to_string();

        let title = if subject.is_empty() { "VieLang Notification" } else { subject };
        let msg = TeamsMessage {
            webhook_url,
            title: title.to_string(),
            body: body.to_string(),
        };
        self.teams.send_message(&msg).await
    }

    async fn dispatch_telegram(&self, body: &str, target: &NotificationTarget) -> Result<(), String> {
        let bot_token = target.target_config["botToken"]
            .as_str()
            .ok_or_else(|| "Missing botToken in Telegram target config".to_string())?
            .to_string();

        let chat_id = target.target_config["chatId"]
            .as_str()
            .ok_or_else(|| "Missing chatId in Telegram target config".to_string())?
            .to_string();

        let msg = TelegramMessage { bot_token, chat_id, text: body.to_string() };
        self.telegram.send_message(&msg).await
    }

    async fn dispatch_webhook(
        &self,
        body:    &str,
        context: &serde_json::Value,
        target:  &NotificationTarget,
    ) -> Result<(), String> {
        let url = target.target_config["url"]
            .as_str()
            .ok_or_else(|| "Missing url in Webhook target config".to_string())?
            .to_string();

        let method = target.target_config["method"]
            .as_str()
            .unwrap_or("POST")
            .to_string();

        let headers: HashMap<String, String> = target.target_config["headers"]
            .as_object()
            .map(|obj| obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect())
            .unwrap_or_default();

        let hmac_secret = target.target_config["hmacSecret"]
            .as_str()
            .map(String::from);

        let payload = serde_json::json!({
            "subject": context.get("subject").and_then(|v| v.as_str()).unwrap_or(""),
            "body":    body,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });

        let msg = WebhookMessage { url, method, headers, body: payload, hmac_secret };
        self.webhook.send_message(&msg).await
    }

    async fn dispatch_mobile_push(
        &self,
        subject: &str,
        body:    &str,
        target:  &NotificationTarget,
    ) -> Result<(), String> {
        let fcm = match &self.fcm {
            Some(f) => f,
            None => {
                warn!("MobilePush: FCM not configured");
                return Ok(());
            }
        };

        let tokens: Vec<String> = target.target_config["fcmTokens"]
            .as_array()
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect())
            .unwrap_or_default();

        if tokens.is_empty() {
            warn!("MobilePush: no FCM tokens in target config");
            return Ok(());
        }

        let sent = fcm.send_to_tokens(&tokens, subject, body, None).await;
        info!(sent = sent, total = tokens.len(), "FCM MobilePush sent");
        Ok(())
    }
}
