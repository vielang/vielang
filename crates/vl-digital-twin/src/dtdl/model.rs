//! DTDL v3 model definitions — Interface, Property, Telemetry, Command, Relationship, Component.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Schema types ─────────────────────────────────────────────────────────────

/// Primitive and complex schema types supported by DTDL v3.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "@type", rename_all = "camelCase")]
pub enum DtdlSchema {
    Boolean,
    Date,
    DateTime,
    Double,
    Duration,
    Float,
    Integer,
    Long,
    String,
    /// Geospatial types
    Point,
    MultiPoint,
    LineString,
    Polygon,
    /// Complex: enum with named values
    Enum {
        #[serde(rename = "enumValues")]
        enum_values: Vec<EnumValue>,
    },
    /// Complex: map with key/value schemas
    Map {
        #[serde(rename = "mapKey")]
        map_key: Box<MapKey>,
        #[serde(rename = "mapValue")]
        map_value: Box<MapValue>,
    },
    /// Complex: object with named fields
    Object {
        fields: Vec<ObjectField>,
    },
    /// Array of a sub-schema
    Array {
        #[serde(rename = "elementSchema")]
        element_schema: Box<DtdlSchema>,
    },
}

impl Default for DtdlSchema {
    fn default() -> Self {
        Self::Double
    }
}

impl DtdlSchema {
    /// Validate a JSON value against this schema.
    pub fn validate(&self, value: &serde_json::Value) -> bool {
        match self {
            Self::Boolean => value.is_boolean(),
            Self::Double | Self::Float => value.is_f64(),
            Self::Integer => value.is_i64(),
            Self::Long => value.is_i64() || value.is_u64(),
            Self::String | Self::Date | Self::DateTime | Self::Duration => value.is_string(),
            Self::Point | Self::MultiPoint | Self::LineString | Self::Polygon => value.is_object(),
            Self::Enum { enum_values } => {
                if let Some(s) = value.as_str() {
                    enum_values.iter().any(|ev| ev.name == s)
                } else if let Some(n) = value.as_i64() {
                    enum_values.iter().any(|ev| ev.enum_value == Some(n))
                } else {
                    false
                }
            }
            Self::Object { fields } => {
                if let Some(obj) = value.as_object() {
                    fields.iter().all(|f| {
                        match obj.get(&f.name) {
                            Some(v) => f.schema.validate(v),
                            None => !f.required,
                        }
                    })
                } else {
                    false
                }
            }
            Self::Array { element_schema } => {
                if let Some(arr) = value.as_array() {
                    arr.iter().all(|v| element_schema.validate(v))
                } else {
                    false
                }
            }
            Self::Map { .. } => value.is_object(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnumValue {
    pub name: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "enumValue", default)]
    pub enum_value: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapKey {
    pub name: String,
    pub schema: DtdlSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MapValue {
    pub name: String,
    pub schema: DtdlSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectField {
    pub name: String,
    pub schema: DtdlSchema,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

// ── Semantic types (DTDL v3 quantitative units) ──────────────────────────────

/// Semantic type annotation for DTDL properties/telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum SemanticType {
    Temperature,
    Pressure,
    Humidity,
    Velocity,
    Acceleration,
    AngularVelocity,
    Current,
    Voltage,
    Power,
    Energy,
    Force,
    Frequency,
    Illuminance,
    Luminance,
    Mass,
    MassFlowRate,
    Volume,
    VolumeFlowRate,
    Length,
    Area,
    TimeSpan,
    Latitude,
    Longitude,
    Altitude,
}

/// Unit for semantic types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Unit {
    pub symbol: String,
    pub name: String,
    #[serde(default)]
    pub conversion_factor: Option<f64>,
    #[serde(default)]
    pub conversion_offset: Option<f64>,
}

// ── Content types ────────────────────────────────────────────────────────────

/// A DTDL Property — writable or read-only state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtdlProperty {
    pub name: String,
    pub schema: DtdlSchema,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub writable: bool,
    #[serde(rename = "@type", default)]
    pub semantic_type: Option<SemanticType>,
    #[serde(default)]
    pub unit: Option<String>,
    /// Default value (JSON).
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
    /// Valid range for numeric types.
    #[serde(default)]
    pub min_value: Option<f64>,
    #[serde(default)]
    pub max_value: Option<f64>,
}

/// A DTDL Telemetry — time-series data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtdlTelemetry {
    pub name: String,
    pub schema: DtdlSchema,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "@type", default)]
    pub semantic_type: Option<SemanticType>,
    #[serde(default)]
    pub unit: Option<String>,
    /// Sampling interval hint (milliseconds).
    #[serde(default)]
    pub sample_interval_ms: Option<u64>,
}

