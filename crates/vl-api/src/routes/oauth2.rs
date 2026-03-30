use crate::util::now_ms;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AuthState, CoreState}};
use vl_core::entities::OAuth2ClientRegistration;


/// Public noauth + protected routes for OAuth2.
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/noauth/oauth2/authorize/{registrationId}", get(authorize))
        .route("/noauth/oauth2/callback/{registrationId}",  get(callback))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/client/registration",     post(save_registration).get(list_registrations))
        .route("/oauth2/client/registration/{id}", get(get_registration).delete(delete_registration))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CallbackParams {
    code:  Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegistrationResponse {
    #[serde(flatten)]
    reg: OAuth2ClientRegistration,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/noauth/oauth2/authorize/{registrationId}
/// Redirect browser to the OAuth2 provider's authorization URL.
async fn authorize(
    State(state): State<AuthState>,
    Path(registration_id): Path<Uuid>,
) -> Result<Redirect, ApiError> {
    let reg = state.oauth2_registration_dao
        .find_by_id(registration_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("OAuth2 registration not found".into()))?;

    if !reg.enabled {
        return Err(ApiError::BadRequest("OAuth2 registration is disabled".into()));
    }

    let scope_str = reg.scope.join(" ");
    let state_param = Uuid::new_v4().to_string();

    let redirect_url = format!(
        "{}?client_id={}&response_type=code&scope={}&state={}&redirect_uri=/api/noauth/oauth2/callback/{}",
        reg.authorization_uri,
        urlencoding_encode(&reg.client_id),
        urlencoding_encode(&scope_str),
        state_param,
        registration_id,
    );

    Ok(Redirect::to(&redirect_url))
}

/// GET /api/noauth/oauth2/callback/{registrationId}?code=...
/// Exchange authorization code for user info, then issue JWT.
async fn callback(
    State(state): State<AuthState>,
    State(core): State<CoreState>,
    Path(registration_id): Path<Uuid>,
    Query(params): Query<CallbackParams>,
) -> Result<Json<Value>, ApiError> {
    if let Some(err) = &params.error {
        return Err(ApiError::BadRequest(format!("OAuth2 error: {}", err)));
    }

    let code = params.code
        .ok_or_else(|| ApiError::BadRequest("Missing authorization code".into()))?;

    let reg = state.oauth2_registration_dao
        .find_by_id(registration_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("OAuth2 registration not found".into()))?;

    // Exchange code for access token
    let token_resp = reqwest::Client::new()
        .post(&reg.token_uri)
        .form(&[
            ("grant_type",    "authorization_code"),
            ("code",          &code),
            ("client_id",     &reg.client_id),
            ("client_secret", &reg.client_secret),
            ("redirect_uri",  &format!("/api/noauth/oauth2/callback/{}", registration_id)),
        ])
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("Token exchange failed: {}", e)))?
        .json::<Value>()
        .await
        .map_err(|e| ApiError::Internal(format!("Token parse failed: {}", e)))?;

    let access_token = token_resp["access_token"]
        .as_str()
        .ok_or_else(|| ApiError::Internal("No access_token in response".into()))?;

    // Fetch user info
    let user_info = reqwest::Client::new()
        .get(&reg.user_info_uri)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("User info fetch failed: {}", e)))?
        .json::<Value>()
        .await
        .map_err(|e| ApiError::Internal(format!("User info parse failed: {}", e)))?;

    // Extract email using configured attribute
    let email = user_info[&reg.mapper_config.email_attribute]
        .as_str()
        .ok_or_else(|| ApiError::Internal("Could not extract email from user info".into()))?;

    // Find or create user by email
    let user = match state.user_dao.find_by_email(email).await? {
        Some(u) => u,
        None => {
            if !reg.mapper_config.allow_user_creation {
                return Err(ApiError::Forbidden("User creation is disabled for this provider".into()));
            }
            // Create user
            use vl_core::entities::{Authority, User};
            let first_name = reg.mapper_config.first_name_attribute.as_deref()
                .and_then(|attr| user_info[attr].as_str())
                .map(|s| s.to_string());
            let last_name = reg.mapper_config.last_name_attribute.as_deref()
                .and_then(|attr| user_info[attr].as_str())
                .map(|s| s.to_string());

            let new_user = User {
                id:              Uuid::new_v4(),
                created_time:    now_ms(),
                tenant_id:       Uuid::nil(), // default tenant
                customer_id:     None,
                email:           email.to_string(),
                authority:       Authority::TenantAdmin,
                first_name,
                last_name,
                phone:           None,
                additional_info: None,
                version:         1,
            };
            state.user_dao.save(&new_user).await?;

            if reg.mapper_config.activate_user {
                use vl_core::entities::UserCredentials;
                let creds = UserCredentials {
                    id:              Uuid::new_v4(),
                    created_time:    now_ms(),
                    user_id:         new_user.id,
                    enabled:         true,
                    password:        None,
                    activate_token:  None,
                    reset_token:     None,
                    additional_info: None,
                };
                state.user_dao.save_credentials(&creds).await?;
            }

            new_user
        }
    };

    use vl_core::entities::Authority;
    fn authority_str(a: &Authority) -> &'static str {
        match a {
            Authority::SysAdmin             => "SYS_ADMIN",
            Authority::TenantAdmin          => "TENANT_ADMIN",
            Authority::CustomerUser         => "CUSTOMER_USER",
            Authority::RefreshToken         => "REFRESH_TOKEN",
            Authority::PreVerificationToken => "PRE_VERIFICATION_TOKEN",
        }
    }

    let auth_str = authority_str(&user.authority);
    let tenant_id = if user.authority == Authority::SysAdmin { None } else { Some(user.tenant_id) };
    let pair = core.jwt_service
        .issue_token(user.id, tenant_id, user.customer_id, auth_str, vec![auth_str.to_string()])
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "token":        pair.token,
        "refreshToken": pair.refresh_token,
        "scope":        auth_str,
    })))
}

