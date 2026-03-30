/// Partner API — white-label reseller tenant onboarding & usage reporting
///
/// Auth: `X-Partner-Key` header (looked up in admin_settings key='partnerApiKey')
/// These are PUBLIC routes (not behind JWT middleware) but do their own key validation.
use axum::{
    extract::State,
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use vl_core::entities::{AdminSettings, Authority, Tenant, User, UserCredentials};

use crate::{error::ApiError, state::AppState};

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/partner/tenants",        axum::routing::post(onboard_tenant))
        .route("/v1/partner/usage-summary",  get(usage_summary))
        .route("/v1/partner/register-key",   axum::routing::post(register_partner_key))
}

// ── Auth helper ───────────────────────────────────────────────────────────────

async fn authenticate_partner(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Uuid, ApiError> {
    let key = headers
        .get("X-Partner-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing X-Partner-Key header".into()))?;

    state.admin_settings_dao
        .find_tenant_by_partner_key(key)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::Unauthorized("Invalid X-Partner-Key".into()))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardTenantRequest {
    pub company_name:  String,
    pub admin_email:   String,
    pub plan:          Option<String>,
    pub max_devices:   Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardTenantResponse {
    pub tenant_id:        Uuid,
    pub admin_email:      String,
    pub plan:             String,
    pub activation_token: String,
    pub mqtt_broker_url:  String,
    pub login_url:        String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantUsage {
    pub name:            String,
    pub tenant_id:       Uuid,
    pub plan:            String,
    pub devices_active:  i64,
    pub assets:          i64,
    pub users:           i64,
    pub dashboards:      i64,
    pub alarms:          i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummaryResponse {
    pub partner_tenant_id:  Uuid,
    pub billing_period:     String,
    pub tenants:            Vec<TenantUsage>,
    pub total_devices:      i64,
    pub total_tenants:      usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterKeyRequest {
    /// The partner API key to register for this tenant
    pub api_key: String,
    /// The tenant ID this key belongs to
    pub tenant_id: Uuid,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/v1/partner/register-key — (SYS_ADMIN or tenant themselves; no key auth yet)
/// Registers an X-Partner-Key for a given tenant so they can use the Partner API.
/// In production this would be SYS_ADMIN only; for demo it's open.
async fn register_partner_key(
    State(state): State<AppState>,
    Json(req): Json<RegisterKeyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let now = chrono::Utc::now().timestamp_millis();
    let entry = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    req.tenant_id,
        key:          "partnerApiKey".into(),
        json_value:   json!({ "key": req.api_key }),
    };
    state.admin_settings_dao.save(&entry).await?;
    Ok(Json(json!({ "tenantId": req.tenant_id, "apiKey": req.api_key, "registered": true })))
}

/// POST /api/v1/partner/tenants — onboard a new sub-tenant under a partner
async fn onboard_tenant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<OnboardTenantRequest>,
) -> Result<Json<OnboardTenantResponse>, ApiError> {
    let partner_tenant_id = authenticate_partner(&state, &headers).await?;

    let now = chrono::Utc::now().timestamp_millis();

    // 1. Get default tenant profile
    let profile_id = state.tenant_profile_dao.find_default().await?
        .ok_or_else(|| ApiError::Internal("No default tenant profile".into()))?
        .id;

    // 2. Create tenant
    let tenant = Tenant {
        id:                Uuid::new_v4(),
        created_time:      now,
        tenant_profile_id: profile_id,
        title:             payload.company_name.clone(),
        region:            None,
        country:           None,
        state:             None,
        city:              None,
        address:           None,
        address2:          None,
        zip:               None,
        phone:             None,
        email:             Some(payload.admin_email.clone()),
        additional_info:   None,
        version:           1,
    };
    let saved_tenant = state.tenant_dao.save(&tenant).await?;

    // 3. Record this tenant's parent partner
    let parent_entry = AdminSettings {
        id:           Uuid::new_v4(),
        created_time: now,
        tenant_id:    saved_tenant.id,
        key:          "partnerTenantId".into(),
        json_value:   json!({
            "tenantId": partner_tenant_id.to_string(),
            "plan":     payload.plan.clone().unwrap_or_else(|| "BASIC".into()),
            "maxDevices": payload.max_devices.unwrap_or(100),
        }),
    };
    let _ = state.admin_settings_dao.save(&parent_entry).await;

    // 4. Create TENANT_ADMIN user
    let activation_token = Uuid::new_v4().to_string().replace('-', "");
    let user = User {
        id:              Uuid::new_v4(),
        created_time:    now,
        tenant_id:       saved_tenant.id,
        customer_id:     None,
        email:           payload.admin_email.clone(),
        authority:       Authority::TenantAdmin,
        first_name:      Some("Tenant".into()),
        last_name:       Some("Admin".into()),
        phone:           None,
        additional_info: None,
        version:         1,
    };
    state.user_dao.save(&user).await?;
    let creds = UserCredentials {
        id:              Uuid::new_v4(),
        created_time:    now,
        user_id:         user.id,
        enabled:         false, // Must activate via token
        password:        None,
        activate_token:  Some(activation_token.clone()),
        reset_token:     None,
        additional_info: None,
    };
    state.user_dao.save_credentials(&creds).await?;

    Ok(Json(OnboardTenantResponse {
        tenant_id:        saved_tenant.id,
        admin_email:      payload.admin_email,
        plan:             payload.plan.unwrap_or_else(|| "BASIC".into()),
        activation_token,
        mqtt_broker_url:  "mqtt://localhost:1883".into(),
        login_url:        "http://localhost:8080".into(),
    }))
}

/// GET /api/v1/partner/usage-summary — return usage for all sub-tenants
async fn usage_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UsageSummaryResponse>, ApiError> {
    let partner_tenant_id = authenticate_partner(&state, &headers).await?;

    // Find all sub-tenants
    let sub_tenant_ids = state.admin_settings_dao
        .find_partner_sub_tenants(partner_tenant_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let billing_period = chrono::Utc::now().format("%Y-%m").to_string();
    let mut tenants = Vec::new();
    let mut total_devices = 0i64;

    for tid in &sub_tenant_ids {
        let usage = state.usage_info_dao
            .get_tenant_usage(*tid)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        // Get plan from admin_settings
        let plan = state.admin_settings_dao
            .find_by_key(*tid, "partnerTenantId")
            .await
            .ok()
            .flatten()
            .and_then(|s| s.json_value.get("plan").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_else(|| "BASIC".into());

        // Get tenant title
        let name = state.tenant_dao
            .find_by_id(*tid)
            .await
            .ok()
            .flatten()
            .map(|t| t.title)
            .unwrap_or_else(|| tid.to_string());

        total_devices += usage.devices;
        tenants.push(TenantUsage {
            name,
            tenant_id:      *tid,
            plan,
            devices_active: usage.devices,
            assets:         usage.assets,
            users:          usage.users,
            dashboards:     usage.dashboards,
            alarms:         usage.alarms,
        });
    }

    Ok(Json(UsageSummaryResponse {
        partner_tenant_id,
        billing_period,
        tenants,
        total_devices,
        total_tenants: sub_tenant_ids.len(),
    }))
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;

    use vl_auth::password;
    use vl_config::VieLangConfig;
    use vl_core::entities::{Authority, User, UserCredentials as UC};
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

    async fn create_tenant_admin_with_id(pool: &PgPool, email: &str, pwd: &str, tenant_id: Uuid) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(), tenant_id,
            customer_id: None, email: email.into(), authority: Authority::TenantAdmin,
            first_name: Some("Partner".into()), last_name: Some("Admin".into()),
            phone: None, additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pwd).unwrap();
        let creds = UC {
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

    async fn post_json_key(app: axum::Router, uri: &str, key: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("X-Partner-Key", key)
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn get_key(app: axum::Router, uri: &str, key: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("X-Partner-Key", key)
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn partner_onboard_tenant_requires_valid_key(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        // No key
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/v1/partner/tenants")
                .header("content-type", "application/json")
                .body(Body::from(json!({"companyName":"Test","adminEmail":"a@b.com"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn partner_can_register_key_and_onboard_tenants(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        // Step 1: Register a partner API key for a tenant
        let partner_tenant_id = Uuid::new_v4();
        create_tenant_admin_with_id(&pool, "partner@acme.com", "pass123", partner_tenant_id).await;

        let reg_resp = post_json(
            app.clone(),
            "/api/v1/partner/register-key",
            json!({ "tenantId": partner_tenant_id, "apiKey": "acme-test-key-001" }),
        ).await;
        assert_eq!(reg_resp.status(), StatusCode::OK);
        let reg_body = body_json(reg_resp).await;
        assert_eq!(reg_body["registered"], true);

        // Step 2: Onboard a sub-tenant using the key
        let onboard_resp = post_json_key(
            app.clone(),
            "/api/v1/partner/tenants",
            "acme-test-key-001",
            json!({
                "companyName": "Factory XYZ",
                "adminEmail":  "admin@factory-xyz.vn",
                "plan":        "BUSINESS",
                "maxDevices":  500
            }),
        ).await;
        assert_eq!(onboard_resp.status(), StatusCode::OK);
        let onboard_body = body_json(onboard_resp).await;
        assert!(onboard_body["tenantId"].is_string(), "tenantId must be present");
        assert_eq!(onboard_body["adminEmail"], "admin@factory-xyz.vn");
        assert_eq!(onboard_body["plan"], "BUSINESS");
        assert!(onboard_body["activationToken"].is_string(), "activationToken must be present");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn partner_usage_summary_returns_sub_tenants(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        // Register partner + key
        let partner_tenant_id = Uuid::new_v4();
        create_tenant_admin_with_id(&pool, "partner-usage@acme.com", "pass123", partner_tenant_id).await;

        post_json(app.clone(), "/api/v1/partner/register-key",
            json!({ "tenantId": partner_tenant_id, "apiKey": "acme-usage-key" })).await;

        // Onboard 2 tenants
        post_json_key(app.clone(), "/api/v1/partner/tenants", "acme-usage-key",
            json!({"companyName": "Tenant A", "adminEmail": "a@a.com", "plan": "BASIC"})).await;
        post_json_key(app.clone(), "/api/v1/partner/tenants", "acme-usage-key",
            json!({"companyName": "Tenant B", "adminEmail": "b@b.com", "plan": "BUSINESS"})).await;

        // Get usage summary
        let summary_resp = get_key(app.clone(), "/api/v1/partner/usage-summary", "acme-usage-key").await;
        assert_eq!(summary_resp.status(), StatusCode::OK);

        let summary = body_json(summary_resp).await;
        assert_eq!(summary["totalTenants"], 2);
        assert!(summary["billingPeriod"].is_string());
        assert!(summary["tenants"].is_array());
        let tenants = summary["tenants"].as_array().unwrap();
        assert_eq!(tenants.len(), 2);
        // Each tenant entry has required fields
        for t in tenants {
            assert!(t["name"].is_string());
            assert!(t["tenantId"].is_string());
            assert!(t["plan"].is_string());
            assert!(t["devicesActive"].is_number());
        }
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn partner_usage_summary_requires_valid_key(pool: PgPool) {
        let app = test_app(pool.clone()).await;

        let resp = get_key(app, "/api/v1/partner/usage-summary", "invalid-key-xyz").await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
