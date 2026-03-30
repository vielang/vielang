use uuid::Uuid;
use tracing::{debug, warn, error};

use crate::edge::proto::edge::{
    UplinkMsg, DownlinkMsg,
    TelemetryUploadRequestMsg, KeyValueProto, KeyValueType,
};

/// Callback trait injected từ vl-api để handle uplink events.
/// vl-cluster không phụ thuộc vào vl-dao/vl-rule-engine, dùng trait injection.
#[async_trait::async_trait]
pub trait EdgeUplinkHandler: Send + Sync + 'static {
    /// Xác thực Edge bằng routing_key + secret.
    /// Returns (edge_id, tenant_id) nếu thành công.
    async fn authenticate_edge(
        &self,
        routing_key: &str,
        secret: &str,
    ) -> Result<(Uuid, Uuid), AuthError>;

    /// Lưu telemetry từ Edge device lên cloud DB.
    async fn save_edge_telemetry(
        &self,
        edge_id:   Uuid,
        tenant_id: Uuid,
        device_id: Uuid,
        entries:   Vec<TelemetryEntry>,
    ) -> Result<(), String>;

    /// Forward RPC call từ Edge device lên cloud rule engine.
    async fn handle_edge_rpc(
        &self,
        edge_id:    Uuid,
        tenant_id:  Uuid,
        device_id:  Uuid,
        request_id: i64,
        method:     String,
        params:     String,
    ) -> Result<(), String>;

    /// Handle device connect event từ Edge.
    async fn handle_device_connect(
        &self,
        edge_id:   Uuid,
        tenant_id: Uuid,
        device_id: Uuid,
    ) -> Result<(), String>;

    /// Handle device disconnect event từ Edge.
    async fn handle_device_disconnect(
        &self,
        edge_id:   Uuid,
        tenant_id: Uuid,
        device_id: Uuid,
    ) -> Result<(), String>;

    /// Gửi initial sync (push all entities Edge cần biết) sau khi Edge connect.
    async fn send_initial_sync(
        &self,
        edge_id:   Uuid,
        tenant_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, String>;
}

/// Error type cho authentication.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Edge not found")]
    NotFound,
    #[error("Invalid secret")]
    InvalidSecret,
    #[error("Database error: {0}")]
    Database(String),
}

/// Parsed telemetry entry (từ proto TsKvListProto).
#[derive(Debug, Clone)]
pub struct TelemetryEntry {
    pub ts:    i64,
    pub key:   String,
    pub value: TelemetryValue,
}

/// Parsed telemetry value.
#[derive(Debug, Clone)]
pub enum TelemetryValue {
    Bool(bool),
    Long(i64),
    Double(f64),
    String(String),
    Json(serde_json::Value),
}

/// Process một UplinkMsg từ Edge.
/// Returns ACK DownlinkMsg với cùng msg_id.
pub async fn process_uplink(
    edge_id:   Uuid,
    tenant_id: Uuid,
    msg:       UplinkMsg,
    handler:   &dyn EdgeUplinkHandler,
) -> Result<DownlinkMsg, String> {
    let uplink_id = msg.uplink_msg_id;

    // Process telemetry
    for telemetry in &msg.telemetry_msg {
        if let Err(e) = process_telemetry(edge_id, tenant_id, telemetry, handler).await {
            warn!(edge_id = %edge_id, "Telemetry error: {}", e);
        }
    }

    // Process RPC calls
    for rpc in &msg.device_rpc_call_msg {
        let device_id = match rpc.device_id_msc.parse::<Uuid>() {
            Ok(id) => id,
            Err(_) => { warn!("Invalid device_id in RPC: {}", rpc.device_id_msc); continue; }
        };
        if let Err(e) = handler.handle_edge_rpc(
            edge_id, tenant_id, device_id,
            rpc.request_id, rpc.method.clone(), rpc.params.clone(),
        ).await {
            warn!(edge_id = %edge_id, "RPC error: {}", e);
        }
    }

    // Process connect events
    for conn in &msg.connect_request_msg {
        // connect_request_msg also used for auth (first msg) — here handle device connects
        // The routing_key here is a device routing key (not edge auth)
        debug!(edge_id = %edge_id, "Device connect via edge: {}", conn.routing_key);
    }

    // Process disconnect events
    for disc in &msg.disconnect_request_msg {
        debug!(edge_id = %edge_id, "Device disconnect via edge: {}", disc.routing_key);
    }

    // ACK: return DownlinkMsg with same msg_id
    Ok(DownlinkMsg {
        downlink_msg_id: uplink_id,
        ..Default::default()
    })
}

async fn process_telemetry(
    edge_id:   Uuid,
    tenant_id: Uuid,
    msg:       &TelemetryUploadRequestMsg,
    handler:   &dyn EdgeUplinkHandler,
) -> Result<(), String> {
    let device_id: Uuid = msg.entity_id_msc.parse()
        .map_err(|_| format!("Invalid device_id: {}", msg.entity_id_msc))?;

    let entries: Vec<TelemetryEntry> = msg.data.iter().flat_map(|ts_list| {
        ts_list.kv.iter().filter_map(|kv| {
            parse_kv_entry(ts_list.ts, kv)
        }).collect::<Vec<_>>()
    }).collect();

    if !entries.is_empty() {
        handler.save_edge_telemetry(edge_id, tenant_id, device_id, entries).await?;
    }
    Ok(())
}

fn parse_kv_entry(ts: i64, kv: &KeyValueProto) -> Option<TelemetryEntry> {
    let value = match kv.r#type {
        t if t == KeyValueType::BooleanV as i32 => TelemetryValue::Bool(kv.bool_v),
        t if t == KeyValueType::LongV    as i32 => TelemetryValue::Long(kv.long_v),
        t if t == KeyValueType::DoubleV  as i32 => TelemetryValue::Double(kv.double_v),
        t if t == KeyValueType::StringV  as i32 => TelemetryValue::String(kv.string_v.clone()),
        t if t == KeyValueType::JsonV    as i32 => {
            let json = serde_json::from_slice(&kv.json_v).unwrap_or(serde_json::Value::Null);
            TelemetryValue::Json(json)
        }
        _ => return None,
    };
    Some(TelemetryEntry { ts, key: kv.key.clone(), value })
}
