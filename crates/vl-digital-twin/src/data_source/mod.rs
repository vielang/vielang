//! Phase 41 — Industrial data source abstraction layer.
//!
//! Unified trait for connecting to various industrial data sources:
//! OPC UA, Modbus, MQTT, HTTP REST, Sparkplug B, and custom protocols.
//! The twin client uses these to ingest telemetry from diverse sources.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Data point ───────────────────────────────────────────────────────────────

/// A single data point from any source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Source node identifier (OPC UA NodeId, Modbus register, MQTT topic, etc.).
    pub source_id: String,
    /// Telemetry key name.
    pub key: String,
    /// Value (always JSON for flexibility).
    pub value: serde_json::Value,
    /// Timestamp in milliseconds.
    pub timestamp_ms: i64,
    /// Quality/status of the reading.
    pub quality: DataQuality,
    /// Optional metadata.
    pub metadata: HashMap<String, String>,
}

/// Data quality indicator (based on OPC UA quality codes).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataQuality {
    Good,
    Uncertain,
    Bad,
    /// Communication error.
    CommError,
    /// Sensor malfunction.
    SensorError,
    /// Value out of range.
    OutOfRange,
    /// Timestamp inconsistency.
    TimeError,
}

impl Default for DataQuality {
    fn default() -> Self {
        Self::Good
    }
}

impl DataQuality {
    pub fn is_good(&self) -> bool {
        matches!(self, Self::Good)
    }

    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Good | Self::Uncertain)
    }
}

// ── Data source configuration ────────────────────────────────────────────────

/// Configuration for an industrial data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    pub id: Uuid,
    pub name: String,
    pub source_type: DataSourceType,
    pub enabled: bool,
    /// Connection string / URL.
    pub endpoint: String,
    /// Polling interval (ms). None = event-driven / subscription.
    pub poll_interval_ms: Option<u64>,
    /// Authentication credentials.
    pub auth: Option<DataSourceAuth>,
    /// Key mappings: source_key → twin_key.
    pub key_mappings: HashMap<String, String>,
    /// Custom configuration (protocol-specific).
    pub custom: HashMap<String, serde_json::Value>,
}

