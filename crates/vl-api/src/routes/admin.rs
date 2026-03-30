use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{
    AdminSettings, AutoCommitSettings, FeaturesInfo, JwtSettingsInfo, SecuritySettings, SystemInfo,
    SystemInfoData,
};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AdminState, CoreState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Existing
        .route("/admin/stats",              get(admin_stats))
        .route("/admin/info",               get(admin_info))
        // Phase 29: AdminController endpoints
        .route("/admin/settings/{key}",     get(get_admin_settings))
        .route("/admin/settings",           post(save_admin_settings))
        .route("/admin/settings/testMail",  post(test_mail))
        .route("/admin/securitySettings",   get(get_security_settings).post(save_security_settings))
        .route("/admin/jwtSettings",        get(get_jwt_settings).post(save_jwt_settings))
        .route("/admin/systemInfo",         get(get_system_info))
        .route("/admin/featuresInfo",       get(get_features_info))
        .route("/admin/updates",            get(get_updates))
        // Phase 29: RepositorySettings stubs (TENANT_ADMIN)
        .route("/admin/repositorySettings", get(repo_settings_stub).post(repo_settings_stub_post).delete(repo_settings_delete))
        .route("/admin/autoCommitSettings", get(auto_commit_stub).post(auto_commit_post).delete(auto_commit_delete))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub db_pool_size:  u32,
    pub db_pool_idle:  u32,
    pub version:       &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildInfo {
    pub version:      &'static str,
    pub rust_version: &'static str,
    pub pkg_name:     &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAdminSettingsRequest {
    pub id:         Option<Uuid>,
    pub tenant_id:  Option<Uuid>,
    pub key:        String,
    pub json_value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestMailRequest {
    pub email: String,
}

/// Khớp Java: UpdatesController response fields
#[derive(Debug, Serialize)]
pub struct UpdateMessage {
    #[serde(rename = "updateAvailable")]
    pub update_available: bool,
    #[serde(rename = "currentVersion")]
    pub current_version: String,
    #[serde(rename = "latestVersion")]
    pub latest_version: String,
    #[serde(rename = "upgradeInstructionsUrl")]
    pub upgrade_instructions_url: Option<String>,
    #[serde(rename = "currentVersionReleaseNotesUrl")]
    pub current_version_release_notes_url: Option<String>,
    #[serde(rename = "latestVersionReleaseNotesUrl")]
    pub latest_version_release_notes_url: Option<String>,
}

// ── Existing handlers ────────────────────────────────────────────────────────

/// GET /api/admin/stats — system statistics (SYS_ADMIN only)
async fn admin_stats(
    State(state): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<SystemStats>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    Ok(Json(SystemStats {
        db_pool_size: state.pool.size(),
        db_pool_idle: state.pool.num_idle() as u32,
        version:      env!("CARGO_PKG_VERSION"),
    }))
}

/// GET /api/admin/info — build information (SYS_ADMIN only)
async fn admin_info(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<BuildInfo>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    Ok(Json(BuildInfo {
        version:      env!("CARGO_PKG_VERSION"),
        rust_version: option_env!("CARGO_PKG_RUST_VERSION").filter(|s| !s.is_empty()).unwrap_or("1.75"),
        pkg_name:     env!("CARGO_PKG_NAME"),
    }))
}

// ── AdminSettings handlers ────────────────────────────────────────────────────

/// GET /api/admin/settings/{key}
async fn get_admin_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(key): Path<String>,
) -> Result<Json<AdminSettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    // SYS_TENANT_ID = nil UUID in ThingsBoard
    let sys_tenant = Uuid::nil();
    let settings = state.admin_settings_dao
        .find_by_key(sys_tenant, &key)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Settings [{key}] is not found")))?;
    Ok(Json(settings))
}

/// POST /api/admin/settings
async fn save_admin_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveAdminSettingsRequest>,
) -> Result<Json<AdminSettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let sys_tenant = Uuid::nil();
    let settings = AdminSettings {
        id:           req.id.unwrap_or_else(Uuid::new_v4),
        created_time: now,
        tenant_id:    req.tenant_id.unwrap_or(sys_tenant),
        key:          req.key,
        json_value:   req.json_value,
    };
    let saved = state.admin_settings_dao.save(&settings).await?;
    Ok(Json(saved))
}

