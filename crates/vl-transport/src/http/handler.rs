use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, warn};

use vl_cache::TbCache;
use vl_core::entities::{ActivityEvent, TbMsg, msg_type};
use vl_dao::{postgres::{device::DeviceDao, kv::KvDao}, DbPool, TimeseriesDao};
use vl_queue::{TbProducer, topics};

use crate::auth::{AuthDevice, authenticate_by_token};
use crate::mqtt::telemetry::{save_client_attributes, save_telemetry};

// ── Shared state ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct HttpTransportState {
    pub pool:             DbPool,
    pub ts_dao:           Arc<dyn TimeseriesDao>,
    pub rule_engine_tx:   Arc<Option<mpsc::Sender<TbMsg>>>,
    pub queue_producer:   Arc<dyn TbProducer>,
    pub cache:            Arc<dyn TbCache>,
    pub ws_tx:            broadcast::Sender<TbMsg>,
    pub activity_tx:      mpsc::Sender<ActivityEvent>,
    pub rpc_pending:      Arc<crate::RpcPendingRegistry>,
}

pub fn router(state: HttpTransportState) -> Router {
    Router::new()
        .route("/api/v1/{token}/telemetry",         post(post_telemetry))
        .route("/api/v1/{token}/attributes",        post(post_attributes).get(get_attributes))
        .route("/api/v1/{token}/attributes/updates",get(get_attribute_updates))
        .route("/api/v1/{token}/rpc",               post(server_rpc).get(poll_rpc))
        .route("/api/v1/{token}/rpc/{request_id}",  post(respond_rpc))
        .route("/api/v1/{token}/claim",             post(claim_device))
        .route("/api/v1/{token}/provision",         post(provision_device))
        .route("/api/v1/{token}/firmware",          get(firmware_info))
        .route("/api/v1/{token}/software",          get(software_info))
        .with_state(state)
}

// ── Telemetry ─────────────────────────────────────────────────────────────────

