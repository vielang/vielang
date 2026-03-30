use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// RPC request status
/// Java: org.thingsboard.server.common.data.rpc.RpcStatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RpcStatus {
    /// Request queued, waiting to be sent
    Queued,
    /// Request sent to device
    Sent,
    /// Request delivered to device
    Delivered,
    /// Request completed successfully
    Successful,
    /// Request timed out
    Timeout,
    /// Request expired before delivery
    Expired,
    /// Request failed
    Failed,
    /// Request deleted
    Deleted,
}

impl RpcStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RpcStatus::Queued => "QUEUED",
            RpcStatus::Sent => "SENT",
            RpcStatus::Delivered => "DELIVERED",
            RpcStatus::Successful => "SUCCESSFUL",
            RpcStatus::Timeout => "TIMEOUT",
            RpcStatus::Expired => "EXPIRED",
            RpcStatus::Failed => "FAILED",
            RpcStatus::Deleted => "DELETED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "QUEUED" => RpcStatus::Queued,
            "SENT" => RpcStatus::Sent,
            "DELIVERED" => RpcStatus::Delivered,
            "SUCCESSFUL" => RpcStatus::Successful,
            "TIMEOUT" => RpcStatus::Timeout,
            "EXPIRED" => RpcStatus::Expired,
            "FAILED" => RpcStatus::Failed,
            "DELETED" => RpcStatus::Deleted,
            _ => RpcStatus::Queued,
        }
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self,
            RpcStatus::Successful
                | RpcStatus::Timeout
                | RpcStatus::Expired
                | RpcStatus::Failed
                | RpcStatus::Deleted
        )
    }
}

/// Persistent RPC request
/// Java: org.thingsboard.server.common.data.rpc.Rpc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rpc {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub device_id: Uuid,

    /// RPC request ID (for correlation)
    pub request_id: i32,

    /// Expiration time in milliseconds
    pub expiration_time: i64,

    /// Request body
    pub request: RpcRequest,

    /// Response body (if completed)
    pub response: Option<serde_json::Value>,

    /// Current status
    pub status: RpcStatus,

    /// Additional metadata
    pub additional_info: Option<serde_json::Value>,
}

impl Rpc {
    /// Check if the RPC is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        now > self.expiration_time
    }
}

/// RPC request body
/// Java: org.thingsboard.server.common.data.rpc.RpcRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// RPC method name (e.g., "getValue", "setValue")
    pub method: String,

    /// RPC parameters (JSON)
    pub params: serde_json::Value,

    /// Whether this is a one-way request (no response expected)
    #[serde(default)]
    pub oneway: bool,

    /// Request timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout: i64,

    /// Additional metadata
    pub additional_info: Option<serde_json::Value>,
}

fn default_timeout() -> i64 {
    10000 // 10 seconds default
}

/// RPC response from device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Response data
    pub data: serde_json::Value,
    /// Error message if any
    pub error: Option<String>,
}

/// Two-way RPC request for REST API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoWayRpcRequest {
    pub method: String,
    pub params: serde_json::Value,
    #[serde(default)]
    pub persistent: bool,
    #[serde(default = "default_timeout")]
    pub timeout: i64,
    pub retries: Option<i32>,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
}

/// One-way RPC request for REST API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneWayRpcRequest {
    pub method: String,
    pub params: serde_json::Value,
    #[serde(default)]
    pub persistent: bool,
    pub retries: Option<i32>,
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<serde_json::Value>,
}