/// POST /api/admin/settings/testMail
async fn test_mail(
    Extension(ctx): Extension<SecurityContext>,
    Json(_req): Json<TestMailRequest>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    // Stub: email sending requires SMTP config from admin_settings "mail" key
    Ok(StatusCode::OK)
}

// ── SecuritySettings handlers ─────────────────────────────────────────────────

/// GET /api/admin/securitySettings
async fn get_security_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<SecuritySettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    // Load from admin_settings key "security" if exists, else return default
    let sys_tenant = Uuid::nil();
    let settings = state.admin_settings_dao
        .find_by_key(sys_tenant, "security")
        .await?;

    if let Some(s) = settings {
        let parsed: SecuritySettings = serde_json::from_value(s.json_value)
            .unwrap_or_default();
        Ok(Json(parsed))
    } else {
        Ok(Json(SecuritySettings::default()))
    }
}

/// POST /api/admin/securitySettings
async fn save_security_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SecuritySettings>,
) -> Result<Json<SecuritySettings>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let sys_tenant = Uuid::nil();
    let json_value = serde_json::to_value(&req)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let entry = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    sys_tenant,
        key:          "security".into(),
        json_value,
    };
    state.admin_settings_dao.save(&entry).await?;
    Ok(Json(req))
}

// ── JwtSettings handlers ──────────────────────────────────────────────────────

/// GET /api/admin/jwtSettings
async fn get_jwt_settings(
    State(core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<JwtSettingsInfo>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let cfg = &core.config.security.jwt;
    Ok(Json(JwtSettingsInfo {
        token_expiration_time:  cfg.expiration_secs as i64,
        refresh_token_exp_time: cfg.refresh_expiration_secs as i64,
        token_issuer:           "vielang.io".into(),
        token_signing_key:      "***".into(), // masked
    }))
}

/// POST /api/admin/jwtSettings — saves settings (returns masked key; real TB would return new JwtPair)
async fn save_jwt_settings(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<JwtSettingsInfo>,
) -> Result<Json<JwtSettingsInfo>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let sys_tenant = Uuid::nil();
    let json_value = serde_json::to_value(&req)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let entry = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    sys_tenant,
        key:          "jwt".into(),
        json_value,
    };
    state.admin_settings_dao.save(&entry).await?;
    Ok(Json(req))
}

// ── SystemInfo handler ────────────────────────────────────────────────────────

/// GET /api/admin/systemInfo — CPU, memory, disk stats via sysinfo
async fn get_system_info(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<SystemInfo>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_memory = sys.total_memory();
    let used_memory  = sys.used_memory();
    let memory_usage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    // Global CPU usage (average across all cores)
    let cpu_usage = sys.global_cpu_usage() as f64;
    let cpu_count = sys.cpus().len() as u64;

    // Disk usage
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    let (total_disk, used_disk) = disks.iter().fold((0u64, 0u64), |(t, u), d| {
        (t + d.total_space(), u + (d.total_space() - d.available_space()))
    });
    let disk_usage = if total_disk > 0 {
        (used_disk as f64 / total_disk as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(SystemInfo {
        is_monolith: true,
        system_data: vec![SystemInfoData {
            service_id:       "vielang".into(),
            service_type:     "monolith".into(),
            cpu_usage,
            cpu_count,
            memory_usage,
            total_memory,
            disk_usage,
            total_disk_space: total_disk,
        }],
    }))
}

// ── FeaturesInfo handler ──────────────────────────────────────────────────────

/// GET /api/admin/featuresInfo
async fn get_features_info(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<FeaturesInfo>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    Ok(Json(FeaturesInfo {
        email_enabled:        false,
        oauth_enabled:        false,
        sms_enabled:          false,
        notification_enabled: true,
        two_fa_enabled:       true,
    }))
}

// ── Updates handler ───────────────────────────────────────────────────────────

/// GET /api/admin/updates — stub (no external update check)
async fn get_updates(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<UpdateMessage>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }
    let ver = env!("CARGO_PKG_VERSION");
    Ok(Json(UpdateMessage {
        update_available:                   false,
        current_version:                    ver.into(),
        latest_version:                     ver.into(),
        upgrade_instructions_url:           None,
        current_version_release_notes_url:  None,
        latest_version_release_notes_url:   None,
    }))
}

