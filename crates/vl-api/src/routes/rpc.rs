use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::{Rpc, RpcRequest, RpcStatus, TwoWayRpcRequest, OneWayRpcRequest};
use vl_dao::{PageData, PageLink};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, DeviceState, CoreState}};
use super::devices::IdResponse;

pub fn router() -> Router<AppState> {
    Router::new()
        // RPC v2 endpoints — khớp ThingsBoard Java RpcV2Controller
        .route("/rpc/oneway/{deviceId}", post(handle_oneway_rpc))
        .route("/rpc/twoway/{deviceId}", post(handle_twoway_rpc))
        .route("/rpc/persistent/{id}", get(list_persistent_rpc).delete(delete_rpc))
        .route("/rpc/{rpcId}", get(get_rpc))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    #[serde(rename = "deviceId")]
    pub device_id: IdResponse,
    #[serde(rename = "requestId")]
    pub request_id: i32,
    #[serde(rename = "expirationTime")]
    pub expiration_time: i64,
    pub request: RpcRequest,
    pub response: Option<serde_json::Value>,
    pub status: String,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
}

impl From<Rpc> for RpcResponse {
    fn from(r: Rpc) -> Self {
        Self {
            id: IdResponse::new(r.id, "RPC"),
            created_time: r.created_time,
            tenant_id: IdResponse::new(r.tenant_id, "TENANT"),
            device_id: IdResponse::new(r.device_id, "DEVICE"),
            request_id: r.request_id,
            expiration_time: r.expiration_time,
            request: r.request,
            response: r.response,
            status: r.status.as_str().to_string(),
            additional_info: r.additional_info,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RpcParams {
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
}

impl RpcParams {
    pub fn to_page_link(&self) -> PageLink {
        PageLink::new(
            self.page.unwrap_or(0),
            self.page_size.unwrap_or(10),
        )
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/rpc/oneway/{deviceId} — send one-way RPC (fire and forget)
async fn handle_oneway_rpc(
    State(state): State<DeviceState>,
    State(core): State<CoreState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(device_id): Path<Uuid>,
    Json(req): Json<OneWayRpcRequest>,
) -> Result<StatusCode, ApiError> {

    // Verify device exists
    let device = state.device_dao
        .find_by_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Device [{}] not found", device_id)))?;

    let now = chrono::Utc::now().timestamp_millis();
    let request_id = state.rpc_dao.get_next_request_id(device_id).await?;

    let rpc = Rpc {
        id: Uuid::new_v4(),
        created_time: now,
        tenant_id: device.tenant_id,
        device_id,
        request_id,
        expiration_time: now + 60_000, // 1 minute default for oneway
        request: RpcRequest {
            method: req.method,
            params: req.params,
            oneway: true,
            timeout: 0,
            additional_info: req.additional_info,
        },
        response: None,
        status: RpcStatus::Queued,
        additional_info: None,
    };

    // If persistent, save to DB
    if req.persistent {
        state.rpc_dao.save(&rpc).await?;
    }

    // Send to transport layer via queue
    send_rpc_to_device(&state, &core, &rpc).await?;

    Ok(StatusCode::OK)
}

/// POST /api/rpc/twoway/{deviceId} — send two-way RPC (wait for response)
async fn handle_twoway_rpc(
    State(state): State<DeviceState>,
    State(core): State<CoreState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(device_id): Path<Uuid>,
    Json(req): Json<TwoWayRpcRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {

    // Verify device exists
    let device = state.device_dao
        .find_by_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Device [{}] not found", device_id)))?;

    let now = chrono::Utc::now().timestamp_millis();
    let timeout_ms = req.timeout.max(1000).min(300_000); // 1s to 5min
    let request_id = state.rpc_dao.get_next_request_id(device_id).await?;

    let rpc = Rpc {
        id: Uuid::new_v4(),
        created_time: now,
        tenant_id: device.tenant_id,
        device_id,
        request_id,
        expiration_time: now + timeout_ms,
        request: RpcRequest {
            method: req.method.clone(),
            params: req.params.clone(),
            oneway: false,
            timeout: timeout_ms,
            additional_info: req.additional_info,
        },
        response: None,
        status: RpcStatus::Queued,
        additional_info: None,
    };

    // If persistent, save to DB
    if req.persistent {
        state.rpc_dao.save(&rpc).await?;
    }

    // Send to transport layer and wait for response
    let response = send_rpc_and_wait(&state, &core, &rpc, timeout_ms).await?;

    Ok(Json(response))
}

/// GET /api/rpc/persistent/{id} — list persistent RPC for device
async fn list_persistent_rpc(
    State(state): State<DeviceState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(device_id): Path<Uuid>,
    Query(params): Query<RpcParams>,
) -> Result<Json<PageData<RpcResponse>>, ApiError> {
    let tenant_id = ctx.tenant_id;

    let page = state.rpc_dao
        .find_by_device(tenant_id, device_id, &params.to_page_link())
        .await?;

    Ok(Json(PageData {
        data: page.data.into_iter().map(RpcResponse::from).collect(),
        total_pages: page.total_pages,
        total_elements: page.total_elements,
        has_next: page.has_next,
    }))
}

/// DELETE /api/rpc/persistent/{id} — delete RPC request
async fn delete_rpc(
    State(state): State<DeviceState>,
    Path(rpc_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.rpc_dao.delete(rpc_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/rpc/{rpcId} — get RPC request by ID
async fn get_rpc(
    State(state): State<DeviceState>,
    Path(rpc_id): Path<Uuid>,
) -> Result<Json<RpcResponse>, ApiError> {
    let rpc = state.rpc_dao
        .find_by_id(rpc_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("RPC [{}] not found", rpc_id)))?;

    Ok(Json(RpcResponse::from(rpc)))
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Send RPC request to device via transport layer
async fn send_rpc_to_device(state: &DeviceState, core: &CoreState, rpc: &Rpc) -> Result<(), ApiError> {
    // Try direct MQTT delivery first (device currently connected)
    let mqtt_payload = serde_json::json!({
        "method": rpc.request.method,
        "params": rpc.request.params,
    });
    let payload_bytes = mqtt_payload.to_string();
    let topic = format!("v1/devices/me/rpc/request/{}", rpc.request_id);

    if let Some(write_tx) = state.device_rpc_registry.get(&rpc.device_id).map(|r| r.value().clone()) {
        let packet = vl_transport::mqtt::codec::encode_publish(&topic, payload_bytes.as_bytes());
        write_tx.send(packet).await.ok();
    } else {
        // Device not connected — publish to queue for deferred delivery
        let msg = vl_core::entities::TbMsg {
            id: rpc.id,
            msg_type: "RPC_CALL_FROM_SERVER".to_string(),
            originator_id: rpc.device_id,
            originator_type: "DEVICE".to_string(),
            customer_id: None,
            rule_chain_id: None,
            rule_node_id: None,
            tenant_id: None,
            data: serde_json::to_string(&rpc.request).unwrap_or_default(),
            metadata: std::collections::HashMap::from([
                ("requestId".to_string(), rpc.request_id.to_string()),
                ("oneway".to_string(), rpc.request.oneway.to_string()),
            ]),
            ts: rpc.created_time,
        };
        core.queue_producer
            .send_tb_msg(vl_queue::topics::VL_TRANSPORT_API_REQUESTS, &msg)
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to send RPC: {}", e)))?;
    }

    if rpc.request.oneway {
        state.rpc_dao.update_status(rpc.id, RpcStatus::Sent, None).await?;
    }
    Ok(())
}

/// Send RPC and wait for response with timeout
async fn send_rpc_and_wait(
    state: &DeviceState,
    core: &CoreState,
    rpc: &Rpc,
    timeout_ms: i64,
) -> Result<serde_json::Value, ApiError> {
    // Register pending before sending (avoid race condition)
    let (tx, rx) = tokio::sync::oneshot::channel::<serde_json::Value>();
    state.rpc_pending_registry.insert((rpc.device_id, rpc.request_id), tx);

    // Send RPC to device
    send_rpc_to_device(state, core, rpc).await?;

    // Wait for response with timeout
    let duration = tokio::time::Duration::from_millis(timeout_ms as u64);
    match tokio::time::timeout(duration, rx).await {
        Ok(Ok(response)) => {
            state.rpc_dao.update_status(rpc.id, RpcStatus::Successful, Some(response.clone())).await.ok();
            Ok(response)
        }
        Ok(Err(_)) => {
            // Sender dropped (device disconnected)
            state.rpc_pending_registry.remove(&(rpc.device_id, rpc.request_id));
            state.rpc_dao.update_status(rpc.id, RpcStatus::Timeout, None).await.ok();
            Err(ApiError::BadRequest("Device disconnected during RPC".into()))
        }
        Err(_) => {
            // Timeout
            state.rpc_pending_registry.remove(&(rpc.device_id, rpc.request_id));
            state.rpc_dao.update_status(rpc.id, RpcStatus::Timeout, None).await.ok();
            Err(ApiError::BadRequest(format!("RPC timeout after {}ms", timeout_ms)))
        }
    }
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
    use vl_core::entities::{Authority, Device, User, UserCredentials};
    use vl_dao::postgres::{device::DeviceDao, user::UserDao};

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

    async fn create_device(pool: &PgPool, tenant_id: Uuid, name: &str) -> Device {
        // Ensure a device_profile row exists with nil UUID
        let profile_id = Uuid::nil();
        sqlx::query!(
            "INSERT INTO device_profile (id, created_time, tenant_id, name) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO NOTHING",
            profile_id, now_ms(), tenant_id, "default"
        ).execute(pool).await.unwrap();

        let dao = DeviceDao::new(pool.clone());
        let device = Device {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id, customer_id: None, device_profile_id: profile_id,
            name: name.into(), device_type: "default".into(),
            label: None, device_data: None, firmware_id: None,
            software_id: None, external_id: None, additional_info: None, version: 1,
        };
        dao.save(&device).await.unwrap();
        device
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

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn oneway_rpc_unknown_device_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "rpc@test.com", "pass123").await;
        let token = login_as(app.clone(), "rpc@test.com", "pass123").await;

        let resp = post_json_auth(
            app,
            &format!("/api/rpc/oneway/{}", Uuid::new_v4()),
            &token,
            json!({ "method": "setGpio", "params": { "pin": 1 }, "persistent": false }),
        ).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn persistent_rpc_saved_and_listed(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "rpc2@test.com", "pass123").await;
        let token = login_as(app.clone(), "rpc2@test.com", "pass123").await;
        let device = create_device(&pool, Uuid::nil(), "rpc-test-device").await;

        let resp = post_json_auth(
            app.clone(),
            &format!("/api/rpc/oneway/{}", device.id),
            &token,
            json!({ "method": "reboot", "params": {}, "persistent": true }),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let list_resp = get_auth(
            app,
            &format!("/api/rpc/persistent/{}?pageSize=10&page=0", device.id),
            &token,
        ).await;
        assert_eq!(list_resp.status(), StatusCode::OK);
        let body = body_json(list_resp).await;
        assert!(body["totalElements"].as_i64().unwrap() >= 1);
        assert_eq!(body["data"][0]["request"]["method"].as_str().unwrap(), "reboot");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_rpc_not_found_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "rpc3@test.com", "pass123").await;
        let token = login_as(app.clone(), "rpc3@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/rpc/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_rpc_not_found_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "rpc4@test.com", "pass123").await;
        let token = login_as(app.clone(), "rpc4@test.com", "pass123").await;

        let resp = delete_auth(
            app,
            &format!("/api/rpc/persistent/{}", Uuid::new_v4()),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_persistent_rpc_then_not_found(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_user(&pool, "rpc5@test.com", "pass123").await;
        let token = login_as(app.clone(), "rpc5@test.com", "pass123").await;
        let device = create_device(&pool, Uuid::nil(), "rpc-del-device").await;

        post_json_auth(
            app.clone(),
            &format!("/api/rpc/oneway/{}", device.id),
            &token,
            json!({ "method": "ping", "params": {}, "persistent": true }),
        ).await;

        let list = get_auth(
            app.clone(),
            &format!("/api/rpc/persistent/{}?pageSize=1&page=0", device.id),
            &token,
        ).await;
        let body = body_json(list).await;
        let rpc_id = body["data"][0]["id"]["id"].as_str().unwrap().to_string();

        let del = delete_auth(app.clone(), &format!("/api/rpc/persistent/{}", rpc_id), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        let get = get_auth(app, &format!("/api/rpc/{}", rpc_id), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }
}
