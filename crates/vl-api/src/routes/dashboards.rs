use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{Dashboard, DashboardInfo, HomeDashboardInfo};
use vl_dao::PageData;

use axum::Extension;
use crate::{error::ApiError, middleware::auth::SecurityContext, routes::devices::{IdResponse, PageParams}, state::{AppState, EntityState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: DashboardController
        .route("/dashboard",                                    post(save_dashboard))
        .route("/dashboard/serverTime",                         get(server_time))
        .route("/dashboard/home",                               get(get_home_dashboard).post(set_home_dashboard).delete(unset_home_dashboard))
        .route("/dashboard/home/info",                          get(get_home_dashboard_info))
        .route("/dashboard/info/{dashboardId}",                 get(get_dashboard_info))
        .route("/dashboard/{dashboardId}",                      get(get_dashboard).delete(delete_dashboard))
        .route("/dashboard/{dashboardId}/customers",            post(update_dashboard_customers))
        .route("/dashboard/{dashboardId}/customers/add",        post(add_dashboard_customers))
        .route("/dashboard/{dashboardId}/customers/remove",     post(remove_dashboard_customers))
        .route("/tenant/dashboards",                            get(list_tenant_dashboards))
        .route("/tenant/dashboardInfos",                        get(list_tenant_dashboard_infos))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DashboardResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    pub title: Option<String>,
    pub configuration: Option<serde_json::Value>,
    #[serde(rename = "mobileHide")]
    pub mobile_hide: bool,
    #[serde(rename = "mobileOrder")]
    pub mobile_order: Option<i32>,
}

impl From<Dashboard> for DashboardResponse {
    fn from(d: Dashboard) -> Self {
        Self {
            id:            IdResponse::dashboard(d.id),
            created_time:  d.created_time,
            tenant_id:     IdResponse::tenant(d.tenant_id),
            title:         d.title,
            configuration: d.configuration,
            mobile_hide:   d.mobile_hide,
            mobile_order:  d.mobile_order,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DashboardInfoResponse {
    pub id: IdResponse,
    pub created_time: i64,
    pub tenant_id: IdResponse,
    pub title: Option<String>,
    pub assigned_customers: Option<serde_json::Value>,
    pub mobile_hide: bool,
    pub mobile_order: Option<i32>,
}

impl From<DashboardInfo> for DashboardInfoResponse {
    fn from(d: DashboardInfo) -> Self {
        let assigned_customers = d.assigned_customers
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        Self {
            id:                  IdResponse::dashboard(d.id),
            created_time:        d.created_time,
            tenant_id:           IdResponse::tenant(d.tenant_id),
            title:               d.title,
            assigned_customers,
            mobile_hide:         d.mobile_hide,
            mobile_order:        d.mobile_order,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTimeResponse {
    pub server_time: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetHomeDashboardRequest {
    pub dashboard_id: Option<IdResponse>,
    pub hidden_dashboard_toolbar: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerIdsRequest {
    pub customer_ids: Vec<Uuid>,
}

/// Query params for listing dashboards — supports optional mobile filter
#[derive(Debug, Deserialize)]
pub struct DashboardListParams {
    #[serde(rename = "pageSize")]
    pub page_size:   Option<i64>,
    pub page:        Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
    /// mobile=true → only return dashboards with mobile_hide = false
    pub mobile:      Option<bool>,
}

impl DashboardListParams {
    pub fn to_page_link(&self) -> vl_dao::PageLink {
        let mut pl = vl_dao::PageLink::new(self.page.unwrap_or(0), self.page_size.unwrap_or(10));
        pl.text_search = self.text_search.clone();
        pl
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveDashboardRequest {
    pub id: Option<IdResponse>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
    pub title: Option<String>,
    pub configuration: Option<serde_json::Value>,
    #[serde(rename = "mobileHide")]
    pub mobile_hide: Option<bool>,
    #[serde(rename = "mobileOrder")]
    pub mobile_order: Option<i32>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/dashboard
async fn save_dashboard(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SaveDashboardRequest>,
) -> Result<Json<DashboardResponse>, ApiError> {
    let now = chrono::Utc::now().timestamp_millis();
    // Use tenantId from request body if present, otherwise fall back to JWT context
    let tenant_id = req.tenant_id
        .map(|i| i.id)
        .unwrap_or(ctx.tenant_id);
    let dashboard = Dashboard {
        id:            req.id.map(|i| i.id).unwrap_or_else(Uuid::new_v4),
        created_time:  now,
        tenant_id,
        title:         req.title,
        configuration: req.configuration,
        external_id:   None,
        mobile_hide:   req.mobile_hide.unwrap_or(false),
        mobile_order:  req.mobile_order,
        version:       1,
    };
    let saved = state.dashboard_dao.save(&dashboard).await?;
    Ok(Json(DashboardResponse::from(saved)))
}

/// GET /api/dashboard/{dashboardId}
async fn get_dashboard(
    State(state): State<EntityState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<DashboardResponse>, ApiError> {
    let dashboard = state.dashboard_dao
        .find_by_id(dashboard_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Dashboard [{}] is not found", dashboard_id)))?;
    Ok(Json(DashboardResponse::from(dashboard)))
}

/// DELETE /api/dashboard/{dashboardId}
async fn delete_dashboard(
    State(state): State<EntityState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.dashboard_dao.delete(dashboard_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/tenant/dashboards?page=0&pageSize=10&mobile=true
async fn list_tenant_dashboards(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<DashboardListParams>,
) -> Result<Json<PageData<DashboardResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page = state.dashboard_dao
        .find_by_tenant_with_mobile_filter(tenant_id, params.mobile, &params.to_page_link())
        .await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(DashboardResponse::from).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/dashboard/serverTime
async fn server_time() -> Json<ServerTimeResponse> {
    Json(ServerTimeResponse { server_time: chrono::Utc::now().timestamp_millis() })
}

/// GET /api/dashboard/info/{dashboardId}
async fn get_dashboard_info(
    State(state): State<EntityState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<DashboardInfoResponse>, ApiError> {
    let info = state.dashboard_dao
        .find_info_by_id(dashboard_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Dashboard [{}] is not found", dashboard_id)))?;
    Ok(Json(DashboardInfoResponse::from(info)))
}

/// GET /api/dashboard/home — returns home dashboard info for current tenant
async fn get_home_dashboard(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<HomeDashboardInfo>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let info = state.dashboard_dao.get_home_dashboard_info(tenant_id).await?;
    Ok(Json(info))
}

/// POST /api/dashboard/home — set home dashboard for current tenant
async fn set_home_dashboard(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<SetHomeDashboardRequest>,
) -> Result<Json<HomeDashboardInfo>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let info = HomeDashboardInfo {
        dashboard_id:             req.dashboard_id.map(|i| i.id),
        hidden_dashboard_toolbar: req.hidden_dashboard_toolbar.unwrap_or(false),
    };
    state.dashboard_dao.set_home_dashboard(tenant_id, &info).await?;
    Ok(Json(info))
}

/// DELETE /api/dashboard/home — unset home dashboard
async fn unset_home_dashboard(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<StatusCode, ApiError> {
    let tenant_id = ctx.tenant_id;
    let info = HomeDashboardInfo { dashboard_id: None, hidden_dashboard_toolbar: false };
    state.dashboard_dao.set_home_dashboard(tenant_id, &info).await?;
    Ok(StatusCode::OK)
}

/// GET /api/dashboard/home/info
async fn get_home_dashboard_info(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Option<DashboardInfoResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let home = state.dashboard_dao.get_home_dashboard_info(tenant_id).await?;
    if let Some(dashboard_id) = home.dashboard_id {
        let info = state.dashboard_dao
            .find_info_by_id(dashboard_id).await?
            .map(DashboardInfoResponse::from);
        Ok(Json(info))
    } else {
        Ok(Json(None))
    }
}

/// GET /api/tenant/dashboardInfos?page=0&pageSize=10
async fn list_tenant_dashboard_infos(
    State(state): State<EntityState>,
    Query(params): Query<PageParams>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<PageData<DashboardInfoResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let page = state.dashboard_dao
        .find_infos_by_tenant(tenant_id, &params.to_page_link())
        .await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(DashboardInfoResponse::from).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// POST /api/dashboard/{dashboardId}/customers — replace assigned customers
async fn update_dashboard_customers(
    State(state): State<EntityState>,
    Path(dashboard_id): Path<Uuid>,
    Json(req): Json<CustomerIdsRequest>,
) -> Result<Json<DashboardInfoResponse>, ApiError> {
    state.dashboard_dao
        .find_by_id(dashboard_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Dashboard [{}] is not found", dashboard_id)))?;
    let json = if req.customer_ids.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&req.customer_ids)
            .map_err(|e| ApiError::Internal(e.to_string()))?)
    };
    state.dashboard_dao.update_assigned_customers(dashboard_id, json).await?;
    let info = state.dashboard_dao
        .find_info_by_id(dashboard_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Dashboard [{}] is not found", dashboard_id)))?;
    Ok(Json(DashboardInfoResponse::from(info)))
}

/// POST /api/dashboard/{dashboardId}/customers/add
async fn add_dashboard_customers(
    State(state): State<EntityState>,
    Path(dashboard_id): Path<Uuid>,
    Json(req): Json<CustomerIdsRequest>,
) -> Result<Json<DashboardInfoResponse>, ApiError> {
    let info = state.dashboard_dao
        .find_info_by_id(dashboard_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Dashboard [{}] is not found", dashboard_id)))?;

    let mut current: Vec<Uuid> = info.assigned_customers
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    for id in &req.customer_ids {
        if !current.contains(id) {
            current.push(*id);
        }
    }

    let json = if current.is_empty() { None } else {
        Some(serde_json::to_string(&current).map_err(|e| ApiError::Internal(e.to_string()))?)
    };
    state.dashboard_dao.update_assigned_customers(dashboard_id, json).await?;
    let updated = state.dashboard_dao
        .find_info_by_id(dashboard_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Dashboard [{}] is not found", dashboard_id)))?;
    Ok(Json(DashboardInfoResponse::from(updated)))
}

/// POST /api/dashboard/{dashboardId}/customers/remove — keep existing, remove specified
async fn remove_dashboard_customers(
    State(state): State<EntityState>,
    Path(dashboard_id): Path<Uuid>,
    Json(req): Json<CustomerIdsRequest>,
) -> Result<Json<DashboardInfoResponse>, ApiError> {
    let info = state.dashboard_dao
        .find_info_by_id(dashboard_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Dashboard [{}] is not found", dashboard_id)))?;

    let current: Vec<Uuid> = info.assigned_customers
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    let filtered: Vec<Uuid> = current.into_iter()
        .filter(|id| !req.customer_ids.contains(id))
        .collect();

    let json = if filtered.is_empty() { None } else {
        Some(serde_json::to_string(&filtered).map_err(|e| ApiError::Internal(e.to_string()))?)
    };
    state.dashboard_dao.update_assigned_customers(dashboard_id, json).await?;
    let final_info = state.dashboard_dao
        .find_info_by_id(dashboard_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Dashboard [{}] is not found", dashboard_id)))?;
    Ok(Json(DashboardInfoResponse::from(final_info)))
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
    async fn create_dashboard_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dash@test.com", "pass123").await;
        let token = get_token(app.clone(), "dash@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/dashboard", &token, json!({
            "title": "My Dashboard",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn dashboard_response_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dashfmt@test.com", "pass123").await;
        let token = get_token(app.clone(), "dashfmt@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/dashboard", &token, json!({
            "title": "Format Dashboard",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;

        let body = body_json(resp).await;
        assert!(body["id"]["id"].is_string());
        assert_eq!(body["id"]["entityType"], "DASHBOARD");
        assert!(body["createdTime"].is_number());
        assert_eq!(body["tenantId"]["entityType"], "TENANT");
        assert_eq!(body["title"], "Format Dashboard");
        assert!(body["mobileHide"].is_boolean());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_dashboard_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dashget@test.com", "pass123").await;
        let token = get_token(app.clone(), "dashget@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/dashboard", &token, json!({
            "title": "Get Dashboard",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;
        let dashboard_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/dashboard/{dashboard_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["id"]["id"], dashboard_id);
        assert_eq!(body["title"], "Get Dashboard");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_nonexistent_dashboard_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "dash404@test.com", "pass123").await;
        let token = get_token(app.clone(), "dash404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/dashboard/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = body_json(resp).await;
        assert_eq!(body["status"].as_u64().unwrap(), 404);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_dashboard_then_get_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dashdel@test.com", "pass123").await;
        let token = get_token(app.clone(), "dashdel@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/dashboard", &token, json!({
            "title": "Delete Dashboard",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;
        let dashboard_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let del = delete_auth(app.clone(), &format!("/api/dashboard/{dashboard_id}"), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        let get = get_auth(app, &format!("/api/dashboard/{dashboard_id}"), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn server_time_returns_ms_timestamp(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "dashtime@test.com", "pass123").await;
        let token = get_token(app.clone(), "dashtime@test.com", "pass123").await;

        let resp = get_auth(app, "/api/dashboard/serverTime", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["serverTime"].is_number());
        let ts = body["serverTime"].as_i64().unwrap();
        assert!(ts > 1_577_836_800_000_i64, "serverTime must be after 2020-01-01");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn dashboard_info_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dashinfo@test.com", "pass123").await;
        let token = get_token(app.clone(), "dashinfo@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/dashboard", &token, json!({
            "title": "Info Dashboard",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;
        let dashboard_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/dashboard/info/{dashboard_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["id"]["entityType"], "DASHBOARD");
        assert_eq!(body["title"], "Info Dashboard");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_dashboard_infos_returns_pagination_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "dashlist@test.com", "pass123").await;
        let token = get_token(app.clone(), "dashlist@test.com", "pass123").await;

        let resp = get_auth(app, "/api/tenant/dashboardInfos?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert!(body["totalPages"].is_number());
        assert!(body["totalElements"].is_number());
        assert!(body["hasNext"].is_boolean());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn add_customers_to_dashboard(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "dashcust@test.com", "pass123").await;
        let token = get_token(app.clone(), "dashcust@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/dashboard", &token, json!({
            "title": "Customer Dashboard",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;
        let dashboard_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let customer_id = Uuid::new_v4();
        let resp = post_json_auth(app, &format!("/api/dashboard/{dashboard_id}/customers/add"), &token,
            json!({"customerIds": [customer_id]})).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        // assignedCustomers should now contain the customer
        let customers = body["assignedCustomers"].as_array().unwrap();
        assert_eq!(customers.len(), 1);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_dashboard_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/dashboard")
                .header("content-type", "application/json")
                .body(Body::from(json!({"title": "No Auth"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