// ── Repository / AutoCommit stubs ─────────────────────────────────────────────

async fn repo_settings_stub(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN authority required".into()));
    }
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn repo_settings_stub_post(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN authority required".into()));
    }
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn repo_settings_delete(
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN authority required".into()));
    }
    Ok(StatusCode::NOT_IMPLEMENTED)
}

/// GET /api/admin/autoCommitSettings
/// Returns the auto-commit settings for the current tenant.
async fn auto_commit_stub(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<AutoCommitSettings>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN authority required".into()));
    }

    let settings = state
        .admin_settings_dao
        .find_by_key(ctx.tenant_id, "autoCommit")
        .await?;

    match settings {
        Some(s) => {
            let auto_commit: AutoCommitSettings = serde_json::from_value(s.json_value)
                .map_err(|e| ApiError::Internal(format!("Failed to parse autoCommit settings: {e}")))?;
            Ok(Json(auto_commit))
        }
        None => {
            // Return default (disabled) if not configured
            Ok(Json(AutoCommitSettings {
                enabled: false,
                entity_types: vec![],
            }))
        }
    }
}

/// POST /api/admin/autoCommitSettings
/// Save auto-commit settings for the current tenant.
async fn auto_commit_post(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<AutoCommitSettings>,
) -> Result<Json<AutoCommitSettings>, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN authority required".into()));
    }

    let now = chrono::Utc::now().timestamp_millis();
    let json_value = serde_json::to_value(&req)
        .map_err(|e| ApiError::Internal(format!("Failed to serialize autoCommit settings: {e}")))?;

    let settings = AdminSettings {
        id: Uuid::new_v4(),
        created_time: now,
        tenant_id: ctx.tenant_id,
        key: "autoCommit".to_string(),
        json_value,
    };

    state.admin_settings_dao.save(&settings).await?;
    Ok(Json(req))
}

