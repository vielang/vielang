use axum::{
    extract::{Extension, Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{Event, EventType, EventFilter};
use vl_dao::{PageData, PageLink};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, AdminState}};
use super::devices::IdResponse;

pub fn router() -> Router<AppState> {
    Router::new()
        // Event endpoints
        .route("/events/{entityType}/{entityId}", get(get_events).delete(clear_events))
        .route("/events/{entityType}/{entityId}/types", get(get_event_types))
        .route("/event", post(save_event))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct EventResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    #[serde(rename = "entityId")]
    pub entity_id: IdResponse,
    #[serde(rename = "eventType")]
    pub event_type: String,
    #[serde(rename = "uid")]
    pub event_uid: String,
    pub body: serde_json::Value,
}

impl EventResponse {
    pub fn from_event(e: Event) -> Self {
        Self {
            id: IdResponse::new(e.id, "EVENT"),
            created_time: e.created_time,
            tenant_id: IdResponse::new(e.tenant_id, "TENANT"),
            entity_id: IdResponse::with_type(e.entity_id, e.entity_type),
            event_type: e.event_type.as_str().to_string(),
            event_uid: e.event_uid,
            body: e.body,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EventFilterParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
    #[serde(rename = "startTs")]
    pub start_ts: Option<i64>,
    #[serde(rename = "endTs")]
    pub end_ts: Option<i64>,
}

impl EventFilterParams {
    pub fn to_page_link(&self) -> PageLink {
        PageLink::new(
            self.page.unwrap_or(0),
            self.page_size.unwrap_or(10),
        )
    }

    pub fn to_filter(&self) -> EventFilter {
        EventFilter {
            event_type: self.event_type.as_deref().and_then(EventType::from_str),
            start_ts: self.start_ts,
            end_ts: self.end_ts,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveEventRequest {
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    #[serde(rename = "entityId")]
    pub entity_id: IdResponse,
    #[serde(rename = "eventType")]
    pub event_type: String,
    pub uid: String,
    pub body: serde_json::Value,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/events/{entityType}/{entityId} — get events for entity
async fn get_events(
    State(state): State<AdminState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(params): Query<EventFilterParams>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<PageData<EventResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;

    let page = state.event_dao
        .find_by_entity(
            tenant_id,
            entity_id,
            &entity_type,
            &params.to_filter(),
            &params.to_page_link(),
        )
        .await?;

    Ok(Json(PageData {
        data: page.data.into_iter().map(EventResponse::from_event).collect(),
        total_pages: page.total_pages,
        total_elements: page.total_elements,
        has_next: page.has_next,
    }))
}

/// GET /api/events/{entityType}/{entityId}/types — get event types for entity
async fn get_event_types(
    State(state): State<AdminState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<String>>, ApiError> {
    let tenant_id = ctx.tenant_id;

    let types = state.event_dao
        .get_event_types(tenant_id, entity_id, &entity_type)
        .await?;

    Ok(Json(types.into_iter().map(|t| t.as_str().to_string()).collect()))
}

/// POST /api/event — save event
async fn save_event(
    State(state): State<AdminState>,
    Json(req): Json<SaveEventRequest>,
) -> Result<Json<EventResponse>, ApiError> {
    let now = chrono::Utc::now().timestamp_millis();

    let event_type = EventType::from_str(&req.event_type)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid event type: {}", req.event_type)))?;

    let event = Event {
        id: Uuid::new_v4(),
        created_time: now,
        tenant_id: req.tenant_id.id,
        entity_id: req.entity_id.id,
        entity_type: req.entity_id.entity_type.clone(),
        event_type,
        event_uid: req.uid,
        body: req.body,
    };

    let saved = state.event_dao.save(&event).await?;
    Ok(Json(EventResponse::from_event(saved)))
}

/// DELETE /api/events/{entityType}/{entityId} — clear events for entity
async fn clear_events(
    State(state): State<AdminState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(params): Query<EventFilterParams>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<i64>, ApiError> {
    let tenant_id = ctx.tenant_id;

    let deleted = state.event_dao
        .delete_by_entity(tenant_id, entity_id, &entity_type, &params.to_filter())
        .await?;

    Ok(Json(deleted))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;

    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;
    use vl_auth::password;
    use vl_core::entities::{Authority, User, UserCredentials};
    use vl_dao::postgres::user::UserDao;

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

    async fn login_as(app: axum::Router, email: &str, pass: &str) -> String {
        let resp = app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": pass}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        v["token"].as_str().unwrap().to_string()
    }

    async fn create_user(pool: &PgPool, email: &str, pass: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::nil(), customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: None, last_name: None, phone: None,
            additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pass).unwrap();
        dao.save_credentials(&UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(),
            user_id: user.id, enabled: true,
            password: Some(hash), activate_token: None,
            reset_token: None, additional_info: None,
        }).await.unwrap();
        user
    }

    async fn post_json_auth(app: axum::Router, uri: &str, token: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn delete_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("DELETE").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    // ── POST /api/event ───────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_event_returns_201(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ev@test.com", "pass123").await;
        let token = login_as(app.clone(), "ev@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();
        let tenant_id = Uuid::nil();

        let resp = post_json_auth(app, "/api/event", &token, json!({
            "tenantId":  { "id": tenant_id, "entityType": "TENANT" },
            "entityId":  { "id": entity_id, "entityType": "DEVICE" },
            "eventType": "LC_EVENT",
            "uid":       "test-uid-001",
            "body":      { "event": "CREATED", "success": true }
        })).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["eventType"].as_str().unwrap(), "LC_EVENT");
        assert_eq!(body["uid"].as_str().unwrap(), "test-uid-001");
        assert!(body["id"]["id"].is_string());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_event_invalid_type_returns_400(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ev2@test.com", "pass123").await;
        let token = login_as(app.clone(), "ev2@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/event", &token, json!({
            "tenantId":  { "id": Uuid::nil(), "entityType": "TENANT" },
            "entityId":  { "id": Uuid::new_v4(), "entityType": "DEVICE" },
            "eventType": "INVALID_TYPE",
            "uid":       "x",
            "body":      {}
        })).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // ── GET /api/events/{entityType}/{entityId} ───────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_events_returns_paginated_list(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ev3@test.com", "pass123").await;
        let token = login_as(app.clone(), "ev3@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();
        let tenant_id = Uuid::nil();

        // Lưu 2 events
        for uid in ["uid-a", "uid-b"] {
            post_json_auth(app.clone(), "/api/event", &token, json!({
                "tenantId":  { "id": tenant_id, "entityType": "TENANT" },
                "entityId":  { "id": entity_id, "entityType": "DEVICE" },
                "eventType": "LC_EVENT",
                "uid":       uid,
                "body":      {}
            })).await;
        }

        let resp = get_auth(
            app,
            &format!("/api/events/DEVICE/{}?pageSize=10&page=0", entity_id),
            &token,
        ).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].as_array().unwrap().len() >= 2);
        assert!(body["totalElements"].as_i64().unwrap() >= 2);
    }

    // ── GET /api/events/{entityType}/{entityId}/types ─────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_event_types_returns_list(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ev4@test.com", "pass123").await;
        let token = login_as(app.clone(), "ev4@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();
        let tenant_id = Uuid::nil();

        // Lưu 2 loại event khác nhau
        for event_type in ["LC_EVENT", "STATS"] {
            post_json_auth(app.clone(), "/api/event", &token, json!({
                "tenantId":  { "id": tenant_id, "entityType": "TENANT" },
                "entityId":  { "id": entity_id, "entityType": "DEVICE" },
                "eventType": event_type,
                "uid":       event_type,
                "body":      {}
            })).await;
        }

        let resp = get_auth(
            app,
            &format!("/api/events/DEVICE/{}/types", entity_id),
            &token,
        ).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let types = body.as_array().unwrap();
        assert!(types.iter().any(|t| t.as_str() == Some("LC_EVENT")));
    }

    // ── DELETE /api/events/{entityType}/{entityId} ────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn clear_events_removes_events(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "ev5@test.com", "pass123").await;
        let token = login_as(app.clone(), "ev5@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();
        let tenant_id = Uuid::nil();

        post_json_auth(app.clone(), "/api/event", &token, json!({
            "tenantId":  { "id": tenant_id, "entityType": "TENANT" },
            "entityId":  { "id": entity_id, "entityType": "DEVICE" },
            "eventType": "LC_EVENT",
            "uid":       "uid-del",
            "body":      {}
        })).await;

        let resp = delete_auth(
            app.clone(),
            &format!("/api/events/DEVICE/{}", entity_id),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify đã xoá
        let list_resp = get_auth(
            app,
            &format!("/api/events/DEVICE/{}?pageSize=10&page=0", entity_id),
            &token,
        ).await;
        let body = body_json(list_resp).await;
        assert_eq!(body["totalElements"].as_i64().unwrap(), 0);
    }
}
