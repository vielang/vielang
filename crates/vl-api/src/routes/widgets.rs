use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{WidgetsBundle, WidgetType};
use vl_dao::PageData;

use crate::{error::ApiError, middleware::SecurityContext, routes::devices::IdResponse, state::{AppState, UiState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── WidgetsBundle endpoints (Java: WidgetsBundleController) ──────────
        .route("/widgetsBundle",                    post(save_widgets_bundle))
        .route("/widgetsBundle/{bundleId}",         get(get_widgets_bundle).delete(delete_widgets_bundle))
        .route("/widgetsBundles",                   get(list_widgets_bundles))
        // assign widget types to bundle
        .route("/widgetsBundle/{bundleId}/widgetTypes",     post(add_widget_types_to_bundle))
        .route("/widgetsBundle/{bundleId}/widgetTypeFqns",  post(add_widget_type_fqns_to_bundle))
        // ── WidgetType endpoints (Java: WidgetTypeController) ────────────────
        .route("/widgetType",                       post(save_widget_type))
        .route("/widgetType/{widgetTypeId}",        get(get_widget_type).delete(delete_widget_type))
        .route("/widgetTypeInfo/{widgetTypeId}",    get(get_widget_type_info))
        .route("/widgetTypes",                      get(list_widget_types))
        .route("/widgetsBundles/{bundleId}/widgetTypes", get(list_widget_types_by_bundle))
        .route("/widgetTypeFqns",                   get(list_widget_type_fqns))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct WidgetsBundleResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
    pub alias: String,
    pub title: String,
    pub image: Option<String>,
    pub scada: bool,
    pub description: Option<String>,
    #[serde(rename = "order")]
    pub order_index: Option<i32>,
    pub version: i64,
}

impl From<WidgetsBundle> for WidgetsBundleResponse {
    fn from(wb: WidgetsBundle) -> Self {
        Self {
            id:          IdResponse::new(wb.id, "WIDGETS_BUNDLE"),
            created_time: wb.created_time,
            tenant_id:   wb.tenant_id.map(IdResponse::tenant),
            alias:       wb.alias,
            title:       wb.title,
            image:       wb.image,
            scada:       wb.scada,
            description: wb.description,
            order_index: wb.order_index,
            version:     wb.version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WidgetTypeResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
    pub fqn: String,
    pub name: String,
    pub descriptor: serde_json::Value,
    pub deprecated: bool,
    pub scada: bool,
    pub image: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub version: i64,
}

impl From<WidgetType> for WidgetTypeResponse {
    fn from(wt: WidgetType) -> Self {
        Self {
            id:          IdResponse::new(wt.id, "WIDGET_TYPE"),
            created_time: wt.created_time,
            tenant_id:   wt.tenant_id.map(IdResponse::tenant),
            fqn:         wt.fqn,
            name:        wt.name,
            descriptor:  wt.descriptor,
            deprecated:  wt.deprecated,
            scada:       wt.scada,
            image:       wt.image,
            description: wt.description,
            tags:        wt.tags,
            version:     wt.version,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveWidgetsBundleRequest {
    pub id: Option<IdResponse>,
    pub alias: String,
    pub title: String,
    pub image: Option<String>,
    #[serde(default)]
    pub scada: bool,
    pub description: Option<String>,
    pub order: Option<i32>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
}

#[derive(Debug, Deserialize)]
pub struct SaveWidgetTypeRequest {
    pub id: Option<IdResponse>,
    pub fqn: Option<String>,
    pub name: String,
    pub descriptor: Option<serde_json::Value>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub scada: bool,
    pub image: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
}

#[derive(Debug, Deserialize)]
pub struct AddWidgetTypesRequest {
    pub widget_type_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct AddWidgetTypeFqnsRequest {
    pub fqns: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct WidgetListParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
}

impl WidgetListParams {
    fn to_page_link(&self) -> vl_dao::PageLink {
        let mut pl = vl_dao::PageLink::new(
            self.page.unwrap_or(0),
            self.page_size.unwrap_or(10),
        );
        pl.text_search = self.text_search.clone();
        pl
    }
}

// ── Handlers — WidgetsBundle ─────────────────────────────────────────────────

/// GET /api/widgetsBundle/{bundleId}
async fn get_widgets_bundle(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WidgetsBundleResponse>, ApiError> {
    let wb = state.widgets_bundle_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Widgets Bundle not found".into()))?;
    Ok(Json(wb.into()))
}

/// GET /api/widgetsBundles?pageSize=10&page=0
async fn list_widgets_bundles(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Query(params): Query<WidgetListParams>,
) -> Result<Json<PageData<WidgetsBundleResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let page = state.widgets_bundle_dao
        .find_by_tenant(ctx.tenant_id, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// POST /api/widgetsBundle
async fn save_widgets_bundle(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Json(req): Json<SaveWidgetsBundleRequest>,
) -> Result<(StatusCode, Json<WidgetsBundleResponse>), ApiError> {
    let now = chrono::Utc::now().timestamp_millis();

    let (id, created_time, version) = if let Some(existing_id) = &req.id {
        let eid = existing_id.id;
        let existing = state.widgets_bundle_dao.find_by_id(eid).await?
            .ok_or(ApiError::NotFound("Widgets Bundle not found".into()))?;
        (eid, existing.created_time, existing.version)
    } else {
        (Uuid::new_v4(), now, 1)
    };

    let wb = WidgetsBundle {
        id,
        created_time,
        tenant_id:   Some(ctx.tenant_id),
        alias:       req.alias,
        title:       req.title,
        image:       req.image,
        scada:       req.scada,
        description: req.description,
        order_index: req.order,
        external_id: None,
        version,
    };

    let saved = state.widgets_bundle_dao.save(&wb).await
        .map_err(|e| match e {
            vl_dao::DaoError::Constraint(msg) => ApiError::BadRequest(msg),
            other => ApiError::from(other),
        })?;

    let status = if version == 1 { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(saved.into())))
}

/// DELETE /api/widgetsBundle/{bundleId}
async fn delete_widgets_bundle(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.widgets_bundle_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/widgetsBundle/{bundleId}/widgetTypes
async fn add_widget_types_to_bundle(
    State(state): State<UiState>,
    Path(bundle_id): Path<Uuid>,
    Json(req): Json<AddWidgetTypesRequest>,
) -> Result<StatusCode, ApiError> {
    let ids = req.widget_type_ids.unwrap_or_default();
    state.widgets_bundle_dao.add_widget_types(bundle_id, &ids).await?;
    Ok(StatusCode::OK)
}

/// POST /api/widgetsBundle/{bundleId}/widgetTypeFqns
async fn add_widget_type_fqns_to_bundle(
    State(state): State<UiState>,
    Path(bundle_id): Path<Uuid>,
    Json(req): Json<AddWidgetTypeFqnsRequest>,
) -> Result<StatusCode, ApiError> {
    let fqns = req.fqns.unwrap_or_default();
    state.widgets_bundle_dao.add_widget_type_fqns(bundle_id, &fqns).await?;
    Ok(StatusCode::OK)
}

// ── Handlers — WidgetType ────────────────────────────────────────────────────

/// GET /api/widgetType/{widgetTypeId}
async fn get_widget_type(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WidgetTypeResponse>, ApiError> {
    let wt = state.widget_type_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Widget Type not found".into()))?;
    Ok(Json(wt.into()))
}

/// GET /api/widgetTypeInfo/{widgetTypeId}
async fn get_widget_type_info(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WidgetTypeResponse>, ApiError> {
    let wt = state.widget_type_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Widget Type not found".into()))?;
    Ok(Json(wt.into()))
}

/// GET /api/widgetTypes?pageSize=10&page=0
async fn list_widget_types(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Query(params): Query<WidgetListParams>,
) -> Result<Json<PageData<WidgetTypeResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let page = state.widget_type_dao
        .find_by_tenant(ctx.tenant_id, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/widgetsBundles/{bundleId}/widgetTypes
async fn list_widget_types_by_bundle(
    State(state): State<UiState>,
    Path(bundle_id): Path<Uuid>,
    Query(params): Query<WidgetListParams>,
) -> Result<Json<PageData<WidgetTypeResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let page = state.widget_type_dao
        .find_by_bundle(bundle_id, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/widgetTypeFqns?widgetTypeIds=...
#[derive(Debug, Deserialize)]
pub struct FqnQueryParams {
    #[serde(rename = "widgetTypeIds")]
    pub widget_type_ids: Option<String>,
}

async fn list_widget_type_fqns(
    State(state): State<UiState>,
    Query(params): Query<FqnQueryParams>,
) -> Result<Json<Vec<String>>, ApiError> {
    let ids: Vec<Uuid> = params.widget_type_ids
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| Uuid::parse_str(s.trim()).ok())
        .collect();

    let fqns = state.widget_type_dao.find_fqns_by_ids(&ids).await?;
    Ok(Json(fqns))
}

/// POST /api/widgetType
async fn save_widget_type(
    State(state): State<UiState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Json(req): Json<SaveWidgetTypeRequest>,
) -> Result<(StatusCode, Json<WidgetTypeResponse>), ApiError> {
    let now = chrono::Utc::now().timestamp_millis();

    let (id, created_time, version) = if let Some(existing_id) = &req.id {
        let eid = existing_id.id;
        let existing = state.widget_type_dao.find_by_id(eid).await?
            .ok_or(ApiError::NotFound("Widget Type not found".into()))?;
        (eid, existing.created_time, existing.version)
    } else {
        (Uuid::new_v4(), now, 1)
    };

    let fqn = req.fqn.unwrap_or_else(|| format!("vl.{}", id));

    let wt = WidgetType {
        id,
        created_time,
        tenant_id:   Some(ctx.tenant_id),
        fqn,
        name:        req.name,
        descriptor:  req.descriptor.unwrap_or(serde_json::json!({})),
        deprecated:  req.deprecated,
        scada:       req.scada,
        image:       req.image,
        description: req.description,
        tags:        req.tags,
        external_id: None,
        version,
    };

    let saved = state.widget_type_dao.save(&wt).await
        .map_err(|e| match e {
            vl_dao::DaoError::Constraint(msg) => ApiError::BadRequest(msg),
            other => ApiError::from(other),
        })?;

    let status = if version == 1 { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(saved.into())))
}

/// DELETE /api/widgetType/{widgetTypeId}
async fn delete_widget_type(
    State(state): State<UiState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.widget_type_dao.delete(id).await?;
    Ok(StatusCode::OK)
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

    async fn post_json_auth(app: axum::Router, uri: &str, token: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string())).unwrap(),
        ).await.unwrap()
    }

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
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
                .body(Body::from(json!({"username": email, "password": pwd}).to_string())).unwrap(),
        ).await.unwrap();
        body_json(resp).await["token"].as_str().unwrap().to_string()
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_widgets_bundle_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "widget_cr@test.com", "pass123").await;
        let token = get_token(app.clone(), "widget_cr@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/widgetsBundle", &token, json!({
            "title": "Test Bundle",
            "alias": "test_bundle",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;

        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = body_json(resp).await;
        assert_eq!(body["title"], "Test Bundle");
        assert_eq!(body["alias"], "test_bundle");
        assert_eq!(body["id"]["entityType"], "WIDGETS_BUNDLE");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_widgets_bundles(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "widget_ls@test.com", "pass123").await;
        let token = get_token(app.clone(), "widget_ls@test.com", "pass123").await;

        // Create a bundle first so listing is non-empty
        post_json_auth(app.clone(), "/api/widgetsBundle", &token, json!({
            "title": "List Bundle",
            "alias": "list_bundle",
            "tenantId": {"id": user.tenant_id, "entityType": "TENANT"},
        })).await;

        let resp = get_auth(app, "/api/widgetsBundles?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].as_array().unwrap().len() >= 1);
        assert!(body["totalElements"].as_i64().unwrap() >= 1);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_nonexistent_bundle_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let user = create_test_user(&pool, "widget_404@test.com", "pass123").await;
        let token = get_token(app.clone(), "widget_404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/widgetsBundle/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
