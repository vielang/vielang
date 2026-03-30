use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use vl_core::entities::AdminSettings;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AdminState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/mail/config/template", get(get_mail_config_template)
            .post(save_mail_config_template)
            .delete(delete_mail_config_template))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MailConfigTemplate {
    pub name:            String,
    pub subject:         String,
    pub body:            String,
    pub additional_info: Value,
}

impl Default for MailConfigTemplate {
    fn default() -> Self {
        Self {
            name:            String::new(),
            subject:         String::new(),
            body:            String::new(),
            additional_info: Value::Object(Default::default()),
        }
    }
}

const MAIL_SETTINGS_KEY: &str = "mailServerConfiguration";

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/mail/config/template — get mail config template (SYS_ADMIN only)
async fn get_mail_config_template(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<MailConfigTemplate>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let template = state.admin_settings_dao
        .find_by_key(Uuid::nil(), MAIL_SETTINGS_KEY)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .and_then(|s| serde_json::from_value(s.json_value).ok())
        .unwrap_or_default();
    Ok(Json(template))
}

/// POST /api/mail/config/template — save mail config template (SYS_ADMIN only)
async fn save_mail_config_template(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<MailConfigTemplate>,
) -> Result<Json<MailConfigTemplate>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let json_value = serde_json::to_value(&body)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let now = chrono::Utc::now().timestamp_millis();
    let s = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    Uuid::nil(),
        key:          MAIL_SETTINGS_KEY.into(),
        json_value,
    };
    state.admin_settings_dao.save(&s).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(body))
}

/// DELETE /api/mail/config/template — reset mail config template (SYS_ADMIN only)
async fn delete_mail_config_template(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let s = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    Uuid::nil(),
        key:          MAIL_SETTINGS_KEY.into(),
        json_value:   serde_json::to_value(MailConfigTemplate::default()).unwrap(),
    };
    state.admin_settings_dao.save(&s).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::OK)
}
