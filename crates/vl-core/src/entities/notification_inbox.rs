use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInbox {
    pub id:                Uuid,
    pub tenant_id:         Uuid,
    pub recipient_user_id: Uuid,
    pub subject:           Option<String>,
    pub body:              String,
    pub notification_type: Option<String>,
    pub severity:          String,
    pub status:            String,
    pub sent_time:         i64,
    pub read_time:         Option<i64>,
    pub additional_config: serde_json::Value,
}
