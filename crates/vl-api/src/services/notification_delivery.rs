use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn};
use vl_core::entities::NotificationInbox;
use vl_dao::{MobileSessionDao, NotificationInboxDao};
use crate::notification::channels::NotificationChannel;
use crate::services::fcm_push::FcmPushService;

pub struct NotificationDeliveryService {
    pub inbox_dao:          Arc<NotificationInboxDao>,
    pub mobile_session_dao: Arc<MobileSessionDao>,
    pub fcm:                Option<Arc<FcmPushService>>,
    /// Registered delivery channels (EMAIL, SLACK, SMS, TEAMS, WEBHOOK, TELEGRAM).
    channels:               Vec<Arc<dyn NotificationChannel>>,
}

impl NotificationDeliveryService {
    pub fn new(
        inbox_dao:          Arc<NotificationInboxDao>,
        mobile_session_dao: Arc<MobileSessionDao>,
        fcm:                Option<Arc<FcmPushService>>,
        channels:           Vec<Arc<dyn NotificationChannel>>,
    ) -> Self {
        Self { inbox_dao, mobile_session_dao, fcm, channels }
    }

    /// Deliver a notification to a list of user IDs.
    /// Saves to inbox AND sends FCM push if configured.
    pub async fn deliver(
        &self,
        tenant_id:         Uuid,
        subject:           Option<&str>,
        body:              &str,
        notification_type: Option<&str>,
        severity:          &str,
        recipients:        &[Uuid],
    ) {
        let now = chrono::Utc::now().timestamp_millis();

        // 1. Save to notification inbox
        for &user_id in recipients {
            let inbox = NotificationInbox {
                id:                Uuid::new_v4(),
                tenant_id,
                recipient_user_id: user_id,
                subject:           subject.map(str::to_owned),
                body:              body.to_owned(),
                notification_type: notification_type.map(str::to_owned),
                severity:          severity.to_owned(),
                status:            "SENT".to_owned(),
                sent_time:         now,
                read_time:         None,
                additional_config: serde_json::json!({}),
            };
            if let Err(e) = self.inbox_dao.save(&inbox).await {
                warn!("Failed to save notification inbox for user {user_id}: {e}");
            }
        }

        // 2. Send FCM push if configured
        if let Some(fcm) = &self.fcm {
            match self.mobile_session_dao.find_tokens_for_users(recipients).await {
                Err(e) => warn!("Failed to load FCM tokens: {e}"),
                Ok(tokens) if tokens.is_empty() => {},
                Ok(tokens) => {
                    let title = subject.unwrap_or("VieLang Notification");
                    let sent = fcm.send_to_tokens(&tokens, title, body, None).await;
                    info!("FCM push: {sent}/{} tokens for {notification_type:?}", tokens.len());
                }
            }
        }
    }

    /// Deliver a notification via a specific channel delivery method.
    /// Saves to inbox, sends FCM push, AND routes to the matching channel.
    /// Errors on individual channels are logged but do not fail the whole delivery.
    pub async fn deliver_with_method(
        &self,
        tenant_id:         Uuid,
        subject:           Option<&str>,
        body:              &str,
        notification_type: Option<&str>,
        severity:          &str,
        delivery_method:   &str,
        recipient:         &str,
        recipients:        &[Uuid],
    ) {
        // Save to inbox + FCM as before
        self.deliver(tenant_id, subject, body, notification_type, severity, recipients).await;

        // Route to the matching channel
        let subj = subject.unwrap_or("");
        let method_upper = delivery_method.to_uppercase();

        let mut matched = false;
        for channel in &self.channels {
            if channel.method() == method_upper {
                matched = true;
                if let Err(e) = channel.send(recipient, subj, body).await {
                    warn!(
                        method     = %method_upper,
                        recipient  = %recipient,
                        error      = %e,
                        "Channel delivery failed"
                    );
                }
            }
        }

        if !matched && !method_upper.is_empty() {
            warn!(
                method = %method_upper,
                "No registered channel for delivery method"
            );
        }
    }

    /// Returns the list of registered delivery method names.
    pub fn registered_methods(&self) -> Vec<&'static str> {
        self.channels.iter().map(|c| c.method()).collect()
    }
}