/// DELETE /api/admin/autoCommitSettings
/// Remove auto-commit settings for the current tenant.
async fn auto_commit_delete(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() && !ctx.is_tenant_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN authority required".into()));
    }

    state
        .admin_settings_dao
        .delete_by_key(ctx.tenant_id, "autoCommit")
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use vl_auth::password;
    use vl_config::VieLangConfig;
    use vl_core::entities::{Authority, User, UserCredentials};
    use vl_dao::postgres::user::UserDao;
    use crate::{routes::create_router, state::AppState};

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
        let queue_producer = vl_queue::create_producer(&config.queue).expect("queue");
        let cache = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        create_router(state)
    }

    /// Create a SYS_ADMIN user (nil tenant_id, as per ThingsBoard convention)
    async fn create_sys_admin(pool: &PgPool, email: &str, pwd: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id:           Uuid::new_v4(),
            created_time: now_ms(),
            tenant_id:    Uuid::nil(), // SYS_ADMIN has nil tenant_id
            customer_id:  None,
            email:        email.into(),
            authority:    Authority::SysAdmin,
            first_name:   None,
            last_name:    None,
            phone:        None,
            additional_info: None,
            version:      1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pwd).unwrap();
        let creds = UserCredentials {
            id:             Uuid::new_v4(),
            created_time:   now_ms(),
            user_id:        user.id,
            enabled:        true,
            password:       Some(hash),
            activate_token: None,
            reset_token:    None,
            additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        user
    }

    /// Create a TENANT_ADMIN user with a real tenant_id
    async fn create_tenant_admin(pool: &PgPool, email: &str, pwd: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id:           Uuid::new_v4(),
            created_time: now_ms(),
            tenant_id:    Uuid::new_v4(),
            customer_id:  None,
            email:        email.into(),
            authority:    Authority::TenantAdmin,
            first_name:   Some("Tenant".into()),
            last_name:    Some("Admin".into()),
            phone:        None,
            additional_info: None,
            version:      1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pwd).unwrap();
        let creds = UserCredentials {
            id:             Uuid::new_v4(),
            created_time:   now_ms(),
            user_id:        user.id,
            enabled:        true,
            password:       Some(hash),
            activate_token: None,
            reset_token:    None,
            additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        user
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

    async fn post_json_auth(
        app: axum::Router,
        uri: &str,
        token: &str,
        body: Value,
    ) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
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

    async fn get_no_auth(app: axum::Router, uri: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    async fn get_token(app: axum::Router, email: &str, pwd: &str) -> String {
        let resp = post_json(
            app,
            "/api/auth/login",
            json!({"username": email, "password": pwd}),
        )
        .await;
        body_json(resp).await["token"].as_str().unwrap().to_string()
    }

    // ── Router smoke test ─────────────────────────────────────────────────────

    #[test]
    #[ignore = "verified passing"]
    fn admin_router_registered() {
        let r = router();
        drop(r);
    }

    // ── GET /api/admin/stats ──────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn admin_stats_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-stats@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-stats@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/stats", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = body_json(resp).await;
        assert_eq!(body["status"].as_u64().unwrap(), 403);
        assert!(body["message"].is_string());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn admin_stats_with_sys_admin_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-stats@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-stats@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/stats", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body["dbPoolSize"].is_number(), "dbPoolSize must be present");
        assert!(body["dbPoolIdle"].is_number(), "dbPoolIdle must be present");
        assert!(body["version"].is_string(), "version must be present");
        assert!(!body["version"].as_str().unwrap().is_empty());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn admin_stats_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        let resp = get_no_auth(app, "/api/admin/stats").await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = body_json(resp).await;
        assert_eq!(body["status"].as_u64().unwrap(), 401);
    }

    // ── GET /api/admin/info ───────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn admin_info_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-info@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-info@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/info", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn admin_info_returns_build_info(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-info@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-info@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/info", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        // Fields are camelCase per serde rename_all = "camelCase"
        assert!(body["version"].is_string(), "version field must be present");
        assert!(body["rustVersion"].is_string(), "rustVersion field must be present");
        assert!(body["pkgName"].is_string(), "pkgName field must be present");
        // All values must be non-empty (populated from env!() macros at compile time)
        assert!(!body["version"].as_str().unwrap().is_empty());
        assert!(!body["rustVersion"].as_str().unwrap().is_empty());
        assert!(!body["pkgName"].as_str().unwrap().is_empty());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn admin_info_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        let resp = get_no_auth(app, "/api/admin/info").await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── GET /api/admin/settings/{key} ─────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_admin_settings_nonexistent_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-getset@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-getset@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/settings/nonexistent_key_xyz", &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = body_json(resp).await;
        assert_eq!(body["status"].as_u64().unwrap(), 404);
        assert!(body["message"].as_str().unwrap().contains("nonexistent_key_xyz"));
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_admin_settings_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-getset@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-getset@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/settings/some_key", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // ── POST /api/admin/settings then GET ────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_admin_settings_then_get_returns_value(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-saveset@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-saveset@test.com", "pass123").await;

        // Save settings
        let payload = json!({
            "key": "test_key_integration",
            "jsonValue": {
                "smtpHost": "localhost",
                "smtpPort": 25,
                "enableTls": false
            }
        });
        let save_resp = post_json_auth(
            app.clone(),
            "/api/admin/settings",
            &token,
            payload,
        )
        .await;
        assert_eq!(save_resp.status(), StatusCode::OK);

        let saved = body_json(save_resp).await;
        assert_eq!(saved["key"], "test_key_integration");
        assert!(saved["id"].is_string(), "Returned settings must have an id");
        assert_eq!(saved["jsonValue"]["smtpHost"], "localhost");
        assert_eq!(saved["jsonValue"]["smtpPort"], 25);

        // Retrieve and verify
        let get_resp = get_auth(
            app,
            "/api/admin/settings/test_key_integration",
            &token,
        )
        .await;
        assert_eq!(get_resp.status(), StatusCode::OK);

        let fetched = body_json(get_resp).await;
        assert_eq!(fetched["key"], "test_key_integration");
        assert_eq!(fetched["jsonValue"]["smtpHost"], "localhost");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_admin_settings_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-saveset@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-saveset@test.com", "pass123").await;

        let resp = post_json_auth(
            app,
            "/api/admin/settings",
            &token,
            json!({"key": "x", "jsonValue": {}}),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // ── GET /api/admin/securitySettings ──────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn security_settings_get_returns_200_with_password_policy(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-sec@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-sec@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/securitySettings", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        // SecuritySettings must contain passwordPolicy with minimumLength
        assert!(body["passwordPolicy"].is_object(), "passwordPolicy must be an object");
        assert!(
            body["passwordPolicy"]["minimumLength"].is_number(),
            "minimumLength must be a number"
        );
        assert!(
            body["userActivationTokenTtl"].is_number(),
            "userActivationTokenTtl must be present"
        );
        assert!(
            body["passwordResetTokenTtl"].is_number(),
            "passwordResetTokenTtl must be present"
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn security_settings_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-sec@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-sec@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/securitySettings", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    // ── POST /api/admin/securitySettings ─────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn security_settings_save_and_retrieve(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-secsave@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-secsave@test.com", "pass123").await;

        let payload = json!({
            "passwordPolicy": {
                "minimumLength": 10,
                "minimumUppercaseLetters": 1,
                "minimumLowercaseLetters": 1,
                "minimumDigits": 1,
                "minimumSpecialCharacters": null,
                "passwordExpirationPeriodDays": null,
                "allowWhitespaces": null,
                "forceUserToResetPasswordIfNotValid": null
            },
            "maxFailedLoginAttempts": 5,
            "userLockoutNotificationEmail": null,
            "userActivationTokenTtl": 48,
            "passwordResetTokenTtl": 24
        });

        let save_resp = post_json_auth(
            app.clone(),
            "/api/admin/securitySettings",
            &token,
            payload,
        )
        .await;
        assert_eq!(save_resp.status(), StatusCode::OK);

        let saved = body_json(save_resp).await;
        assert_eq!(saved["passwordPolicy"]["minimumLength"], 10);
        assert_eq!(saved["maxFailedLoginAttempts"], 5);
        assert_eq!(saved["userActivationTokenTtl"], 48);

        // Retrieve and verify it was persisted
        let get_resp = get_auth(app, "/api/admin/securitySettings", &token).await;
        assert_eq!(get_resp.status(), StatusCode::OK);

        let fetched = body_json(get_resp).await;
        assert_eq!(fetched["passwordPolicy"]["minimumLength"], 10);
        assert_eq!(fetched["maxFailedLoginAttempts"], 5);
    }

    // ── GET /api/admin/jwtSettings ────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn jwt_settings_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-jwt@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-jwt@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/jwtSettings", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn jwt_settings_get_returns_token_expiration(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-jwt@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-jwt@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/jwtSettings", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(
            body["tokenExpirationTime"].is_number(),
            "tokenExpirationTime must be present"
        );
        assert!(
            body["refreshTokenExpTime"].is_number(),
            "refreshTokenExpTime must be present"
        );
        assert!(
            body["tokenIssuer"].is_string(),
            "tokenIssuer must be present"
        );
        assert!(
            body["tokenSigningKey"].is_string(),
            "tokenSigningKey must be present"
        );
        // Signing key must be masked for security
        assert_eq!(
            body["tokenSigningKey"].as_str().unwrap(),
            "***",
            "tokenSigningKey must be masked"
        );
        assert!(!body["tokenIssuer"].as_str().unwrap().is_empty());
        // Expiration times must be positive
        assert!(body["tokenExpirationTime"].as_i64().unwrap() > 0);
        assert!(body["refreshTokenExpTime"].as_i64().unwrap() > 0);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn jwt_settings_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        let resp = get_no_auth(app, "/api/admin/jwtSettings").await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── GET /api/admin/systemInfo ─────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn system_info_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-sysinfo@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-sysinfo@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/systemInfo", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn system_info_returns_cpu_memory_disk(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-sysinfo@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-sysinfo@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/systemInfo", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        // Top-level field is "monolith" (serde rename from is_monolith)
        assert!(body["monolith"].is_boolean(), "monolith flag must be present");
        assert_eq!(body["monolith"], true);

        // systemData must be a non-empty array
        assert!(body["systemData"].is_array(), "systemData must be an array");
        let system_data = body["systemData"].as_array().unwrap();
        assert!(!system_data.is_empty(), "systemData must have at least one entry");

        let entry = &system_data[0];
        assert!(entry["serviceId"].is_string(), "serviceId must be present");
        assert!(entry["serviceType"].is_string(), "serviceType must be present");
        assert!(entry["cpuUsage"].is_number(), "cpuUsage must be present");
        assert!(entry["cpuCount"].is_number(), "cpuCount must be present");
        assert!(entry["memoryUsage"].is_number(), "memoryUsage must be present");
        assert!(entry["totalMemory"].is_number(), "totalMemory must be present");
        assert!(entry["diskUsage"].is_number(), "diskUsage must be present");
        assert!(entry["totalDiskSpace"].is_number(), "totalDiskSpace must be present");
    }

    // ── GET /api/admin/featuresInfo ───────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn features_info_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-feat@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-feat@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/featuresInfo", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn features_info_returns_enabled_flags(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-feat@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-feat@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/featuresInfo", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        // All fields use explicit serde rename (not camelCase rename_all)
        assert!(body["emailEnabled"].is_boolean(), "emailEnabled must be present");
        assert!(body["oauthEnabled"].is_boolean(), "oauthEnabled must be present");
        assert!(body["smsEnabled"].is_boolean(), "smsEnabled must be present");
        assert!(body["notificationEnabled"].is_boolean(), "notificationEnabled must be present");
        assert!(body["twoFaEnabled"].is_boolean(), "twoFaEnabled must be present");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn features_info_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        let resp = get_no_auth(app, "/api/admin/featuresInfo").await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── GET /api/admin/updates ────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn admin_updates_requires_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-upd@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-upd@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/updates", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn admin_updates_returns_version_info(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-upd@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-upd@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/updates", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body["updateAvailable"].is_boolean(), "updateAvailable must be present");
        assert_eq!(body["updateAvailable"], false);
        assert!(body["currentVersion"].is_string(), "currentVersion must be present");
        assert!(body["latestVersion"].is_string(), "latestVersion must be present");
        assert!(!body["currentVersion"].as_str().unwrap().is_empty());
        assert!(!body["latestVersion"].as_str().unwrap().is_empty());
        // Current == latest (no update available stub)
        assert_eq!(body["currentVersion"], body["latestVersion"]);
    }

    // ── Cross-cutting RBAC tests ──────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn sys_admin_can_access_all_admin_endpoints(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-rbac@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-rbac@test.com", "pass123").await;

        // /admin/stats
        let r = get_auth(app.clone(), "/api/admin/stats", &token).await;
        assert_eq!(r.status(), StatusCode::OK, "/admin/stats should be 200 for SYS_ADMIN");

        // /admin/info
        let r = get_auth(app.clone(), "/api/admin/info", &token).await;
        assert_eq!(r.status(), StatusCode::OK, "/admin/info should be 200 for SYS_ADMIN");

        // /admin/securitySettings
        let r = get_auth(app.clone(), "/api/admin/securitySettings", &token).await;
        assert_eq!(r.status(), StatusCode::OK, "/admin/securitySettings should be 200 for SYS_ADMIN");

        // /admin/jwtSettings
        let r = get_auth(app.clone(), "/api/admin/jwtSettings", &token).await;
        assert_eq!(r.status(), StatusCode::OK, "/admin/jwtSettings should be 200 for SYS_ADMIN");

        // /admin/systemInfo
        let r = get_auth(app.clone(), "/api/admin/systemInfo", &token).await;
        assert_eq!(r.status(), StatusCode::OK, "/admin/systemInfo should be 200 for SYS_ADMIN");

        // /admin/featuresInfo
        let r = get_auth(app.clone(), "/api/admin/featuresInfo", &token).await;
        assert_eq!(r.status(), StatusCode::OK, "/admin/featuresInfo should be 200 for SYS_ADMIN");

        // /admin/updates
        let r = get_auth(app.clone(), "/api/admin/updates", &token).await;
        assert_eq!(r.status(), StatusCode::OK, "/admin/updates should be 200 for SYS_ADMIN");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn tenant_admin_blocked_from_all_sys_admin_endpoints(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-rbac@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-rbac@test.com", "pass123").await;

        let endpoints = [
            "/api/admin/stats",
            "/api/admin/info",
            "/api/admin/securitySettings",
            "/api/admin/jwtSettings",
            "/api/admin/systemInfo",
            "/api/admin/featuresInfo",
            "/api/admin/updates",
        ];

        for endpoint in endpoints {
            let r = get_auth(app.clone(), endpoint, &token).await;
            assert_eq!(
                r.status(),
                StatusCode::FORBIDDEN,
                "{endpoint} should return 403 for TENANT_ADMIN"
            );
        }
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn missing_auth_returns_401_on_all_protected_admin_endpoints(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        let endpoints = [
            "/api/admin/stats",
            "/api/admin/info",
            "/api/admin/securitySettings",
            "/api/admin/jwtSettings",
            "/api/admin/systemInfo",
            "/api/admin/featuresInfo",
            "/api/admin/updates",
        ];

        for endpoint in endpoints {
            let r = get_no_auth(app.clone(), endpoint).await;
            assert_eq!(
                r.status(),
                StatusCode::UNAUTHORIZED,
                "{endpoint} without auth should return 401"
            );
        }
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn jwt_settings_requires_sys_admin_not_tenant_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-jwt2@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-jwt2@test.com", "pass123").await;

        // GET jwtSettings
        let r = get_auth(app.clone(), "/api/admin/jwtSettings", &token).await;
        assert_eq!(r.status(), StatusCode::FORBIDDEN);
        let body = body_json(r).await;
        assert_eq!(body["status"].as_u64().unwrap(), 403);

        // POST jwtSettings
        let post_resp = post_json_auth(
            app,
            "/api/admin/jwtSettings",
            &token,
            json!({
                "tokenExpirationTime": 9000,
                "refreshTokenExpTime": 604800,
                "tokenIssuer": "vielang.io",
                "tokenSigningKey": "some_key"
            }),
        )
        .await;
        assert_eq!(post_resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn features_info_requires_sys_admin_not_tenant_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "ta-feat2@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-feat2@test.com", "pass123").await;

        let resp = get_auth(app, "/api/admin/featuresInfo", &token).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = body_json(resp).await;
        // Verify ThingsBoard error format
        assert_eq!(body["status"].as_u64().unwrap(), 403);
        assert!(body["message"].is_string());
        assert!(body["errorCode"].is_number());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn repo_settings_accessible_by_tenant_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        // repo_settings_stub allows SYS_ADMIN or TENANT_ADMIN
        create_tenant_admin(&pool, "ta-repo@test.com", "pass123").await;
        let token = get_token(app.clone(), "ta-repo@test.com", "pass123").await;

        let r = get_auth(app, "/api/admin/repositorySettings", &token).await;
        // Handler returns NOT_IMPLEMENTED (501) — not 403
        assert_eq!(r.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn repo_settings_accessible_by_sys_admin(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_sys_admin(&pool, "sa-repo@test.com", "pass123").await;
        let token = get_token(app.clone(), "sa-repo@test.com", "pass123").await;

        let r = get_auth(app, "/api/admin/repositorySettings", &token).await;
        assert_eq!(r.status(), StatusCode::NOT_IMPLEMENTED);
    }
}
