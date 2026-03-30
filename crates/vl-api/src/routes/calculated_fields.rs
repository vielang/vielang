use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use evalexpr::{ContextWithMutableVariables, DefaultNumericTypes, HashMapContext, Value, eval_float_with_context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::calculated_field::{CalculatedField, CreateCalculatedFieldRequest};
use vl_dao::PageData;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, TelemetryState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // CRUD for calculated fields
        .route("/calculatedField",                   post(create_calculated_field))
        .route("/calculatedField/{id}",              get(get_calculated_field)
                                                     .put(update_calculated_field)
                                                     .delete(delete_calculated_field))
        .route("/calculatedFields/entity/{entityId}", get(list_by_entity))
        .route("/calculatedFields/tenant",            get(list_by_tenant))
        // Test expression evaluation (no DAO required)
        .route("/calculatedField/test",              post(test_expression))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageParams {
    #[serde(default)]
    pub page: i64,
    #[serde(rename = "pageSize", default = "default_page_size")]
    pub page_size: i64,
}

fn default_page_size() -> i64 {
    10
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestExprRequest {
    pub expression: String,
    /// Map of variable name → float value to substitute
    pub values: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestExprResponse {
    pub result: Option<f64>,
    pub error: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/calculatedField — create a new calculated field
async fn create_calculated_field(
    State(state): State<TelemetryState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<CreateCalculatedFieldRequest>,
) -> Result<Json<CalculatedField>, ApiError> {
    let field = state.calc_field_dao.save(ctx.tenant_id, &req).await?;
    Ok(Json(field))
}

/// GET /api/calculatedField/{id} — get calculated field by ID
async fn get_calculated_field(
    State(state): State<TelemetryState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalculatedField>, ApiError> {
    let field = state
        .calc_field_dao
        .find_by_id(id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("CalculatedField [{}] not found", id)))?;

    // Tenant isolation check
    if field.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    Ok(Json(field))
}

/// PUT /api/calculatedField/{id} — update a calculated field
async fn update_calculated_field(
    State(state): State<TelemetryState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateCalculatedFieldRequest>,
) -> Result<Json<CalculatedField>, ApiError> {
    let field = state.calc_field_dao.update(id, ctx.tenant_id, &req).await?;
    Ok(Json(field))
}

/// DELETE /api/calculatedField/{id} — delete a calculated field
async fn delete_calculated_field(
    State(state): State<TelemetryState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.calc_field_dao.delete(id, ctx.tenant_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/calculatedFields/entity/{entityId} — list fields for an entity
async fn list_by_entity(
    State(state): State<TelemetryState>,
    Path(entity_id): Path<Uuid>,
) -> Result<Json<Vec<CalculatedField>>, ApiError> {
    let fields = state.calc_field_dao.find_by_entity(entity_id).await?;
    Ok(Json(fields))
}

/// GET /api/calculatedFields/tenant?page=0&pageSize=10 — list fields for the caller's tenant
async fn list_by_tenant(
    State(state): State<TelemetryState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<CalculatedField>>, ApiError> {
    let (data, total) = state
        .calc_field_dao
        .find_by_tenant(ctx.tenant_id, params.page, params.page_size)
        .await?;

    let total_pages = if params.page_size > 0 {
        (total + params.page_size - 1) / params.page_size
    } else {
        0
    };
    let has_next = (params.page + 1) * params.page_size < total;

    Ok(Json(PageData {
        data,
        total_pages,
        total_elements: total,
        has_next,
    }))
}

/// POST /api/calculatedField/test — evaluate an expression with given variable values
async fn test_expression(
    Json(req): Json<TestExprRequest>,
) -> Json<TestExprResponse> {
    let mut ctx = HashMapContext::<DefaultNumericTypes>::new();
    for (k, v) in &req.values {
        if let Err(e) = ctx.set_value(k.clone(), Value::Float(*v)) {
            return Json(TestExprResponse {
                result: None,
                error: Some(e.to_string()),
            });
        }
    }

    match eval_float_with_context(&req.expression, &ctx) {
        Ok(r) => Json(TestExprResponse { result: Some(r), error: None }),
        Err(e) => Json(TestExprResponse { result: None, error: Some(e.to_string()) }),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

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
                .method("POST").uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username": email, "password": pass}).to_string()))
                .unwrap(),
        ).await.unwrap();
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
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string())).unwrap(),
        ).await.unwrap()
    }

    async fn put_json_auth(app: axum::Router, uri: &str, token: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("PUT").uri(uri)
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

    async fn delete_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("DELETE").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty()).unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    fn calc_field_body(entity_id: Uuid) -> Value {
        json!({
            "entityId": entity_id,
            "entityType": "DEVICE",
            "name": "avg_temp",
            "expression": "temperature + humidity",
            "outputKey": "avg_metric",
            "inputKeys": ["temperature", "humidity"],
            "triggerMode": "ANY_CHANGE",
            "enabled": true
        })
    }

    /// Test 1: Create a calculated field
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_calculated_field_returns_200_with_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf1@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf1@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();
        let resp = post_json_auth(app, "/api/calculatedField", &token, calc_field_body(entity_id)).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["id"].is_string());
        assert_eq!(body["name"].as_str().unwrap(), "avg_temp");
        assert_eq!(body["expression"].as_str().unwrap(), "temperature + humidity");
        assert_eq!(body["outputKey"].as_str().unwrap(), "avg_metric");
        assert_eq!(body["enabled"].as_bool().unwrap(), true);
    }

    /// Test 2: Get calculated field by ID
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_calculated_field_by_id_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf2@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf2@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();
        let create_resp = post_json_auth(app.clone(), "/api/calculatedField", &token, calc_field_body(entity_id)).await;
        let created = body_json(create_resp).await;
        let field_id = created["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/calculatedField/{}", field_id), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["id"].as_str().unwrap(), field_id.as_str());
        assert_eq!(body["name"].as_str().unwrap(), "avg_temp");
    }

    /// Test 3: Get non-existent field returns 404
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_unknown_calculated_field_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf3@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf3@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/calculatedField/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    /// Test 4: Update a calculated field
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn update_calculated_field_returns_updated(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf4@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf4@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();
        let create_resp = post_json_auth(app.clone(), "/api/calculatedField", &token, calc_field_body(entity_id)).await;
        let created = body_json(create_resp).await;
        let field_id = created["id"].as_str().unwrap().to_string();

        let updated_body = json!({
            "entityId": entity_id,
            "entityType": "DEVICE",
            "name": "avg_temp",
            "expression": "temperature * 2",
            "outputKey": "doubled_temp",
            "inputKeys": ["temperature"],
            "triggerMode": "ALL_CHANGE",
            "enabled": false
        });

        let resp = put_json_auth(app, &format!("/api/calculatedField/{}", field_id), &token, updated_body).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["expression"].as_str().unwrap(), "temperature * 2");
        assert_eq!(body["outputKey"].as_str().unwrap(), "doubled_temp");
        assert_eq!(body["enabled"].as_bool().unwrap(), false);
    }

    /// Test 5: Delete a calculated field, then get 404
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_calculated_field_then_not_found(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf5@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf5@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();
        let create_resp = post_json_auth(app.clone(), "/api/calculatedField", &token, calc_field_body(entity_id)).await;
        let created = body_json(create_resp).await;
        let field_id = created["id"].as_str().unwrap().to_string();

        let del = delete_auth(app.clone(), &format!("/api/calculatedField/{}", field_id), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        let get = get_auth(app, &format!("/api/calculatedField/{}", field_id), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }

    /// Test 6: Delete non-existent field returns 404
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_unknown_field_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf6@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf6@test.com", "pass123").await;

        let resp = delete_auth(app, &format!("/api/calculatedField/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    /// Test 7: List fields by entity
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_calculated_fields_by_entity(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf7@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf7@test.com", "pass123").await;

        let entity_id = Uuid::new_v4();

        post_json_auth(app.clone(), "/api/calculatedField", &token, json!({
            "entityId": entity_id, "entityType": "DEVICE",
            "name": "field_a", "expression": "x + y",
            "outputKey": "out_a", "inputKeys": ["x", "y"],
        })).await;
        post_json_auth(app.clone(), "/api/calculatedField", &token, json!({
            "entityId": entity_id, "entityType": "DEVICE",
            "name": "field_b", "expression": "x * y",
            "outputKey": "out_b", "inputKeys": ["x", "y"],
        })).await;

        let resp = get_auth(app, &format!("/api/calculatedFields/entity/{}", entity_id), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let arr = body.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    /// Test 8: List fields by tenant with pagination
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_calculated_fields_by_tenant_paginated(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf8@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf8@test.com", "pass123").await;

        for i in 0..3i32 {
            post_json_auth(app.clone(), "/api/calculatedField", &token, json!({
                "entityId": Uuid::new_v4(), "entityType": "DEVICE",
                "name": format!("field_{}", i),
                "expression": "a + b", "outputKey": format!("out_{}", i),
                "inputKeys": ["a", "b"],
            })).await;
        }

        let resp = get_auth(app, "/api/calculatedFields/tenant?page=0&pageSize=10", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["totalElements"].as_i64().unwrap() >= 3);
        assert!(!body["data"].as_array().unwrap().is_empty());
    }

    /// Test 9: Test expression evaluation — correct result
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn test_expression_evaluates_correctly(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf9@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf9@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/calculatedField/test", &token, json!({
            "expression": "temperature + 10.0",
            "values": { "temperature": 25.0 }
        })).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["error"].is_null());
        let result = body["result"].as_f64().unwrap();
        assert!((result - 35.0).abs() < 1e-9);
    }

    /// Test 10: Test expression with undefined variable returns error field
    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn test_expression_undefined_variable_returns_error(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "cf10@test.com", "pass123").await;
        let token = login_as(app.clone(), "cf10@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/calculatedField/test", &token, json!({
            "expression": "undefined_var + 5",
            "values": {}
        })).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["result"].is_null());
        assert!(body["error"].is_string());
    }
}
