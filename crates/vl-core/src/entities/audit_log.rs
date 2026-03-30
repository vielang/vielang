use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditActionType {
    #[serde(rename = "LOGIN")]               Login,
    #[serde(rename = "LOGOUT")]              Logout,
    #[serde(rename = "LOCKOUT")]             Lockout,
    #[serde(rename = "LOGIN_FAILED")]        LoginFailed,
    #[serde(rename = "ADDED")]               Added,
    #[serde(rename = "UPDATED")]             Updated,
    #[serde(rename = "DELETED")]             Deleted,
    #[serde(rename = "CREDENTIALS_UPDATED")] CredentialsUpdated,
    #[serde(rename = "ACTIVATED_BY_USER")]   ActivatedByUser,
    #[serde(rename = "SUSPENDED")]           Suspended,
    #[serde(rename = "CREDENTIALS_READ")]    CredentialsRead,
}

impl AuditActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Login              => "LOGIN",
            Self::Logout             => "LOGOUT",
            Self::Lockout            => "LOCKOUT",
            Self::LoginFailed        => "LOGIN_FAILED",
            Self::Added              => "ADDED",
            Self::Updated            => "UPDATED",
            Self::Deleted            => "DELETED",
            Self::CredentialsUpdated => "CREDENTIALS_UPDATED",
            Self::ActivatedByUser    => "ACTIVATED_BY_USER",
            Self::Suspended          => "SUSPENDED",
            Self::CredentialsRead    => "CREDENTIALS_READ",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "LOGIN"               => Self::Login,
            "LOGOUT"              => Self::Logout,
            "LOCKOUT"             => Self::Lockout,
            "LOGIN_FAILED"        => Self::LoginFailed,
            "ADDED"               => Self::Added,
            "UPDATED"             => Self::Updated,
            "DELETED"             => Self::Deleted,
            "CREDENTIALS_UPDATED" => Self::CredentialsUpdated,
            "ACTIVATED_BY_USER"   => Self::ActivatedByUser,
            "SUSPENDED"           => Self::Suspended,
            _                     => Self::CredentialsRead,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditActionStatus {
    #[serde(rename = "SUCCESS")] Success,
    #[serde(rename = "FAILURE")] Failure,
}

impl AuditActionStatus {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Success => "SUCCESS", Self::Failure => "FAILURE" }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    pub id:                     Uuid,
    pub created_time:           i64,
    pub tenant_id:              Uuid,
    pub user_id:                Option<Uuid>,
    pub user_name:              Option<String>,
    pub action_type:            AuditActionType,
    pub action_data:            serde_json::Value,
    pub action_status:          AuditActionStatus,
    pub action_failure_details: Option<String>,
    pub entity_type:            Option<String>,
    pub entity_id:              Option<Uuid>,
    pub entity_name:            Option<String>,
}
