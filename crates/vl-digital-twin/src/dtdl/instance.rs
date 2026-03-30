//! DTDL twin instances — live state of a twin conforming to an interface.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::model::DtdlInterface;

/// A live digital twin instance — the runtime state of a twin that conforms
/// to a DTDL interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinInstance {
    /// Unique identifier for this twin instance.
    pub twin_id: Uuid,
    /// DTMI of the interface this twin conforms to.
    pub model_id: String,
    /// Human-readable name.
    pub display_name: String,
    /// Current property values (name → JSON value).
    pub properties: HashMap<String, serde_json::Value>,
    /// Latest telemetry values (name → (timestamp_ms, JSON value)).
    pub telemetry: HashMap<String, (i64, serde_json::Value)>,
    /// Component instances (component_name → nested TwinInstance).
    pub components: HashMap<String, Box<TwinInstance>>,
    /// Custom metadata / tags.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Tenant isolation.
    pub tenant_id: Uuid,
    /// Creation timestamp (ms).
    pub created_at: i64,
    /// Last modification timestamp (ms).
    pub updated_at: i64,
    /// ETag for optimistic concurrency.
    pub etag: String,
}

impl TwinInstance {
    pub fn new(twin_id: Uuid, model_id: &str, display_name: &str, tenant_id: Uuid) -> Self {
        let now = crate::components::device::current_time_ms();
        Self {
            twin_id,
            model_id: model_id.into(),
            display_name: display_name.into(),
            properties: HashMap::new(),
            telemetry: HashMap::new(),
            components: HashMap::new(),
            metadata: HashMap::new(),
            tenant_id,
            created_at: now,
            updated_at: now,
            etag: Uuid::new_v4().to_string(),
        }
    }

    /// Set a property value, updating the modification timestamp and etag.
    pub fn set_property(&mut self, name: &str, value: serde_json::Value) {
        self.properties.insert(name.into(), value);
        self.updated_at = crate::components::device::current_time_ms();
        self.etag = Uuid::new_v4().to_string();
    }

    /// Update a telemetry value with timestamp.
    pub fn update_telemetry(&mut self, name: &str, value: serde_json::Value, timestamp_ms: i64) {
        self.telemetry.insert(name.into(), (timestamp_ms, value));
        self.updated_at = crate::components::device::current_time_ms();
    }

    /// Get property value by name.
    pub fn get_property(&self, name: &str) -> Option<&serde_json::Value> {
        self.properties.get(name)
    }

    /// Get latest telemetry value by name.
    pub fn get_telemetry(&self, name: &str) -> Option<(&serde_json::Value, i64)> {
        self.telemetry.get(name).map(|(ts, v)| (v, *ts))
    }