/// A DTDL Command — device-invocable operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtdlCommand {
    pub name: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub request: Option<CommandPayload>,
    #[serde(default)]
    pub response: Option<CommandPayload>,
    /// If true, command requires device acknowledgement.
    #[serde(default)]
    pub is_twoway: bool,
    /// If true, shows confirmation dialog in UI.
    #[serde(default)]
    pub dangerous: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub name: String,
    pub schema: DtdlSchema,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
}

/// A DTDL Relationship — typed edge to another twin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtdlRelationship {
    pub name: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Target interface ID (DTMI). If None, any twin can be the target.
    #[serde(default)]
    pub target: Option<String>,
    /// Max number of targets allowed (None = unlimited).
    #[serde(default)]
    pub max_multiplicity: Option<u32>,
    /// Min number of targets required (0 = optional).
    #[serde(default)]
    pub min_multiplicity: Option<u32>,
    /// Properties on the relationship edge itself.
    #[serde(default)]
    pub properties: Vec<DtdlProperty>,
    /// Whether this is a containment relationship (spatial hierarchy).
    #[serde(default)]
    pub is_containment: bool,
}

/// A DTDL Component — composition of another interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtdlComponent {
    pub name: String,
    /// Referenced interface DTMI.
    pub schema: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

// ── Interface ────────────────────────────────────────────────────────────────

/// DTDL Interface — the core model definition for a digital twin type.
///
/// Identified by a DTMI (Digital Twin Model Identifier), e.g.:
/// `dtmi:vielang:TemperatureSensor;1`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtdlInterface {
    /// Digital Twin Model Identifier, e.g. `dtmi:vielang:TemperatureSensor;1`
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type", default = "default_interface_type")]
    pub r#type: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "@context", default = "default_context")]
    pub context: String,
    /// Parent interface IDs (DTMI) this extends.
    #[serde(default)]
    pub extends: Vec<String>,
    /// Properties — writable/read-only state values.
    #[serde(default)]
    pub properties: Vec<DtdlProperty>,
    /// Telemetry — time-series data points.
    #[serde(default)]
    pub telemetry: Vec<DtdlTelemetry>,
    /// Commands — device-invocable operations.
    #[serde(default)]
    pub commands: Vec<DtdlCommand>,
    /// Relationships — typed edges to other twins.
    #[serde(default)]
    pub relationships: Vec<DtdlRelationship>,
    /// Components — embedded sub-interfaces.
    #[serde(default)]
    pub components: Vec<DtdlComponent>,
    /// Custom metadata / annotations.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

fn default_interface_type() -> String {
    "Interface".into()
}

fn default_context() -> String {
    "dtmi:dtdl:context;3".into()
}

impl DtdlInterface {
    /// Parse the version number from the DTMI (e.g. `dtmi:foo:bar;2` → `2`).
    pub fn version(&self) -> u32 {
        self.id
            .rsplit_once(';')
            .and_then(|(_, v)| v.parse().ok())
            .unwrap_or(1)
    }

    /// Extract the model path from the DTMI (e.g. `dtmi:vielang:Sensor;1` → `vielang:Sensor`).
    pub fn model_path(&self) -> &str {
        let without_prefix = self.id.strip_prefix("dtmi:").unwrap_or(&self.id);
        without_prefix
            .rsplit_once(';')
            .map(|(path, _)| path)
            .unwrap_or(without_prefix)
    }

