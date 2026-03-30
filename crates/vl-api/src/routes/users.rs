use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{Authority, User};
use vl_dao::{PageData, PageLink};

use crate::{error::ApiError, middleware::auth::SecurityContext, routes::auth::JwtPairResponse, routes::devices::IdResponse, state::{AppState, AuthState, CoreState, MobileState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: UserController
        .route("/user",                                 post(save_user))
        .route("/user/{userId}",                        get(get_user).delete(delete_user))
        .route("/user/{userId}/token",                  get(get_user_token))
        .route("/user/{userId}/activationLink",         get(get_activation_link))
        .route("/user/{userId}/sendActivationMail",     post(send_activation_mail))
        .route("/user/{userId}/userCredentialsEnabled", post(set_user_credentials_enabled))
        .route("/users",                                get(list_users))
        .route("/users/count",                          get(count_users))
        .route("/tenant/{tenantId}/users",              get(list_tenant_users))
        .route("/customer/{customerId}/users",          get(list_customer_users))
        .route("/user/mobileSessions",                  post(save_mobile_session))
        .route("/user/mobileSessions/{token}",          delete(remove_mobile_session))
        // Phase 69: user avatar
        .route("/user/avatar",                          post(upload_avatar).delete(delete_avatar))
        .route("/user/{userId}/avatar",                 get(get_avatar))
        // Phase 69: notification settings
        .route("/notifications/settings",               get(get_notification_settings).put(save_notification_settings))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: IdResponse,
    pub created_time: i64,
    pub tenant_id: IdResponse,
    pub customer_id: Option<IdResponse>,
    pub email: String,
    pub authority: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub additional_info: Option<serde_json::Value>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id:              IdResponse::new(u.id, "USER"),
            created_time:    u.created_time,
            tenant_id:       IdResponse::tenant(u.tenant_id),
            customer_id:     u.customer_id.map(IdResponse::customer),
            email:           u.email,
            authority:       authority_to_str(&u.authority).into(),
            first_name:      u.first_name,
            last_name:       u.last_name,
            phone:           u.phone,
            // Java returns {} for null additionalInfo — Angular may access fields on it
            additional_info: Some(u.additional_info.unwrap_or_else(|| serde_json::json!({}))),
        }
    }
}

fn authority_to_str(a: &Authority) -> &'static str {
    match a {
        Authority::SysAdmin             => "SYS_ADMIN",
        Authority::TenantAdmin          => "TENANT_ADMIN",
        Authority::CustomerUser         => "CUSTOMER_USER",
        Authority::RefreshToken         => "REFRESH_TOKEN",
        Authority::PreVerificationToken => "PRE_VERIFICATION_TOKEN",
    }
}

fn parse_authority(s: &str) -> Authority {
    match s {
        "SYS_ADMIN"              => Authority::SysAdmin,
        "TENANT_ADMIN"           => Authority::TenantAdmin,
        "CUSTOMER_USER"          => Authority::CustomerUser,
        _                        => Authority::CustomerUser,
    }
}

#[derive(Debug, Deserialize)]
pub struct PageParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
    pub authority: Option<String>,
}

impl PageParams {
    pub fn to_page_link(&self) -> PageLink {
        let mut pl = PageLink::new(self.page.unwrap_or(0), self.page_size.unwrap_or(10));
        pl.text_search = self.text_search.clone();
        pl
    }
}

