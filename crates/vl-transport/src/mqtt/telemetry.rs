use std::sync::Arc;

use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use vl_core::entities::{AttributeKvEntry, AttributeScope, TsRecord};
use vl_dao::{postgres::kv::KvDao, TimeseriesDao};

use crate::error::TransportError;

/// Parse và lưu telemetry payload vào timeseries backend.
/// Hỗ trợ 2 format ThingsBoard:
/// - Simple:   `{"temperature": 25.5, "humidity": 60}`
/// - With ts:  `[{"ts": 1234567890000, "values": {"temperature": 25.5}}]`
pub async fn save_telemetry(
    ts_dao: &Arc<dyn TimeseriesDao>,
    entity_type: &str,
    entity_id: Uuid,
    payload: &[u8],
) -> Result<usize, TransportError> {
    let now_ms = Utc::now().timestamp_millis();

    let json: Value = serde_json::from_slice(payload)
        .map_err(|e| TransportError::InvalidPayload(format!("Invalid JSON: {e}")))?;

    match &json {
        Value::Array(arr) => {
            let mut count = 0;
            for item in arr {
                let ts = item.get("ts").and_then(|v| v.as_i64()).unwrap_or(now_ms);
                let Some(values) = item.get("values").and_then(|v| v.as_object()) else {
                    continue;
                };
                for (key_str, value) in values {
                    let record = build_ts_record(entity_id, key_str, ts, value);
                    ts_dao.save_latest(entity_type, &record).await?;
                    // save history — best-effort
                    if let Err(e) = ts_dao.save(entity_type, &record).await {
                        tracing::warn!("save ts history failed: {e}");
                    }
                    count += 1;
                }
            }
            Ok(count)
        }
        Value::Object(map) => {
            let mut count = 0;
            for (key_str, value) in map {
                let record = build_ts_record(entity_id, key_str, now_ms, value);
                ts_dao.save_latest(entity_type, &record).await?;
                if let Err(e) = ts_dao.save(entity_type, &record).await {
                    tracing::warn!("save ts history failed: {e}");
                }
                count += 1;
            }
            Ok(count)
        }
        _ => Err(TransportError::InvalidPayload(
            "Telemetry payload must be a JSON object or array".into(),
        )),
    }
}

/// Parse và lưu client attributes payload (JSON object).
/// Attributes luôn ở PostgreSQL — dùng KvDao trực tiếp.
pub async fn save_client_attributes(
    kv_dao: &KvDao,
    entity_id: Uuid,
    payload: &[u8],
) -> Result<usize, TransportError> {
    let now_ms = Utc::now().timestamp_millis();

    let json: Value = serde_json::from_slice(payload)
        .map_err(|e| TransportError::InvalidPayload(format!("Invalid JSON: {e}")))?;

    let map = json
        .as_object()
        .ok_or_else(|| TransportError::InvalidPayload("Attributes must be a JSON object".into()))?;

    let mut count = 0;
    for (key_str, value) in map {
        let key_id = kv_dao.get_or_create_key(key_str).await?;
        let attr = build_attr_entry(entity_id, key_id, now_ms, AttributeScope::ClientScope, value);
        kv_dao.save_attribute(&attr).await?;
        count += 1;
    }

    Ok(count)
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

pub fn build_ts_record(entity_id: Uuid, key: &str, ts: i64, value: &Value) -> TsRecord {
    let mut record = TsRecord {
        entity_id,
        key: key.to_string(),
        ts,
        bool_v: None,
        str_v: None,
        long_v: None,
        dbl_v: None,
        json_v: None,
    };
    fill_kv_value(&mut record, value);
    record
}

fn build_attr_entry(
    entity_id: Uuid,
    key_id: i32,
    ts: i64,
    scope: AttributeScope,
    value: &Value,
) -> AttributeKvEntry {
    let mut attr = AttributeKvEntry {
        entity_id,
        attribute_type: scope,
        attribute_key: key_id,
        last_update_ts: ts,
        bool_v: None,
        str_v: None,
        long_v: None,
        dbl_v: None,
        json_v: None,
        version: 0,
    };
    match value {
        Value::Bool(b) => attr.bool_v = Some(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                attr.long_v = Some(i);
            } else if let Some(f) = n.as_f64() {
                attr.dbl_v = Some(f);
            } else {
                attr.str_v = Some(n.to_string());
            }
        }
        Value::String(s) => attr.str_v = Some(s.clone()),
        Value::Object(_) | Value::Array(_) => attr.json_v = Some(value.clone()),
        Value::Null => attr.str_v = Some("null".to_string()),
    }
    attr
}

fn fill_kv_value(record: &mut TsRecord, value: &Value) {
    match value {
        Value::Bool(b) => record.bool_v = Some(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                record.long_v = Some(i);
            } else if let Some(f) = n.as_f64() {
                record.dbl_v = Some(f);
            } else {
                record.str_v = Some(n.to_string());
            }
        }
        Value::String(s) => record.str_v = Some(s.clone()),
        Value::Object(_) | Value::Array(_) => record.json_v = Some(value.clone()),
        Value::Null => record.str_v = Some("null".to_string()),
    }
}
