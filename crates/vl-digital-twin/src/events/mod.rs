use bevy::prelude::*;
use uuid::Uuid;

use crate::api::AggregationType;

/// Fired when a telemetry value is received from the backend WebSocket.
#[derive(Message, Debug, Clone)]
pub struct TelemetryUpdate {
    /// cmdId used in the subscription → maps to the device
    pub cmd_id:    i32,
    pub device_id: Uuid,
    pub key:       String,
    pub value:     f64,
    pub ts:        i64,
}

/// Fired when an alarm state changes.
#[derive(Message, Debug, Clone)]
pub struct AlarmUpdate {
    pub device_id:  Uuid,
    pub alarm_type: String,
    pub severity:   String,
    pub active:     bool,
}

/// Fired when the WS connection status changes.
#[derive(Message, Debug, Clone)]
pub enum WsStatusEvent {
    Connected,
    Disconnected(String),
}

/// Request to fetch historical timeseries data for a device+range.
/// Fired by UI (Fetch Range button); handled by playback_system.
#[derive(Message, Debug, Clone)]
pub struct FetchHistoryRequest {
    pub device_id: Uuid,
    pub keys:      Vec<String>,
    pub start_ts:  i64,
    pub end_ts:    i64,
    pub agg:       AggregationType,
}

/// Fired when a shared attribute update is received from the backend.
#[derive(Message, Debug, Clone)]
pub struct AttributeUpdate {
    pub device_id: Uuid,
    pub key:       String,
    pub value:     serde_json::Value,
}

// ── RPC events ────────────────────────────────────────────────────────────────

/// UI fires this event when the user triggers an RPC command.
#[derive(Message, Debug, Clone)]
pub struct SendRpcRequest {
    pub device_id:  Uuid,
    pub method:     String,
    pub params:     serde_json::Value,
    pub is_twoway:  bool,
    /// Wall-clock timestamp (ms) when the request was sent.
    pub sent_at:    i64,
}

/// Fired when an async RPC call completes (success or error).
#[derive(Message, Debug, Clone)]
pub struct RpcResult {
    pub device_id:   Uuid,
    pub device_name: String,
    pub method:      String,
    pub result:      Result<serde_json::Value, String>,
    pub sent_at:     i64,
}
