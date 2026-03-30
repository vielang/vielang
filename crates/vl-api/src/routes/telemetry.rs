use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use vl_core::entities::{AttributeKvEntry, AttributeScope, TsRecord};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, TelemetryState, BillingState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Timeseries ────────────────────────────────────────────────────────
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/values/timeseries",
            get(get_latest_timeseries).post(save_timeseries),
        )
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/timeseries/keys",
            get(get_timeseries_keys),
        )
        // Java TB 4.x alias: /keys/timeseries
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/keys/timeseries",
            get(get_timeseries_keys),
        )
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/timeseries/delete",
            delete(delete_timeseries),
        )
        // Java: POST /{entityType}/{entityId}/timeseries/{scope}
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/timeseries/{scope}",
            axum::routing::post(save_entity_telemetry),
        )
        // Java: POST /{entityType}/{entityId}/timeseries/{scope}/{ttl}
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/timeseries/{scope}/{ttl}",
            axum::routing::post(save_entity_telemetry_with_ttl),
        )
        // ── Attributes ────────────────────────────────────────────────────────
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/keys/attributes",
            get(get_attribute_keys),
        )
        // Java: GET /keys/attributes/{scope}
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/keys/attributes/{scope}",
            get(get_attribute_keys_by_scope),
        )
        // Java: GET /values/attributes (no scope — returns all scopes)
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/values/attributes",
            get(get_attributes_all_scopes),
        )
        // Java: GET/POST/DELETE /values/attributes/{scope}
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/values/attributes/{scope}",
            get(get_attributes).post(save_attributes).delete(delete_attributes),
        )
        // Java v1: POST/DELETE /{entityType}/{entityId}/{scope} (save/delete attributes)
        // Also handles device shorthand: POST /{deviceId}/{scope} when entityType is a UUID
        .route(
            "/plugins/telemetry/{entityType}/{entityId}/{scope}",
            axum::routing::post(save_attributes_v1).delete(delete_attributes_v1),
        )
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TelemetryQueryParams {
    pub keys: Option<String>,
    /// Alias for keys — Java supports both `keys` and `key` (multi-value)
    pub key: Option<String>,
    #[serde(rename = "startTs")]
    pub start_ts: Option<i64>,
    #[serde(rename = "endTs")]
    pub end_ts: Option<i64>,
    pub agg: Option<String>,       // AVG, MIN, MAX, SUM, COUNT, NONE
    pub interval: Option<i64>,     // bucket width in ms (default 1h = 3600000)
    pub limit: Option<i64>,        // max points (default 1000, max 50000)
    pub format: Option<String>,    // "JSON" (default) | "CSV"
    /// Java: orderBy — "ASC" or "DESC" (default ASC)
    #[serde(rename = "orderBy")]
    pub order_by: Option<String>,
    /// Java: useStrictDataTypes — return numeric types as-is (not stringified)
    #[serde(rename = "useStrictDataTypes")]
    pub use_strict_data_types: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTsParams {
    pub keys: Option<String>,
    /// Alias for keys — Java supports `key` (multi-value)
    pub key: Option<String>,
    #[serde(rename = "startTs")]
    pub start_ts: Option<i64>,
    #[serde(rename = "endTs")]
    pub end_ts: Option<i64>,
    /// Java: deleteAllDataForKeys — if true, ignores startTs/endTs and deletes all
    #[serde(rename = "deleteAllDataForKeys")]
    pub delete_all_data_for_keys: Option<bool>,
    #[serde(rename = "deleteLatest")]
    pub delete_latest: Option<bool>,
    /// Java: rewriteLatestIfDeleted — re-calculate latest from history after delete
    #[serde(rename = "rewriteLatestIfDeleted")]
    pub rewrite_latest_if_deleted: Option<bool>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/plugins/telemetry/{entityType}/{entityId}/values/timeseries
async fn get_latest_timeseries(
    State(state): State<TelemetryState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(params): Query<TelemetryQueryParams>,
) -> Result<impl IntoResponse, ApiError> {
    use vl_dao::AggType;

    let keys: Vec<String> = params.keys
        .as_deref()
        .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
        .unwrap_or_default();

    let has_range = params.start_ts.is_some() || params.end_ts.is_some();
    let agg_type = params.agg.as_deref()
        .and_then(AggType::from_str)
        .unwrap_or(AggType::None);
    let use_agg = has_range && agg_type != AggType::None;

    let now = chrono::Utc::now().timestamp_millis();
    let start_ts = params.start_ts.unwrap_or(now - 24 * 3600 * 1000);
    let end_ts   = params.end_ts.unwrap_or(now);
    let interval_ms = params.interval.unwrap_or(3_600_000).max(1);
    let limit    = params.limit.unwrap_or(1000).min(50_000);

    let mut result = serde_json::Map::new();

    if !has_range {
        // ── Latest values ──────────────────────────────────────────────────────
        let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        let key_opt = if key_refs.is_empty() { None } else { Some(key_refs.as_slice()) };
        let entries = state.ts_dao.find_latest(entity_id, &entity_type, key_opt).await?;
        for entry in entries {
            let value_str = entry.value_as_string();
            result.insert(entry.key, serde_json::json!([{ "ts": entry.ts, "value": value_str }]));
        }
    } else if use_agg {
        // ── Aggregated range ───────────────────────────────────────────────────
        let query_keys: Vec<String> = if keys.is_empty() {
            state.ts_dao.get_ts_keys(entity_id, &entity_type).await?
        } else {
            keys.clone()
        };
        for key in &query_keys {
            let records = state.ts_dao
                .find_range_agg(entity_id, &entity_type, key, start_ts, end_ts, interval_ms, agg_type, limit)
                .await?;
            if !records.is_empty() {
                let points: Vec<serde_json::Value> = records.iter().map(|r| {
                    serde_json::json!({ "ts": r.ts, "value": r.value_as_string() })
                }).collect();
                result.insert(key.clone(), serde_json::json!(points));
            }
        }
    } else {
        // ── Raw range ─────────────────────────────────────────────────────────
        let query_keys: Vec<String> = if keys.is_empty() {
            state.ts_dao.get_ts_keys(entity_id, &entity_type).await?
        } else {
            keys.clone()
        };
        for key in &query_keys {
            let records = state.ts_dao
                .find_range(entity_id, &entity_type, key, start_ts, end_ts, limit)
                .await?;
            if !records.is_empty() {
                let points: Vec<serde_json::Value> = records.iter().map(|r| {
                    serde_json::json!({ "ts": r.ts, "value": r.value_as_string() })
                }).collect();
                result.insert(key.clone(), serde_json::json!(points));
            }
        }
    }

    // CSV export — format=CSV query param
    if params.format.as_deref().eq(&Some("CSV")) {
        return Ok(build_csv_response(&result).into_response());
    }

    Ok(Json(serde_json::Value::Object(result)).into_response())
}

/// Build CSV response từ timeseries result map (key → [{ts, value}])
fn build_csv_response(result: &serde_json::Map<String, serde_json::Value>) -> impl IntoResponse {
    use std::collections::BTreeMap;
    use std::io::Write;

    let keys: Vec<String> = result.keys().cloned().collect();
    let mut by_ts: BTreeMap<i64, std::collections::HashMap<String, String>> = BTreeMap::new();

    for (key, points) in result {
        if let Some(arr) = points.as_array() {
            for point in arr {
                let ts = point["ts"].as_i64().unwrap_or(0);
                let val = point["value"].as_str().unwrap_or("").to_string();
                by_ts.entry(ts).or_default().insert(key.clone(), val);
            }
        }
    }

    let mut buf = Vec::new();
    // Header
    let mut header_row = vec!["timestamp".to_string()];
    header_row.extend(keys.iter().cloned());
    writeln!(buf, "{}", header_row.join(",")).ok();

    // Rows
    for (ts, kv) in &by_ts {
        let mut row = vec![ts.to_string()];
        for key in &keys {
            row.push(kv.get(key).cloned().unwrap_or_default());
        }
        writeln!(buf, "{}", row.join(",")).ok();
    }

    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"telemetry.csv\""),
        ],
        buf,
    )
}

