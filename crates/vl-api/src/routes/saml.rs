/// SAML 2.0 routes — P4
///
/// Public (no auth):
///   GET  /api/noauth/saml/{tenantId}/login     — SP-initiated redirect to IdP
///   POST /api/noauth/saml/{tenantId}/acs       — Assertion Consumer Service (IdP posts here)
///   GET  /api/noauth/saml/{tenantId}/metadata  — SP metadata XML
///
/// Protected (JWT required):
///   GET  /api/admin/samlSettings               — get tenant SAML config
///   POST /api/admin/samlSettings               — save tenant SAML config
///   DELETE /api/admin/samlSettings             — delete tenant SAML config

use axum::{
    extract::{Extension, Path, State, Form},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_auth::saml::{SamlAuthProvider, SamlConfig};
use vl_dao::{SamlConfigDao, TenantSamlConfig};
use vl_core::entities::{Authority, User, UserCredentials};

use crate::{
    error::ApiError,
    middleware::auth::SecurityContext,
    state::{AppState, AuthState, CoreState},
};

// ── Routers ───────────────────────────────────────────────────────────────────

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/noauth/saml/{tenantId}/login",    get(saml_login))
        .route("/noauth/saml/{tenantId}/acs",      post(saml_acs))
        .route("/noauth/saml/{tenantId}/metadata", get(saml_metadata))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/admin/samlSettings",  get(get_saml_settings).post(save_saml_settings))
        .route("/admin/samlSettings",  delete(delete_saml_settings))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSamlRequest {
    pub enabled:          bool,
    pub entity_id:        String,
    pub sso_url:          String,
    pub slo_url:          Option<String>,
    pub idp_certificate:  String,
    pub sp_private_key:   Option<String>,
    pub sp_certificate:   Option<String>,
    pub email_attr:       Option<String>,
    pub first_name_attr:  Option<String>,
    pub last_name_attr:   Option<String>,
    pub force_authn:      Option<bool>,
    pub name_id_format:   Option<String>,
    pub default_authority: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SamlSettingsResponse {
    pub id:               Uuid,
    pub tenant_id:        Uuid,
    pub enabled:          bool,
    pub entity_id:        String,
    pub sso_url:          String,
    pub slo_url:          Option<String>,
    pub idp_certificate:  String,
    pub email_attr:       String,
    pub first_name_attr:  String,
    pub last_name_attr:   String,
    pub force_authn:      bool,
    pub name_id_format:   String,
    pub default_authority: String,
    pub created_time:     i64,
    pub updated_time:     i64,
    // sp_private_key excluded from response
}

impl From<TenantSamlConfig> for SamlSettingsResponse {
    fn from(c: TenantSamlConfig) -> Self {
        Self {
            id:               c.id,
            tenant_id:        c.tenant_id,
            enabled:          c.enabled,
            entity_id:        c.entity_id,
            sso_url:          c.sso_url,
            slo_url:          c.slo_url,
            idp_certificate:  c.idp_certificate,
            email_attr:       c.email_attr,
            first_name_attr:  c.first_name_attr,
            last_name_attr:   c.last_name_attr,
            force_authn:      c.force_authn,
            name_id_format:   c.name_id_format,
            default_authority: c.default_authority,
            created_time:     c.created_time,
            updated_time:     c.updated_time,
        }
    }
}

/// ACS POST form body from IdP
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AcsForm {
    #[serde(rename = "SAMLResponse")]
    saml_response: String,
    #[serde(rename = "RelayState")]
    relay_state: Option<String>,
}

// ── Public handlers ───────────────────────────────────────────────────────────

/// SP-initiated SSO: build AuthnRequest and redirect to IdP.
pub async fn saml_login(
    State(state): State<CoreState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let dao = SamlConfigDao::new(state.pool.clone());
    let cfg = dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Not found".into()))?;

    if !cfg.enabled {
        return Err(ApiError::BadRequest("SAML is not enabled for this tenant".into()));
    }

    let acs_url = format!(
        "{}/api/noauth/saml/{}/acs",
        state.config.server.base_url(),
        tenant_id
    );

    let provider = build_provider(&cfg, &acs_url);
    let relay = tenant_id.to_string();
    let (redirect_url, _request_id) = provider.build_auth_request(&relay)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Redirect::temporary(&redirect_url).into_response())
}

/// ACS: receive SAMLResponse from IdP, validate, provision user, issue JWT.
pub async fn saml_acs(
    State(state): State<CoreState>,
    State(auth): State<AuthState>,
    Path(tenant_id): Path<Uuid>,
    Form(form): Form<AcsForm>,
) -> Result<Response, ApiError> {
    let dao = SamlConfigDao::new(state.pool.clone());
    let cfg = dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Not found".into()))?;

    let acs_url = format!(
        "{}/api/noauth/saml/{}/acs",
        state.config.server.base_url(),
        tenant_id
    );

    let provider = build_provider(&cfg, &acs_url);
    let user_info = provider.process_response(&form.saml_response)
        .map_err(|e| ApiError::Unauthorized(e.to_string()))?;

    // Find or provision user
    let user = auth.user_dao.find_by_email(&user_info.email).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let user = match user {
        Some(u) => u,
        None => {
            // Auto-provision user from SAML assertion (no local password — SSO-only)
            let now = chrono::Utc::now().timestamp_millis();
            let new_user = User {
                id:              Uuid::new_v4(),
                created_time:    now,
                tenant_id,
                customer_id:     None,
                email:           user_info.email.clone(),
                authority:       parse_authority(&cfg.default_authority),
                first_name:      user_info.first_name.clone(),
                last_name:       user_info.last_name.clone(),
                phone:           None,
                additional_info: Some(serde_json::json!({ "ssoEnabled": true })),
                version:         0,
            };
            let saved = auth.user_dao.save(&new_user).await
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            // Create enabled credentials with no password (SSO-only)
            let creds = UserCredentials {
                id:              Uuid::new_v4(),
                created_time:    now,
                user_id:         saved.id,
                enabled:         true,
                password:        None,
                activate_token:  None,
                reset_token:     None,
                additional_info: None,
            };
            auth.user_dao.save_credentials(&creds).await
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            saved
        }
    };

    let authority_str = authority_to_str(&user.authority);
    let jwt_tenant_id = if user.authority == Authority::SysAdmin { None } else { Some(user.tenant_id) };
    let pair = state.jwt_service
        .issue_token(user.id, jwt_tenant_id, user.customer_id, authority_str, vec![authority_str.to_string()])
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Redirect to frontend with token in fragment (ThingsBoard style)
    let frontend_url = state.config.server.base_url();
    let redirect = format!(
        "{}/?accessToken={}&refreshToken={}",
        frontend_url, pair.token, pair.refresh_token
    );

    Ok(Redirect::temporary(&redirect).into_response())
}

/// SP metadata XML — give this URL to IdP administrators.
pub async fn saml_metadata(
    State(state): State<CoreState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    let dao = SamlConfigDao::new(state.pool.clone());
    let cfg = dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Not found".into()))?;

    let acs_url = format!(
        "{}/api/noauth/saml/{}/acs",
        state.config.server.base_url(),
        tenant_id
    );

    let provider = build_provider(&cfg, &acs_url);
    let xml = provider.metadata_xml();

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    ).into_response())
}

