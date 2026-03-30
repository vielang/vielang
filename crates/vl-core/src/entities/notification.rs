use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── NotificationTemplate ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplate {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub name: String,
    pub notification_type: NotificationType,
    pub subject_template: Option<String>,
    pub body_template: String,
    pub additional_config: Option<serde_json::Value>,
    pub enabled: bool,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationType {
    Email,
    Slack,
    MicrosoftTeams,
    Webhook,
    Sms,
    MobilePush,
    Telegram,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email          => "EMAIL",
            Self::Slack          => "SLACK",
            Self::MicrosoftTeams => "MICROSOFT_TEAMS",
            Self::Webhook        => "WEBHOOK",
            Self::Sms            => "SMS",
            Self::MobilePush     => "MOBILE_PUSH",
            Self::Telegram       => "TELEGRAM",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "EMAIL"           => Some(Self::Email),
            "SLACK"           => Some(Self::Slack),
            "MICROSOFT_TEAMS" => Some(Self::MicrosoftTeams),
            "WEBHOOK"         => Some(Self::Webhook),
            "SMS"             => Some(Self::Sms),
            "MOBILE_PUSH"     => Some(Self::MobilePush),
            "TELEGRAM"        => Some(Self::Telegram),
            _                 => None,
        }
    }
}

// ── NotificationTarget ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTarget {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub name: String,
    pub target_type: String,
    pub target_config: serde_json::Value,
    pub version: i64,
}

// ── NotificationRule ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRule {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub name: String,
    pub template_id: Uuid,
    pub trigger_type: TriggerType,
    pub trigger_config: serde_json::Value,
    pub recipients_config: serde_json::Value,
    pub additional_config: Option<serde_json::Value>,
    pub enabled: bool,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TriggerType {
    Alarm,
    DeviceActivity,
    EntityAction,
    RuleEngine,
    ApiUsageLimit,
}

impl TriggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Alarm          => "ALARM",
            Self::DeviceActivity => "DEVICE_ACTIVITY",
            Self::EntityAction   => "ENTITY_ACTION",
            Self::RuleEngine     => "RULE_ENGINE",
            Self::ApiUsageLimit  => "API_USAGE_LIMIT",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ALARM"           => Some(Self::Alarm),
            "DEVICE_ACTIVITY" => Some(Self::DeviceActivity),
            "ENTITY_ACTION"   => Some(Self::EntityAction),
            "RULE_ENGINE"     => Some(Self::RuleEngine),
            "API_USAGE_LIMIT" => Some(Self::ApiUsageLimit),
            _                 => None,
        }
    }
}

// ── NotificationRequest ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRequest {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub rule_id: Option<Uuid>,
    pub template_id: Uuid,
    pub info: serde_json::Value,
    pub status: NotificationStatus,
    pub error: Option<String>,
    pub sent_time: Option<i64>,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationStatus {
    Scheduled,
    Processing,
    Sent,
    Failed,
}

impl NotificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scheduled  => "SCHEDULED",
            Self::Processing => "PROCESSING",
            Self::Sent       => "SENT",
            Self::Failed     => "FAILED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SCHEDULED"  => Some(Self::Scheduled),
            "PROCESSING" => Some(Self::Processing),
            "SENT"       => Some(Self::Sent),
            "FAILED"     => Some(Self::Failed),
            _            => None,
        }
    }
}