    /// Find a property definition by name.
    pub fn property(&self, name: &str) -> Option<&DtdlProperty> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// Find a telemetry definition by name.
    pub fn telemetry_def(&self, name: &str) -> Option<&DtdlTelemetry> {
        self.telemetry.iter().find(|t| t.name == name)
    }

    /// Find a command definition by name.
    pub fn command(&self, name: &str) -> Option<&DtdlCommand> {
        self.commands.iter().find(|c| c.name == name)
    }

    /// Find a relationship definition by name.
    pub fn relationship(&self, name: &str) -> Option<&DtdlRelationship> {
        self.relationships.iter().find(|r| r.name == name)
    }

    /// All writable properties.
    pub fn writable_properties(&self) -> Vec<&DtdlProperty> {
        self.properties.iter().filter(|p| p.writable).collect()
    }

    /// Validate that the DTMI format is correct.
    pub fn validate_dtmi(dtmi: &str) -> bool {
        dtmi.starts_with("dtmi:")
            && dtmi.contains(';')
            && dtmi
                .rsplit_once(';')
                .map(|(_, v)| v.parse::<u32>().is_ok())
                .unwrap_or(false)
    }
}

// ── Built-in industrial interfaces ───────────────────────────────────────────

/// Factory methods for common industrial twin interfaces.
impl DtdlInterface {
    pub fn temperature_sensor() -> Self {
        Self {
            id: "dtmi:vielang:TemperatureSensor;1".into(),
            r#type: "Interface".into(),
            display_name: Some("Temperature Sensor".into()),
            description: Some("Standard industrial temperature sensor with humidity".into()),
            context: default_context(),
            extends: vec![],
            properties: vec![
                DtdlProperty {
                    name: "firmware_version".into(),
                    schema: DtdlSchema::String,
                    display_name: Some("Firmware Version".into()),
                    description: None,
                    writable: false,
                    semantic_type: None,
                    unit: None,
                    default_value: None,
                    min_value: None,
                    max_value: None,
                },
                DtdlProperty {
                    name: "sampling_rate".into(),
                    schema: DtdlSchema::Integer,
                    display_name: Some("Sampling Rate".into()),
                    description: Some("Telemetry sampling interval in seconds".into()),
                    writable: true,
                    semantic_type: None,
                    unit: Some("s".into()),
                    default_value: Some(serde_json::json!(10)),
                    min_value: Some(1.0),
                    max_value: Some(3600.0),
                },
            ],
            telemetry: vec![
                DtdlTelemetry {
                    name: "temperature".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Temperature".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Temperature),
                    unit: Some("°C".into()),
                    sample_interval_ms: Some(10_000),
                },
                DtdlTelemetry {
                    name: "humidity".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Humidity".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Humidity),
                    unit: Some("%RH".into()),
                    sample_interval_ms: Some(10_000),
                },
                DtdlTelemetry {
                    name: "battery".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Battery Level".into()),
                    description: None,
                    semantic_type: None,
                    unit: Some("%".into()),
                    sample_interval_ms: Some(60_000),
                },
            ],
            commands: vec![
                DtdlCommand {
                    name: "calibrate".into(),
                    display_name: Some("Calibrate Sensor".into()),
                    description: Some("Run sensor calibration routine".into()),
                    request: None,
                    response: Some(CommandPayload {
                        name: "result".into(),
                        schema: DtdlSchema::String,
                        display_name: Some("Calibration Result".into()),
                    }),
                    is_twoway: true,
                    dangerous: true,
                },
                DtdlCommand {
                    name: "setThreshold".into(),
                    display_name: Some("Set Alert Threshold".into()),
                    description: None,
                    request: Some(CommandPayload {
                        name: "threshold".into(),
                        schema: DtdlSchema::Object {
                            fields: vec![
                                ObjectField {
                                    name: "min".into(),
                                    schema: DtdlSchema::Double,
                                    display_name: None,
                                    description: None,
                                    required: false,
                                },
                                ObjectField {
                                    name: "max".into(),
                                    schema: DtdlSchema::Double,
                                    display_name: None,
                                    description: None,
                                    required: false,
                                },
                            ],
                        },
                        display_name: None,
                    }),
                    response: None,
                    is_twoway: false,
                    dangerous: false,
                },
            ],
            relationships: vec![
                DtdlRelationship {
                    name: "locatedIn".into(),
                    display_name: Some("Located In".into()),
                    description: Some("Spatial containment relationship".into()),
                    target: Some("dtmi:vielang:Space;1".into()),
                    max_multiplicity: Some(1),
                    min_multiplicity: None,
                    properties: vec![],
                    is_containment: false,
                },
            ],
            components: vec![],
            metadata: HashMap::new(),
        }
    }