// ── Admin handlers ────────────────────────────────────────────────────────────

pub async fn get_saml_settings(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = ctx.tenant_id;

    let dao = SamlConfigDao::new(state.pool.clone());
    let cfg = dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Not found".into()))?;

    Ok((StatusCode::OK, Json(SamlSettingsResponse::from(cfg))).into_response())
}

pub async fn save_saml_settings(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveSamlRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = ctx.tenant_id;

    let now = chrono::Utc::now().timestamp_millis();
    let dao = SamlConfigDao::new(state.pool.clone());

    let existing_id = dao.find_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map(|c| c.id)
        .unwrap_or_else(Uuid::new_v4);

    let cfg = TenantSamlConfig {
        id:               existing_id,
        tenant_id,
        enabled:          req.enabled,
        entity_id:        req.entity_id,
        sso_url:          req.sso_url,
        slo_url:          req.slo_url,
        idp_certificate:  req.idp_certificate,
        sp_private_key:   req.sp_private_key,
        sp_certificate:   req.sp_certificate,
        email_attr:       req.email_attr.unwrap_or_else(|| {
            "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress".into()
        }),
        first_name_attr:  req.first_name_attr.unwrap_or_else(|| {
            "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname".into()
        }),
        last_name_attr:   req.last_name_attr.unwrap_or_else(|| {
            "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname".into()
        }),
        force_authn:      req.force_authn.unwrap_or(false),
        name_id_format:   req.name_id_format.unwrap_or_else(|| {
            "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".into()
        }),
        default_authority: req.default_authority.unwrap_or_else(|| "TENANT_ADMIN".into()),
        created_time:     now,
        updated_time:     now,
    };

    dao.upsert(&cfg).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok((StatusCode::OK, Json(SamlSettingsResponse::from(cfg))).into_response())
}

pub async fn delete_saml_settings(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<impl IntoResponse, ApiError> {
    let tenant_id = ctx.tenant_id;

    let dao = SamlConfigDao::new(state.pool.clone());
    dao.delete_by_tenant(tenant_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(StatusCode::OK.into_response())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_provider(cfg: &TenantSamlConfig, acs_url: &str) -> SamlAuthProvider {
    SamlAuthProvider::new(SamlConfig {
        entity_id:         cfg.entity_id.clone(),
        sso_url:           cfg.sso_url.clone(),
        acs_url:           acs_url.to_string(),
        idp_certificate:   cfg.idp_certificate.clone(),
        email_attr:        cfg.email_attr.clone(),
        first_name_attr:   cfg.first_name_attr.clone(),
        last_name_attr:    cfg.last_name_attr.clone(),
        force_authn:       cfg.force_authn,
        name_id_format:    cfg.name_id_format.clone(),
        default_authority: cfg.default_authority.clone(),
    })
}

fn authority_to_str(a: &Authority) -> &'static str {
    match a {
        Authority::SysAdmin     => "SYS_ADMIN",
        Authority::TenantAdmin  => "TENANT_ADMIN",
        Authority::CustomerUser  => "CUSTOMER_USER",
        _                       => "CUSTOMER_USER",
    }
}

fn parse_authority(s: &str) -> Authority {
    match s {
        "SYS_ADMIN"     => Authority::SysAdmin,
        "TENANT_ADMIN"  => Authority::TenantAdmin,
        _               => Authority::CustomerUser,
    }
}