/// Supported data source protocols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSourceType {
    /// OPC UA — Industrial automation standard.
    OpcUa {
        /// OPC UA security mode.
        security_mode: OpcUaSecurityMode,
        /// Node IDs to subscribe/poll.
        node_ids: Vec<String>,
        /// Namespace index for node IDs.
        namespace_index: u16,
    },
    /// Modbus TCP/RTU — Legacy PLC/SCADA protocol.
    Modbus {
        variant: ModbusVariant,
        /// Slave/unit ID.
        unit_id: u8,
        /// Register definitions.
        registers: Vec<ModbusRegister>,
    },
    /// MQTT — Lightweight pub/sub.
    Mqtt {
        topics: Vec<String>,
        qos: u8,
        /// JSON path to extract value from payload.
        value_path: Option<String>,
    },
    /// HTTP REST — Polling-based.
    HttpRest {
        method: String,
        headers: HashMap<String, String>,
        /// JSON path to extract values from response.
        value_paths: HashMap<String, String>,
    },
    /// Sparkplug B — Industrial MQTT.
    SparkplugB {
        group_id: String,
        edge_node_id: String,
        device_id: Option<String>,
    },
    /// BACnet — Building automation.
    BacNet {
        device_instance: u32,
        object_ids: Vec<String>,
    },
    /// Custom protocol with plugin-style handler.
    Custom {
        protocol_name: String,
        config: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpcUaSecurityMode {
    None,
    Sign,
    SignAndEncrypt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModbusVariant {
    Tcp,
    Rtu,
}

/// Modbus register definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusRegister {
    pub address: u16,
    pub register_type: ModbusRegisterType,
    pub key: String,
    pub data_type: ModbusDataType,
    /// Scale factor (multiply raw value by this).
    pub scale: f64,
    /// Offset (add to scaled value).
    pub offset: f64,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModbusRegisterType {
    Coil,
    DiscreteInput,
    HoldingRegister,
    InputRegister,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModbusDataType {
    UInt16,
    Int16,
    UInt32,
    Int32,
    Float32,
    Float64,
    Boolean,
}

/// Authentication for data sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSourceAuth {
    /// Username/password.
    Basic { username: String, password: String },
    /// Certificate-based (OPC UA, MQTT TLS).
    Certificate { cert_path: String, key_path: String, ca_path: Option<String> },
    /// Bearer token.
    Token { token: String },
    /// API key.
    ApiKey { header: String, key: String },
    /// Anonymous.
    Anonymous,
}

// ── Data source status ───────────────────────────────────────────────────────

/// Runtime status of a data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceStatus {
    pub source_id: Uuid,
    pub connected: bool,
    pub last_data_at: Option<i64>,
    pub error: Option<String>,
    pub points_received: u64,
    pub points_errors: u64,
    /// Average latency (ms) between source timestamp and receipt.
    pub avg_latency_ms: Option<f64>,
}

impl DataSourceStatus {
    pub fn new(source_id: Uuid) -> Self {
        Self {
            source_id,
            connected: false,
            last_data_at: None,
            error: None,
            points_received: 0,
            points_errors: 0,
            avg_latency_ms: None,
        }
    }

    pub fn record_success(&mut self, timestamp_ms: i64) {
        self.connected = true;
        self.last_data_at = Some(timestamp_ms);
        self.points_received += 1;
        self.error = None;
    }

    pub fn record_error(&mut self, error: &str) {
        self.points_errors += 1;
        self.error = Some(error.into());
    }

    /// Availability percentage.
    pub fn availability_pct(&self) -> f64 {
        let total = self.points_received + self.points_errors;
        if total == 0 {
            return 0.0;
        }
        self.points_received as f64 / total as f64 * 100.0
    }
}

// ── Data source manager ──────────────────────────────────────────────────────

/// Central manager for all data sources — Bevy resource.
#[derive(Resource, Default)]
pub struct DataSourceManager {
    /// Registered data sources.
    pub sources: HashMap<Uuid, DataSourceConfig>,
    /// Runtime status per source.
    pub status: HashMap<Uuid, DataSourceStatus>,
    /// Incoming data buffer (drained by twin update systems).
    pub data_buffer: Vec<(Uuid, DataPoint)>,
}

impl DataSourceManager {
    /// Register a new data source.
    pub fn register(&mut self, config: DataSourceConfig) {
        let id = config.id;
        self.status.insert(id, DataSourceStatus::new(id));
        self.sources.insert(id, config);
    }

    /// Remove a data source.
    pub fn remove(&mut self, source_id: Uuid) {
        self.sources.remove(&source_id);
        self.status.remove(&source_id);
    }

    /// Push a data point from a source.
    pub fn push_data(&mut self, source_id: Uuid, point: DataPoint) {
        if let Some(status) = self.status.get_mut(&source_id) {
            if point.quality.is_usable() {
                status.record_success(point.timestamp_ms);
            } else {
                status.record_error(&format!("Bad quality: {:?}", point.quality));
            }
        }

        // Apply key mapping
        let mapped_key = self.sources.get(&source_id)
            .and_then(|cfg| cfg.key_mappings.get(&point.key))
            .cloned();

        let mut mapped_point = point;
        if let Some(key) = mapped_key {
            mapped_point.key = key;
        }

        self.data_buffer.push((source_id, mapped_point));
    }

    /// Drain all buffered data points.
    pub fn drain_data(&mut self) -> Vec<(Uuid, DataPoint)> {
        std::mem::take(&mut self.data_buffer)
    }

    /// Get source by ID.
    pub fn get_source(&self, id: Uuid) -> Option<&DataSourceConfig> {
        self.sources.get(&id)
    }

    /// Get status by source ID.
    pub fn get_status(&self, id: Uuid) -> Option<&DataSourceStatus> {
        self.status.get(&id)
    }

    /// All enabled sources.
    pub fn enabled_sources(&self) -> Vec<&DataSourceConfig> {
        self.sources.values().filter(|s| s.enabled).collect()
    }

    /// Overall health: percentage of connected sources.
    pub fn health_pct(&self) -> f64 {
        let enabled: Vec<_> = self.sources.values().filter(|s| s.enabled).collect();
        if enabled.is_empty() {
            return 100.0;
        }
        let connected = enabled.iter()
            .filter(|s| self.status.get(&s.id).map(|st| st.connected).unwrap_or(false))
            .count();
        connected as f64 / enabled.len() as f64 * 100.0
    }
}

// ── OPC UA node helpers ──────────────────────────────────────────────────────

/// Build an OPC UA data source config.
pub fn opc_ua_source(
    name: &str,
    endpoint: &str,
    node_ids: Vec<&str>,
    poll_interval_ms: u64,
) -> DataSourceConfig {
    DataSourceConfig {
        id: Uuid::new_v4(),
        name: name.into(),
        source_type: DataSourceType::OpcUa {
            security_mode: OpcUaSecurityMode::None,
            node_ids: node_ids.into_iter().map(String::from).collect(),
            namespace_index: 2,
        },
        enabled: true,
        endpoint: endpoint.into(),
        poll_interval_ms: Some(poll_interval_ms),
        auth: Some(DataSourceAuth::Anonymous),
        key_mappings: HashMap::new(),
        custom: HashMap::new(),
    }
}

/// Build a Modbus TCP data source config.
pub fn modbus_tcp_source(
    name: &str,
    endpoint: &str,
    unit_id: u8,
    registers: Vec<ModbusRegister>,
    poll_interval_ms: u64,
) -> DataSourceConfig {
    DataSourceConfig {
        id: Uuid::new_v4(),
        name: name.into(),
        source_type: DataSourceType::Modbus {
            variant: ModbusVariant::Tcp,
            unit_id,
            registers,
        },
        enabled: true,
        endpoint: endpoint.into(),
        poll_interval_ms: Some(poll_interval_ms),
        auth: None,
        key_mappings: HashMap::new(),
        custom: HashMap::new(),
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_quality_checks() {
        assert!(DataQuality::Good.is_good());
        assert!(DataQuality::Good.is_usable());
        assert!(!DataQuality::Uncertain.is_good());
        assert!(DataQuality::Uncertain.is_usable());
        assert!(!DataQuality::Bad.is_usable());
    }

    #[test]
    fn data_source_manager_register() {
        let mut mgr = DataSourceManager::default();
        let cfg = opc_ua_source("PLC-1", "opc.tcp://192.168.1.10:4840", vec!["ns=2;s=Temperature"], 1000);
        let id = cfg.id;
        mgr.register(cfg);
        assert_eq!(mgr.sources.len(), 1);
        assert!(mgr.get_source(id).is_some());
        assert!(mgr.get_status(id).is_some());
    }

    #[test]
    fn push_and_drain_data() {
        let mut mgr = DataSourceManager::default();
        let cfg = opc_ua_source("PLC-1", "opc.tcp://localhost:4840", vec![], 1000);
        let id = cfg.id;
        mgr.register(cfg);

        mgr.push_data(id, DataPoint {
            source_id: "ns=2;s=Temperature".into(),
            key: "temperature".into(),
            value: serde_json::json!(25.5),
            timestamp_ms: 1000,
            quality: DataQuality::Good,
            metadata: HashMap::new(),
        });

        let data = mgr.drain_data();
        assert_eq!(data.len(), 1);
        assert!(mgr.drain_data().is_empty()); // drained
    }

    #[test]
    fn key_mapping() {
        let mut mgr = DataSourceManager::default();
        let mut cfg = opc_ua_source("PLC-1", "opc.tcp://localhost:4840", vec![], 1000);
        cfg.key_mappings.insert("ns2_temp".into(), "temperature".into());
        let id = cfg.id;
        mgr.register(cfg);

        mgr.push_data(id, DataPoint {
            source_id: "node-1".into(),
            key: "ns2_temp".into(),
            value: serde_json::json!(25.5),
            timestamp_ms: 1000,
            quality: DataQuality::Good,
            metadata: HashMap::new(),
        });

        let data = mgr.drain_data();
        assert_eq!(data[0].1.key, "temperature"); // mapped
    }

    #[test]
    fn status_tracking() {
        let mut status = DataSourceStatus::new(Uuid::nil());
        status.record_success(1000);
        status.record_success(2000);
        status.record_error("Timeout");
        assert_eq!(status.points_received, 2);
        assert_eq!(status.points_errors, 1);
        assert!((status.availability_pct() - 66.67).abs() < 1.0);
    }

    #[test]
    fn health_percentage() {
        let mut mgr = DataSourceManager::default();
        let cfg1 = opc_ua_source("A", "url1", vec![], 1000);
        let id1 = cfg1.id;
        let cfg2 = opc_ua_source("B", "url2", vec![], 1000);
        mgr.register(cfg1);
        mgr.register(cfg2);

        // Only one connected
        mgr.status.get_mut(&id1).unwrap().connected = true;
        assert!((mgr.health_pct() - 50.0).abs() < 1.0);
    }

    #[test]
    fn modbus_source_config() {
        let cfg = modbus_tcp_source(
            "Motor VFD",
            "192.168.1.20:502",
            1,
            vec![ModbusRegister {
                address: 40001,
                register_type: ModbusRegisterType::HoldingRegister,
                key: "motor_speed".into(),
                data_type: ModbusDataType::Float32,
                scale: 1.0,
                offset: 0.0,
                unit: Some("RPM".into()),
            }],
            500,
        );
        assert!(matches!(cfg.source_type, DataSourceType::Modbus { .. }));
    }

    #[test]
    fn json_roundtrip() {
        let cfg = opc_ua_source("Test", "opc.tcp://localhost", vec!["ns=2;s=Test"], 1000);
        let json = serde_json::to_string(&cfg).expect("serialize");
        let recovered: DataSourceConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(recovered.name, "Test");
    }
}
