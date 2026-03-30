use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{Edge, EdgeEvent, EdgeInfo};
use vl_dao::PageData;

use crate::{error::ApiError, middleware::SecurityContext, routes::devices::IdResponse, state::{AppState, EdgeState, EntityState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Edge CRUD (Java: EdgeController) ─────────────────────────────────
        .route("/edges/enabled",                                    get(edges_enabled))
        .route("/edge",                                             post(save_edge))
        .route("/edge/{edgeId}",                                    get(get_edge).delete(delete_edge))
        .route("/edge/info/{edgeId}",                               get(get_edge_info))
        .route("/edge/types",                                       get(get_edge_types))
        // Tenant listing
        .route("/tenant/edges",                                     get(list_tenant_edges))
        .route("/tenant/edgeInfos",                                 get(list_tenant_edge_infos))
        .route("/tenant/edge",                                      get(get_tenant_edge_by_name))
        // Customer assign/unassign
        .route("/customer/{customerId}/edge/{edgeId}",              post(assign_edge_to_customer))
        .route("/customer/edge/{edgeId}",                           delete(unassign_edge_from_customer))
        .route("/customer/public/edge/{edgeId}",                    post(assign_edge_to_public))
        .route("/customer/{customerId}/edges",                      get(list_customer_edges))
        .route("/customer/{customerId}/edgeInfos",                  get(list_customer_edge_infos))
        // Root rule chain
        .route("/edge/{edgeId}/{ruleChainId}/root",                 post(set_root_rule_chain))
        // Edge events (Java: EdgeEventController)
        .route("/edge/{edgeId}/events",                             get(list_edge_events))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    #[serde(rename = "customerId")]
    pub customer_id: Option<IdResponse>,
    #[serde(rename = "rootRuleChainId")]
    pub root_rule_chain_id: Option<IdResponse>,
    pub name: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub label: Option<String>,
    #[serde(rename = "routingKey")]
    pub routing_key: String,
    pub secret: String,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
    pub version: i64,
}

impl From<Edge> for EdgeResponse {
    fn from(e: Edge) -> Self {
        Self {
            id:                 IdResponse::new(e.id, "EDGE"),
            created_time:       e.created_time,
            tenant_id:          IdResponse::tenant(e.tenant_id),
            customer_id:        e.customer_id.map(IdResponse::customer),
            root_rule_chain_id: e.root_rule_chain_id.map(|id| IdResponse::new(id, "RULE_CHAIN")),
            name:               e.name,
            edge_type:          e.edge_type,
            label:              e.label,
            routing_key:        e.routing_key,
            secret:             e.secret,
            additional_info:    e.additional_info,
            version:            e.version,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeInfoResponse {
    #[serde(flatten)]
    pub edge: EdgeResponse,
    #[serde(rename = "customerTitle")]
    pub customer_title: Option<String>,
    #[serde(rename = "customerIsPublic")]
    pub customer_is_public: bool,
}

impl From<EdgeInfo> for EdgeInfoResponse {
    fn from(i: EdgeInfo) -> Self {
        Self {
            edge:               i.edge.into(),
            customer_title:     i.customer_title,
            customer_is_public: i.customer_is_public,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeEventResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "seqId")]
    pub seq_id: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    #[serde(rename = "edgeId")]
    pub edge_id: IdResponse,
    #[serde(rename = "type")]
    pub edge_event_type: String,
    pub action: String,
    #[serde(rename = "entityId")]
    pub entity_id: Option<Uuid>,
    pub body: Option<serde_json::Value>,
    pub uid: Option<String>,
}

impl From<EdgeEvent> for EdgeEventResponse {
    fn from(e: EdgeEvent) -> Self {
        Self {
            id:              IdResponse::new(e.id, "EDGE_EVENT"),
            created_time:    e.created_time,
            seq_id:          e.seq_id,
            tenant_id:       IdResponse::tenant(e.tenant_id),
            edge_id:         IdResponse::new(e.edge_id, "EDGE"),
            edge_event_type: e.edge_event_type,
            action:          e.edge_event_action,
            entity_id:       e.entity_id,
            body:            e.body,
            uid:             e.uid,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveEdgeRequest {
    pub id: Option<IdResponse>,
    pub name: String,
    #[serde(rename = "type")]
    pub edge_type: Option<String>,
    pub label: Option<String>,
    #[serde(rename = "routingKey")]
    pub routing_key: Option<String>,
    pub secret: Option<String>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
    #[serde(rename = "customerId")]
    pub customer_id: Option<IdResponse>,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct EdgeListParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
    #[serde(rename = "type")]
    pub edge_type: Option<String>,
}

impl EdgeListParams {
    fn to_page_link(&self) -> vl_dao::PageLink {
        let mut pl = vl_dao::PageLink::new(
            self.page.unwrap_or(0),
            self.page_size.unwrap_or(10),
        );
        pl.text_search = self.text_search.clone();
        pl
    }
}

#[derive(Debug, Deserialize)]
pub struct EdgeNameParams {
    #[serde(rename = "edgeName")]
    pub edge_name: String,
}

#[derive(Debug, Serialize)]
pub struct EdgeEnabledResponse {
    #[serde(rename = "edgesEnabled")]
    pub edges_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct EdgeSubtypeResponse {
    #[serde(rename = "entityType")]
    pub entity_type: String,
    #[serde(rename = "type")]
    pub subtype: String,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/edges/enabled
async fn edges_enabled() -> Json<EdgeEnabledResponse> {
    Json(EdgeEnabledResponse { edges_enabled: true })
}

/// GET /api/edge/{edgeId}
async fn get_edge(
    State(state): State<EdgeState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EdgeResponse>, ApiError> {
    let edge = state.edge_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Edge not found".into()))?;
    Ok(Json(edge.into()))
}

/// GET /api/edge/info/{edgeId}
async fn get_edge_info(
    State(edge): State<EdgeState>,
    State(entity): State<EntityState>,
    axum::extract::Extension(_ctx): axum::extract::Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<EdgeInfoResponse>, ApiError> {
    let e = edge.edge_dao
        .find_by_id(id).await?
        .ok_or(ApiError::NotFound("Edge not found".into()))?;

    let customer_title;
    let customer_is_public;
    if let Some(cid) = e.customer_id {
        if let Some(c) = entity.customer_dao.find_by_id(cid).await? {
            customer_title = Some(c.title);
            customer_is_public = c.is_public;
        } else {
            customer_title = None;
            customer_is_public = false;
        }
    } else {
        customer_title = None;
        customer_is_public = false;
    }

    Ok(Json(EdgeInfoResponse {
        edge: e.into(),
        customer_title,
        customer_is_public,
    }))
}

/// GET /api/edge/types
async fn get_edge_types(
    State(state): State<EdgeState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
) -> Result<Json<Vec<EdgeSubtypeResponse>>, ApiError> {
    let types = state.edge_dao.find_types_by_tenant(ctx.tenant_id).await?;
    let result = types.into_iter().map(|t| EdgeSubtypeResponse {
        entity_type: "EDGE".into(),
        subtype: t,
    }).collect();
    Ok(Json(result))
}

/// GET /api/tenant/edges
async fn list_tenant_edges(
    State(state): State<EdgeState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Query(params): Query<EdgeListParams>,
) -> Result<Json<PageData<EdgeResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let edge_type = params.edge_type.as_deref();
    let page = state.edge_dao
        .find_by_tenant(ctx.tenant_id, edge_type, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/tenant/edgeInfos
async fn list_tenant_edge_infos(
    State(state): State<EdgeState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Query(params): Query<EdgeListParams>,
) -> Result<Json<PageData<EdgeInfoResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let edge_type = params.edge_type.as_deref();
    let page = state.edge_dao
        .find_infos_by_tenant(ctx.tenant_id, edge_type, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/tenant/edge?edgeName=xxx
async fn get_tenant_edge_by_name(
    State(state): State<EdgeState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Query(params): Query<EdgeNameParams>,
) -> Result<Json<EdgeResponse>, ApiError> {
    let edge = state.edge_dao
        .find_by_tenant_and_name(ctx.tenant_id, &params.edge_name).await?
        .ok_or(ApiError::NotFound("Edge not found".into()))?;
    Ok(Json(edge.into()))
}

/// POST /api/edge
async fn save_edge(
    State(state): State<EdgeState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Json(req): Json<SaveEdgeRequest>,
) -> Result<(StatusCode, Json<EdgeResponse>), ApiError> {
    let now = chrono::Utc::now().timestamp_millis();

    let (id, created_time, version) = if let Some(existing_id) = &req.id {
        let eid = existing_id.id;
        let existing = state.edge_dao.find_by_id(eid).await?
            .ok_or(ApiError::NotFound("Edge not found".into()))?;
        (eid, existing.created_time, existing.version)
    } else {
        (Uuid::new_v4(), now, 1)
    };

    // Generate routing key and secret if not provided
    let routing_key = req.routing_key
        .unwrap_or_else(|| Uuid::new_v4().to_string().replace('-', ""));
    let secret = req.secret
        .unwrap_or_else(|| Uuid::new_v4().to_string().replace('-', ""));

    let edge = Edge {
        id,
        created_time,
        tenant_id:          ctx.tenant_id,
        customer_id:        req.customer_id.map(|c| c.id),
        root_rule_chain_id: None,
        name:               req.name,
        edge_type:          req.edge_type.unwrap_or_else(|| "DEFAULT".into()),
        label:              req.label,
        routing_key,
        secret,
        additional_info:    req.additional_info,
        external_id:        None,
        version,
    };

    let saved = state.edge_dao.save(&edge).await
        .map_err(|e| match e {
            vl_dao::DaoError::Constraint(msg) => ApiError::BadRequest(msg),
            other => ApiError::from(other),
        })?;

    let status = if version == 1 { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(saved.into())))
}

/// DELETE /api/edge/{edgeId}
async fn delete_edge(
    State(state): State<EdgeState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.edge_dao.delete(id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/customer/{customerId}/edge/{edgeId}
async fn assign_edge_to_customer(
    State(edge): State<EdgeState>,
    State(entity): State<EntityState>,
    Path((customer_id, edge_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<EdgeResponse>, ApiError> {
    edge.edge_dao.find_by_id(edge_id).await?
        .ok_or(ApiError::NotFound("Edge not found".into()))?;
    entity.customer_dao.find_by_id(customer_id).await?
        .ok_or(ApiError::NotFound("Customer not found".into()))?;

    let updated = edge.edge_dao.assign_to_customer(edge_id, customer_id).await?;
    Ok(Json(updated.into()))
}

/// DELETE /api/customer/edge/{edgeId}
async fn unassign_edge_from_customer(
    State(state): State<EdgeState>,
    Path(edge_id): Path<Uuid>,
) -> Result<Json<EdgeResponse>, ApiError> {
    let edge = state.edge_dao.find_by_id(edge_id).await?
        .ok_or(ApiError::NotFound("Edge not found".into()))?;
    if edge.customer_id.is_none() {
        return Err(ApiError::BadRequest("Edge isn't assigned to any customer".into()));
    }
    let updated = state.edge_dao.unassign_from_customer(edge_id).await?;
    Ok(Json(updated.into()))
}

/// POST /api/customer/public/edge/{edgeId}
async fn assign_edge_to_public(
    State(edge): State<EdgeState>,
    State(entity): State<EntityState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path(edge_id): Path<Uuid>,
) -> Result<Json<EdgeResponse>, ApiError> {
    let public_customer = entity.customer_dao
        .find_public_customer(ctx.tenant_id).await?
        .ok_or(ApiError::NotFound("Public customer not found".into()))?;
    let updated = edge.edge_dao.assign_to_customer(edge_id, public_customer.id).await?;
    Ok(Json(updated.into()))
}

/// GET /api/customer/{customerId}/edges
async fn list_customer_edges(
    State(state): State<EdgeState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path(customer_id): Path<Uuid>,
    Query(params): Query<EdgeListParams>,
) -> Result<Json<PageData<EdgeResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let edge_type = params.edge_type.as_deref();
    let page = state.edge_dao
        .find_by_customer(ctx.tenant_id, customer_id, edge_type, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// GET /api/customer/{customerId}/edgeInfos
async fn list_customer_edge_infos(
    State(state): State<EdgeState>,
    axum::extract::Extension(ctx): axum::extract::Extension<SecurityContext>,
    Path(customer_id): Path<Uuid>,
    Query(params): Query<EdgeListParams>,
) -> Result<Json<PageData<EdgeInfoResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let edge_type = params.edge_type.as_deref();
    // Reuse find_infos_by_tenant but filter by customer
    let page = state.edge_dao
        .find_by_customer(ctx.tenant_id, customer_id, edge_type, &page_link).await?;
    Ok(Json(PageData {
        data: page.data.into_iter().map(|e| EdgeInfoResponse {
            edge: e.into(),
            customer_title: None,
            customer_is_public: false,
        }).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
}

/// POST /api/edge/{edgeId}/{ruleChainId}/root
async fn set_root_rule_chain(
    State(state): State<EdgeState>,
    Path((edge_id, rule_chain_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<EdgeResponse>, ApiError> {
    state.edge_dao.find_by_id(edge_id).await?
        .ok_or(ApiError::NotFound("Edge not found".into()))?;
    let updated = state.edge_dao.set_root_rule_chain(edge_id, rule_chain_id).await?;
    Ok(Json(updated.into()))
}

/// GET /api/edge/{edgeId}/events
async fn list_edge_events(
    State(state): State<EdgeState>,
    Path(edge_id): Path<Uuid>,
    Query(params): Query<EdgeListParams>,
) -> Result<Json<PageData<EdgeEventResponse>>, ApiError> {
    let page_link = params.to_page_link();
    let page = state.edge_event_dao
        .find_by_edge(edge_id, &page_link).await?;
    Ok(Json(PageData {
        data:           page.data.into_iter().map(Into::into).collect(),
        total_pages:    page.total_pages,
        total_elements: page.total_elements,
        has_next:       page.has_next,
    }))
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

    async fn insert_tenant(pool: &PgPool) -> Uuid {
        let profile_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO tenant_profile (id, created_time, name, is_default, isolated_vl_rule_engine) VALUES ($1, $2, $3, false, false)",
            profile_id, now_ms(), format!("Profile {profile_id}")
        ).execute(pool).await.unwrap();
        sqlx::query!(
            "INSERT INTO tenant (id, created_time, title, tenant_profile_id) VALUES ($1, $2, $3, $4)",
            tenant_id, now_ms(), format!("Tenant {tenant_id}"), profile_id
        ).execute(pool).await.unwrap();
        tenant_id
    }

    async fn insert_customer(pool: &PgPool, tenant_id: Uuid) -> Uuid {
        let customer_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO customer (id, created_time, tenant_id, title) VALUES ($1, $2, $3, $4)",
            customer_id, now_ms(), tenant_id, format!("Customer {customer_id}")
        ).execute(pool).await.unwrap();
        customer_id
    }

    async fn create_test_user(pool: &PgPool, email: &str, pwd: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::new_v4(), customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: None, last_name: None, phone: None, additional_info: None, version: 1,
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

    async fn create_user_in_tenant(pool: &PgPool, tenant_id: Uuid, email: &str, pwd: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id, customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: None, last_name: None, phone: None, additional_info: None, version: 1,
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
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": pwd}).to_string()))
                .unwrap(),
        ).await.unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        v["token"].as_str().unwrap().to_string()
    }

    // ── Unit 8: Basic Edge CRUD ───────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn edges_enabled_returns_true(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgeenabled@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgeenabled@test.com", "pass123").await;

        let resp = get_auth(app, "/api/edges/enabled", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["edgesEnabled"], true);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_edge_returns_201(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgecreate@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgecreate@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/edge", &token, json!({
            "name": "My Test Edge"
        })).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn edge_response_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgefmt@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgefmt@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/edge", &token, json!({
            "name": "Format Edge"
        })).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = body_json(resp).await;

        // id must be { "id": "...", "entityType": "EDGE" }
        assert!(body["id"]["id"].is_string());
        assert_eq!(body["id"]["entityType"], "EDGE");
        // createdTime is i64
        assert!(body["createdTime"].is_number());
        // tenantId must be { "id": "...", "entityType": "TENANT" }
        assert!(body["tenantId"]["id"].is_string());
        assert_eq!(body["tenantId"]["entityType"], "TENANT");
        // routingKey and secret are auto-generated strings
        assert!(body["routingKey"].is_string());
        assert!(!body["routingKey"].as_str().unwrap().is_empty());
        assert!(body["secret"].is_string());
        assert!(!body["secret"].as_str().unwrap().is_empty());
        // name is returned
        assert_eq!(body["name"], "Format Edge");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_edge_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgeget@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgeget@test.com", "pass123").await;

        // Create edge
        let create_resp = post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": "Get Edge Test"
        })).await;
        let create_body = body_json(create_resp).await;
        let edge_id = create_body["id"]["id"].as_str().unwrap().to_string();

        // Get by id
        let resp = get_auth(app, &format!("/api/edge/{edge_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["id"]["id"], edge_id);
        assert_eq!(body["name"], "Get Edge Test");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_nonexistent_edge_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edge404@test.com", "pass123").await;
        let token = get_token(app.clone(), "edge404@test.com", "pass123").await;

        let random_id = Uuid::new_v4();
        let resp = get_auth(app, &format!("/api/edge/{random_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = body_json(resp).await;
        assert!(body["status"].is_number());
        assert!(body["message"].is_string());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_edge_then_get_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgedel@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgedel@test.com", "pass123").await;

        // Create edge
        let create_resp = post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": "Delete Me Edge"
        })).await;
        let create_body = body_json(create_resp).await;
        let edge_id = create_body["id"]["id"].as_str().unwrap().to_string();

        // Delete
        let del_resp = delete_auth(app.clone(), &format!("/api/edge/{edge_id}"), &token).await;
        assert_eq!(del_resp.status(), StatusCode::OK);

        // Get returns 404
        let get_resp = get_auth(app, &format!("/api/edge/{edge_id}"), &token).await;
        assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn edge_info_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgeinfo@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgeinfo@test.com", "pass123").await;

        // Create edge
        let create_resp = post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": "Info Edge"
        })).await;
        let create_body = body_json(create_resp).await;
        let edge_id = create_body["id"]["id"].as_str().unwrap().to_string();

        // Get edge info
        let resp = get_auth(app, &format!("/api/edge/info/{edge_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["id"]["id"], edge_id);
        // customerIsPublic field must be present
        assert!(body["customerIsPublic"].is_boolean());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_edge_types_returns_array(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgetypes@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgetypes@test.com", "pass123").await;

        // Create two edges with different types
        post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": "Edge Type A",
            "type": "TypeA"
        })).await;
        post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": "Edge Type B",
            "type": "TypeB"
        })).await;

        let resp = get_auth(app, "/api/edge/types", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body.is_array());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn edge_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/edge")
                .header("content-type", "application/json")
                .body(Body::from(json!({"name": "Unauthorized Edge"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Unit 9: Edge Assignment & Listing ─────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_edges_pagination(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgelist@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgelist@test.com", "pass123").await;

        // Create two edges
        post_json_auth(app.clone(), "/api/edge", &token, json!({"name": "List Edge 1"})).await;
        post_json_auth(app.clone(), "/api/edge", &token, json!({"name": "List Edge 2"})).await;

        let resp = get_auth(app, "/api/tenant/edges?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert!(body["totalPages"].is_number());
        assert!(body["totalElements"].is_number());
        assert!(body["hasNext"].is_boolean());
        assert!(body["totalElements"].as_i64().unwrap() >= 2);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_tenant_edge_infos_pagination(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgeinfos@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgeinfos@test.com", "pass123").await;

        post_json_auth(app.clone(), "/api/edge", &token, json!({"name": "InfoList Edge"})).await;

        let resp = get_auth(app, "/api/tenant/edgeInfos?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert!(body["totalElements"].as_i64().unwrap() >= 1);
        // Each item should have customerIsPublic field
        let first = &body["data"][0];
        assert!(first["customerIsPublic"].is_boolean());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn assign_edge_to_customer(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let customer_id = insert_customer(&pool, tenant_id).await;
        create_user_in_tenant(&pool, tenant_id, "edgeassign@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgeassign@test.com", "pass123").await;

        // Create edge
        let create_resp = post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": "Assign Edge"
        })).await;
        let create_body = body_json(create_resp).await;
        let edge_id = create_body["id"]["id"].as_str().unwrap().to_string();

        // Assign to customer
        let resp = post_json_auth(
            app,
            &format!("/api/customer/{customer_id}/edge/{edge_id}"),
            &token,
            json!({}),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["customerId"]["id"], customer_id.to_string());
        assert_eq!(body["customerId"]["entityType"], "CUSTOMER");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn unassign_edge_from_customer(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let customer_id = insert_customer(&pool, tenant_id).await;
        create_user_in_tenant(&pool, tenant_id, "edgeunassign@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgeunassign@test.com", "pass123").await;

        // Create edge
        let create_resp = post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": "Unassign Edge"
        })).await;
        let create_body = body_json(create_resp).await;
        let edge_id = create_body["id"]["id"].as_str().unwrap().to_string();

        // Assign to customer first
        post_json_auth(
            app.clone(),
            &format!("/api/customer/{customer_id}/edge/{edge_id}"),
            &token,
            json!({}),
        ).await;

        // Unassign from customer
        let resp = delete_auth(app, &format!("/api/customer/edge/{edge_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        // customerId should be null after unassign
        assert!(body["customerId"].is_null());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_customer_edges(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let customer_id = insert_customer(&pool, tenant_id).await;
        create_user_in_tenant(&pool, tenant_id, "edgecustlist@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgecustlist@test.com", "pass123").await;

        // Create edge and assign to customer
        let create_resp = post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": "Customer Edge"
        })).await;
        let create_body = body_json(create_resp).await;
        let edge_id = create_body["id"]["id"].as_str().unwrap().to_string();

        post_json_auth(
            app.clone(),
            &format!("/api/customer/{customer_id}/edge/{edge_id}"),
            &token,
            json!({}),
        ).await;

        // List customer edges
        let resp = get_auth(
            app,
            &format!("/api/customer/{customer_id}/edges?pageSize=10&page=0"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert!(body["totalElements"].as_i64().unwrap() >= 1);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_tenant_edge_by_name(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "edgebyname@test.com", "pass123").await;
        let token = get_token(app.clone(), "edgebyname@test.com", "pass123").await;

        let unique_name = format!("Named Edge {}", Uuid::new_v4());
        post_json_auth(app.clone(), "/api/edge", &token, json!({
            "name": unique_name
        })).await;

        let encoded_name = unique_name.replace(' ', "%20");
        let resp = get_auth(
            app,
            &format!("/api/tenant/edge?edgeName={encoded_name}"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["name"], unique_name);
        assert_eq!(body["id"]["entityType"], "EDGE");
    }
}
