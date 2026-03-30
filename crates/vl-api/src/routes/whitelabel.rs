use axum::{
    extract::{Extension, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use vl_core::entities::AdminSettings;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AdminState}};

// ── Routers ───────────────────────────────────────────────────────────────────

/// Protected routes — require JWT (TENANT_ADMIN or SYS_ADMIN)
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/whitelabel/settings", get(get_branding).post(save_branding))
}

/// Public route — no auth required (used by login page to fetch branding)
pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/noauth/whitelabel/loginTheme", get(login_theme))
        // Flutter PE ThingsBoard app expects this exact camelCase path
        .route("/noauth/whiteLabel/loginWhiteLabelParams", get(login_theme))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrandingConfig {
    /// Product name shown to end customers
    pub product_name:           Option<String>,
    pub company_name:           Option<String>,
    pub logo_url:               Option<String>,
    pub logo_url_dark:          Option<String>,
    pub favicon:                Option<String>,
    pub primary_color:          Option<String>,
    pub accent_color:           Option<String>,
    pub support_email:          Option<String>,
    pub support_phone:          Option<String>,
    pub documentation_url:      Option<String>,
    pub terms_url:              Option<String>,
    pub privacy_url:            Option<String>,
    pub footer_text:            Option<String>,
    #[serde(rename = "hideVieLangBranding", default)]
    pub hide_vielang_branding: bool,
    pub custom_css:             Option<String>,
    /// The custom domain this branding applies to (e.g. factory.acme.vn)
    pub domain:                 Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginThemeQuery {
    pub domain: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/whitelabel/settings — get current tenant's white-label branding
async fn get_branding(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Value>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    let settings = state.admin_settings_dao
        .find_by_key(ctx.tenant_id, "platformBranding")
        .await?
        .map(|s| s.json_value)
        .unwrap_or(Value::Object(Default::default()));
    Ok(Json(settings))
}

/// POST /api/whitelabel/settings — save white-label branding for current tenant
async fn save_branding(
    State(state): State<AdminState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(body): Json<BrandingConfig>,
) -> Result<Json<Value>, ApiError> {
    if !ctx.is_tenant_admin() && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }
    let now = chrono::Utc::now().timestamp_millis();
    let json_value = serde_json::to_value(&body)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Also register the domain for loginTheme lookup
    if let Some(domain) = &body.domain {
        let domain_entry = AdminSettings {
            id:           Uuid::new_v4(),
            created_time: now,
            tenant_id:    ctx.tenant_id,
            key:          "whitelabelDomain".into(),
            json_value:   serde_json::json!({ "domain": domain }),
        };
        let _ = state.admin_settings_dao.save(&domain_entry).await;
    }

    let entry = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    ctx.tenant_id,
        key:          "platformBranding".into(),
        json_value:   json_value.clone(),
    };
    state.admin_settings_dao.save(&entry).await?;
    Ok(Json(json_value))
}

/// GET /api/noauth/whitelabel/loginTheme?domain=xxx — public, returns branding for a domain
async fn login_theme(
    State(state): State<AdminState>,
    Query(params): Query<LoginThemeQuery>,
) -> Result<Json<Value>, ApiError> {
    let Some(domain) = params.domain else {
        // No domain → return empty branding (default VieLang UI)
        return Ok(Json(Value::Object(Default::default())));
    };

    let Some(tenant_id) = state.admin_settings_dao
        .find_tenant_by_whitelabel_domain(&domain)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))? else {
        // Domain not registered → empty branding
        return Ok(Json(Value::Object(Default::default())));
    };

    let branding = state.admin_settings_dao
        .find_by_key(tenant_id, "platformBranding")
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map(|s| s.json_value)
        .unwrap_or(Value::Object(Default::default()));

    Ok(Json(branding))
}