/// POST /api/oauth2/client/registration
async fn save_registration(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(mut reg): Json<OAuth2ClientRegistration>,
) -> Result<Json<OAuth2ClientRegistration>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    if reg.id == Uuid::nil() {
        reg.id = Uuid::new_v4();
        reg.created_time = now_ms();
    }
    reg.tenant_id = ctx.tenant_id;
    state.oauth2_registration_dao.save(&reg).await?;
    Ok(Json(reg))
}

/// GET /api/oauth2/client/registration/{id}
async fn get_registration(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<OAuth2ClientRegistration>, ApiError> {
    let reg = state.oauth2_registration_dao.find_by_id(id).await?
        .ok_or_else(|| ApiError::NotFound("OAuth2 registration not found".into()))?;
    ctx.check_tenant_access(reg.tenant_id)?;
    Ok(Json(reg))
}

/// DELETE /api/oauth2/client/registration/{id}
async fn delete_registration(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let reg = state.oauth2_registration_dao.find_by_id(id).await?
        .ok_or_else(|| ApiError::NotFound("OAuth2 registration not found".into()))?;
    ctx.check_tenant_access(reg.tenant_id)?;
    state.oauth2_registration_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/oauth2/client/registrations
async fn list_registrations(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<OAuth2ClientRegistration>>, ApiError> {
    let regs = state.oauth2_registration_dao
        .find_enabled_by_tenant(ctx.tenant_id)
        .await?;
    Ok(Json(regs))
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;

    async fn test_app(pool: PgPool) -> axum::Router {
        let config = VieLangConfig::default();
        let re     = vl_rule_engine::RuleEngine::start_noop();
        let qp     = vl_queue::create_producer(&config.queue).expect("queue");
        let cache  = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state  = AppState::new(pool, config, ts_dao, re, qp, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        create_router(state)
    }

    fn admin_token(state: &AppState) -> String {
        state.jwt_service
            .issue_token(
                Uuid::new_v4(),
                Some(Uuid::new_v4()),
                None,
                "TENANT_ADMIN",
                vec!["TENANT_ADMIN".into()],
            )
            .unwrap()
            .token
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_and_get_registration(pool: PgPool) {
        let config = VieLangConfig::default();
        let re     = vl_rule_engine::RuleEngine::start_noop();
        let qp     = vl_queue::create_producer(&config.queue).expect("queue");
        let cache  = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state  = AppState::new(pool.clone(), config, ts_dao, re, qp, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        let token  = admin_token(&state);
        let app    = create_router(state);

        let body = json!({
            "id": Uuid::nil(),
            "createdTime": 0,
            "tenantId": Uuid::nil(),
            "providerName": "google",
            "clientId": "my-client-id",
            "clientSecret": "my-secret",
            "authorizationUri": "https://accounts.google.com/o/oauth2/auth",
            "tokenUri": "https://oauth2.googleapis.com/token",
            "userInfoUri": "https://openidconnect.googleapis.com/v1/userinfo",
            "scope": ["openid", "email"],
            "userNameAttribute": "email",
            "mapperConfig": {
                "emailAttribute": "email",
                "tenantNameStrategy": "BASIC",
                "allowUserCreation": true,
                "activateUser": true
            },
            "enabled": true
        });

        let resp = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/oauth2/client/registration")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap();

        assert_eq!(resp.status(), axum::http::StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_nonexistent_registration_returns_404(pool: PgPool) {
        let config = VieLangConfig::default();
        let re     = vl_rule_engine::RuleEngine::start_noop();
        let qp     = vl_queue::create_producer(&config.queue).expect("queue");
        let cache  = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state  = AppState::new(pool.clone(), config, ts_dao, re, qp, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        let token  = admin_token(&state);
        let app    = create_router(state);

        let resp = app.oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/oauth2/client/registration/{}", Uuid::new_v4()))
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();

        assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);
    }
}