    pub fn wind_turbine() -> Self {
        Self {
            id: "dtmi:vielang:WindTurbine;1".into(),
            r#type: "Interface".into(),
            display_name: Some("Wind Turbine".into()),
            description: Some("Industrial wind turbine with power generation monitoring".into()),
            context: default_context(),
            extends: vec![],
            properties: vec![
                DtdlProperty {
                    name: "rated_power_kw".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Rated Power".into()),
                    description: Some("Nameplate rated power output".into()),
                    writable: false,
                    semantic_type: Some(SemanticType::Power),
                    unit: Some("kW".into()),
                    default_value: None,
                    min_value: None,
                    max_value: None,
                },
                DtdlProperty {
                    name: "operating_mode".into(),
                    schema: DtdlSchema::Enum {
                        enum_values: vec![
                            EnumValue { name: "stopped".into(), display_name: Some("Stopped".into()), enum_value: Some(0), description: None },
                            EnumValue { name: "starting".into(), display_name: Some("Starting".into()), enum_value: Some(1), description: None },
                            EnumValue { name: "running".into(), display_name: Some("Running".into()), enum_value: Some(2), description: None },
                            EnumValue { name: "maintenance".into(), display_name: Some("Maintenance".into()), enum_value: Some(3), description: None },
                            EnumValue { name: "fault".into(), display_name: Some("Fault".into()), enum_value: Some(4), description: None },
                        ],
                    },
                    display_name: Some("Operating Mode".into()),
                    description: None,
                    writable: true,
                    semantic_type: None,
                    unit: None,
                    default_value: Some(serde_json::json!("stopped")),
                    min_value: None,
                    max_value: None,
                },
            ],
            telemetry: vec![
                DtdlTelemetry {
                    name: "rpm".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Rotor Speed".into()),
                    description: None,
                    semantic_type: Some(SemanticType::AngularVelocity),
                    unit: Some("RPM".into()),
                    sample_interval_ms: Some(1_000),
                },
                DtdlTelemetry {
                    name: "power_kw".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Power Output".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Power),
                    unit: Some("kW".into()),
                    sample_interval_ms: Some(1_000),
                },
                DtdlTelemetry {
                    name: "wind_speed".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Wind Speed".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Velocity),
                    unit: Some("m/s".into()),
                    sample_interval_ms: Some(5_000),
                },
                DtdlTelemetry {
                    name: "vibration".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Vibration Level".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Velocity),
                    unit: Some("mm/s".into()),
                    sample_interval_ms: Some(1_000),
                },
                DtdlTelemetry {
                    name: "bearing_temp".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Bearing Temperature".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Temperature),
                    unit: Some("°C".into()),
                    sample_interval_ms: Some(5_000),
                },
            ],
            commands: vec![
                DtdlCommand {
                    name: "setSpeed".into(),
                    display_name: Some("Set Rotor Speed".into()),
                    description: None,
                    request: Some(CommandPayload {
                        name: "rpm".into(),
                        schema: DtdlSchema::Double,
                        display_name: Some("Target RPM".into()),
                    }),
                    response: None,
                    is_twoway: false,
                    dangerous: false,
                },
                DtdlCommand {
                    name: "emergencyStop".into(),
                    display_name: Some("Emergency Stop".into()),
                    description: Some("Immediately halt all rotation".into()),
                    request: None,
                    response: None,
                    is_twoway: false,
                    dangerous: true,
                },
                DtdlCommand {
                    name: "getStatus".into(),
                    display_name: Some("Get Status".into()),
                    description: None,
                    request: None,
                    response: Some(CommandPayload {
                        name: "status".into(),
                        schema: DtdlSchema::Object {
                            fields: vec![
                                ObjectField { name: "mode".into(), schema: DtdlSchema::String, display_name: None, description: None, required: true },
                                ObjectField { name: "uptime_hours".into(), schema: DtdlSchema::Double, display_name: None, description: None, required: true },
                                ObjectField { name: "error_code".into(), schema: DtdlSchema::Integer, display_name: None, description: None, required: false },
                            ],
                        },
                        display_name: None,
                    }),
                    is_twoway: true,
                    dangerous: false,
                },
            ],
            relationships: vec![
                DtdlRelationship {
                    name: "locatedIn".into(),
                    display_name: Some("Located In".into()),
                    description: None,
                    target: Some("dtmi:vielang:Space;1".into()),
                    max_multiplicity: Some(1),
                    min_multiplicity: None,
                    properties: vec![],
                    is_containment: false,
                },
                DtdlRelationship {
                    name: "monitoredBy".into(),
                    display_name: Some("Monitored By".into()),
                    description: Some("SCADA/monitoring system watching this turbine".into()),
                    target: None,
                    max_multiplicity: None,
                    min_multiplicity: None,
                    properties: vec![],
                    is_containment: false,
                },
            ],
            components: vec![],
            metadata: HashMap::new(),
        }
    }