async fn post_telemetry(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let auth = match auth(&token, &state).await {
        Some(a) => a,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    match save_telemetry(&state.ts_dao, "DEVICE", auth.device_id, &body).await {
        Ok(n) => {
            debug!(device_id = %auth.device_id, entries = n, "HTTP telemetry saved");
            let ts = now_ms();
            state.activity_tx.send(ActivityEvent::Telemetry { device_id: auth.device_id, ts }).await.ok();
            let data = String::from_utf8_lossy(&body).to_string();
            let msg = TbMsg::new(msg_type::POST_TELEMETRY_REQUEST, auth.device_id, "DEVICE", &data)
                .with_tenant(auth.tenant_id);
            send_to_rule_engine(&state.rule_engine_tx, msg.clone());
            publish_to_queue(&state.queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
            let _ = state.ws_tx.send(msg);
            StatusCode::OK.into_response()
        }
        Err(e) => {
            warn!(device_id = %auth.device_id, error = %e, "HTTP telemetry save failed");
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

// ── Attributes ────────────────────────────────────────────────────────────────

async fn post_attributes(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let auth = match auth(&token, &state).await {
        Some(a) => a,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let kv_dao = KvDao::new(state.pool.clone());
    match save_client_attributes(&kv_dao, auth.device_id, &body).await {
        Ok(n) => {
            debug!(device_id = %auth.device_id, entries = n, "HTTP client attributes saved");
            let data = String::from_utf8_lossy(&body).to_string();
            let msg = TbMsg::new(msg_type::POST_ATTRIBUTES_REQUEST, auth.device_id, "DEVICE", &data)
                .with_tenant(auth.tenant_id);
            send_to_rule_engine(&state.rule_engine_tx, msg.clone());
            publish_to_queue(&state.queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
            let _ = state.ws_tx.send(msg);
            StatusCode::OK.into_response()
        }
        Err(e) => {
            warn!(device_id = %auth.device_id, error = %e, "HTTP attributes save failed");
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

#[derive(Deserialize)]
struct AttributeQuery {
    #[serde(rename = "clientKeys")]
    client_keys: Option<String>,
    #[serde(rename = "sharedKeys")]
    shared_keys: Option<String>,
}

async fn get_attributes(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
    Query(params): Query<AttributeQuery>,
) -> impl IntoResponse {
    let auth = match auth(&token, &state).await {
        Some(a) => a,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let kv_dao = KvDao::new(state.pool.clone());

    let client = fetch_scope_attrs(
        &kv_dao, auth.device_id,
        vl_core::entities::AttributeScope::ClientScope,
        params.client_keys.as_deref(),
    ).await;

    let shared = fetch_scope_attrs(
        &kv_dao, auth.device_id,
        vl_core::entities::AttributeScope::SharedScope,
        params.shared_keys.as_deref(),
    ).await;

    Json(json!({ "client": client, "shared": shared })).into_response()
}

// ── Attribute Updates long-poll ───────────────────────────────────────────────

#[derive(Deserialize)]
struct TimeoutQuery {
    timeout: Option<u64>,
}

/// GET /api/v1/{token}/attributes/updates?timeout=30000
/// Long-polls for shared attribute updates pushed by the server.
async fn get_attribute_updates(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
    Query(params): Query<TimeoutQuery>,
) -> impl IntoResponse {
    let auth = match auth(&token, &state).await {
        Some(a) => a,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let timeout_ms = params.timeout.unwrap_or(30_000).min(90_000);
    let device_id  = auth.device_id;
    let mut rx     = state.ws_tx.subscribe();

    let result = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        async move {
            loop {
                match rx.recv().await {
                    Ok(msg)
                        if msg.originator_id == device_id
                            && msg.msg_type == msg_type::ATTRIBUTE_UPDATED =>
                    {
                        return serde_json::from_str::<Value>(&msg.data)
                            .unwrap_or(json!({}));
                    }
                    Ok(_) => continue,
                    Err(_) => return json!({}),
                }
            }
        },
    )
    .await;

    match result {
        Ok(attrs) => Json(attrs).into_response(),
        Err(_)    => Json(json!({})).into_response(), // timeout → empty
    }
}

// ── RPC ───────────────────────────────────────────────────────────────────────

/// POST /api/v1/{token}/rpc — device-to-server (client-side) RPC
async fn server_rpc(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let auth = match auth(&token, &state).await {
        Some(a) => a,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };
    let data = String::from_utf8_lossy(&body).to_string();
    let msg = TbMsg::new(msg_type::RPC_CALL_FROM_SERVER, auth.device_id, "DEVICE", &data)
        .with_tenant(auth.tenant_id);
    send_to_rule_engine(&state.rule_engine_tx, msg.clone());
    publish_to_queue(&state.queue_producer, topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
    StatusCode::OK.into_response()
}

/// GET /api/v1/{token}/rpc?timeout=30000 — device polls for pending server-initiated RPC
async fn poll_rpc(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
    Query(params): Query<TimeoutQuery>,
) -> impl IntoResponse {
    let auth = match auth(&token, &state).await {
        Some(a) => a,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let timeout_ms = params.timeout.unwrap_or(30_000).min(90_000);
    let device_id  = auth.device_id;
    let mut rx     = state.ws_tx.subscribe();

    let result = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        async move {
            loop {
                match rx.recv().await {
                    Ok(msg)
                        if msg.originator_id == device_id
                            && msg.msg_type == msg_type::RPC_CALL_FROM_SERVER =>
                    {
                        return serde_json::from_str::<Value>(&msg.data)
                            .unwrap_or(json!({}));
                    }
                    Ok(_) => continue,
                    Err(_) => return json!({}),
                }
            }
        },
    )
    .await;

    match result {
        Ok(rpc_req) => Json(rpc_req).into_response(),
        Err(_) => (StatusCode::REQUEST_TIMEOUT, Json(json!({}))).into_response(),
    }
}

/// POST /api/v1/{token}/rpc/{request_id} — device responds to server-initiated RPC
async fn respond_rpc(
    Path((token, request_id)): Path<(String, String)>,
    State(state): State<HttpTransportState>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let auth = match auth(&token, &state).await {
        Some(a) => a,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let Ok(req_id) = request_id.parse::<i32>() else {
        return (StatusCode::BAD_REQUEST, "Invalid request ID").into_response();
    };

    let response: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

    if let Some((_, tx)) = state.rpc_pending.remove(&(auth.device_id, req_id)) {
        tx.send(response).ok();
        StatusCode::OK.into_response()
    } else {
        debug!(device_id = %auth.device_id, request_id = %req_id, "HTTP RPC response: no pending request found");
        StatusCode::NOT_FOUND.into_response()
    }
}

// ── Claim device ──────────────────────────────────────────────────────────────

async fn claim_device(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
) -> impl IntoResponse {
    match auth(&token, &state).await {
        Some(_) => StatusCode::OK.into_response(),
        None    => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }
}

// ── Device provisioning ───────────────────────────────────────────────────────

/// POST /api/v1/{token}/provision — device self-provisioning.
///
/// Java: DeviceApiController.provisionDevice()
///
/// Payload:
/// ```json
/// {
///   "deviceName": "NEW_DEVICE",
///   "provisionDeviceKey": "profile_provision_key",
///   "provisionDeviceSecret": "profile_secret"
/// }
/// ```
///
/// Flow:
/// 1. Look up device profile by `provisionDeviceKey`
/// 2. Validate `provisionDeviceSecret` against profile_data
/// 3. Based on provision_type:
///    - AllowCreateNewDevices: create device + credentials
///    - CheckPreProvisionedDevices: device must already exist
/// 4. Return credentials
async fn provision_device(
    Path(_token): Path<String>,
    State(state): State<HttpTransportState>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let payload: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({
            "status": "FAILURE",
            "errorMsg": "Invalid JSON payload"
        }))).into_response(),
    };

    let device_name = match payload.get("deviceName").and_then(|v| v.as_str()) {
        Some(n) if !n.is_empty() => n.to_string(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({
            "status": "FAILURE",
            "errorMsg": "deviceName is required"
        }))).into_response(),
    };

    let provision_key = match payload.get("provisionDeviceKey").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return (StatusCode::BAD_REQUEST, Json(json!({
            "status": "FAILURE",
            "errorMsg": "provisionDeviceKey is required"
        }))).into_response(),
    };

    let provision_secret = payload
        .get("provisionDeviceSecret")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // 1. Look up device profile by provision key.
    let profile_dao = vl_dao::postgres::device_profile::DeviceProfileDao::new(state.pool.clone());
    let profile = match profile_dao.find_by_provision_key(&provision_key).await {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::BAD_REQUEST, Json(json!({
            "status": "FAILURE",
            "errorMsg": "Invalid provision key"
        }))).into_response(),
        Err(e) => {
            warn!("Provision key lookup failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "status": "FAILURE",
                "errorMsg": "Internal error"
            }))).into_response();
        }
    };

    // 2. Validate provision secret from profile_data.provisionConfiguration.provisionDeviceSecret
    let expected_secret = profile
        .profile_data
        .as_ref()
        .and_then(|d| d.get("provisionConfiguration"))
        .and_then(|c| c.get("provisionDeviceSecret"))
        .and_then(|s| s.as_str())
        .unwrap_or("");

    if !expected_secret.is_empty() && provision_secret != expected_secret {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "status": "FAILURE",
            "errorMsg": "Invalid provision secret"
        }))).into_response();
    }

    // 3. Check provision type.
    use vl_core::entities::device_profile::DeviceProvisionType;
    let device_dao = DeviceDao::new(state.pool.clone());

    match profile.provision_type {
        DeviceProvisionType::Disabled => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "status": "FAILURE",
                "errorMsg": "Device provisioning is disabled for this profile"
            }))).into_response();
        }
        DeviceProvisionType::AllowCreateNewDevices => {
            // Check if device already exists.
            if let Ok(Some(existing)) = device_dao.find_by_name(profile.tenant_id, &device_name).await {
                // Device exists — return its credentials.
                if let Ok(Some(creds)) = device_dao.get_credentials(existing.id).await {
                    return Json(json!({
                        "status": "SUCCESS",
                        "credentialsType": creds.credentials_type,
                        "credentialsValue": creds.credentials_id
                    })).into_response();
                }
            }

            // Create new device.
            let device_type = payload
                .get("deviceType")
                .and_then(|v| v.as_str())
                .unwrap_or(&profile.name);
            let now = chrono::Utc::now().timestamp_millis();
            let device = vl_core::entities::Device {
                id:                uuid::Uuid::new_v4(),
                created_time:      now,
                tenant_id:         profile.tenant_id,
                customer_id:       None,
                device_profile_id: profile.id,
                name:              device_name.clone(),
                device_type:       device_type.to_string(),
                label:             None,
                device_data:       None,
                firmware_id:       profile.firmware_id,
                software_id:       profile.software_id,
                external_id:       None,
                additional_info:   None,
                version:           1,
            };

            if let Err(e) = device_dao.save(&device).await {
                warn!("Failed to save provisioned device: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "status": "FAILURE",
                    "errorMsg": "Failed to create device"
                }))).into_response();
            }

            // Generate access token credentials.
            let access_token = uuid::Uuid::new_v4().to_string().replace('-', "");
            let creds = vl_core::entities::DeviceCredentials {
                id:               uuid::Uuid::new_v4(),
                created_time:     now,
                device_id:        device.id,
                credentials_type: vl_core::entities::DeviceCredentialsType::AccessToken,
                credentials_id:   access_token.clone(),
                credentials_value: None,
            };

            if let Err(e) = device_dao.save_credentials(&creds).await {
                warn!("Failed to save credentials for provisioned device: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "status": "FAILURE",
                    "errorMsg": "Failed to create credentials"
                }))).into_response();
            }

            debug!(device_name = %device_name, device_id = %device.id, "Device provisioned successfully");

            Json(json!({
                "status": "SUCCESS",
                "credentialsType": "ACCESS_TOKEN",
                "credentialsValue": access_token
            })).into_response()
        }
        DeviceProvisionType::CheckPreProvisionedDevices => {
            // Device must already exist.
            let device = match device_dao.find_by_name(profile.tenant_id, &device_name).await {
                Ok(Some(d)) => d,
                Ok(None) => return (StatusCode::BAD_REQUEST, Json(json!({
                    "status": "FAILURE",
                    "errorMsg": "Device not found (pre-provisioned mode)"
                }))).into_response(),
                Err(e) => {
                    warn!("Device lookup failed: {e}");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "status": "FAILURE",
                        "errorMsg": "Internal error"
                    }))).into_response();
                }
            };

            // Return existing credentials.
            match device_dao.get_credentials(device.id).await {
                Ok(Some(creds)) => {
                    Json(json!({
                        "status": "SUCCESS",
                        "credentialsType": creds.credentials_type,
                        "credentialsValue": creds.credentials_id
                    })).into_response()
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "status": "FAILURE",
                    "errorMsg": "Device credentials not found"
                }))).into_response(),
            }
        }
        DeviceProvisionType::X509CertificateChain => {
            // X509 provisioning — validate certificate from payload.
            // Payload should contain: {"credentialsType": "X509_CERTIFICATE", "hash": "sha3_of_cert"}
            // or the raw certificate PEM in "credentialsValue".
            let cert_hash = payload
                .get("hash")
                .and_then(|v| v.as_str())
                .or_else(|| payload.get("credentialsValue").and_then(|v| v.as_str()));

            let Some(hash) = cert_hash else {
                return (StatusCode::BAD_REQUEST, Json(json!({
                    "status": "FAILURE",
                    "errorMsg": "X509 provisioning requires 'hash' or 'credentialsValue' in payload"
                }))).into_response();
            };

            // Check if device with this X509 hash already exists.
            if let Ok(Some(existing)) = device_dao.find_by_credentials_id(hash).await {
                return Json(json!({
                    "status": "SUCCESS",
                    "credentialsType": "X509_CERTIFICATE",
                    "credentialsValue": hash
                })).into_response();
            }

            // Create new device with X509 credentials.
            let now = chrono::Utc::now().timestamp_millis();
            let device = vl_core::entities::Device {
                id:                uuid::Uuid::new_v4(),
                created_time:      now,
                tenant_id:         profile.tenant_id,
                customer_id:       None,
                device_profile_id: profile.id,
                name:              device_name.clone(),
                device_type:       profile.name.clone(),
                label:             None,
                device_data:       None,
                firmware_id:       profile.firmware_id,
                software_id:       profile.software_id,
                external_id:       None,
                additional_info:   None,
                version:           1,
            };

            if let Err(e) = device_dao.save(&device).await {
                warn!("Failed to save X509 provisioned device: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "status": "FAILURE",
                    "errorMsg": "Failed to create device"
                }))).into_response();
            }

            let creds = vl_core::entities::DeviceCredentials {
                id:               uuid::Uuid::new_v4(),
                created_time:     now,
                device_id:        device.id,
                credentials_type: vl_core::entities::DeviceCredentialsType::X509Certificate,
                credentials_id:   hash.to_string(),
                credentials_value: payload.get("credentialsValue").and_then(|v| v.as_str()).map(String::from),
            };

            if let Err(e) = device_dao.save_credentials(&creds).await {
                warn!("Failed to save X509 credentials: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "status": "FAILURE",
                    "errorMsg": "Failed to create credentials"
                }))).into_response();
            }

            debug!(device_name = %device_name, device_id = %device.id, "X509 device provisioned");
            Json(json!({
                "status": "SUCCESS",
                "credentialsType": "X509_CERTIFICATE",
                "credentialsValue": hash
            })).into_response()
        }
    }
}

