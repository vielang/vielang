use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How the simulator sends telemetry to the platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    /// Bypass transport — write directly to DB (fast, no protocol testing)
    Direct,
    /// Send via HTTP Device API (POST /api/v1/{token}/telemetry)
    Http,
    /// Publish via MQTT broker (topic: v1/devices/me/telemetry)
    Mqtt,
}

impl Default for TransportMode {
    fn default() -> Self { Self::Direct }
}

impl TransportMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "http" => Self::Http,
            "mqtt" => Self::Mqtt,
            _ => Self::Direct,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Http => "http",
            Self::Mqtt => "mqtt",
        }
    }
}

/// A simulator configuration — defines how a virtual device generates telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulatorConfig {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub device_id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub interval_ms: i64,
    pub telemetry_schema: Vec<TelemetryFieldDef>,
    pub script: Option<String>,
    #[serde(default)]
    pub transport_mode: TransportMode,
    pub created_time: i64,
    pub updated_time: i64,
}

/// One telemetry field definition within a simulator config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryFieldDef {
    pub key: String,
    pub data_type: SimDataType,
    pub generator: GeneratorType,
}

/// Data type for a simulated telemetry field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SimDataType {
    Double,
    Long,
    Boolean,
    String,
}

/// Generator pattern for producing values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeneratorType {
    Random { min: f64, max: f64 },
    SineWave {
        amplitude: f64,
        offset: f64,
        #[serde(rename = "periodMs")]
        period_ms: i64,
    },
    Linear {
        start: f64,
        step: f64,
        max: Option<f64>,
    },
    Constant { value: serde_json::Value },
    Script { expression: String },
}

/// Runtime status of a simulation (not persisted).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SimulationStatus {
    Running,
    Stopped,
    Error,
}

/// API response for simulation runtime status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationStatusResponse {
    pub config_id: Uuid,
    pub config_name: String,
    pub device_id: Uuid,
    pub status: SimulationStatus,
    pub tick_count: u64,
    pub last_tick_ts: Option<i64>,
    pub error_message: Option<String>,
}

/// API request to create or update a simulator config.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSimulatorRequest {
    pub device_id: Uuid,
    pub name: String,
    pub interval_ms: i64,
    pub telemetry_schema: Vec<TelemetryFieldDef>,
    pub script: Option<String>,
    pub enabled: bool,
    #[serde(default)]
    pub transport_mode: TransportMode,
}

// ── Device Templates ─────────────────────────────────────────────────────────

/// A predefined device template with telemetry schema and Wokwi diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub telemetry_schema: Vec<TelemetryFieldDef>,
    pub diagram: serde_json::Value,
    pub is_builtin: bool,
    pub tenant_id: Option<Uuid>,
    pub created_time: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDeviceTemplateRequest {
    pub name: String,
    pub description: String,
    pub category: String,
    pub telemetry_schema: Vec<TelemetryFieldDef>,
    pub diagram: serde_json::Value,
}

/// Wokwi component info served from the static registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WokwiComponentInfo {
    pub component_type: String,
    pub category: String,
    pub label: String,
    pub telemetry_keys: Vec<String>,
}

// ── Phase 4: Schematic models ────────────────────────────────────────────────

/// A visual schematic linking simulated devices in a JointJS graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulatorSchematic {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub graph_data: serde_json::Value,
    pub created_time: i64,
    pub updated_time: i64,
}

/// Links a JointJS graph node to a SimulatorConfig.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicNodeConfig {
    pub id: Uuid,
    pub schematic_id: Uuid,
    pub node_id: String,
    pub simulator_config_id: Option<Uuid>,
    pub node_type: String,
    pub properties: serde_json::Value,
}

/// API request to create or update a schematic.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchematicRequest {
    pub name: String,
    pub graph_data: serde_json::Value,
}

/// API request to save a schematic node.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSchematicNodeRequest {
    pub node_id: String,
    pub simulator_config_id: Option<Uuid>,
    pub node_type: String,
    pub properties: serde_json::Value,
}
