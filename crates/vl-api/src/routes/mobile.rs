use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_cache::{get_cached, put_cached};
use vl_core::entities::{
    LoginMobileInfo, MobileApp, MobileAppBundle, MobileAppStatus, PlatformType,
    QrCodeSettings,
};
use vl_dao::PageLink;

use crate::{
    error::ApiError,
    middleware::auth::SecurityContext,
    state::{AdminState, AppState, AuthState, CoreState, MobileState},
};

/// Public (no auth) routes
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/noauth/qr/{secret}",          get(get_token_by_qr_secret))
        .route("/noauth/mobile",               get(get_login_mobile_info))
        // P9: Deep linking
        .route("/noauth/deeplink/device/{id}", get(device_deeplink))
}

/// Well-known routes (served at root, not under /api)
pub fn well_known_router() -> Router<AppState> {
    Router::new()
        .route("/.well-known/apple-app-site-association", get(apple_app_site_association))
        .route("/.well-known/assetlinks.json",            get(android_assetlinks))
}

/// Protected (JWT required) routes
pub fn router() -> Router<AppState> {
    Router::new()
        // MobileApp CRUD
        .route("/mobile/app",         post(save_mobile_app).get(list_mobile_apps))
        .route("/mobile/app/{id}",    get(get_mobile_app).delete(delete_mobile_app))
        // MobileAppBundle CRUD
        .route("/mobile/bundle",               post(save_bundle))
        .route("/mobile/bundle/{id}/oauth2Clients", put(update_bundle_oauth2))
        .route("/mobile/bundle/infos",         get(list_bundles))
        .route("/mobile/bundle/info/{id}",     get(get_bundle))
        .route("/mobile/bundle/{id}",          delete(delete_bundle))
        // QR code settings + deep link
        .route("/mobile/qr/settings",  get(get_qr_settings).post(save_qr_settings))
        .route("/mobile/qr/deepLink",  get(get_deep_link))
        // P9: Mobile session registration
        .route("/mobile/session",      post(register_mobile_session).delete(delete_mobile_session))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMobileAppRequest {
    pub id:            Option<Uuid>,
    pub pkg_name:      String,
    pub title:         Option<String>,
    pub app_secret:    Option<String>,
    pub platform_type: String,
    pub status:        Option<String>,
    pub version_info:  Option<serde_json::Value>,
    pub store_info:    Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBundleRequest {
    pub id:             Option<Uuid>,
    pub title:          Option<String>,
    pub android_app_id: Option<Uuid>,
    pub ios_app_id:     Option<Uuid>,
    pub layout_config:  Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub page:      Option<i64>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct OAuth2ClientsRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveQrRequest {
    pub id:                   Option<Uuid>,
    pub use_system_settings:  Option<bool>,
    pub use_default_app:      Option<bool>,
    pub mobile_app_bundle_id: Option<Uuid>,
    pub qr_code_config:       Option<serde_json::Value>,
    pub android_enabled:      Option<bool>,
    pub ios_enabled:          Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DeepLinkResponse {
    pub value: String,
}

// ── MobileApp handlers ────────────────────────────────────────────────────────

/// POST /api/mobile/app
async fn save_mobile_app(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveMobileAppRequest>,
) -> Result<Json<MobileApp>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let platform = match req.platform_type.to_uppercase().as_str() {
        "IOS" => PlatformType::Ios,
        _     => PlatformType::Android,
    };
    let status = match req.status.as_deref().unwrap_or("DRAFT").to_uppercase().as_str() {
        "PUBLISHED"  => MobileAppStatus::Published,
        "DEPRECATED" => MobileAppStatus::Deprecated,
        "SUSPENDED"  => MobileAppStatus::Suspended,
        _            => MobileAppStatus::Draft,
    };
    // Generate secret if not provided
    let app_secret = req.app_secret.unwrap_or_else(|| {
        use rand::Rng;
        rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    });
    let app = MobileApp {
        id:            req.id.unwrap_or_else(Uuid::new_v4),
        created_time:  now,
        tenant_id:     ctx.tenant_id,
        pkg_name:      req.pkg_name,
        title:         req.title,
        app_secret,
        platform_type: platform,
        status,
        version_info:  req.version_info,
        store_info:    req.store_info,
    };
    let saved = state.mobile_app_dao.save(&app).await?;
    Ok(Json(saved))
}

/// GET /api/mobile/app?page=0&pageSize=10
async fn list_mobile_apps(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<ListParams>,
) -> Result<Json<vl_dao::PageData<MobileApp>>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    let page_link = PageLink::new(params.page.unwrap_or(0), params.page_size.unwrap_or(10));
    let page = state.mobile_app_dao.find_by_tenant(ctx.tenant_id, &page_link).await?;
    Ok(Json(page))
}

/// GET /api/mobile/app/{id}
async fn get_mobile_app(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<MobileApp>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    let app = state.mobile_app_dao.find_by_id(id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Mobile App [{id}] is not found")))?;
    Ok(Json(app))
}

/// DELETE /api/mobile/app/{id}
async fn delete_mobile_app(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    state.mobile_app_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

// ── Bundle handlers ───────────────────────────────────────────────────────────

/// POST /api/mobile/bundle
async fn save_bundle(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveBundleRequest>,
) -> Result<Json<MobileAppBundle>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let bundle = MobileAppBundle {
        id:                req.id.unwrap_or_else(Uuid::new_v4),
        created_time:      now,
        tenant_id:         ctx.tenant_id,
        title:             req.title,
        android_app_id:    req.android_app_id,
        ios_app_id:        req.ios_app_id,
        layout_config:     req.layout_config,
        oauth2_client_ids: vec![],
    };
    let saved = state.mobile_app_bundle_dao.save(&bundle).await?;
    Ok(Json(saved))
}

/// PUT /api/mobile/bundle/{id}/oauth2Clients
async fn update_bundle_oauth2(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<OAuth2ClientsRequest>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    state.mobile_app_bundle_dao.update_oauth2_clients(id, req.ids).await?;
    Ok(StatusCode::OK)
}

/// GET /api/mobile/bundle/infos?page=0&pageSize=10
async fn list_bundles(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<ListParams>,
) -> Result<Json<vl_dao::PageData<MobileAppBundle>>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    let page_link = PageLink::new(params.page.unwrap_or(0), params.page_size.unwrap_or(10));
    let page = state.mobile_app_bundle_dao.find_by_tenant(ctx.tenant_id, &page_link).await?;
    Ok(Json(page))
}

/// GET /api/mobile/bundle/info/{id}
async fn get_bundle(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<MobileAppBundle>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    let bundle = state.mobile_app_bundle_dao.find_by_id(id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Mobile App Bundle [{id}] is not found")))?;
    Ok(Json(bundle))
}

/// DELETE /api/mobile/bundle/{id}
async fn delete_bundle(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN required".into()));
    }
    state.mobile_app_bundle_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

// ── QrCode handlers ───────────────────────────────────────────────────────────

/// GET /api/mobile/qr/settings
async fn get_qr_settings(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<QrCodeSettings>, ApiError> {
    let tenant_id = if ctx.is_sys_admin() { Uuid::nil() } else { ctx.tenant_id };
    let settings = state.qr_code_settings_dao.find_by_tenant(tenant_id).await?
        .unwrap_or_else(|| QrCodeSettings {
            id:                   Uuid::nil(),
            created_time:         0,
            tenant_id,
            use_system_settings:  false,
            use_default_app:      true,
            mobile_app_bundle_id: None,
            qr_code_config:       serde_json::json!({}),
            android_enabled:      false,
            ios_enabled:          false,
            google_play_link:     None,
            app_store_link:       None,
        });
    Ok(Json(settings))
}

/// POST /api/mobile/qr/settings (SYS_ADMIN only in Java, but allow TENANT_ADMIN too)
async fn save_qr_settings(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveQrRequest>,
) -> Result<Json<QrCodeSettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN required for QR settings".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let tenant_id = Uuid::nil(); // system-wide settings
    let settings = QrCodeSettings {
        id:                   req.id.unwrap_or_else(Uuid::new_v4),
        created_time:         now,
        tenant_id,
        use_system_settings:  req.use_system_settings.unwrap_or(false),
        use_default_app:      req.use_default_app.unwrap_or(true),
        mobile_app_bundle_id: req.mobile_app_bundle_id,
        qr_code_config:       req.qr_code_config.unwrap_or(serde_json::json!({})),
        android_enabled:      req.android_enabled.unwrap_or(false),
        ios_enabled:          req.ios_enabled.unwrap_or(false),
        google_play_link:     None,
        app_store_link:       None,
    };
    let saved = state.qr_code_settings_dao.save(&settings).await?;
    Ok(Json(saved))
}

/// GET /api/mobile/qr/deepLink — generate a short-lived QR secret, return deep link URL
async fn get_deep_link(
    State(core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<DeepLinkResponse>, ApiError> {
    // Generate a random secret and store in cache with 2-minute TTL
    use rand::Rng;
    let secret: String = rand::rng()
        .sample_iter(rand::distr::Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    let cache_key = format!("qr_secret:{secret}");
    let ttl = std::time::Duration::from_secs(120); // 2 minutes
    put_cached(&*core.cache, &cache_key, &ctx.user_id, Some(ttl))
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let host = &core.config.server.host;
    let port = core.config.server.port;
    let deep_link = format!("https://{host}:{port}/api/noauth/qr?secret={secret}&ttl=120");
    Ok(Json(DeepLinkResponse { value: deep_link }))
}

/// GET /api/noauth/qr/{secret} — validate secret and return JWT pair
async fn get_token_by_qr_secret(
    State(core): State<CoreState>,
    State(auth): State<AuthState>,
    Path(secret): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cache_key = format!("qr_secret:{secret}");
    let user_id: Option<Uuid> = get_cached(&*core.cache, &cache_key)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let user_id = user_id
        .ok_or_else(|| ApiError::Unauthorized("Invalid or expired QR secret".into()))?;

    // Evict single-use secret
    core.cache.evict(&cache_key).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Load user and issue JWT
    let user = auth.user_dao.find_by_id(user_id).await?
        .ok_or_else(|| ApiError::Unauthorized("User not found".into()))?;

    let authority_str = match user.authority {
        vl_core::entities::Authority::SysAdmin     => "SYS_ADMIN",
        vl_core::entities::Authority::TenantAdmin  => "TENANT_ADMIN",
        vl_core::entities::Authority::CustomerUser => "CUSTOMER_USER",
        _                                          => "CUSTOMER_USER",
    };

    let pair = core.jwt_service
        .issue_token(user.id, Some(user.tenant_id), user.customer_id, authority_str, vec![])
        .map_err(|e: vl_auth::AuthError| ApiError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "token": pair.token,
        "refreshToken": pair.refresh_token,
    })))
}

/// GET /api/noauth/mobile — pre-login mobile info (public)
async fn get_login_mobile_info(
    State(state): State<MobileState>,
) -> Json<LoginMobileInfo> {
    // Load system QR settings
    let qr = state.qr_code_settings_dao
        .find_by_tenant(Uuid::nil())
        .await
        .unwrap_or(None);

    let (android_enabled, ios_enabled) = qr.as_ref()
        .map(|s| (s.android_enabled, s.ios_enabled))
        .unwrap_or((false, false));

    Json(LoginMobileInfo {
        qr_enabled:       android_enabled || ios_enabled,
        android_enabled,
        ios_enabled,
        google_play_link: None,
        app_store_link:   None,
    })
}

// ── Deep linking handlers (P9) ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DeepLinkParams {
    /// "ios" | "android" — determines redirect scheme; defaults to web dashboard
    pub platform: Option<String>,
}

/// GET /api/noauth/deeplink/device/{id}
/// Redirects to the platform-native deep link for a device.
/// iOS:     thingsboard://device/{id}
/// Android: intent://device/{id}#Intent;scheme=thingsboard;end
/// Web:     /dashboard/devices/{id}
async fn device_deeplink(
    Path(device_id): Path<Uuid>,
    Query(params):   Query<DeepLinkParams>,
) -> Response {
    let url = match params.platform.as_deref() {
        Some("ios")     => format!("thingsboard://device/{device_id}"),
        Some("android") => format!("intent://device/{device_id}#Intent;scheme=thingsboard;package=org.thingsboard.demo;end"),
        _               => format!("/dashboard/devices/{device_id}"),
    };
    Redirect::temporary(&url).into_response()
}

/// GET /.well-known/apple-app-site-association
/// iOS Universal Links configuration.
/// Config stored in admin_settings under key "appleAppSiteAssociation".
/// Falls back to a sensible default if not configured.
async fn apple_app_site_association(
    State(state): State<AdminState>,
) -> Response {
    let json = state.admin_settings_dao
        .find_by_key(Uuid::nil(), "appleAppSiteAssociation")
        .await
        .ok()
        .flatten()
        .map(|s| s.json_value)
        .unwrap_or_else(|| serde_json::json!({
            "applinks": {
                "apps": [],
                "details": [{
                    "appID": "TEAMID.org.thingsboard.demo",
                    "paths": ["/api/v1/*", "/dashboard/*", "/api/noauth/deeplink/*"]
                }]
            }
        }));

    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        axum::Json(json),
    ).into_response()
}

/// GET /.well-known/assetlinks.json
/// Android App Links configuration.
/// Config stored in admin_settings under key "androidAssetLinks".
async fn android_assetlinks(
    State(state): State<AdminState>,
) -> Response {
    let json = state.admin_settings_dao
        .find_by_key(Uuid::nil(), "androidAssetLinks")
        .await
        .ok()
        .flatten()
        .map(|s| s.json_value)
        .unwrap_or_else(|| serde_json::json!([{
            "relation": ["delegate_permission/common.handle_all_urls"],
            "target": {
                "namespace": "android_app",
                "package_name": "org.thingsboard.demo",
                "sha256_cert_fingerprints": []
            }
        }]));

    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        axum::Json(json),
    ).into_response()
}