    /// ISA-95 Space model — used for spatial hierarchy (Site/Area/Line/Cell).
    pub fn space() -> Self {
        Self {
            id: "dtmi:vielang:Space;1".into(),
            r#type: "Interface".into(),
            display_name: Some("Space".into()),
            description: Some("ISA-95 spatial entity: Site, Area, Line, or Cell".into()),
            context: default_context(),
            extends: vec![],
            properties: vec![
                DtdlProperty {
                    name: "space_type".into(),
                    schema: DtdlSchema::Enum {
                        enum_values: vec![
                            EnumValue { name: "Enterprise".into(), display_name: None, enum_value: Some(0), description: None },
                            EnumValue { name: "Site".into(), display_name: None, enum_value: Some(1), description: None },
                            EnumValue { name: "Area".into(), display_name: None, enum_value: Some(2), description: None },
                            EnumValue { name: "Line".into(), display_name: None, enum_value: Some(3), description: None },
                            EnumValue { name: "Cell".into(), display_name: None, enum_value: Some(4), description: None },
                            EnumValue { name: "Building".into(), display_name: None, enum_value: Some(5), description: None },
                            EnumValue { name: "Floor".into(), display_name: None, enum_value: Some(6), description: None },
                            EnumValue { name: "Zone".into(), display_name: None, enum_value: Some(7), description: None },
                        ],
                    },
                    display_name: Some("Space Type".into()),
                    description: None,
                    writable: false,
                    semantic_type: None,
                    unit: None,
                    default_value: None,
                    min_value: None,
                    max_value: None,
                },
                DtdlProperty {
                    name: "latitude".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Latitude".into()),
                    description: None,
                    writable: true,
                    semantic_type: Some(SemanticType::Latitude),
                    unit: Some("°".into()),
                    default_value: None,
                    min_value: Some(-90.0),
                    max_value: Some(90.0),
                },
                DtdlProperty {
                    name: "longitude".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Longitude".into()),
                    description: None,
                    writable: true,
                    semantic_type: Some(SemanticType::Longitude),
                    unit: Some("°".into()),
                    default_value: None,
                    min_value: Some(-180.0),
                    max_value: Some(180.0),
                },
            ],
            telemetry: vec![],
            commands: vec![],
            relationships: vec![
                DtdlRelationship {
                    name: "contains".into(),
                    display_name: Some("Contains".into()),
                    description: Some("Spatial containment (parent → child)".into()),
                    target: None,
                    max_multiplicity: None,
                    min_multiplicity: None,
                    properties: vec![],
                    is_containment: true,
                },
            ],
            components: vec![],
            metadata: HashMap::new(),
        }
    }

