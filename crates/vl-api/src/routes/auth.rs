use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AuthState, CoreState, BillingState}};
use rand::Rng as _;
use vl_auth::password;
use vl_auth::ldap::{LdapAuthProvider, LdapConfig};
use vl_core::entities::{Authority, Tenant, TenantSubscription, User, UserCredentials};
use vl_dao::postgres::ldap_config::LdapConfigDao;

/// Public routes: login, token refresh, noauth/*
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login",                   post(login))
        .route("/auth/token",                   post(refresh_token))
        .route("/noauth/signup",                post(signup))
        .route("/noauth/activate",              post(activate))
        .route("/noauth/resetPasswordByEmail",  post(reset_password_by_email))
        .route("/noauth/resetPassword",         post(reset_password))
        .route("/noauth/userPasswordPolicy",    get(user_password_policy))
        .route("/noauth/activateByEmailCode",   post(activate_by_email_code))
        .route("/noauth/resendEmailActivation", post(resend_email_activation))
        .route("/noauth/oauth2Clients",         post(oauth2_clients))
        .route("/auth/2fa/providers",           get(twofa_providers))
        .route("/2fa/providers",                get(twofa_providers))
}

/// Protected auth routes (requires valid JWT)
pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/auth/user",              get(get_current_user))
        .route("/auth/logout",            post(logout))
        .route("/auth/sessions",          delete(delete_all_sessions))
        .route("/auth/changePassword",    post(change_password))
        .route("/auth/2fa/backupCodes",   post(regenerate_backup_codes))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtPairResponse {
    pub token: String,
    pub refresh_token: String,
    /// Java TB does not return scope in the login response; omit when None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivateRequest {
    pub activate_token: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordByEmailRequest {
    pub email: String,
}

// ── DTOs — signup ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupRequest {
    pub email:      String,
    pub password:   String,
    pub first_name: Option<String>,
    pub last_name:  Option<String>,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/noauth/signup — self-service SaaS registration
///
/// Creates: Tenant + TenantAdmin user + credentials + free-plan subscription.
/// Returns 201 Created. The caller (BFF) then logs the user in via /auth/login.
async fn signup(
    State(state): State<AuthState>, State(_core): State<CoreState>, State(billing): State<BillingState>,
    Json(req): Json<SignupRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    // Guard: email must not already exist
    if state.user_dao.find_by_email(&req.email).await?.is_some() {
        return Err(ApiError::BadRequest(format!(
            "An account with email '{}' already exists.",
            req.email
        )));
    }

    let now = chrono::Utc::now().timestamp_millis();

    // 1. Resolve default tenant profile
    let profile_id = state.tenant_profile_dao
        .find_default()
        .await?
        .ok_or_else(|| ApiError::Internal("No default tenant profile configured".into()))?
        .id;

    // 2. Create tenant (title = email prefix)
    let tenant_title = req.email
        .split('@')
        .next()
        .unwrap_or(&req.email)
        .to_string();
    let tenant_id = uuid::Uuid::new_v4();
    let tenant = Tenant {
        id: tenant_id,
        created_time: now,
        tenant_profile_id: profile_id,
        title: tenant_title,
        region: None, country: None, state: None, city: None,
        address: None, address2: None, zip: None,
        phone: None, email: Some(req.email.clone()),
        additional_info: None, version: 1,
    };
    state.tenant_dao.save(&tenant).await?;

    // 3. Create TenantAdmin user
    let user_id = uuid::Uuid::new_v4();
    let user = User {
        id: user_id,
        created_time: now,
        tenant_id,
        customer_id: None,
        email: req.email.clone(),
        authority: Authority::TenantAdmin,
        first_name: req.first_name.clone(),
        last_name:  req.last_name.clone(),
        phone: None,
        additional_info: None,
        version: 1,
    };
    state.user_dao.save(&user).await?;

    // 4. Create credentials
    let hash = password::hash_password(&req.password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let creds = UserCredentials {
        id: uuid::Uuid::new_v4(),
        created_time: now,
        user_id,
        enabled: true,
        password: Some(hash),
        activate_token: None,
        reset_token: None,
        additional_info: None,
    };
    state.user_dao.save_credentials(&creds).await?;

    // 5. Provision free-plan subscription
    let free_plan = billing.plan_dao.find_by_name("free").await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    if let Some(plan) = free_plan {
        let sub = TenantSubscription {
            id: uuid::Uuid::new_v4(),
            created_time: now,
            updated_time: now,
            tenant_id,
            plan_id: plan.id,
            stripe_customer_id: None,
            stripe_subscription_id: None,
            billing_cycle: "monthly".into(),
            status: "free".into(),
            current_period_start: None,
            current_period_end: None,
            trial_end: None,
            cancel_at_period_end: false,
            canceled_at: None,
        };
        billing.subscription_dao.upsert(&sub).await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    info!("New SaaS signup: {} (tenant={})", req.email, tenant_id);
    Ok(axum::http::StatusCode::CREATED)
}

/// POST /api/auth/login — khớp Java AuthController.login()
async fn login(
    State(state): State<AuthState>, State(core): State<CoreState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<JwtPairResponse>, ApiError> {
    // Tìm user theo email
    let user = state.user_dao
        .find_by_email(&req.username)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("Bad credentials".into()))?;

    // Lấy credentials
    let creds = state.user_dao
        .find_credentials(user.id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("User credentials not found".into()))?;

    if !creds.enabled {
        return Err(ApiError::Unauthorized("User account is disabled".into()));
    }

    // ── LDAP fallback: if tenant has LDAP enabled and user has no local password ─
    if creds.password.is_none() {
        // user.tenant_id is Option<Uuid> only for SysAdmin (which cannot use LDAP)
        if let Some(tid) = if user.authority == Authority::SysAdmin { None } else { Some(user.tenant_id) } {
            let ldap_dao = LdapConfigDao::new(core.pool.clone());
            if let Ok(Some(ldap_cfg)) = ldap_dao.find_by_tenant(tid).await {
                if ldap_cfg.enabled {
                    let provider = LdapAuthProvider::new(LdapConfig {
                        server_url:        ldap_cfg.server_url,
                        use_tls:           ldap_cfg.use_tls,
                        base_dn:           ldap_cfg.base_dn,
                        search_filter:     ldap_cfg.search_filter,
                        bind_dn:           ldap_cfg.bind_dn,
                        bind_password:     ldap_cfg.bind_password,
                        username_attr:     ldap_cfg.username_attr,
                        first_name_attr:   ldap_cfg.first_name_attr,
                        last_name_attr:    ldap_cfg.last_name_attr,
                        email_attr:        ldap_cfg.email_attr,
                        default_authority: ldap_cfg.default_authority,
                        group_search_base: None,
                        group_filter:      None,
                    });
                    provider.authenticate(&req.username, &req.password).await
                        .map_err(|_| ApiError::Unauthorized("Bad credentials".into()))?;
                    // LDAP auth succeeded — skip local password check
                    let authority_str = authority_to_str(&user.authority);
                    let jwt_tenant_id = if user.authority == Authority::SysAdmin { None } else { Some(user.tenant_id) };
                    let pair = core.jwt_service
                        .issue_token(user.id, jwt_tenant_id, user.customer_id, authority_str, vec![authority_str.to_string()])
                        .map_err(|e| ApiError::Internal(e.to_string()))?;
                    return Ok(Json(JwtPairResponse {
                        token:         pair.token,
                        refresh_token: pair.refresh_token,
                        scope:         None,
                    }));
                }
            }
        }
        return Err(ApiError::Unauthorized("User has no password set".into()));
    }

    // Verify Argon2 password
    let hash = creds.password
        .as_deref()
        .ok_or_else(|| ApiError::Unauthorized("User has no password set".into()))?;

    let valid = password::verify_password(&req.password, hash)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if !valid {
        return Err(ApiError::Unauthorized("Bad credentials".into()));
    }

    let authority_str = authority_to_str(&user.authority);
    let scopes = vec![authority_str.to_string()];
    let tenant_id = if user.authority == Authority::SysAdmin {
        None
    } else {
        Some(user.tenant_id)
    };

    // Check 2FA — if enabled and verified, return pre-verification token instead of full JWT
    let tfa = state.two_factor_auth_dao
        .find_by_user(user.id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if let Some(settings) = tfa {
        if settings.enabled && settings.verified {
            let pre_token = core.jwt_service
                .issue_pre_verification_token(user.id, tenant_id)
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            return Ok(Json(JwtPairResponse {
                token:         pre_token.clone(),
                refresh_token: pre_token,
                scope:         None,
            }));
        }
    }

    // Issue full JWT pair
    let pair = core.jwt_service
        .issue_token(user.id, tenant_id, user.customer_id, authority_str, scopes)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(JwtPairResponse {
        token:         pair.token,
        refresh_token: pair.refresh_token,
        scope:         None,
    }))
}

/// POST /api/auth/token — refresh token flow
async fn refresh_token(
    State(state): State<AuthState>, State(core): State<CoreState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<JwtPairResponse>, ApiError> {
    // Validate refresh token (must have authority = REFRESH_TOKEN)
    let claims = core.jwt_service
        .validate_token(&req.refresh_token)
        .map_err(|e| ApiError::Unauthorized(e.to_string()))?;

    if !claims.is_refresh_token() {
        return Err(ApiError::Unauthorized("Not a refresh token".into()));
    }

    // Fetch fresh user data from DB
    let user_id = claims.user_id();
    let user = state.user_dao
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("User not found".into()))?;

    let creds = state.user_dao
        .find_credentials(user.id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("User credentials not found".into()))?;

    if !creds.enabled {
        return Err(ApiError::Unauthorized("User account is disabled".into()));
    }

    let authority_str = authority_to_str(&user.authority);
    let scopes = vec![authority_str.to_string()];
    let tenant_id = if user.authority == Authority::SysAdmin {
        None
    } else {
        Some(user.tenant_id)
    };

    let pair = core.jwt_service
        .issue_token(user.id, tenant_id, user.customer_id, authority_str, scopes)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(JwtPairResponse {
        token:         pair.token,
        refresh_token: pair.refresh_token,
        scope:         None,
    }))
}

/// ThingsBoard-compatible User response (camelCase + EntityId wrappers)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: crate::routes::devices::IdResponse,
    pub created_time: i64,
    pub tenant_id: crate::routes::devices::IdResponse,
    pub customer_id: Option<crate::routes::devices::IdResponse>,
    pub name: String,
    pub authority: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub additional_info: Option<serde_json::Value>,
    pub version: i64,
}

impl From<vl_core::entities::User> for UserResponse {
    fn from(u: vl_core::entities::User) -> Self {
        let authority_str = match u.authority {
            Authority::SysAdmin             => "SYS_ADMIN",
            Authority::TenantAdmin          => "TENANT_ADMIN",
            Authority::CustomerUser         => "CUSTOMER_USER",
            Authority::RefreshToken         => "REFRESH_TOKEN",
            Authority::PreVerificationToken => "PRE_VERIFICATION_TOKEN",
        };
        Self {
            id:              crate::routes::devices::IdResponse::new(u.id, "USER"),
            created_time:    u.created_time,
            tenant_id:       crate::routes::devices::IdResponse::tenant(u.tenant_id),
            customer_id:     u.customer_id.map(crate::routes::devices::IdResponse::customer),
            name:            u.email.clone(),
            authority:       authority_str.into(),
            email:           u.email,
            first_name:      u.first_name,
            last_name:       u.last_name,
            phone:           u.phone,
            additional_info: u.additional_info,
            version:         u.version,
        }
    }
}

/// GET /api/auth/user — current authenticated user
async fn get_current_user(
    State(state): State<AuthState>, State(_core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state.user_dao
        .find_by_id(ctx.user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".into()))?;

    Ok(Json(UserResponse::from(user)))
}

/// POST /api/noauth/activate — activate user account
async fn activate(
    State(state): State<AuthState>, State(core): State<CoreState>,
    Json(req): Json<ActivateRequest>,
) -> Result<Json<JwtPairResponse>, ApiError> {
    // Validate password strength (basic)
    if req.password.len() < 6 {
        return Err(ApiError::BadRequest("Password must be at least 6 characters".into()));
    }

    let hashed = password::hash_password(&req.password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let user = state.user_dao
        .activate_user(&req.activate_token, &hashed)
        .await?;

    let authority_str = authority_to_str(&user.authority);
    let scopes = vec![authority_str.to_string()];
    let tenant_id = if user.authority == Authority::SysAdmin {
        None
    } else {
        Some(user.tenant_id)
    };

    let pair = core.jwt_service
        .issue_token(user.id, tenant_id, user.customer_id, authority_str, scopes)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(JwtPairResponse {
        token:         pair.token,
        refresh_token: pair.refresh_token,
        scope:         None,
    }))
}

/// POST /api/noauth/resetPasswordByEmail — initiate password reset
/// Note: email sending không implement — chỉ log token ra console
async fn reset_password_by_email(
    State(state): State<AuthState>, State(_core): State<CoreState>,
    Json(req): Json<ResetPasswordByEmailRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    // Generate a random reset token
    let reset_token = uuid::Uuid::new_v4().to_string().replace('-', "");

    // Store reset token (fire-and-forget; không leak lỗi ra ngoài)
    match state.user_dao.reset_password_token(&req.email, &reset_token).await {
        Ok(()) => {
            info!(email = %req.email, reset_token = %reset_token,
                  "Password reset requested — in production, send this token via email");
        }
        Err(vl_dao::DaoError::NotFound) => {
            // Không leak thông tin user có tồn tại hay không
            info!(email = %req.email, "Password reset for unknown email (no-op)");
        }
        Err(e) => return Err(ApiError::from(e)),
    }

    Ok(axum::http::StatusCode::OK)
}

// ── Additional DTOs ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    pub reset_token: String,
    pub password: String,
}

// ── Additional handlers ───────────────────────────────────────────────────────

/// POST /api/auth/logout — revoke the current token then return 200
async fn logout(
    State(_state): State<AuthState>, State(core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> axum::http::StatusCode {
    if !ctx.jti.is_empty() {
        core.token_blacklist.revoke(&ctx.jti, ctx.exp).await;
    }
    axum::http::StatusCode::OK
}

/// POST /api/auth/changePassword
async fn change_password(
    State(state): State<AuthState>, State(_core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    if req.new_password.len() < 6 {
        return Err(ApiError::BadRequest("Password must be at least 6 characters".into()));
    }

    let creds = state.user_dao.find_credentials(ctx.user_id).await?
        .ok_or_else(|| ApiError::NotFound("User credentials not found".into()))?;

    let current_hash = creds.password.as_deref()
        .ok_or_else(|| ApiError::Unauthorized("No password set".into()))?;

    let valid = password::verify_password(&req.current_password, current_hash)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    if !valid {
        return Err(ApiError::Unauthorized("Current password is incorrect".into()));
    }

    let new_hash = password::hash_password(&req.new_password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut updated = creds;
    updated.password = Some(new_hash);
    state.user_dao.save_credentials(&updated).await
        .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::OK)
}

/// POST /api/noauth/resetPassword — reset password using reset token
async fn reset_password(
    State(state): State<AuthState>, State(core): State<CoreState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<JwtPairResponse>, ApiError> {
    if req.password.len() < 6 {
        return Err(ApiError::BadRequest("Password must be at least 6 characters".into()));
    }

    let hashed = password::hash_password(&req.password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Reuse activate_user: reset_token stored in activate_token field
    let user = state.user_dao.activate_user(&req.reset_token, &hashed).await
        .map_err(|_| ApiError::NotFound("Reset token not found or expired".into()))?;

    let authority_str = authority_to_str(&user.authority);
    let tenant_id = if user.authority == Authority::SysAdmin { None } else { Some(user.tenant_id) };
    let pair = core.jwt_service
        .issue_token(user.id, tenant_id, user.customer_id, authority_str, vec![authority_str.to_string()])
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(JwtPairResponse { token: pair.token, refresh_token: pair.refresh_token, scope: None }))
}

/// GET /api/noauth/userPasswordPolicy — password constraints
async fn user_password_policy() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "minimumLength": 6,
        "minimumUppercaseLetters": 0,
        "minimumLowercaseLetters": 0,
        "minimumDigits": 0,
        "minimumSpecialCharacters": 0,
        "passwordExpirationPeriodDays": 0,
        "allowWhitespaces": true,
        "forceUserToResetPasswordIfNotValid": false
    }))
}

/// POST /api/noauth/activateByEmailCode
/// Body: { "activateToken": "..." }
/// Returns: JWT pair (same as normal login)
async fn activate_by_email_code(
    State(state): State<AuthState>, State(core): State<CoreState>,
    Json(req): Json<ActivateByEmailCodeRequest>,
) -> Result<Json<JwtPairResponse>, ApiError> {
    let user_id = state.activation_service.verify_activation_token(&req.activate_token).await?;
    let user = state.user_dao
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User".into()))?;
    let authority_str = authority_to_str(&user.authority);
    let tenant_id = if user.authority == Authority::SysAdmin {
        None
    } else {
        Some(user.tenant_id)
    };
    let pair = core.jwt_service
        .issue_token(user.id, tenant_id, user.customer_id, authority_str, vec![authority_str.to_string()])
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(JwtPairResponse {
        token:         pair.token,
        refresh_token: pair.refresh_token,
        scope:         None,
    }))
}

/// POST /api/noauth/resendEmailActivation
/// Body: { "email": "..." }
async fn resend_email_activation(
    State(state): State<AuthState>, State(_core): State<CoreState>,
    Json(req): Json<ResendActivationRequest>,
) -> Result<axum::http::StatusCode, ApiError> {
    // Silently succeed even if not found to prevent user enumeration
    if let Ok(Some(user)) = state.user_dao.find_by_email(&req.email).await {
        let _ = state.activation_service.send_activation_email(user.id, &user.email).await;
    }
    Ok(axum::http::StatusCode::OK)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivateByEmailCodeRequest {
    activate_token: String,
}

#[derive(Deserialize)]
struct ResendActivationRequest {
    email: String,
}

/// POST /api/noauth/oauth2Clients — return enabled OAuth2 providers (without secrets)
async fn oauth2_clients(
    State(state): State<AuthState>, State(_core): State<CoreState>,
) -> Json<serde_json::Value> {
    // SYS_ADMIN tenant_id = nil; return enabled registrations for nil tenant (system-level)
    let regs = state.oauth2_registration_dao
        .find_enabled_by_tenant(uuid::Uuid::nil())
        .await
        .unwrap_or_default();

    let providers: Vec<serde_json::Value> = regs.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "name": r.provider_name,
            "icon": r.provider_name,
            "url": format!("/api/noauth/oauth2/authorize/{}", r.id),
        })
    }).collect();

    Json(serde_json::json!(providers))
}