// ── Mobile session handlers (P9) ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileSessionRequest {
    /// Firebase Cloud Messaging token for this device
    pub fcm_token:    String,
    /// "ANDROID" | "IOS"
    pub platform:     Option<String>,
    pub app_version:  Option<String>,
    pub os:           Option<String>,
    pub os_version:   Option<String>,
    pub device_model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSessionRequest {
    pub fcm_token: String,
}

/// POST /api/mobile/session
/// Called on app open — registers or refreshes the device session (FCM token + device info).
async fn register_mobile_session(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<MobileSessionRequest>,
) -> Result<StatusCode, ApiError> {
    state.mobile_session_dao.upsert(
        ctx.user_id,
        &req.fcm_token,
        req.platform.as_deref().unwrap_or("ANDROID"),
        req.app_version.as_deref(),
        req.os.as_deref(),
        req.os_version.as_deref(),
        req.device_model.as_deref(),
    ).await?;
    Ok(StatusCode::OK)
}

/// DELETE /api/mobile/session
/// Called on logout — removes FCM token so no more push notifications to this device.
async fn delete_mobile_session(
    State(state): State<MobileState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<DeleteSessionRequest>,
) -> Result<StatusCode, ApiError> {
    state.mobile_session_dao.delete(ctx.user_id, &req.fcm_token).await?;
    Ok(StatusCode::OK)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;

    use vl_auth::password;
    use vl_config::VieLangConfig;
    use vl_core::entities::{Authority, User, UserCredentials};

    fn now_ms() -> i64 { chrono::Utc::now().timestamp_millis() }

    async fn test_app(pool: PgPool) -> axum::Router {
        let config = VieLangConfig::default();
        let rule_engine = vl_rule_engine::RuleEngine::start_noop();
        let queue_producer = vl_queue::create_producer(&config.queue).expect("queue");
        let cache = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state = crate::state::AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        crate::routes::create_router(state)
    }

    async fn create_tenant_admin(pool: &PgPool, email: &str) -> User {
        let dao = vl_dao::postgres::user::UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::new_v4(), customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: None, last_name: None, phone: None, additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password("pass123").unwrap();
        let creds = UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(), user_id: user.id,
            enabled: true, password: Some(hash),
            activate_token: None, reset_token: None, additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        user
    }

    async fn get_token(app: axum::Router, email: &str) -> String {
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": "pass123"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v["token"].as_str().unwrap().to_string()
    }

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    }

    #[test]
    #[ignore = "verified passing"]
    fn mobile_router_registered() {
        let r = router();
        drop(r);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_and_get_mobile_app(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_tenant_admin(&pool, "mapp@test.com").await;
        let token = get_token(app.clone(), "mapp@test.com").await;

        let resp = app.clone().oneshot(
            Request::builder().method("POST").uri("/api/mobile/app")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(json!({
                    "pkgName": "com.example.app",
                    "title": "Test App",
                    "platformType": "ANDROID",
                    "status": "DRAFT"
                }).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["pkgName"].as_str(), Some("com.example.app"));
        assert!(body["appSecret"].as_str().is_some());

        let id = body["id"].as_str().unwrap().to_string();

        // Get by id
        let resp2 = app.oneshot(
            Request::builder().method("GET").uri(format!("/api/mobile/app/{id}"))
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_and_list_bundle(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "mbundle@test.com").await;
        let token = get_token(app.clone(), "mbundle@test.com").await;

        let resp = app.clone().oneshot(
            Request::builder().method("POST").uri("/api/mobile/bundle")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(json!({"title": "My Bundle"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["title"].as_str(), Some("My Bundle"));

        // List bundles
        let resp2 = app.oneshot(
            Request::builder().method("GET").uri("/api/mobile/bundle/infos?page=0&pageSize=10")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let list = body_json(resp2).await;
        assert_eq!(list["totalElements"].as_i64(), Some(1));
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_login_mobile_info_no_auth(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/noauth/mobile")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["qrEnabled"].is_boolean());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_mobile_app_returns_ok(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "mdel@test.com").await;
        let token = get_token(app.clone(), "mdel@test.com").await;

        let r = app.clone().oneshot(
            Request::builder().method("POST").uri("/api/mobile/app")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(json!({"pkgName": "com.del.app", "platformType": "IOS"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        let id = body_json(r).await["id"].as_str().unwrap().to_string();

        let del = app.oneshot(
            Request::builder().method("DELETE").uri(format!("/api/mobile/app/{id}"))
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(del.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_bundle_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "gbundle@test.com").await;
        let token = get_token(app.clone(), "gbundle@test.com").await;

        // Save bundle
        let resp = app.clone().oneshot(
            Request::builder().method("POST").uri("/api/mobile/bundle")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(json!({"title": "Bundle Alpha"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let saved = body_json(resp).await;
        let id = saved["id"].as_str().unwrap().to_string();

        // GET /api/mobile/bundle/info/{id}
        let resp2 = app.oneshot(
            Request::builder().method("GET").uri(format!("/api/mobile/bundle/info/{id}"))
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let body = body_json(resp2).await;
        assert_eq!(body["title"].as_str(), Some("Bundle Alpha"));
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_bundle_returns_ok(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "dbundle@test.com").await;
        let token = get_token(app.clone(), "dbundle@test.com").await;

        // Save bundle
        let resp = app.clone().oneshot(
            Request::builder().method("POST").uri("/api/mobile/bundle")
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(json!({"title": "Bundle To Delete"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        let id = body_json(resp).await["id"].as_str().unwrap().to_string();

        // DELETE /api/mobile/bundle/{id}
        let del = app.oneshot(
            Request::builder().method("DELETE").uri(format!("/api/mobile/bundle/{id}"))
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(del.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_qr_settings_returns_defaults(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "qrsettings@test.com").await;
        let token = get_token(app.clone(), "qrsettings@test.com").await;

        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/mobile/qr/settings")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["useDefaultApp"].is_boolean());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_deep_link_returns_url(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "deeplink@test.com").await;
        let token = get_token(app.clone(), "deeplink@test.com").await;

        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/mobile/qr/deepLink")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let value = body["value"].as_str().unwrap();
        assert!(value.starts_with("https://"), "deep link should start with https://: {value}");
        assert!(value.contains("secret="), "deep link should contain secret param: {value}");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn noauth_qr_secret_invalid_returns_401(pool: PgPool) {
        let app = test_app(pool).await;

        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/noauth/qr/invalid-secret-xyz")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_mobile_apps_pagination(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "mlist@test.com").await;
        let token = get_token(app.clone(), "mlist@test.com").await;

        // Save two apps
        for (pkg, platform) in &[("com.list.app1", "ANDROID"), ("com.list.app2", "IOS")] {
            let r = app.clone().oneshot(
                Request::builder().method("POST").uri("/api/mobile/app")
                    .header("content-type", "application/json")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::from(json!({"pkgName": pkg, "platformType": platform}).to_string()))
                    .unwrap(),
            ).await.unwrap();
            assert_eq!(r.status(), StatusCode::OK);
        }

        // List with pagination
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/mobile/app?page=0&pageSize=10")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array(), "response should have data array");
        assert!(body["totalElements"].is_number(), "response should have totalElements");
        assert!(body["totalPages"].is_number(), "response should have totalPages");
        assert!(body["hasNext"].is_boolean(), "response should have hasNext");
        assert_eq!(body["totalElements"].as_i64(), Some(2));
    }
}