// ── Tests ──────────────────────────────────────────────────────────────────────

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
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, {
            let (tx, _) = tokio::sync::mpsc::channel(1); tx
        });
        create_router(state)
    }

    async fn create_tenant_admin(pool: &PgPool, email: &str, pwd: &str) -> (User, Uuid) {
        let dao = UserDao::new(pool.clone());
        let tenant_id = Uuid::new_v4();
        let user = User {
            id:              Uuid::new_v4(),
            created_time:    now_ms(),
            tenant_id,
            customer_id:     None,
            email:           email.into(),
            authority:       Authority::TenantAdmin,
            first_name:      Some("ACME".into()),
            last_name:       Some("Admin".into()),
            phone:           None,
            additional_info: None,
            version:         1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pwd).unwrap();
        let creds = UserCredentials {
            id:              Uuid::new_v4(),
            created_time:    now_ms(),
            user_id:         user.id,
            enabled:         true,
            password:        Some(hash),
            activate_token:  None,
            reset_token:     None,
            additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        (user, tenant_id)
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

    async fn get_no_auth(app: axum::Router, uri: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    async fn get_token(app: axum::Router, email: &str, pwd: &str) -> String {
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": pwd}).to_string()))
                .unwrap(),
        ).await.unwrap();
        body_json(resp).await["token"].as_str().unwrap().to_string()
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn save_and_get_branding_round_trip(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "acme@test.com", "pass123").await;
        let token = get_token(app.clone(), "acme@test.com", "pass123").await;

        let branding = json!({
            "productName": "Smart Factory Pro",
            "companyName": "ACME Technology",
            "primaryColor": "#1976D2",
            "accentColor": "#FF5722",
            "supportEmail": "support@acme.vn",
            "footerText": "© 2026 ACME Technology",
            "hideVieLangBranding": true,
            "domain": "factory.acme.vn"
        });

        // Save branding
        let save_resp = post_json_auth(app.clone(), "/api/whitelabel/settings", &token, branding).await;
        assert_eq!(save_resp.status(), StatusCode::OK, "save branding must return 200");

        let saved = body_json(save_resp).await;
        assert_eq!(saved["productName"], "Smart Factory Pro");
        assert_eq!(saved["companyName"], "ACME Technology");
        assert_eq!(saved["primaryColor"], "#1976D2");
        assert_eq!(saved["hideVieLangBranding"], true);

        // Retrieve branding
        let get_resp = get_auth(app.clone(), "/api/whitelabel/settings", &token).await;
        assert_eq!(get_resp.status(), StatusCode::OK, "get branding must return 200");

        let fetched = body_json(get_resp).await;
        assert_eq!(fetched["productName"], "Smart Factory Pro");
        assert_eq!(fetched["companyName"], "ACME Technology");
        assert_eq!(fetched["supportEmail"], "support@acme.vn");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn get_branding_returns_empty_when_not_set(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "empty@test.com", "pass123").await;
        let token = get_token(app.clone(), "empty@test.com", "pass123").await;

        let resp = get_auth(app, "/api/whitelabel/settings", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body.is_object(), "should return empty object when no branding set");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn login_theme_returns_branding_for_registered_domain(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_tenant_admin(&pool, "domain-wl@test.com", "pass123").await;
        let token = get_token(app.clone(), "domain-wl@test.com", "pass123").await;

        // Register branding with domain
        let branding = json!({
            "productName": "Domain Brand",
            "primaryColor": "#00FF00",
            "domain": "myplatform.example.com"
        });
        let save = post_json_auth(app.clone(), "/api/whitelabel/settings", &token, branding).await;
        assert_eq!(save.status(), StatusCode::OK);

        // Fetch login theme by domain (public endpoint, no auth)
        let resp = get_no_auth(
            app.clone(),
            "/api/noauth/whitelabel/loginTheme?domain=myplatform.example.com",
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert_eq!(body["productName"], "Domain Brand");
        assert_eq!(body["primaryColor"], "#00FF00");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn login_theme_returns_empty_for_unknown_domain(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        let resp = get_no_auth(app, "/api/noauth/whitelabel/loginTheme?domain=unknown.notregistered.io").await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body.is_object());
        // Must be empty — no branding registered for this domain
        assert_eq!(body.as_object().unwrap().len(), 0);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn branding_requires_admin_auth(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        // No auth
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/whitelabel/settings")
                .body(Body::empty()).unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