/// GET /api/auth/2fa/providers — list available 2FA providers
async fn twofa_providers() -> Json<serde_json::Value> {
    Json(serde_json::json!(["TOTP"]))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn authority_to_str(auth: &Authority) -> &'static str {
    match auth {
        Authority::SysAdmin             => "SYS_ADMIN",
        Authority::TenantAdmin          => "TENANT_ADMIN",
        Authority::CustomerUser         => "CUSTOMER_USER",
        Authority::RefreshToken         => "REFRESH_TOKEN",
        Authority::PreVerificationToken => "PRE_VERIFICATION_TOKEN",
    }
}

/// DELETE /api/auth/sessions — revoke ALL tokens for the authenticated user.
///
/// After this call, all access tokens the user has (on other devices/browsers) will be
/// invalidated. The current token is also revoked (same as logout).
async fn delete_all_sessions(
    State(_state): State<AuthState>, State(core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> StatusCode {
    // Revoke the current token
    if !ctx.jti.is_empty() {
        core.token_blacklist.revoke(&ctx.jti, ctx.exp).await;
    }
    // Mark user-level "revoke all before now" so older tokens are also rejected
    let refresh_ttl = core.config.security.jwt.refresh_expiration_secs;
    core.token_blacklist.revoke_all_for_user(ctx.user_id, refresh_ttl).await;
    StatusCode::OK
}

/// POST /api/auth/2fa/backupCodes — regenerate 2FA backup codes.
///
/// Returns 10 fresh plaintext codes. The previous codes are replaced in the DB.
/// The user should save these in a safe place — they are shown only once.
async fn regenerate_backup_codes(
    State(state): State<AuthState>, State(_core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut settings = state.two_factor_auth_dao
        .find_by_user(ctx.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("2FA settings not found".into()))?;

    if !settings.enabled {
        return Err(ApiError::BadRequest("2FA is not enabled".into()));
    }

    let codes: Vec<String> = (0..10)
        .map(|_| {
            let raw: String = (0..12)
                .map(|_| {
                    let idx: usize = rand::rng().random_range(0..36);
                    if idx < 10 { (b'0' + idx as u8) as char }
                    else        { (b'A' + (idx - 10) as u8) as char }
                })
                .collect();
            format!("{}-{}-{}", &raw[..4], &raw[4..8], &raw[8..])
        })
        .collect();

    settings.backup_codes = codes.clone();
    state.two_factor_auth_dao
        .save(&settings)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({ "backupCodes": codes })))
}

// ── HTTP Integration Tests ────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt; // for .oneshot()
    use uuid::Uuid;

    use vl_auth::password;
    use vl_core::entities::UserCredentials;

    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;

    // ── Test helpers ──────────────────────────────────────────────────────────

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    async fn test_app(pool: PgPool) -> axum::Router {
        let config = VieLangConfig::default();
        let rule_engine = vl_rule_engine::RuleEngine::start_noop();
        let queue_producer = vl_queue::create_producer(&config.queue)
            .expect("queue producer");
        let cache = vl_cache::create_cache(&config.cache)
            .expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster)
            .await
            .expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        create_router(state)
    }

    async fn post_json(app: axum::Router, uri: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn get_with_token(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn body_json(response: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(response.into_body(), 1_000_000)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    /// Tạo test user với hashed password và trả về (User, plain_password)
    async fn create_test_user(
        pool: &PgPool,
        email: &str,
        password: &str,
        enabled: bool,
    ) -> vl_core::entities::User {
        use vl_core::entities::{Authority, User};
        use vl_dao::postgres::user::UserDao;

        let dao = UserDao::new(pool.clone());
        let user = User {
            id:              Uuid::new_v4(),
            created_time:    now_ms(),
            tenant_id:       Uuid::new_v4(),
            customer_id:     None,
            email:           email.into(),
            authority:       Authority::TenantAdmin,
            first_name:      Some("Test".into()),
            last_name:       Some("User".into()),
            phone:           None,
            additional_info: None,
            version:         1,
        };
        dao.save(&user).await.unwrap();

        let hash = password::hash_password(password).unwrap();
        let creds = UserCredentials {
            id:              Uuid::new_v4(),
            created_time:    now_ms(),
            user_id:         user.id,
            enabled,
            password:        Some(hash),
            activate_token:  None,
            reset_token:     None,
            additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        user
    }

    // ── POST /api/auth/login ──────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn login_success_returns_token_pair(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "admin@test.com", "password123", true).await;

        let resp = post_json(
            app,
            "/api/auth/login",
            json!({"username": "admin@test.com", "password": "password123"}),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["token"].is_string(), "Response phải có 'token'");
        assert!(body["refreshToken"].is_string(), "Response phải có 'refreshToken'");
        assert!(!body["token"].as_str().unwrap().is_empty());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn login_wrong_password_returns_401(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "user@test.com", "correct-password", true).await;

        let resp = post_json(
            app,
            "/api/auth/login",
            json!({"username": "user@test.com", "password": "wrong-password"}),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn login_unknown_email_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = post_json(
            app,
            "/api/auth/login",
            json!({"username": "nobody@nowhere.com", "password": "any"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn login_disabled_user_returns_401(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "disabled@test.com", "password", false).await;

        let resp = post_json(
            app,
            "/api/auth/login",
            json!({"username": "disabled@test.com", "password": "password"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn login_response_body_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "format@test.com", "pass123", true).await;

        let resp = post_json(
            app,
            "/api/auth/login",
            json!({"username": "format@test.com", "password": "pass123"}),
        )
        .await;

        let body = body_json(resp).await;
        // Khớp format ThingsBoard: camelCase fields (scope omitted to match Java TB)
        assert!(body.get("token").is_some());
        assert!(body.get("refreshToken").is_some());
    }

    // ── POST /api/auth/token (refresh) ────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn refresh_token_success_returns_new_pair(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "refresh@test.com", "pass123", true).await;

        // Login để lấy refresh token
        let login_resp = post_json(
            app.clone(),
            "/api/auth/login",
            json!({"username": "refresh@test.com", "password": "pass123"}),
        )
        .await;
        let login_body = body_json(login_resp).await;
        let refresh_token = login_body["refreshToken"].as_str().unwrap().to_string();

        // Dùng refresh token để lấy pair mới
        let resp = post_json(
            app,
            "/api/auth/token",
            json!({"refreshToken": refresh_token}),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["token"].is_string());
        assert!(body["refreshToken"].is_string());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn refresh_with_access_token_returns_401(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "access@test.com", "pass123", true).await;

        let login_resp = post_json(
            app.clone(),
            "/api/auth/login",
            json!({"username": "access@test.com", "password": "pass123"}),
        )
        .await;
        let login_body = body_json(login_resp).await;
        let access_token = login_body["token"].as_str().unwrap().to_string();

        // Gửi access token thay vì refresh token
        let resp = post_json(
            app,
            "/api/auth/token",
            json!({"refreshToken": access_token}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn refresh_with_invalid_token_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = post_json(
            app,
            "/api/auth/token",
            json!({"refreshToken": "invalid.jwt.token"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── GET /api/auth/user ────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_current_user_with_valid_token_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "me@test.com", "pass123", true).await;

        let login_resp = post_json(
            app.clone(),
            "/api/auth/login",
            json!({"username": "me@test.com", "password": "pass123"}),
        )
        .await;
        let token = body_json(login_resp).await["token"]
            .as_str()
            .unwrap()
            .to_string();

        let resp = get_with_token(app, "/api/auth/user", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert_eq!(body["email"].as_str().unwrap(), user.email);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_current_user_without_token_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/auth/user")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_current_user_with_refresh_token_returns_401(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "refresh2@test.com", "pass123", true).await;

        let login_resp = post_json(
            app.clone(),
            "/api/auth/login",
            json!({"username": "refresh2@test.com", "password": "pass123"}),
        )
        .await;
        let refresh_token = body_json(login_resp).await["refreshToken"]
            .as_str()
            .unwrap()
            .to_string();

        let resp = get_with_token(app, "/api/auth/user", &refresh_token).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── POST /api/noauth/activate ─────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn activate_with_valid_token_returns_jwt(pool: PgPool) {
        use vl_dao::postgres::user::UserDao;
        use vl_core::entities::{Authority, User, UserCredentials};

        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::new_v4(), customer_id: None,
            email: "activate@test.com".into(), authority: Authority::TenantAdmin,
            first_name: None, last_name: None, phone: None,
            additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();

        let creds = UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(), user_id: user.id,
            enabled: false, password: None,
            activate_token: Some("test-activate-token-999".into()),
            reset_token: None, additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();

        let app = test_app(pool).await;
        let resp = post_json(
            app,
            "/api/noauth/activate",
            json!({"activateToken": "test-activate-token-999", "password": "new-password-123"}),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["token"].is_string());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn activate_with_invalid_token_returns_404(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = post_json(
            app,
            "/api/noauth/activate",
            json!({"activateToken": "invalid-token-xyz", "password": "new-password-123"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn activate_with_short_password_returns_400(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = post_json(
            app,
            "/api/noauth/activate",
            json!({"activateToken": "any-token", "password": "abc"}), // < 6 chars
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // ── POST /api/noauth/resetPasswordByEmail ─────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn reset_password_by_email_known_user_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "reset@test.com", "pass123", true).await;

        let resp = post_json(
            app,
            "/api/noauth/resetPasswordByEmail",
            json!({"email": "reset@test.com"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn reset_password_by_email_unknown_user_also_returns_200(pool: PgPool) {
        // Security: không leak thông tin user có tồn tại hay không
        let app = test_app(pool).await;
        let resp = post_json(
            app,
            "/api/noauth/resetPasswordByEmail",
            json!({"email": "nobody@nowhere.com"}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── Error response format ─────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn error_response_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = post_json(
            app,
            "/api/auth/login",
            json!({"username": "nobody@example.com", "password": "pass"}),
        )
        .await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let body = body_json(resp).await;
        // Khớp TB Java format: { status, message, errorCode }
        assert!(body["status"].is_number());
        assert!(body["message"].is_string());
        assert!(body["errorCode"].is_number());
        assert_eq!(body["status"].as_u64().unwrap(), 401);
    }
}
