/// LDAP admin routes — P4
///
/// GET  /api/admin/ldapSettings         — get tenant LDAP config
/// POST /api/admin/ldapSettings         — save tenant LDAP config
/// POST /api/admin/ldapSettings/test    — test LDAP connection

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_auth::ldap::{LdapAuthProvider, LdapConfig};
use vl_dao::{LdapConfigDao, TenantLdapConfig};

use crate::{
    error::ApiError,
    middleware::auth::SecurityContext,
    state::{AppState, CoreState},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/ldapSettings",      get(get_ldap_settings).post(save_ldap_settings))
        .route("/admin/ldapSettings/test", post(test_ldap_connection))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveLdapRequest {
    pub enabled:          bool,
    pub server_url:       String,
    pub use_tls:          bool,
    pub base_dn:          String,
    pub search_filter:    Option<String>,
    pub bind_dn:          String,
    pub bind_password:    String,
    pub username_attr:    Option<String>,
    pub first_name_attr:  Option<String>,
    pub last_name_attr:   Option<String>,
    pub email_attr:       Option<String>,
    pub default_authority: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LdapSettingsResponse {
    pub id:               Uuid,
    pub tenant_id:        Uuid,
    pub enabled:          bool,
    pub server_url:       String,
    pub use_tls:          bool,
    pub base_dn:          String,
    pub search_filter:    String,
    pub bind_dn:          String,
    pub username_attr:    String,
    pub first_name_attr:  String,
    pub last_name_attr:   String,
    pub email_attr:       String,
    pub default_authority: String,
    pub created_time:     i64,
    pub updated_time:     i64,
    // bind_password is intentionally excluded from response
}

impl From<TenantLdapConfig> for LdapSettingsResponse {
    fn from(c: TenantLdapConfig) -> Self {
        Self {
            id:               c.id,
            tenant_id:        c.tenant_id,
            enabled:          c.enabled,
            server_url:       c.server_url,
            use_tls:          c.use_tls,
            base_dn:          c.base_dn,
            search_filter:    c.search_filter,
            bind_dn:          c.bind_dn,
            username_attr:    c.username_attr,
            first_name_attr:  c.first_name_attr,
            last_name_attr:   c.last_name_attr,
            email_attr:       c.email_attr,
            default_authority: c.default_authority,
            created_time:     c.created_time,
            updated_time:     c.updated_time,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestLdapRequest {
    pub server_url:    String,
    pub use_tls:       bool,
    pub bind_dn:       String,
    pub bind_password: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn get_ldap_settings(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = ctx.tenant_id;

    let dao = LdapConfigDao::new(state.pool.clone());
    let cfg = dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    match cfg {
        Some(c) => Ok((StatusCode::OK, Json(LdapSettingsResponse::from(c))).into_response()),
        None    => Err(ApiError::NotFound("LDAP settings not found".into())),
    }
}

pub async fn save_ldap_settings(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveLdapRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = ctx.tenant_id;

    let now = chrono::Utc::now().timestamp_millis();
    let dao = LdapConfigDao::new(state.pool.clone());

    // Preserve existing id if updating
    let existing_id = dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map(|c| c.id)
        .unwrap_or_else(Uuid::new_v4);

    let cfg = TenantLdapConfig {
        id:               existing_id,
        tenant_id,
        enabled:          req.enabled,
        server_url:       req.server_url,
        use_tls:          req.use_tls,
        base_dn:          req.base_dn,
        search_filter:    req.search_filter.unwrap_or_else(|| "(sAMAccountName={username})".into()),
        bind_dn:          req.bind_dn,
        bind_password:    req.bind_password,
        username_attr:    req.username_attr.unwrap_or_else(|| "sAMAccountName".into()),
        first_name_attr:  req.first_name_attr.unwrap_or_else(|| "givenName".into()),
        last_name_attr:   req.last_name_attr.unwrap_or_else(|| "sn".into()),
        email_attr:       req.email_attr.unwrap_or_else(|| "mail".into()),
        default_authority: req.default_authority.unwrap_or_else(|| "TENANT_ADMIN".into()),
        created_time:     now,
        updated_time:     now,
        group_search_base: None,
        group_filter:      None,
    };

    dao.upsert(&cfg).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok((StatusCode::OK, Json(LdapSettingsResponse::from(cfg))).into_response())
}

pub async fn test_ldap_connection(
    Json(req): Json<TestLdapRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let config = LdapConfig {
        server_url:        req.server_url,
        use_tls:           req.use_tls,
        base_dn:           String::new(),
        search_filter:     String::new(),
        bind_dn:           req.bind_dn,
        bind_password:     req.bind_password,
        username_attr:     String::new(),
        first_name_attr:   String::new(),
        last_name_attr:    String::new(),
        email_attr:        String::new(),
        default_authority: String::new(),
        group_search_base: None,
        group_filter:      None,
    };
    let provider = LdapAuthProvider::new(config);
    provider.test_connection().await
        .map_err(|e| ApiError::BadRequest(format!("LDAP test failed: {}", e)))?;

    Ok((StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))).into_response())
}
