use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TwoFactorProvider {
    #[serde(rename = "TOTP")]
    Totp,
    #[serde(rename = "SMS")]
    Sms,
    #[serde(rename = "EMAIL")]
    Email,
}

impl TwoFactorProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Totp  => "TOTP",
            Self::Sms   => "SMS",
            Self::Email => "EMAIL",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "SMS"   => Self::Sms,
            "EMAIL" => Self::Email,
            _       => Self::Totp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorAuthSettings {
    pub user_id:      Uuid,
    pub provider:     TwoFactorProvider,
    pub enabled:      bool,
    pub secret:       String,
    pub backup_codes: Vec<String>,
    pub verified:     bool,
}
