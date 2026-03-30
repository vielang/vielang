//! Sparkplug B protocol support for MQTT transport.
//!
//! Implements the Sparkplug B specification for industrial IoT:
//! - Topic namespace: `spBv1.0/{group_id}/{message_type}/{edge_node_id}/{device_id?}`
//! - Message types: NBIRTH, NDEATH, DBIRTH, DDEATH, NDATA, DDATA, NCMD, DCMD
//! - Metrics: key-value pairs with typed values (int, float, string, bool, bytes)
//!
//! Java: org.thingsboard.server.transport.mqtt.util.sparkplug.*

use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Sparkplug B message types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparkplugMessageType {
    /// Node birth certificate — edge node comes online.
    NBirth,
    /// Node death certificate — edge node goes offline.
    NDeath,
    /// Device birth certificate — device under edge node comes online.
    DBirth,
    /// Device death certificate — device under edge node goes offline.
    DDeath,
    /// Node data — telemetry from edge node itself.
    NData,
    /// Device data — telemetry from device under edge node.
    DData,
    /// Node command — command to edge node.
    NCmd,
    /// Device command — command to device under edge node.
    DCmd,
}

impl SparkplugMessageType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "NBIRTH" => Some(Self::NBirth),
            "NDEATH" => Some(Self::NDeath),
            "DBIRTH" => Some(Self::DBirth),
            "DDEATH" => Some(Self::DDeath),
            "NDATA" => Some(Self::NData),
            "DDATA" => Some(Self::DData),
            "NCMD" => Some(Self::NCmd),
            "DCMD" => Some(Self::DCmd),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NBirth => "NBIRTH",
            Self::NDeath => "NDEATH",
            Self::DBirth => "DBIRTH",
            Self::DDeath => "DDEATH",
            Self::NData => "NDATA",
            Self::DData => "DDATA",
            Self::NCmd => "NCMD",
            Self::DCmd => "DCMD",
        }
    }

    /// Whether this message type carries metric data (telemetry).
    pub fn has_metrics(&self) -> bool {
        matches!(
            self,
            Self::NBirth | Self::DBirth | Self::NData | Self::DData
        )
    }

    /// Whether this is a birth certificate.
    pub fn is_birth(&self) -> bool {
        matches!(self, Self::NBirth | Self::DBirth)
    }

    /// Whether this is a death certificate.
    pub fn is_death(&self) -> bool {
        matches!(self, Self::NDeath | Self::DDeath)
    }
}

/// Parsed Sparkplug B topic.
#[derive(Debug, Clone)]
pub struct SparkplugTopic {
    pub group_id: String,
    pub message_type: SparkplugMessageType,
    pub edge_node_id: String,
    pub device_id: Option<String>,
}

impl SparkplugTopic {
    /// Parse a Sparkplug B topic string.
    ///
    /// Format: `spBv1.0/{group_id}/{message_type}/{edge_node_id}/{device_id?}`
    pub fn parse(topic: &str) -> Option<Self> {
        let parts: Vec<&str> = topic.split('/').collect();
        if parts.len() < 4 || parts[0] != "spBv1.0" {
            return None;
        }

        let message_type = SparkplugMessageType::from_str(parts[2])?;

        Some(Self {
            group_id: parts[1].to_string(),
            message_type,
            edge_node_id: parts[3].to_string(),
            device_id: parts.get(4).map(|s| s.to_string()),
        })
    }

    /// Check if a topic is a Sparkplug B topic.
    pub fn is_sparkplug(topic: &str) -> bool {
        topic.starts_with("spBv1.0/")
    }

    /// Construct a device name from Sparkplug topic components.
    /// For device-level messages: "{edge_node_id}/{device_id}"
    /// For node-level messages: "{edge_node_id}"
    pub fn device_name(&self) -> String {
        match &self.device_id {
            Some(did) => format!("{}/{}", self.edge_node_id, did),
            None => self.edge_node_id.clone(),
        }
    }
}

/// A single Sparkplug B metric (simplified JSON representation).
///
/// In production, this would use Protobuf (prost), but for compatibility
/// we support both JSON and a simplified binary format.
#[derive(Debug, Clone)]
pub struct SparkplugMetric {
    pub name: String,
    pub timestamp: i64,
    pub datatype: SparkplugDataType,
    pub value: SparkplugValue,
}

/// Sparkplug B data types (subset of the full 34-type spec).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparkplugDataType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    Boolean,
    String,
    DateTime,
    Bytes,
    Unknown(u32),
}

impl SparkplugDataType {
    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => Self::Int8,
            2 => Self::Int16,
            3 => Self::Int32,
            4 => Self::Int64,
            5 => Self::UInt8,
            6 => Self::UInt16,
            7 => Self::UInt32,
            8 => Self::UInt64,
            9 => Self::Float,
            10 => Self::Double,
            11 => Self::Boolean,
            12 => Self::String,
            13 => Self::DateTime,
            17 => Self::Bytes,
            other => Self::Unknown(other),
        }
    }
}

/// Sparkplug metric value.
#[derive(Debug, Clone)]
pub enum SparkplugValue {
    Int(i64),
    UInt(u64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    Null,
}

impl SparkplugValue {
    /// Convert to a JSON-compatible serde_json::Value.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Int(v) => serde_json::json!(v),
            Self::UInt(v) => serde_json::json!(v),
            Self::Float(v) => serde_json::json!(v),
            Self::Double(v) => serde_json::json!(v),
            Self::Bool(v) => serde_json::json!(v),
            Self::String(v) => serde_json::json!(v),
            Self::Bytes(v) => serde_json::json!(base64_encode(v)),
            Self::Null => serde_json::Value::Null,
        }
    }
}

fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 63) as usize] as char);
        result.push(CHARS[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Parse Sparkplug B payload from JSON format.
///
/// JSON payload format (ThingsBoard compatibility):
/// ```json
/// {
///   "timestamp": 1234567890,
///   "metrics": [
///     {"name": "temperature", "timestamp": 1234567890, "dataType": "Double", "value": 25.5},
///     {"name": "status", "timestamp": 1234567890, "dataType": "Boolean", "value": true}
///   ]
/// }
/// ```
pub fn parse_sparkplug_json(payload: &[u8]) -> Vec<SparkplugMetric> {
    let body: serde_json::Value = match serde_json::from_slice(payload) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let default_ts = body
        .get("timestamp")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    let metrics = match body.get("metrics").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    metrics
        .iter()
        .filter_map(|m| {
            let name = m.get("name").and_then(|v| v.as_str())?.to_string();
            let ts = m
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or(default_ts);
            let dt_str = m
                .get("dataType")
                .and_then(|v| v.as_str())
                .unwrap_or("Double");
            let raw_value = m.get("value")?;

            let (datatype, value) = match dt_str {
                "Boolean" | "Bool" => (
                    SparkplugDataType::Boolean,
                    SparkplugValue::Bool(raw_value.as_bool().unwrap_or(false)),
                ),
                "Int8" | "Int16" | "Int32" | "Int64" => (
                    SparkplugDataType::Int64,
                    SparkplugValue::Int(raw_value.as_i64().unwrap_or(0)),
                ),
                "UInt8" | "UInt16" | "UInt32" | "UInt64" => (
                    SparkplugDataType::UInt64,
                    SparkplugValue::UInt(raw_value.as_u64().unwrap_or(0)),
                ),
                "Float" => (
                    SparkplugDataType::Float,
                    SparkplugValue::Float(raw_value.as_f64().unwrap_or(0.0) as f32),
                ),
                "Double" => (
                    SparkplugDataType::Double,
                    SparkplugValue::Double(raw_value.as_f64().unwrap_or(0.0)),
                ),
                "String" | "Text" => (
                    SparkplugDataType::String,
                    SparkplugValue::String(
                        raw_value.as_str().unwrap_or("").to_string(),
                    ),
                ),
                _ => (
                    SparkplugDataType::Double,
                    SparkplugValue::Double(raw_value.as_f64().unwrap_or(0.0)),
                ),
            };

            Some(SparkplugMetric {
                name,
                timestamp: ts,
                datatype,
                value,
            })
        })
        .collect()
}

/// Convert Sparkplug metrics to TsRecord entries for timeseries storage.
pub fn metrics_to_ts_records(
    entity_id: Uuid,
    metrics: &[SparkplugMetric],
) -> Vec<vl_core::entities::TsRecord> {
    metrics
        .iter()
        .map(|m| {
            let (bool_v, long_v, dbl_v, str_v) = match &m.value {
                SparkplugValue::Bool(v) => (Some(*v), None, None, None),
                SparkplugValue::Int(v) => (None, Some(*v), None, None),
                SparkplugValue::UInt(v) => (None, Some(*v as i64), None, None),
                SparkplugValue::Float(v) => (None, None, Some(*v as f64), None),
                SparkplugValue::Double(v) => (None, None, Some(*v), None),
                SparkplugValue::String(v) => (None, None, None, Some(v.clone())),
                SparkplugValue::Bytes(_) => (None, None, None, Some("[bytes]".to_string())),
                SparkplugValue::Null => (None, None, None, None),
            };
            vl_core::entities::TsRecord {
                entity_id,
                key: m.name.clone(),
                ts: m.timestamp,
                bool_v,
                long_v,
                dbl_v,
                str_v,
                json_v: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_topic() {
        let topic = "spBv1.0/myGroup/DDATA/edgeNode1/device1";
        let parsed = SparkplugTopic::parse(topic).unwrap();
        assert_eq!(parsed.group_id, "myGroup");
        assert_eq!(parsed.message_type, SparkplugMessageType::DData);
        assert_eq!(parsed.edge_node_id, "edgeNode1");
        assert_eq!(parsed.device_id.as_deref(), Some("device1"));
        assert_eq!(parsed.device_name(), "edgeNode1/device1");
    }

    #[test]
    fn test_parse_node_topic() {
        let topic = "spBv1.0/factory/NBIRTH/plc01";
        let parsed = SparkplugTopic::parse(topic).unwrap();
        assert_eq!(parsed.message_type, SparkplugMessageType::NBirth);
        assert!(parsed.device_id.is_none());
        assert_eq!(parsed.device_name(), "plc01");
    }

    #[test]
    fn test_parse_json_metrics() {
        let payload = r#"{
            "timestamp": 1700000000000,
            "metrics": [
                {"name": "temp", "timestamp": 1700000000000, "dataType": "Double", "value": 25.5},
                {"name": "active", "timestamp": 1700000000000, "dataType": "Boolean", "value": true}
            ]
        }"#;
        let metrics = parse_sparkplug_json(payload.as_bytes());
        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].name, "temp");
        assert_eq!(metrics[1].name, "active");
    }

    #[test]
    fn test_is_sparkplug() {
        assert!(SparkplugTopic::is_sparkplug("spBv1.0/group/NDATA/node"));
        assert!(!SparkplugTopic::is_sparkplug("v1/devices/me/telemetry"));
    }
}