// ── OTA ───────────────────────────────────────────────────────────────────────

async fn firmware_info(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
) -> impl IntoResponse {
    match auth(&token, &state).await {
        Some(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({"errorCode": 2, "message": "No OTA firmware configured"})),
        ).into_response(),
        None => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }
}

async fn software_info(
    Path(token): Path<String>,
    State(state): State<HttpTransportState>,
) -> impl IntoResponse {
    match auth(&token, &state).await {
        Some(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({"errorCode": 2, "message": "No OTA software configured"})),
        ).into_response(),
        None => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn auth(token: &str, state: &HttpTransportState) -> Option<AuthDevice> {
    let device_dao = DeviceDao::new(state.pool.clone());
    authenticate_by_token(token, &device_dao, &state.cache).await
}

async fn fetch_scope_attrs(
    kv_dao:   &KvDao,
    entity_id: uuid::Uuid,
    scope:     vl_core::entities::AttributeScope,
    keys_csv:  Option<&str>,
) -> Value {
    let key_ids_opt: Option<Vec<i32>> = if let Some(csv) = keys_csv {
        let key_names: Vec<String> = csv
            .split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();
        if key_names.is_empty() {
            return json!({});
        }
        match kv_dao.lookup_key_ids(&key_names).await {
            Ok(m) if !m.is_empty() => Some(m.values().copied().collect()),
            _ => return json!({}),
        }
    } else {
        None
    };

    let entries = match kv_dao.find_attributes(entity_id, scope, key_ids_opt.as_deref()).await {
        Ok(e) => e,
        Err(_) => return json!({}),
    };

    let mut obj = serde_json::Map::new();
    for entry in &entries {
        if let Ok(Some(name)) = kv_dao.get_key_name(entry.attribute_key).await {
            let val = attr_to_value(entry);
            obj.insert(name, val);
        }
    }
    Value::Object(obj)
}

fn attr_to_value(entry: &vl_core::entities::AttributeKvEntry) -> Value {
    if let Some(v) = &entry.str_v  { return Value::String(v.clone()); }
    if let Some(v) = entry.long_v  { return json!(v); }
    if let Some(v) = entry.dbl_v   { return json!(v); }
    if let Some(v) = entry.bool_v  { return Value::Bool(v); }
    if let Some(v) = &entry.json_v { return v.clone(); }
    Value::Null
}

fn send_to_rule_engine(tx: &Arc<Option<mpsc::Sender<TbMsg>>>, msg: TbMsg) {
    if let Some(sender) = tx.as_ref() {
        if let Err(e) = sender.try_send(msg) {
            debug!("Rule engine channel full: {}", e);
        }
    }
}

async fn publish_to_queue(producer: &Arc<dyn TbProducer>, topic: &str, msg: &TbMsg) {
    if let Err(e) = producer.send_tb_msg(topic, msg).await {
        debug!("Queue publish error on {}: {}", topic, e);
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
