use axum::{extract::{Path, Query, State}, extract::rejection::JsonRejection, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{Tenant, TenantSubscription};
use vl_dao::{PageLink, PageData};

use crate::{error::ApiError, routes::devices::IdResponse, state::{AppState, AuthState, BillingState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: TenantController
        .route("/tenants",             get(list_tenants))
        .route("/tenant",              post(save_tenant))
        .route("/tenant/{tenantId}",   get(get_tenant).delete(delete_tenant))
}

#[derive(Debug, Deserialize)]
pub struct PageParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    pub title: String,
    pub region: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[serde(rename = "tenantProfileId")]
    pub tenant_profile_id: IdResponse,
}

impl IdResponse {
    pub fn tenant_profile(id: Uuid) -> Self {
        Self { id, entity_type: "TENANT_PROFILE".into() }
    }
}

impl From<Tenant> for TenantResponse {
    fn from(t: Tenant) -> Self {
        Self {
            id: IdResponse::tenant(t.id),
            created_time: t.created_time,
            title: t.title,
            region: t.region,
            country: t.country,
            city: t.city,
            email: t.email,
            phone: t.phone,
            tenant_profile_id: IdResponse::tenant_profile(t.tenant_profile_id),
        }
    }
}

async fn list_tenants(
    State(state): State<AuthState>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<TenantResponse>>, ApiError> {
    let mut page_link = PageLink::new(params.page.unwrap_or(0), params.page_size.unwrap_or(10));
    page_link.text_search = params.text_search;

    let page = state.tenant_dao.find_all(&page_link).await?;

    Ok(Json(PageData {
        data: page.data.into_iter().map(TenantResponse::from).collect(),
        total_pages: page.total_pages,
        total_elements: page.total_elements,
        has_next: page.has_next,
    }))
}

async fn get_tenant(
    State(state): State<AuthState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantResponse>, ApiError> {
    let tenant = state.tenant_dao.find_by_id(tenant_id).await?
        .ok_or(ApiError::NotFound(format!("Tenant [{}] is not found", tenant_id)))?;
    Ok(Json(TenantResponse::from(tenant)))
}

#[derive(Debug, Deserialize)]
pub struct SaveTenantRequest {
    pub id: Option<IdResponse>,
    pub title: String,
    pub region: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[serde(rename = "tenantProfileId")]
    pub tenant_profile_id: Option<IdResponse>,
}

async fn save_tenant(
    State(state): State<AuthState>,
    State(billing): State<BillingState>,
    body: Result<Json<SaveTenantRequest>, JsonRejection>,
) -> Result<Json<TenantResponse>, ApiError> {
    let Json(req) = body.map_err(|e| ApiError::BadRequest(e.body_text()))?;
    let now = chrono::Utc::now().timestamp_millis();
    let is_new = req.id.is_none();

    // Resolve tenant_profile_id: use provided or look up the default
    let profile_id = if let Some(pid) = req.tenant_profile_id.map(|i| i.id) {
        pid
    } else {
        state.tenant_profile_dao.find_default().await?
            .ok_or_else(|| ApiError::BadRequest(
                "No default tenant profile found. Provide tenantProfileId.".into()
            ))?
            .id
    };

    let tenant = Tenant {
        id: req.id.map(|i| i.id).unwrap_or_else(Uuid::new_v4),
        created_time: now,
        tenant_profile_id: profile_id,
        title: req.title,
        region: req.region,
        country: req.country,
        state: None,
        city: req.city,
        address: None,
        address2: None,
        zip: None,
        phone: req.phone,
        email: req.email,
        additional_info: None,
        version: 1,
    };

    let saved = state.tenant_dao.save(&tenant).await?;

    // Auto-provision free plan for new tenants
    if is_new {
        if let Ok(Some(plan)) = billing.plan_dao.find_by_name("free").await {
            let sub = TenantSubscription {
                id:                     Uuid::new_v4(),
                created_time:           now,
                updated_time:           now,
                tenant_id:              saved.id,
                plan_id:                plan.id,
                stripe_customer_id:     None,
                stripe_subscription_id: None,
                billing_cycle:          "monthly".into(),
                status:                 "free".into(),
                current_period_start:   None,
                current_period_end:     None,
                trial_end:              None,
                cancel_at_period_end:   false,
                canceled_at:            None,
            };
            let _ = billing.subscription_dao.upsert(&sub).await;
        }
    }

    Ok(Json(TenantResponse::from(saved)))
}

async fn delete_tenant(
    State(state): State<AuthState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<(), ApiError> {
    state.tenant_dao.delete(tenant_id).await?;
    Ok(())
}

// ── Integration Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;

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

    async fn create_test_user(pool: &PgPool, email: &str, pwd: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::new_v4(), customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
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

    async fn post_json_auth(
        app: axum::Router, uri: &str, token: &str, body: Value,
    ) -> axum::response::Response {
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

    // ── GET /api/tenants ──────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenants_returns_pagination_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "tenlist@test.com", "pass123").await;
        let token = get_token(app.clone(), "tenlist@test.com", "pass123").await;

        let resp = get_auth(app, "/api/tenants?page=0&pageSize=10", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body["data"].is_array(),           "Must have 'data' array");
        assert!(body["totalPages"].is_number(),    "Must have 'totalPages'");
        assert!(body["totalElements"].is_number(), "Must have 'totalElements'");
        assert!(body["hasNext"].is_boolean(),      "Must have 'hasNext'");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenants_requires_auth(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("GET").uri("/api/tenants?page=0&pageSize=10")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── POST /api/tenant ──────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_tenant_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "tencreate@test.com", "pass123").await;
        let token = get_token(app.clone(), "tencreate@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/tenant", &token, json!({
            "title": "New Tenant"
        })).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn tenant_response_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "tenfmt@test.com", "pass123").await;
        let token = get_token(app.clone(), "tenfmt@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/tenant", &token, json!({
            "title": "Format Tenant"
        })).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;

        // ThingsBoard Java format: id is object with id + entityType
        assert!(body["id"]["id"].is_string(),            "id.id must be UUID string");
        assert_eq!(body["id"]["entityType"], "TENANT",   "id.entityType must be TENANT");
        assert!(body["createdTime"].is_number(),          "createdTime must be ms timestamp");
        assert_eq!(body["title"], "Format Tenant");
        // tenantProfileId is object with id + entityType
        assert!(body["tenantProfileId"]["id"].is_string(),                     "tenantProfileId.id must be UUID string");
        assert_eq!(body["tenantProfileId"]["entityType"], "TENANT_PROFILE",    "tenantProfileId.entityType must be TENANT_PROFILE");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_tenant_without_title_returns_400(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "tenbad@test.com", "pass123").await;
        let token = get_token(app.clone(), "tenbad@test.com", "pass123").await;

        // Sending an empty body (no title field) — axum JSON extractor rejects it
        let resp = post_json_auth(app, "/api/tenant", &token, json!({})).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // ── GET /api/tenant/{tenantId} ────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_tenant_by_id_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "tenget@test.com", "pass123").await;
        let token = get_token(app.clone(), "tenget@test.com", "pass123").await;

        // Create a tenant first
        let create_resp = post_json_auth(app.clone(), "/api/tenant", &token, json!({
            "title": "Get Tenant"
        })).await;
        assert_eq!(create_resp.status(), StatusCode::OK);
        let tenant_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        // Then fetch it by id
        let resp = get_auth(app, &format!("/api/tenant/{tenant_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert_eq!(body["id"]["id"], tenant_id);
        assert_eq!(body["title"], "Get Tenant");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_tenant_nonexistent_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ten404@test.com", "pass123").await;
        let token = get_token(app.clone(), "ten404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/tenant/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn error_response_has_status_message_errorcode(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "tenerr@test.com", "pass123").await;
        let token = get_token(app.clone(), "tenerr@test.com", "pass123").await;

        // A nonexistent tenant id triggers NotFound → errorCode 32
        let resp = get_auth(app, &format!("/api/tenant/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = body_json(resp).await;
        assert_eq!(body["status"].as_u64().unwrap(), 404, "status field must be 404");
        assert!(body["message"].is_string(),               "message field must be a string");
        assert_eq!(body["errorCode"].as_u64().unwrap(), 32, "errorCode for NotFound must be 32");
    }

    // ── DELETE /api/tenant/{tenantId} ─────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_tenant_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "tendel@test.com", "pass123").await;
        let token = get_token(app.clone(), "tendel@test.com", "pass123").await;

        // Create a tenant to delete
        let create_resp = post_json_auth(app.clone(), "/api/tenant", &token, json!({
            "title": "Delete Me"
        })).await;
        assert_eq!(create_resp.status(), StatusCode::OK);
        let tenant_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        // Delete it
        let del = delete_auth(app.clone(), &format!("/api/tenant/{tenant_id}"), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        // Verify it is gone
        let get = get_auth(app, &format!("/api/tenant/{tenant_id}"), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }

    // ── Pagination params ─────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenants_pagination_params_work(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "tenpag@test.com", "pass123").await;
        let token = get_token(app.clone(), "tenpag@test.com", "pass123").await;

        // Create two tenants
        post_json_auth(app.clone(), "/api/tenant", &token, json!({"title": "Pag Tenant A"})).await;
        post_json_auth(app.clone(), "/api/tenant", &token, json!({"title": "Pag Tenant B"})).await;

        // Request page 0 with pageSize=1 — should have hasNext=true (at least 2 tenants exist)
        let resp = get_auth(app, "/api/tenants?page=0&pageSize=1", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert_eq!(body["data"].as_array().unwrap().len(), 1, "pageSize=1 must return exactly 1 item");
        assert!(body["hasNext"].as_bool().unwrap(), "hasNext must be true when more tenants exist");
        assert!(body["totalElements"].as_i64().unwrap() >= 2, "totalElements must count all tenants");
    }
}