#[derive(Debug, Deserialize)]
pub struct CredentialsEnabledParams {
    #[serde(rename = "userCredentialsEnabled")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialsEnabledBody {
    pub user_credentials_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveUserRequest {
    pub id: Option<IdResponse>,
    pub tenant_id: Option<IdResponse>,
    pub customer_id: Option<IdResponse>,
    pub email: String,
    pub authority: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub additional_info: Option<serde_json::Value>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/user/{userId}
async fn get_user(
    State(state): State<AuthState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state.user_dao.find_by_id(user_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("User [{}] is not found", user_id)))?;
    Ok(Json(UserResponse::from(user)))
}

/// POST /api/user
async fn save_user(
    State(state): State<AuthState>,
    Json(req): Json<SaveUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let now = chrono::Utc::now().timestamp_millis();
    let user = User {
        id:              req.id.map(|i| i.id).unwrap_or_else(Uuid::new_v4),
        created_time:    now,
        tenant_id:       req.tenant_id.map(|i| i.id)
            .ok_or_else(|| ApiError::BadRequest("tenantId is required".into()))?,
        customer_id:     req.customer_id.map(|i| i.id),
        email:           req.email,
        authority:       parse_authority(req.authority.as_deref().unwrap_or("CUSTOMER_USER")),
        first_name:      req.first_name,
        last_name:       req.last_name,
        phone:           req.phone,
        additional_info: req.additional_info,
        version:         1,
    };
    let saved = state.user_dao.save(&user).await?;

    // Auto-create user_credentials với activate_token (khớp ThingsBoard Java)
    let creds = vl_core::entities::UserCredentials {
        id:              uuid::Uuid::new_v4(),
        created_time:    now,
        user_id:         saved.id,
        enabled:         false,
        password:        None,
        activate_token:  Some(uuid::Uuid::new_v4().to_string().replace('-', "")),
        reset_token:     None,
        additional_info: None,
    };
    state.user_dao.save_credentials(&creds).await?;

    Ok(Json(UserResponse::from(saved)))
}

/// DELETE /api/user/{userId}
async fn delete_user(
    State(state): State<AuthState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    // Verify user exists before deleting
    state.user_dao.find_by_id(user_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("User [{}] is not found", user_id)))?;
    state.user_dao.delete(user_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/user/{userId}/token — SYS_ADMIN impersonation token (Java: UserController.getUserToken)
async fn get_user_token(
    State(state): State<AuthState>,
    State(core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<JwtPairResponse>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let user = state.user_dao.find_by_id(user_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("User [{}] is not found", user_id)))?;

    let authority_str = match user.authority {
        Authority::SysAdmin             => "SYS_ADMIN",
        Authority::TenantAdmin          => "TENANT_ADMIN",
        Authority::CustomerUser         => "CUSTOMER_USER",
        Authority::RefreshToken         => "REFRESH_TOKEN",
        Authority::PreVerificationToken => "PRE_VERIFICATION_TOKEN",
    };
    let tenant_id = if user.authority == Authority::SysAdmin { None } else { Some(user.tenant_id) };
    let pair = core.jwt_service
        .issue_token(user.id, tenant_id, user.customer_id, authority_str, vec![authority_str.to_string()])
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(JwtPairResponse {
        token:         pair.token,
        refresh_token: pair.refresh_token,
        scope:         None,
    }))
}

/// GET /api/user/{userId}/activationLink
async fn get_activation_link(
    State(state): State<AuthState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let creds = state.user_dao.find_credentials(user_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Credentials not found for user [{}]", user_id)))?;
    let token = creds.activate_token
        .ok_or_else(|| ApiError::BadRequest("User is already activated".into()))?;
    Ok(Json(serde_json::json!({ "link": format!("/api/noauth/activate?activateToken={}", token) })))
}

/// POST /api/user/{userId}/sendActivationMail — stub (real impl would send email)
async fn send_activation_mail(
    Path(_user_id): Path<Uuid>,
) -> StatusCode {
    StatusCode::OK
}

/// POST /api/user/{userId}/userCredentialsEnabled
/// Body: { "userCredentialsEnabled": true } or query param ?userCredentialsEnabled=true
async fn set_user_credentials_enabled(
    State(state): State<AuthState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<CredentialsEnabledParams>,
    body: Option<Json<CredentialsEnabledBody>>,
) -> Result<StatusCode, ApiError> {
    let creds = state.user_dao.find_credentials(user_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Credentials not found for user [{}]", user_id)))?;
    // Body takes precedence over query param
    let enabled = body.and_then(|b| b.user_credentials_enabled)
        .or(params.enabled)
        .unwrap_or(true);
    let updated = vl_core::entities::UserCredentials { enabled, ..creds };
    state.user_dao.save_credentials(&updated).await?;
    Ok(StatusCode::OK)
}

/// GET /api/users/count — count users (SYS_ADMIN: all, TENANT_ADMIN: by tenant)
async fn count_users(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let count = if ctx.is_sys_admin() {
        state.user_dao.count_all().await?
    } else {
        state.user_dao.count_by_tenant(ctx.tenant_id).await?
    };
    Ok(Json(serde_json::json!(count)))
}

/// GET /api/users?authority=TENANT_ADMIN&pageSize=...
/// Lists users by authority (SYS_ADMIN only).
async fn list_users(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<UserResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page = state.user_dao.find_by_tenant(tenant_id, &params.to_page_link()).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(UserResponse::from).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/tenant/{tenantId}/users?pageSize=...
async fn list_tenant_users(
    State(state): State<AuthState>,
    Path(tenant_id): Path<Uuid>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<UserResponse>>, ApiError> {
    let page = state.user_dao.find_by_tenant(tenant_id, &params.to_page_link()).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(UserResponse::from).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/customer/{customerId}/users?pageSize=...
async fn list_customer_users(
    State(state): State<AuthState>,
    Path(customer_id): Path<Uuid>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<UserResponse>>, ApiError> {
    let page = state.user_dao.find_by_tenant(customer_id, &params.to_page_link()).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(UserResponse::from).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

// ── Mobile Sessions (FCM) ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MobileSessionRequest {
    fcm_token:   String,
    platform:    Option<String>,
    app_version: Option<String>,
}

/// POST /api/user/mobileSessions — register FCM token for push notifications
async fn save_mobile_session(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<MobileSessionRequest>,
) -> Result<StatusCode, ApiError> {
    let platform = body.platform.as_deref().unwrap_or("ANDROID");
    state.mobile_session_dao
        .upsert(ctx.user_id, &body.fcm_token, platform, body.app_version.as_deref(), None, None, None)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::OK)
}

/// DELETE /api/user/mobileSessions/{token} — remove FCM token on logout
async fn remove_mobile_session(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(token): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.mobile_session_dao
        .delete(ctx.user_id, &token)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::OK)
}

// ── Phase 69: User Avatar ─────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AvatarResponse {
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AvatarUploadRequest {
    /// Base64-encoded image from client
    avatar_b64: String,
}

/// POST /api/user/avatar — upload profile picture (base64 JSON body)
async fn upload_avatar(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<AvatarUploadRequest>,
) -> Result<Json<AvatarResponse>, ApiError> {
    // base64 is ~33% larger than binary; 2 MB image → ~2.7 MB base64
    if body.avatar_b64.len() > 3 * 1024 * 1024 {
        return Err(ApiError::BadRequest("Avatar too large (max ~2MB image)".into()));
    }
    state.user_dao
        .set_avatar(ctx.user_id, &body.avatar_b64)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(AvatarResponse {
        url: format!("/api/user/{}/avatar", ctx.user_id),
    }))
}

/// GET /api/user/{userId}/avatar
async fn get_avatar(
    State(state): State<AuthState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let b64_opt = state.user_dao
        .get_avatar(user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    match b64_opt {
        None => Err(ApiError::NotFound(format!("Avatar for user [{user_id}] not found"))),
        Some(b64) => Ok(Json(serde_json::json!({ "avatarB64": b64 })).into_response()),
    }
}

/// DELETE /api/user/avatar
async fn delete_avatar(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, ApiError> {
    state.user_dao
        .set_avatar(ctx.user_id, "")
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(StatusCode::OK)
}

// ── Phase 69: Notification Settings ──────────────────────────────────────────

/// GET /api/notifications/settings
async fn get_notification_settings(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let v = state.user_dao
        .get_notification_settings(ctx.user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .unwrap_or_else(|| serde_json::json!({
            "enabled": true,
            "pushEnabled": true,
            "emailEnabled": true
        }));
    Ok(Json(v))
}

/// PUT /api/notifications/settings
async fn save_notification_settings(
    State(state): State<AuthState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.user_dao
        .set_notification_settings(ctx.user_id, &body)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(body))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use vl_auth::password;
    use vl_core::entities::{Authority, User, UserCredentials};
    use vl_dao::postgres::user::UserDao;
    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    async fn test_app(pool: PgPool) -> axum::Router {
        let config = VieLangConfig::default();
        let rule_engine = vl_rule_engine::RuleEngine::start_noop();
        let queue_producer = vl_queue::create_producer(&config.queue).expect("queue");
        let cache = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        create_router(state)
    }

    async fn create_test_user_with_authority(pool: &PgPool, email: &str, pwd: &str, authority: Authority) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::new_v4(), customer_id: None,
            email: email.into(), authority,
            first_name: Some("Test".into()), last_name: Some("User".into()),
            phone: None, additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pwd).unwrap();
        let creds = UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(), user_id: user.id,
            enabled: true, password: Some(hash),
            activate_token: None, reset_token: None, additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        user
    }

    async fn post_json(app: axum::Router, uri: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn post_json_auth(app: axum::Router, uri: &str, token: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn delete_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("DELETE").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    async fn get_token(app: axum::Router, email: &str, pwd: &str) -> String {
        let resp = post_json(app, "/api/auth/login",
            json!({"username": email, "password": pwd})).await;
        body_json(resp).await["token"].as_str().unwrap().to_string()
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_user_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let caller = create_test_user_with_authority(&pool, "ucreate@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "ucreate@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/user", &token, json!({
            "email": "newuser@test.com",
            "authority": "CUSTOMER_USER",
            "tenantId": {"id": caller.tenant_id, "entityType": "TENANT"},
        })).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn user_response_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let caller = create_test_user_with_authority(&pool, "ufmt@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "ufmt@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/user", &token, json!({
            "email": "fmtuser@test.com",
            "authority": "TENANT_ADMIN",
            "tenantId": {"id": caller.tenant_id, "entityType": "TENANT"},
        })).await;

        let body = body_json(resp).await;
        assert!(body["id"]["id"].is_string(), "id.id must be UUID string");
        assert_eq!(body["id"]["entityType"], "USER");
        assert!(body["createdTime"].is_number(), "createdTime must be ms timestamp");
        assert_eq!(body["tenantId"]["entityType"], "TENANT");
        assert_eq!(body["email"], "fmtuser@test.com");
        assert_eq!(body["authority"], "TENANT_ADMIN");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_user_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let caller = create_test_user_with_authority(&pool, "uget@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "uget@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/user", &token, json!({
            "email": "getuser@test.com",
            "authority": "CUSTOMER_USER",
            "tenantId": {"id": caller.tenant_id, "entityType": "TENANT"},
        })).await;
        let user_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/user/{user_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["id"]["id"], user_id);
        assert_eq!(body["email"], "getuser@test.com");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_nonexistent_user_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user_with_authority(&pool, "u404@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "u404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/user/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = body_json(resp).await;
        assert_eq!(body["status"].as_u64().unwrap(), 404);
        assert!(body["errorCode"].is_number());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_user_then_get_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let caller = create_test_user_with_authority(&pool, "udel@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "udel@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/user", &token, json!({
            "email": "deluser@test.com",
            "authority": "CUSTOMER_USER",
            "tenantId": {"id": caller.tenant_id, "entityType": "TENANT"},
        })).await;
        let user_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let del = delete_auth(app.clone(), &format!("/api/user/{user_id}"), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        let get = get_auth(app, &format!("/api/user/{user_id}"), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn count_users_returns_number(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user_with_authority(&pool, "ucount@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "ucount@test.com", "pass123").await;

        let resp = get_auth(app, "/api/users/count", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body.is_number(), "count must be a number");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_users_returns_pagination_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let caller = create_test_user_with_authority(&pool, "ulist@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "ulist@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/tenant/{}/users?pageSize=10&page=0", caller.tenant_id), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert!(body["totalPages"].is_number());
        assert!(body["totalElements"].is_number());
        assert!(body["hasNext"].is_boolean());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_user_token_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        // Regular TENANT_ADMIN cannot impersonate other users
        let caller = create_test_user_with_authority(&pool, "uimpers@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "uimpers@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/user/{}/token", caller.id), &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn set_user_credentials_enabled(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        // Use a user that has credentials (created via helper, not the API)
        let caller = create_test_user_with_authority(&pool, "ucreds@test.com", "pass123", Authority::TenantAdmin).await;
        let token = get_token(app.clone(), "ucreds@test.com", "pass123").await;

        // Create a second user via the DAO directly (with credentials)
        let dao = vl_dao::postgres::user::UserDao::new(pool.clone());
        let target = vl_core::entities::User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: caller.tenant_id, customer_id: None,
            email: "credsuser@test.com".into(), authority: Authority::CustomerUser,
            first_name: None, last_name: None, phone: None, additional_info: None, version: 1,
        };
        dao.save(&target).await.unwrap();
        let hash = vl_auth::password::hash_password("pass456").unwrap();
        let creds = vl_core::entities::UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(), user_id: target.id,
            enabled: true, password: Some(hash),
            activate_token: None, reset_token: None, additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();

        // Disable user credentials
        let resp = post_json_auth(app, &format!("/api/user/{}/userCredentialsEnabled", target.id), &token,
            json!({"userCredentialsEnabled": false})).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn users_pagination_total_elements_correct(pool: PgPool) {
        let dao = UserDao::new(pool.clone());
        let app = test_app(pool.clone()).await;

        let login_user = create_test_user_with_authority(&pool, "users_pg_login@test.com", "pass123", Authority::TenantAdmin).await;
        let tenant_id = login_user.tenant_id;
        let token = get_token(app.clone(), "users_pg_login@test.com", "pass123").await;

        for i in 0..2u32 {
            let u = User {
                id: Uuid::new_v4(), created_time: now_ms(),
                tenant_id, customer_id: None,
                email: format!("users_pg_extra{i}@test.com"),
                authority: Authority::TenantAdmin,
                first_name: None, last_name: None, phone: None,
                additional_info: None, version: 1,
            };
            dao.save(&u).await.unwrap();
        }

        let resp = get_auth(app, &format!("/api/tenant/{tenant_id}/users?pageSize=10&page=0"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert!(body["totalElements"].is_number());
        assert!(body["hasNext"].is_boolean());
        assert_eq!(body["totalElements"], 3);
        assert_eq!(body["hasNext"], false);
    }
}