    /// Generic industrial pump.
    pub fn pump() -> Self {
        Self {
            id: "dtmi:vielang:Pump;1".into(),
            r#type: "Interface".into(),
            display_name: Some("Industrial Pump".into()),
            description: Some("Variable frequency drive pump with flow and pressure monitoring".into()),
            context: default_context(),
            extends: vec![],
            properties: vec![
                DtdlProperty {
                    name: "rated_flow".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Rated Flow".into()),
                    description: None,
                    writable: false,
                    semantic_type: Some(SemanticType::VolumeFlowRate),
                    unit: Some("m³/h".into()),
                    default_value: None,
                    min_value: None,
                    max_value: None,
                },
                DtdlProperty {
                    name: "rated_head".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Rated Head".into()),
                    description: None,
                    writable: false,
                    semantic_type: Some(SemanticType::Pressure),
                    unit: Some("m".into()),
                    default_value: None,
                    min_value: None,
                    max_value: None,
                },
            ],
            telemetry: vec![
                DtdlTelemetry {
                    name: "flow_rate".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Flow Rate".into()),
                    description: None,
                    semantic_type: Some(SemanticType::VolumeFlowRate),
                    unit: Some("m³/h".into()),
                    sample_interval_ms: Some(1_000),
                },
                DtdlTelemetry {
                    name: "discharge_pressure".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Discharge Pressure".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Pressure),
                    unit: Some("bar".into()),
                    sample_interval_ms: Some(1_000),
                },
                DtdlTelemetry {
                    name: "motor_current".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Motor Current".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Current),
                    unit: Some("A".into()),
                    sample_interval_ms: Some(1_000),
                },
                DtdlTelemetry {
                    name: "vibration".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Vibration".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Velocity),
                    unit: Some("mm/s".into()),
                    sample_interval_ms: Some(1_000),
                },
                DtdlTelemetry {
                    name: "bearing_temp".into(),
                    schema: DtdlSchema::Double,
                    display_name: Some("Bearing Temperature".into()),
                    description: None,
                    semantic_type: Some(SemanticType::Temperature),
                    unit: Some("°C".into()),
                    sample_interval_ms: Some(5_000),
                },
            ],
            commands: vec![
                DtdlCommand {
                    name: "setSpeed".into(),
                    display_name: Some("Set VFD Speed".into()),
                    description: Some("Set variable frequency drive speed percentage".into()),
                    request: Some(CommandPayload {
                        name: "speed_pct".into(),
                        schema: DtdlSchema::Double,
                        display_name: Some("Speed %".into()),
                    }),
                    response: None,
                    is_twoway: false,
                    dangerous: false,
                },
                DtdlCommand {
                    name: "emergencyStop".into(),
                    display_name: Some("Emergency Stop".into()),
                    description: None,
                    request: None,
                    response: None,
                    is_twoway: false,
                    dangerous: true,
                },
            ],
            relationships: vec![
                DtdlRelationship {
                    name: "locatedIn".into(),
                    display_name: Some("Located In".into()),
                    description: None,
                    target: Some("dtmi:vielang:Space;1".into()),
                    max_multiplicity: Some(1),
                    min_multiplicity: None,
                    properties: vec![],
                    is_containment: false,
                },
            ],
            components: vec![],
            metadata: HashMap::new(),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dtmi_validation() {
        assert!(DtdlInterface::validate_dtmi("dtmi:vielang:Sensor;1"));
        assert!(DtdlInterface::validate_dtmi("dtmi:com:example:Thermostat;2"));
        assert!(!DtdlInterface::validate_dtmi("not_a_dtmi"));
        assert!(!DtdlInterface::validate_dtmi("dtmi:no_version"));
        assert!(!DtdlInterface::validate_dtmi("dtmi:bad;notanum"));
    }

    #[test]
    fn version_extraction() {
        let iface = DtdlInterface::temperature_sensor();
        assert_eq!(iface.version(), 1);
    }

    #[test]
    fn model_path_extraction() {
        let iface = DtdlInterface::temperature_sensor();
        assert_eq!(iface.model_path(), "vielang:TemperatureSensor");
    }

    #[test]
    fn property_lookup() {
        let iface = DtdlInterface::temperature_sensor();
        assert!(iface.property("firmware_version").is_some());
        assert!(iface.property("nonexistent").is_none());
    }

    #[test]
    fn telemetry_lookup() {
        let iface = DtdlInterface::temperature_sensor();
        let t = iface.telemetry_def("temperature").expect("should exist");
        assert_eq!(t.unit.as_deref(), Some("°C"));
    }

    #[test]
    fn command_lookup() {
        let iface = DtdlInterface::wind_turbine();
        let cmd = iface.command("emergencyStop").expect("should exist");
        assert!(cmd.dangerous);
    }

    #[test]
    fn writable_properties() {
        let iface = DtdlInterface::temperature_sensor();
        let writable = iface.writable_properties();
        assert_eq!(writable.len(), 1);
        assert_eq!(writable[0].name, "sampling_rate");
    }

    #[test]
    fn schema_validation_double() {
        let schema = DtdlSchema::Double;
        assert!(schema.validate(&serde_json::json!(42.5)));
        assert!(!schema.validate(&serde_json::json!("text")));
    }

    #[test]
    fn schema_validation_enum() {
        let schema = DtdlSchema::Enum {
            enum_values: vec![
                EnumValue { name: "on".into(), display_name: None, enum_value: Some(1), description: None },
                EnumValue { name: "off".into(), display_name: None, enum_value: Some(0), description: None },
            ],
        };
        assert!(schema.validate(&serde_json::json!("on")));
        assert!(schema.validate(&serde_json::json!(1)));
        assert!(!schema.validate(&serde_json::json!("invalid")));
    }

    #[test]
    fn schema_validation_object() {
        let schema = DtdlSchema::Object {
            fields: vec![
                ObjectField { name: "x".into(), schema: DtdlSchema::Double, display_name: None, description: None, required: true },
                ObjectField { name: "y".into(), schema: DtdlSchema::Double, display_name: None, description: None, required: false },
            ],
        };
        assert!(schema.validate(&serde_json::json!({"x": 1.0, "y": 2.0})));
        assert!(schema.validate(&serde_json::json!({"x": 1.0}))); // y is optional
        assert!(!schema.validate(&serde_json::json!({"y": 2.0}))); // x is required
    }

    #[test]
    fn space_interface_has_containment() {
        let space = DtdlInterface::space();
        let rel = space.relationship("contains").expect("should exist");
        assert!(rel.is_containment);
    }

    #[test]
    fn pump_interface_has_expected_telemetry() {
        let pump = DtdlInterface::pump();
        assert_eq!(pump.telemetry.len(), 5);
        assert!(pump.telemetry_def("flow_rate").is_some());
        assert!(pump.telemetry_def("discharge_pressure").is_some());
    }

    #[test]
    fn json_roundtrip() {
        let iface = DtdlInterface::temperature_sensor();
        let json = serde_json::to_string_pretty(&iface).expect("serialize");
        let recovered: DtdlInterface = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(recovered.id, iface.id);
        assert_eq!(recovered.properties.len(), iface.properties.len());
        assert_eq!(recovered.telemetry.len(), iface.telemetry.len());
    }
}