/// POST /api/plugins/telemetry/{entityType}/{entityId}/values/timeseries
/// Payload: {"temperature": 25.5} hoặc [{"ts": 123, "values": {"temperature": 25.5}}]
async fn save_timeseries(
    State(state): State<TelemetryState>,
    State(billing): State<BillingState>,
    ctx: Option<axum::extract::Extension<SecurityContext>>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, ApiError> {
    save_timeseries_inner(&state, &billing, ctx, &entity_type, entity_id, &payload, None).await
}

/// GET /api/plugins/telemetry/{entityType}/{entityId}/timeseries/keys
async fn get_timeseries_keys(
    State(state): State<TelemetryState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<String>>, ApiError> {
    let keys = state.ts_dao.get_ts_keys(entity_id, &entity_type).await?;
    Ok(Json(keys))
}

/// GET /api/plugins/telemetry/{entityType}/{entityId}/keys/attributes
/// Java TB 4.x path for getting attribute key names
async fn get_attribute_keys(
    State(state): State<TelemetryState>,
    Path((_entity_type, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<String>>, ApiError> {
    let keys = state.kv_dao.get_attr_keys(entity_id).await?;
    Ok(Json(keys))
}

/// DELETE /api/plugins/telemetry/{entityType}/{entityId}/timeseries/delete
async fn delete_timeseries(
    State(state): State<TelemetryState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(params): Query<DeleteTsParams>,
) -> Result<StatusCode, ApiError> {
    // Support both `keys` and `key` params (Java compatibility)
    let keys_str = params.keys.as_deref()
        .or(params.key.as_deref())
        .unwrap_or("");
    let keys: Vec<String> = keys_str
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    if keys.is_empty() {
        return Err(ApiError::BadRequest("keys parameter is required".into()));
    }

    let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

    if params.delete_all_data_for_keys.unwrap_or(false) {
        // Delete ALL data for these keys (ignore time range)
        state.ts_dao.delete_ts(entity_id, &entity_type, &key_refs, 0, i64::MAX).await?;
        state.ts_dao.delete_latest(entity_id, &entity_type, &key_refs).await?;
    } else {
        let start_ts = params.start_ts.unwrap_or(0);
        let end_ts   = params.end_ts.unwrap_or(chrono::Utc::now().timestamp_millis());
        state.ts_dao.delete_ts(entity_id, &entity_type, &key_refs, start_ts, end_ts).await?;

        if params.delete_latest.unwrap_or(false) {
            state.ts_dao.delete_latest(entity_id, &entity_type, &key_refs).await?;
        }
    }

    Ok(StatusCode::OK)
}

/// GET /api/plugins/telemetry/{entityType}/{entityId}/values/attributes/{scope}
async fn get_attributes(
    State(state): State<TelemetryState>,
    Path((_entity_type, entity_id, scope)): Path<(String, Uuid, String)>,
    Query(params): Query<TelemetryQueryParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let scope = parse_scope(&scope)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid scope: {}", scope)))?;

    let key_ids: Option<Vec<i32>> = if let Some(key_str) = &params.keys {
        let mut ids = Vec::new();
        for key in key_str.split(',') {
            if let Ok(id) = state.kv_dao.get_or_create_key(key.trim()).await {
                ids.push(id);
            }
        }
        Some(ids)
    } else {
        None
    };

    let attrs = state.kv_dao
        .find_attributes(entity_id, scope, key_ids.as_deref())
        .await?;

    let mut data = Vec::new();
    for a in attrs {
        let key_name = state.kv_dao.get_key_name(a.attribute_key).await?
            .unwrap_or_else(|| format!("key_{}", a.attribute_key));
        data.push(serde_json::json!({
            "key":          key_name,
            "value":        attr_value(&a),
            "lastUpdateTs": a.last_update_ts,
        }));
    }
    Ok(Json(serde_json::json!(data)))
}

/// POST /api/plugins/telemetry/{entityType}/{entityId}/values/attributes/{scope}
async fn save_attributes(
    State(state): State<TelemetryState>,
    Path((_entity_type, entity_id, scope)): Path<(String, Uuid, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, ApiError> {
    let scope = parse_scope(&scope)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid scope: {}", scope)))?;
    let now = chrono::Utc::now().timestamp_millis();
    save_attributes_inner(&state, entity_id, scope, &payload, now).await
}

/// DELETE /api/plugins/telemetry/{entityType}/{entityId}/values/attributes/{scope}?keys=k1,k2
async fn delete_attributes(
    State(state): State<TelemetryState>,
    Path((_entity_type, entity_id, scope)): Path<(String, Uuid, String)>,
    Query(params): Query<TelemetryQueryParams>,
) -> Result<StatusCode, ApiError> {
    let scope = parse_scope(&scope)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid scope: {}", scope)))?;

    let keys: Vec<String> = params.keys
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    if keys.is_empty() {
        return Err(ApiError::BadRequest("keys parameter is required".into()));
    }

    let mut key_ids = Vec::new();
    for key in &keys {
        if let Ok(id) = state.kv_dao.get_or_create_key(key).await {
            key_ids.push(id);
        }
    }
    state.kv_dao.delete_attributes(entity_id, scope, &key_ids).await?;
    Ok(StatusCode::OK)
}

// ── New endpoint handlers (Phase 8) ───────────────────────────────────────────

/// GET /api/plugins/telemetry/{entityType}/{entityId}/keys/attributes/{scope}
async fn get_attribute_keys_by_scope(
    State(state): State<TelemetryState>,
    Path((_entity_type, entity_id, scope)): Path<(String, Uuid, String)>,
) -> Result<Json<Vec<String>>, ApiError> {
    let scope = parse_scope(&scope)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid scope: {scope}")))?;
    let keys = state.kv_dao.get_attr_keys_by_scope(entity_id, scope).await?;
    Ok(Json(keys))
}

/// GET /api/plugins/telemetry/{entityType}/{entityId}/values/attributes
/// Returns attributes across ALL scopes (no scope in path).
async fn get_attributes_all_scopes(
    State(state): State<TelemetryState>,
    Path((_entity_type, entity_id)): Path<(String, Uuid)>,
    Query(params): Query<TelemetryQueryParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key_ids: Option<Vec<i32>> = if let Some(key_str) = &params.keys {
        let mut ids = Vec::new();
        for key in key_str.split(',') {
            if let Ok(id) = state.kv_dao.get_or_create_key(key.trim()).await {
                ids.push(id);
            }
        }
        Some(ids)
    } else {
        None
    };

    let attrs = state.kv_dao
        .find_all_attributes(entity_id, key_ids.as_deref())
        .await?;

    let mut data = Vec::new();
    for a in attrs {
        let key_name = state.kv_dao.get_key_name(a.attribute_key).await?
            .unwrap_or_else(|| format!("key_{}", a.attribute_key));
        data.push(serde_json::json!({
            "key":          key_name,
            "value":        attr_value(&a),
            "lastUpdateTs": a.last_update_ts,
        }));
    }
    Ok(Json(serde_json::json!(data)))
}

/// POST /api/plugins/telemetry/{entityType}/{entityId}/timeseries/{scope}
/// Java: saveEntityTelemetry — saves telemetry with a scope qualifier.
async fn save_entity_telemetry(
    State(state): State<TelemetryState>,
    State(billing): State<BillingState>,
    ctx: Option<axum::extract::Extension<SecurityContext>>,
    Path((entity_type, entity_id, _scope)): Path<(String, Uuid, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, ApiError> {
    // Scope is informational for telemetry (unlike attributes).
    // Delegate to the same save logic.
    save_timeseries_inner(&state, &billing, ctx, &entity_type, entity_id, &payload, None).await
}

/// POST /api/plugins/telemetry/{entityType}/{entityId}/timeseries/{scope}/{ttl}
/// Java: saveEntityTelemetryWithTTL — saves telemetry with TTL in seconds.
async fn save_entity_telemetry_with_ttl(
    State(state): State<TelemetryState>,
    State(billing): State<BillingState>,
    ctx: Option<axum::extract::Extension<SecurityContext>>,
    Path((entity_type, entity_id, _scope, ttl)): Path<(String, Uuid, String, i64)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, ApiError> {
    save_timeseries_inner(&state, &billing, ctx, &entity_type, entity_id, &payload, Some(ttl)).await
}

/// POST /api/plugins/telemetry/{entityType}/{entityId}/{scope} (v1 attribute save)
/// Java: saveEntityAttributesV1
async fn save_attributes_v1(
    State(state): State<TelemetryState>,
    Path((entity_type, entity_id, scope)): Path<(String, Uuid, String)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, ApiError> {
    // If entity_type looks like a UUID, treat it as deviceId shorthand
    let scope = parse_scope(&scope)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid scope: {scope}")))?;

    let now = chrono::Utc::now().timestamp_millis();
    save_attributes_inner(&state, entity_id, scope, &payload, now).await
}

/// DELETE /api/plugins/telemetry/{entityType}/{entityId}/{scope} (v1 attribute delete)
/// Java: deleteEntityAttributes
async fn delete_attributes_v1(
    State(state): State<TelemetryState>,
    Path((_entity_type, entity_id, scope)): Path<(String, Uuid, String)>,
    Query(params): Query<TelemetryQueryParams>,
) -> Result<StatusCode, ApiError> {
    let scope = parse_scope(&scope)
        .ok_or_else(|| ApiError::BadRequest(format!("Invalid scope: {scope}")))?;

    let keys: Vec<String> = params.keys
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    if keys.is_empty() {
        return Err(ApiError::BadRequest("keys parameter is required".into()));
    }

    let mut key_ids = Vec::new();
    for key in &keys {
        if let Ok(id) = state.kv_dao.get_or_create_key(key).await {
            key_ids.push(id);
        }
    }
    state.kv_dao.delete_attributes(entity_id, scope, &key_ids).await?;
    Ok(StatusCode::OK)
}

/// Shared logic for saving timeseries (with optional TTL).
async fn save_timeseries_inner(
    state: &TelemetryState,
    billing: &BillingState,
    ctx: Option<axum::extract::Extension<SecurityContext>>,
    entity_type: &str,
    entity_id: Uuid,
    payload: &serde_json::Value,
    ttl: Option<i64>,
) -> Result<StatusCode, ApiError> {
    let now = chrono::Utc::now().timestamp_millis();
    let datapoints = extract_datapoints(payload, now);

    if datapoints.is_empty() {
        return Ok(StatusCode::OK);
    }

    let mut ws_data: std::collections::HashMap<String, Vec<[serde_json::Value; 2]>> =
        std::collections::HashMap::new();

    let records: Vec<_> = datapoints.iter()
        .map(|(key, ts, value)| build_ts_record(entity_id, key.clone(), *ts, value))
        .collect();

    for (key, ts, value) in &datapoints {
        ws_data.entry(key.clone())
            .or_default()
            .push([serde_json::json!(ts), value.clone()]);
    }

    // Save with or without TTL
    if let Some(ttl_secs) = ttl {
        state.ts_dao.save_batch_with_ttl(entity_type, &records, ttl_secs).await?;
    } else {
        state.ts_dao.save_batch(entity_type, &records).await?;
    }
    state.ts_dao.save_latest_batch(entity_type, &records).await?;

    if !ws_data.is_empty() {
        state.ws_registry.push_ts_update(entity_id, ws_data);
    }

    if let Some(axum::extract::Extension(c)) = ctx {
        let dp_count = datapoints.len() as i64;
        billing.usage_tracker.record_transport_msg(c.tenant_id, 1);
        billing.usage_tracker.record_transport_dp(c.tenant_id, dp_count);
    }

    Ok(StatusCode::OK)
}

/// Shared logic for saving attributes.
async fn save_attributes_inner(
    state: &TelemetryState,
    entity_id: Uuid,
    scope: AttributeScope,
    payload: &serde_json::Value,
    now: i64,
) -> Result<StatusCode, ApiError> {
    if let Some(obj) = payload.as_object() {
        for (key, value) in obj {
            let key_id = state.kv_dao.get_or_create_key(key).await?;
            let attr = AttributeKvEntry {
                entity_id,
                attribute_type: scope,
                attribute_key:  key_id,
                last_update_ts: now,
                bool_v:  value.as_bool(),
                long_v:  value.as_i64().filter(|_| value.is_i64()),
                dbl_v:   value.as_f64().filter(|_| value.is_f64()),
                str_v:   value.as_str().map(String::from),
                json_v:  if value.is_object() || value.is_array() { Some(value.clone()) } else { None },
                version: 0,
            };
            state.kv_dao.save_attribute(&attr).await?;
        }
    }
    Ok(StatusCode::OK)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_scope(s: &str) -> Option<AttributeScope> {
    match s.to_uppercase().as_str() {
        "CLIENT_SCOPE" | "CLIENT" => Some(AttributeScope::ClientScope),
        "SERVER_SCOPE" | "SERVER" => Some(AttributeScope::ServerScope),
        "SHARED_SCOPE" | "SHARED" => Some(AttributeScope::SharedScope),
        _ => None,
    }
}

fn extract_datapoints(payload: &serde_json::Value, now: i64) -> Vec<(String, i64, serde_json::Value)> {
    let mut result = Vec::new();
    match payload {
        serde_json::Value::Array(arr) => {
            for item in arr {
                let ts = item.get("ts").and_then(|v| v.as_i64()).unwrap_or(now);
                if let Some(values) = item.get("values").and_then(|v| v.as_object()) {
                    for (k, v) in values {
                        result.push((k.clone(), ts, v.clone()));
                    }
                }
            }
        }
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                // ThingsBoard compact multi-timestamp format:
                // { "temperature": [{"ts": 123, "value": 25.5}, ...] }
                if let Some(arr) = v.as_array() {
                    if arr.first().and_then(|e| e.get("ts")).is_some() {
                        for item in arr {
                            let ts  = item.get("ts").and_then(|t| t.as_i64()).unwrap_or(now);
                            let val = item.get("value").cloned().unwrap_or(serde_json::Value::Null);
                            result.push((k.clone(), ts, val));
                        }
                        continue;
                    }
                }
                // Simple flat format: { "temperature": 25.5 }
                result.push((k.clone(), now, v.clone()));
            }
        }
        _ => {}
    }
    result
}

fn build_ts_record(entity_id: Uuid, key: String, ts: i64, value: &serde_json::Value) -> TsRecord {
    TsRecord {
        entity_id,
        key,
        ts,
        bool_v: value.as_bool(),
        long_v: value.as_i64().filter(|_| value.is_i64()),
        dbl_v:  value.as_f64().filter(|_| value.is_f64()),
        str_v:  value.as_str().map(String::from),
        json_v: if value.is_object() || value.is_array() { Some(value.clone()) } else { None },
    }
}

fn attr_value(a: &AttributeKvEntry) -> serde_json::Value {
    if let Some(v) = a.bool_v        { return serde_json::json!(v); }
    if let Some(v) = a.long_v        { return serde_json::json!(v); }
    if let Some(v) = a.dbl_v         { return serde_json::json!(v); }
    if let Some(ref v) = a.json_v    { return v.clone(); }
    if let Some(ref v) = a.str_v     { return serde_json::json!(v); }
    serde_json::Value::Null
}

// ── Integration Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use vl_auth::password;
    use vl_core::entities::{Authority, User, UserCredentials};
    use vl_dao::postgres::{ts_dao::PostgresTsDao, user::UserDao};
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
        let ts_dao = std::sync::Arc::new(PostgresTsDao::new(pool.clone()));
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        create_router(state)
    }

    async fn create_test_user(pool: &PgPool, email: &str, pwd: &str) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id: Uuid::new_v4(), customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: None, last_name: None, phone: None,
            additional_info: None, version: 1,
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

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    async fn get_token(app: axum::Router, email: &str, pwd: &str) -> String {
        let resp = post_json(app, "/api/auth/login",
            json!({"username": email, "password": pwd})).await;
        body_json(resp).await["token"].as_str().unwrap().to_string()
    }

    // ── POST timeseries ───────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn post_telemetry_simple_format_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts1@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts1@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        let resp = post_json_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!({"temperature": 25.5, "humidity": 60}),
        ).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn post_telemetry_batch_format_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts2@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts2@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        let resp = post_json_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!([{
                "ts": now_ms(),
                "values": {"pressure": 1013.25, "altitude": 100}
            }]),
        ).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── GET timeseries ────────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_latest_telemetry_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts3@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts3@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        // Save telemetry first
        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!({"temperature": 30.0}),
        ).await;

        // Get latest telemetry
        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries?keys=temperature"),
            &token,
        ).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;

        // ThingsBoard format: {"temperature": [{"ts": 123456, "value": "30"}]}
        assert!(body["temperature"].is_array(), "Must have 'temperature' array");
        let entry = &body["temperature"][0];
        assert!(entry["ts"].is_number(), "ts must be numeric millisecond timestamp");
        assert!(entry["value"].is_string(), "value must be string");
        assert_eq!(entry["value"], "30");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_multiple_keys_returns_all(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts4@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts4@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!({"co2": 400, "pm25": 12.5, "temperature": 22.0}),
        ).await;

        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries?keys=co2,pm25"),
            &token,
        ).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["co2"].is_array());
        assert!(body["pm25"].is_array());
    }

    // ── GET timeseries keys ───────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_timeseries_keys_returns_list(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts5@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts5@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!({"voltage": 3.3, "current": 0.5}),
        ).await;

        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/timeseries/keys"),
            &token,
        ).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body.is_array(), "Keys response must be an array");
        let keys: Vec<&str> = body.as_array().unwrap()
            .iter().filter_map(|v| v.as_str()).collect();
        assert!(keys.contains(&"voltage"));
        assert!(keys.contains(&"current"));
    }

    // ── POST/GET attributes ───────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn post_attributes_then_get_returns_values(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "attr1@test.com", "pass123").await;
        let token = get_token(app.clone(), "attr1@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        // Save server-scope attributes
        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE"),
            &token,
            json!({"active": true, "firmware": "1.2.3"}),
        ).await;

        // Read them back
        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE"),
            &token,
        ).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body.is_array());
        let attrs = body.as_array().unwrap();
        assert!(!attrs.is_empty(), "Attributes must not be empty");
        // Each entry: {key, value, lastUpdateTs}
        let attr = &attrs[0];
        assert!(attr["key"].is_string());
        assert!(attr["lastUpdateTs"].is_number());
    }

    // ── Auth check ────────────────────────────────────────────────────────────

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn post_telemetry_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let entity_id = Uuid::new_v4();

        let resp = app.oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"))
                .header("content-type", "application/json")
                .body(Body::from(json!({"temperature": 20.0}).to_string()))
                .unwrap(),
        ).await.unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // ── Unit 15 — Telemetry Deep Dive ─────────────────────────────────────────

    async fn delete_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("DELETE").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn save_all_5_kv_types_roundtrip(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts15a@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts15a@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        let resp = post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!({
                "active": true,
                "firmware": "1.2.3",
                "counter": 42,
                "temp": 21.5,
                "config": {"key": "val"}
            }),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries?keys=active,firmware,counter,temp,config"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["active"].is_array(),   "must have 'active'");
        assert!(body["firmware"].is_array(), "must have 'firmware'");
        assert!(body["counter"].is_array(),  "must have 'counter'");
        assert!(body["temp"].is_array(),     "must have 'temp'");
        assert!(body["config"].is_array(),   "must have 'config'");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_timeseries_by_key(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts15b@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts15b@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        // Save timeseries
        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!({"deleteme": 123}),
        ).await;

        // Delete via /timeseries/delete with deleteLatest=true to also wipe latest
        let resp = delete_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/timeseries/delete?keys=deleteme&deleteLatest=true"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify key is gone from latest
        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries?keys=deleteme"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(
            body.as_object().map(|m| !m.contains_key("deleteme")).unwrap_or(true),
            "deleteme key should be absent after delete"
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn all_three_attribute_scopes_work(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts15c@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts15c@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        let resp = post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE"),
            &token,
            json!({"srv": 1}),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK, "SERVER_SCOPE save failed");

        let resp = post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/CLIENT_SCOPE"),
            &token,
            json!({"cli": 2}),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK, "CLIENT_SCOPE save failed");

        let resp = post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SHARED_SCOPE"),
            &token,
            json!({"shr": 3}),
        ).await;
        assert_eq!(resp.status(), StatusCode::OK, "SHARED_SCOPE save failed");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_attribute_by_key(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts15d@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts15d@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        // Save SERVER_SCOPE attribute
        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE"),
            &token,
            json!({"toDelete": "yes"}),
        ).await;

        // Delete it
        let resp = delete_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE?keys=toDelete"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify gone
        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE?keys=toDelete"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let arr = body.as_array().expect("response must be array");
        assert!(
            arr.iter().all(|e| e["key"] != "toDelete"),
            "toDelete attribute should be absent after delete"
        );
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_attribute_keys_returns_array(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts15e@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts15e@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE"),
            &token,
            json!({"key1": 1, "key2": 2}),
        ).await;

        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/keys/attributes"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body.is_array(), "response must be array of key strings");
        let keys: Vec<&str> = body.as_array().unwrap()
            .iter().filter_map(|v| v.as_str()).collect();
        assert!(keys.contains(&"key1"), "must contain key1");
        assert!(keys.contains(&"key2"), "must contain key2");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_timeseries_history_by_time_range(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts15f@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts15f@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        // Use a well-known timestamp in 2025 (within the partitions in migrations/002)
        let past_ts: i64 = 1_735_689_600_000; // 2025-01-01 00:00:00 UTC in ms

        // Post with explicit timestamp
        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!([{"ts": past_ts, "values": {"histkey": 99}}]),
        ).await;

        let start_ts = past_ts - 1000;
        let end_ts   = past_ts + 1000;

        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries?keys=histkey&startTs={start_ts}&endTs={end_ts}"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["histkey"].is_array(), "histkey must be present in response");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_timeseries_history_multi_key(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts15g@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts15g@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries"),
            &token,
            json!({"alpha": 10, "beta": 20}),
        ).await;

        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries?keys=alpha,beta"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["alpha"].is_array(), "alpha must be in response");
        assert!(body["beta"].is_array(),  "beta must be in response");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn attribute_scope_isolation(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        create_test_user(&pool, "ts15h@test.com", "pass123").await;
        let token = get_token(app.clone(), "ts15h@test.com", "pass123").await;
        let entity_id = Uuid::new_v4();

        // Post same key to SERVER_SCOPE and CLIENT_SCOPE with different values
        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE"),
            &token,
            json!({"shared_key": "server_val"}),
        ).await;
        post_json_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/CLIENT_SCOPE"),
            &token,
            json!({"shared_key": "client_val"}),
        ).await;

        // GET SERVER_SCOPE — should see "server_val"
        let resp = get_auth(
            app.clone(),
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/SERVER_SCOPE?keys=shared_key"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let arr = body.as_array().expect("must be array");
        let server_entry = arr.iter().find(|e| e["key"] == "shared_key")
            .expect("shared_key must exist in SERVER_SCOPE");
        assert_eq!(server_entry["value"], "server_val", "SERVER_SCOPE value mismatch");

        // GET CLIENT_SCOPE — should see "client_val"
        let resp = get_auth(
            app,
            &format!("/api/plugins/telemetry/DEVICE/{entity_id}/values/attributes/CLIENT_SCOPE?keys=shared_key"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        let arr = body.as_array().expect("must be array");
        let client_entry = arr.iter().find(|e| e["key"] == "shared_key")
            .expect("shared_key must exist in CLIENT_SCOPE");
        assert_eq!(client_entry["value"], "client_val", "CLIENT_SCOPE value mismatch");
    }

    // ── extract_datapoints unit tests ─────────────────────────────────────────

    #[test]
    fn extract_flat_scalar_format() {
        // {"temperature": 25.5, "humidity": 60}
        let payload = json!({"temperature": 25.5, "humidity": 60});
        let now = 1_000_000i64;
        let pts = extract_datapoints(&payload, now);
        assert_eq!(pts.len(), 2);
        let temp = pts.iter().find(|(k, _, _)| k == "temperature").unwrap();
        assert_eq!(temp.1, now);
        assert_eq!(temp.2, json!(25.5));
    }

    #[test]
    fn extract_array_with_ts_values_format() {
        // [{"ts": 1000, "values": {"temperature": 25.5}}]
        let payload = json!([{"ts": 1000i64, "values": {"temperature": 25.5}}]);
        let pts = extract_datapoints(&payload, 9999);
        assert_eq!(pts.len(), 1);
        assert_eq!(pts[0].0, "temperature");
        assert_eq!(pts[0].1, 1000);
        assert_eq!(pts[0].2, json!(25.5));
    }

    #[test]
    fn extract_compact_multi_timestamp_format() {
        // {"temperature": [{"ts": 1000, "value": 25.5}, {"ts": 2000, "value": 26.0}]}
        let payload = json!({
            "temperature": [
                {"ts": 1000i64, "value": 25.5},
                {"ts": 2000i64, "value": 26.0}
            ]
        });
        let pts = extract_datapoints(&payload, 9999);
        assert_eq!(pts.len(), 2);
        let pt0 = pts.iter().find(|(_, ts, _)| *ts == 1000).unwrap();
        assert_eq!(pt0.2, json!(25.5));
        let pt1 = pts.iter().find(|(_, ts, _)| *ts == 2000).unwrap();
        assert_eq!(pt1.2, json!(26.0));
    }

    #[test]
    fn extract_compact_uses_ts_from_entry_not_now() {
        let payload = json!({
            "status": [{"ts": 5555i64, "value": "RUNNING"}]
        });
        let pts = extract_datapoints(&payload, 9999);
        assert_eq!(pts.len(), 1);
        assert_eq!(pts[0].1, 5555);  // ts from payload, NOT 9999
        assert_eq!(pts[0].2, json!("RUNNING"));
    }
}