    /// Validate all current property values against the interface schema.
    pub fn validate_properties(&self, interface: &DtdlInterface) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for (name, value) in &self.properties {
            if let Some(prop_def) = interface.property(name) {
                if !prop_def.schema.validate(value) {
                    errors.push(ValidationError {
                        field: name.clone(),
                        kind: ValidationErrorKind::SchemaViolation,
                        message: format!(
                            "Property '{}' value does not match schema",
                            name
                        ),
                    });
                }
                if let Some(min) = prop_def.min_value {
                    if let Some(v) = value.as_f64() {
                        if v < min {
                            errors.push(ValidationError {
                                field: name.clone(),
                                kind: ValidationErrorKind::RangeViolation,
                                message: format!(
                                    "Property '{}' = {} is below minimum {}",
                                    name, v, min
                                ),
                            });
                        }
                    }
                }
                if let Some(max) = prop_def.max_value {
                    if let Some(v) = value.as_f64() {
                        if v > max {
                            errors.push(ValidationError {
                                field: name.clone(),
                                kind: ValidationErrorKind::RangeViolation,
                                message: format!(
                                    "Property '{}' = {} exceeds maximum {}",
                                    name, v, max
                                ),
                            });
                        }
                    }
                }
            } else {
                errors.push(ValidationError {
                    field: name.clone(),
                    kind: ValidationErrorKind::UndefinedField,
                    message: format!("Property '{}' is not defined in interface", name),
                });
            }
        }
        errors
    }

    /// Apply default values from the interface for any unset properties.
    pub fn apply_defaults(&mut self, interface: &DtdlInterface) {
        for prop in &interface.properties {
            if !self.properties.contains_key(&prop.name) {
                if let Some(default) = &prop.default_value {
                    self.properties.insert(prop.name.clone(), default.clone());
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub kind: ValidationErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorKind {
    SchemaViolation,
    RangeViolation,
    UndefinedField,
    MissingRequired,
}

// ── Relationship instance ────────────────────────────────────────────────────

/// A live relationship between two twin instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipInstance {
    /// Unique ID for this relationship instance.
    pub relationship_id: Uuid,
    /// Source twin ID.
    pub source_id: Uuid,
    /// Target twin ID.
    pub target_id: Uuid,
    /// Relationship name (matches DtdlRelationship.name).
    pub name: String,
    /// Properties on the relationship edge.
    pub properties: HashMap<String, serde_json::Value>,
    /// ETag for optimistic concurrency.
    pub etag: String,
}

impl RelationshipInstance {
    pub fn new(source_id: Uuid, target_id: Uuid, name: &str) -> Self {
        Self {
            relationship_id: Uuid::new_v4(),
            source_id,
            target_id,
            name: name.into(),
            properties: HashMap::new(),
            etag: Uuid::new_v4().to_string(),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_instance() -> TwinInstance {
        TwinInstance::new(
            Uuid::new_v4(),
            "dtmi:vielang:TemperatureSensor;1",
            "Sensor A",
            Uuid::nil(),
        )
    }

    #[test]
    fn set_and_get_property() {
        let mut inst = make_instance();
        inst.set_property("firmware_version", serde_json::json!("1.2.3"));
        assert_eq!(
            inst.get_property("firmware_version"),
            Some(&serde_json::json!("1.2.3"))
        );
    }

    #[test]
    fn update_and_get_telemetry() {
        let mut inst = make_instance();
        inst.update_telemetry("temperature", serde_json::json!(25.3), 1000);
        let (val, ts) = inst.get_telemetry("temperature").expect("should exist");
        assert_eq!(val, &serde_json::json!(25.3));
        assert_eq!(ts, 1000);
    }

    #[test]
    fn validate_properties_schema_violation() {
        let iface = DtdlInterface::temperature_sensor();
        let mut inst = make_instance();
        inst.set_property("firmware_version", serde_json::json!(123)); // should be string
        let errors = inst.validate_properties(&iface);
        assert!(errors.iter().any(|e| e.kind == ValidationErrorKind::SchemaViolation));
    }

    #[test]
    fn validate_properties_range_violation() {
        let iface = DtdlInterface::temperature_sensor();
        let mut inst = make_instance();
        inst.set_property("sampling_rate", serde_json::json!(99999)); // above max 3600
        let errors = inst.validate_properties(&iface);
        assert!(errors.iter().any(|e| e.kind == ValidationErrorKind::RangeViolation));
    }

    #[test]
    fn validate_undefined_field() {
        let iface = DtdlInterface::temperature_sensor();
        let mut inst = make_instance();
        inst.set_property("ghost_field", serde_json::json!("boo"));
        let errors = inst.validate_properties(&iface);
        assert!(errors.iter().any(|e| e.kind == ValidationErrorKind::UndefinedField));
    }

    #[test]
    fn apply_defaults() {
        let iface = DtdlInterface::temperature_sensor();
        let mut inst = make_instance();
        assert!(inst.get_property("sampling_rate").is_none());
        inst.apply_defaults(&iface);
        assert_eq!(inst.get_property("sampling_rate"), Some(&serde_json::json!(10)));
    }

    #[test]
    fn relationship_instance_creation() {
        let src = Uuid::new_v4();
        let tgt = Uuid::new_v4();
        let rel = RelationshipInstance::new(src, tgt, "locatedIn");
        assert_eq!(rel.source_id, src);
        assert_eq!(rel.target_id, tgt);
        assert_eq!(rel.name, "locatedIn");
    }

    #[test]
    fn etag_changes_on_property_update() {
        let mut inst = make_instance();
        let etag1 = inst.etag.clone();
        inst.set_property("firmware_version", serde_json::json!("2.0"));
        assert_ne!(inst.etag, etag1);
    }
}
