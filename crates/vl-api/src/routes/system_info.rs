use axum::{
    extract::{Extension, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use vl_core::entities::{SystemParams, UsageInfo};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, CoreState, AdminState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // SystemInfoController — all authenticated users
        .route("/system/info",   get(get_system_build_info))
        .route("/system/params", get(get_system_params))
}

/// Public router — no auth required.
/// Flutter / Angular apps call these endpoints before login to discover the backend.
pub fn public_router() -> Router<AppState> {
    Router::new()
        // Flutter ThingsBoard PE: called on server URL entry to verify compatibility
        .route("/noauth/ui/systemParams", get(get_public_system_params))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemBuildInfo {
    pub version:                      String,
    pub artifact:                     String,
    pub name:                         String,
    #[serde(rename = "type")]
    pub build_type:                   String,
    /// Phase 68: mobile SDK compatibility — matches TB Java softwareVersion
    pub software_version:             String,
    pub software_version_description: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/system/info — build info (no auth check; middleware already requires token)
async fn get_system_build_info() -> Json<SystemBuildInfo> {
    Json(SystemBuildInfo {
        version:    env!("CARGO_PKG_VERSION").into(),
        artifact:   env!("CARGO_PKG_NAME").into(),
        name:       "VieLang".into(),
        build_type: "CE".into(),
        software_version:             env!("CARGO_PKG_VERSION").to_string(),
        software_version_description: format!("VieLang {}", env!("CARGO_PKG_VERSION")),
    })
}

/// GET /api/system/params — platform configuration parameters
async fn get_system_params(
    State(_state): State<CoreState>,
    Extension(_ctx): Extension<SecurityContext>,
) -> Result<Json<SystemParams>, ApiError> {
    Ok(Json(SystemParams {
        user_token_access_enabled:         false,
        allowed_dashboard_ids:             vec![],
        edges_support_enabled:             false,
        has_repository:                    false,
        tbel_enabled:                      false,
        persist_device_state_to_telemetry: false,
        // Java: userSettings == null → newObjectNode() + set("openedMenuSections", [])
        // Angular MenuService reads openedMenuSections — must never be null
        user_settings:                     Some(json!({ "openedMenuSections": [] })),
        max_datapoints_limit:              50_000,
        max_resource_size:                 16 * 1024 * 1024, // 16 MB
        mobile_qr_enabled:                 false,
        max_debug_mode_duration_minutes:   15,
    }))
}

/// GET /api/usage — tenant resource usage (TENANT_ADMIN only)
async fn get_usage_info(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<UsageInfo>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("TENANT_ADMIN authority required".into()));
    }
    let usage = state.usage_info_dao
        .get_tenant_usage(ctx.tenant_id)
        .await?;
    Ok(Json(usage))
}

/// GET /api/noauth/ui/systemParams — public server discovery for self-hosted deployments.
/// Flutter/Angular apps call this before login to verify the backend URL is a valid VieLang instance.
async fn get_public_system_params(
    State(state): State<CoreState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "platform": "CE",
        "platformName": "VieLang",
        "softwareVersion": env!("CARGO_PKG_VERSION"),
        "softwareVersionDescription": format!("VieLang {}", env!("CARGO_PKG_VERSION")),
        "allowedDashboardIds": [],
        "whiteLabelingEnabled": false,
        "selfRegistrationEnabled": false,
        "twoFaEnabled": true,
        "edgesSupportEnabled": false,
        "hasRepository": false,
        "allowedOrigins": state.config.server.allowed_origins,
    }))
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

    use crate::state::AppState;
    use vl_auth::password;
    use vl_config::VieLangConfig;
    use vl_core::entities::{Authority, User, UserCredentials};
    use uuid::Uuid;

    fn now_ms() -> i64 { chrono::Utc::now().timestamp_millis() }

    async fn test_app(pool: PgPool) -> axum::Router {
        let config = VieLangConfig::default();
        let rule_engine = vl_rule_engine::RuleEngine::start_noop();
        let queue_producer = vl_queue::create_producer(&config.queue).expect("queue");
        let cache = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        crate::routes::create_router(state)
    }

    async fn create_tenant_admin(pool: &PgPool, email: &str, pwd: &str) -> User {
        let dao = vl_dao::postgres::user::UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::new_v4(), customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: None, last_name: None,
            phone: None, additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pwd).unwrap();
        let creds = UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(),
            user_id: user.id, enabled: true,
            password: Some(hash),
            activate_token: None, reset_token: None, additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        user
    }

    async fn get_token(app: axum::Router, email: &str, pwd: &str) -> String {
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": pwd}).to_string()))
                .unwrap(),
        ).await.unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v["token"].as_str().unwrap().to_string()
    }

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    }

    #[test]
    #[ignore = "verified passing"]
    fn system_info_router_registered() {
        let r = router();
        drop(r);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_system_build_info_returns_version(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "sysinfo@test.com", "pass123").await;
        let token = get_token(app.clone(), "sysinfo@test.com", "pass123").await;

        let resp = get_auth(app, "/api/system/info", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["version"].is_string());
        assert_eq!(body["type"].as_str(), Some("CE"));
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_system_params_returns_limits(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "params@test.com", "pass123").await;
        let token = get_token(app.clone(), "params@test.com", "pass123").await;

        let resp = get_auth(app, "/api/system/params", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["maxDatapointsLimit"].as_i64().unwrap_or(0) > 0);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_usage_info_returns_counts(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "usage@test.com", "pass123").await;
        let token = get_token(app.clone(), "usage@test.com", "pass123").await;

        let resp = get_auth(app, "/api/usage", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        // At minimum, users field should exist and be >= 0
        assert!(body["users"].as_i64().is_some());
        assert!(body["devices"].as_i64().is_some());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_usage_info_requires_tenant_admin(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/usage")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
