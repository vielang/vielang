use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2ClientRegistrationTemplate {
    pub id:                             Uuid,
    pub created_time:                   i64,
    pub additional_info:                Option<serde_json::Value>,
    pub provider_id:                    Option<String>,
    pub name:                           Option<String>,
    pub authorization_uri:              Option<String>,
    pub token_uri:                      Option<String>,
    pub scope:                          Option<String>,
    pub user_info_uri:                  Option<String>,
    pub user_name_attribute_name:       Option<String>,
    pub jwk_set_uri:                    Option<String>,
    pub client_authentication_method:   Option<String>,
    #[serde(rename = "type")]
    pub type_:                          Option<String>,
    pub comment:                        Option<String>,
    pub login_button_icon:              Option<String>,
    pub login_button_label:             Option<String>,
    pub help_link:                      Option<String>,
    pub platforms:                      Option<String>,
}
