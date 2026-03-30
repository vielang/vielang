use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2ClientRegistration {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub provider_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub authorization_uri: String,
    pub token_uri: String,
    pub user_info_uri: String,
    pub scope: Vec<String>,
    pub user_name_attribute: String,
    pub mapper_config: OAuth2MapperConfig,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2MapperConfig {
    pub email_attribute: String,
    #[serde(default)]
    pub first_name_attribute: Option<String>,
    #[serde(default)]
    pub last_name_attribute: Option<String>,
    pub tenant_name_strategy: TenantNameStrategy,
    pub allow_user_creation: bool,
    pub activate_user: bool,
    #[serde(default)]
    pub default_dashboard_id: Option<Uuid>,
}

impl Default for OAuth2MapperConfig {
    fn default() -> Self {
        Self {
            email_attribute: "email".into(),
            first_name_attribute: None,
            last_name_attribute: None,
            tenant_name_strategy: TenantNameStrategy::Basic,
            allow_user_creation: true,
            activate_user: true,
            default_dashboard_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantNameStrategy {
    #[serde(rename = "DOMAIN")]
    Domain,
    #[serde(rename = "CUSTOM")]
    Custom,
    #[serde(rename = "BASIC")]
    Basic,
}
