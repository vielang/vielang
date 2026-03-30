/// OMA LwM2M Standard Object Registry — Objects 0–8.
///
/// Each object definition maps resource IDs to (name, type) pairs so that
/// raw TLV/JSON payloads can be translated to human-readable ThingsBoard
/// telemetry keys and the correct JSON type can be inferred.
///
/// Reference: OMA TS-DM LwM2M Object Specification (v1.1)
/// Also extends the IPSO registry in `ipso.rs` for smart objects (3300+).

use std::collections::HashMap;

/// Resource value type from the OMA object model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceType {
    String,
    Integer,
    Float,
    Boolean,
    Opaque,
    Time,
    ObjectLink,
    None,
}

/// Definition of a single LwM2M resource within an object.
#[derive(Debug, Clone)]
pub struct ResourceDef {
    pub name:     &'static str,
    pub res_type: ResourceType,
    /// True if the resource is mandatory per OMA spec
    pub mandatory: bool,
}

impl ResourceDef {
    const fn new(name: &'static str, res_type: ResourceType, mandatory: bool) -> Self {
        Self { name, res_type, mandatory }
    }
}

/// Definition of a standard LwM2M object.
pub struct ObjectDef {
    pub id:        u16,
    pub name:      &'static str,
    pub resources: &'static [(u16, ResourceDef)],
}

// ── Object 0 — LwM2M Security ─────────────────────────────────────────────────

static OBJECT_0_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("LwM2M Server URI",                ResourceType::String,  true)),
    (1,  ResourceDef::new("Bootstrap Server",                ResourceType::Boolean, true)),
    (2,  ResourceDef::new("Security Mode",                   ResourceType::Integer, true)),
    (3,  ResourceDef::new("Public Key or Identity",          ResourceType::Opaque,  true)),
    (4,  ResourceDef::new("Server Public Key",               ResourceType::Opaque,  true)),
    (5,  ResourceDef::new("Secret Key",                      ResourceType::Opaque,  true)),
    (6,  ResourceDef::new("SMS Security Mode",               ResourceType::Integer, false)),
    (10, ResourceDef::new("Short Server ID",                 ResourceType::Integer, false)),
    (11, ResourceDef::new("Client Hold Off Time",            ResourceType::Integer, false)),
    (12, ResourceDef::new("Bootstrap Server Account Timeout",ResourceType::Integer, false)),
];

// ── Object 1 — LwM2M Server ───────────────────────────────────────────────────

static OBJECT_1_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("Short Server ID",               ResourceType::Integer, true)),
    (1,  ResourceDef::new("Lifetime",                      ResourceType::Integer, true)),
    (2,  ResourceDef::new("Default Min Period",            ResourceType::Integer, false)),
    (3,  ResourceDef::new("Default Max Period",            ResourceType::Integer, false)),
    (5,  ResourceDef::new("Disable Timeout",               ResourceType::Integer, false)),
    (6,  ResourceDef::new("Notification Storing",          ResourceType::Boolean, true)),
    (7,  ResourceDef::new("Binding",                       ResourceType::String,  true)),
    (8,  ResourceDef::new("Registration Update Trigger",   ResourceType::None,    true)),
];

// ── Object 2 — Access Control ─────────────────────────────────────────────────

static OBJECT_2_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("Object ID",             ResourceType::Integer, true)),
    (1,  ResourceDef::new("Object Instance ID",    ResourceType::Integer, true)),
    (2,  ResourceDef::new("ACL",                   ResourceType::Integer, false)),
    (3,  ResourceDef::new("Access Control Owner",  ResourceType::Integer, true)),
];

// ── Object 3 — Device ─────────────────────────────────────────────────────────

static OBJECT_3_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("manufacturer",          ResourceType::String,  false)),
    (1,  ResourceDef::new("model",                 ResourceType::String,  false)),
    (2,  ResourceDef::new("serial_number",         ResourceType::String,  false)),
    (3,  ResourceDef::new("firmware_version",      ResourceType::String,  false)),
    (4,  ResourceDef::new("reboot",                ResourceType::None,    true)),
    (5,  ResourceDef::new("factory_reset",         ResourceType::None,    false)),
    (6,  ResourceDef::new("available_power_source", ResourceType::Integer, false)),
    (7,  ResourceDef::new("power_source_voltage",  ResourceType::Integer, false)),
    (8,  ResourceDef::new("power_source_current",  ResourceType::Integer, false)),
    (9,  ResourceDef::new("battery_level",         ResourceType::Integer, false)),
    (10, ResourceDef::new("memory_free",           ResourceType::Integer, false)),
    (11, ResourceDef::new("error_code",            ResourceType::Integer, true)),
    (12, ResourceDef::new("reset_error_code",      ResourceType::None,    false)),
    (13, ResourceDef::new("current_time",          ResourceType::Time,    false)),
    (14, ResourceDef::new("utc_offset",            ResourceType::String,  false)),
    (15, ResourceDef::new("timezone",              ResourceType::String,  false)),
    (16, ResourceDef::new("supported_bindings",    ResourceType::String,  true)),
    (17, ResourceDef::new("device_type",           ResourceType::String,  false)),
    (18, ResourceDef::new("hardware_version",      ResourceType::String,  false)),
    (19, ResourceDef::new("software_version",      ResourceType::String,  false)),
    (20, ResourceDef::new("battery_status",        ResourceType::Integer, false)),
    (21, ResourceDef::new("memory_total",          ResourceType::Integer, false)),
];

// ── Object 4 — Connectivity Monitoring ────────────────────────────────────────

static OBJECT_4_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("network_bearer",        ResourceType::Integer, true)),
    (1,  ResourceDef::new("available_network_bearer", ResourceType::Integer, true)),
    (2,  ResourceDef::new("radio_signal_strength", ResourceType::Integer, true)),
    (3,  ResourceDef::new("link_quality",          ResourceType::Integer, false)),
    (4,  ResourceDef::new("ip_addresses",          ResourceType::String,  true)),
    (5,  ResourceDef::new("router_ip_address",     ResourceType::String,  false)),
    (6,  ResourceDef::new("link_utilization",      ResourceType::Integer, false)),
    (7,  ResourceDef::new("apn",                   ResourceType::String,  false)),
    (8,  ResourceDef::new("cell_id",               ResourceType::Integer, false)),
    (9,  ResourceDef::new("smnc",                  ResourceType::Integer, false)),
    (10, ResourceDef::new("smcc",                  ResourceType::Integer, false)),
];

// ── Object 5 — Firmware Update ────────────────────────────────────────────────

static OBJECT_5_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("firmware_package",      ResourceType::Opaque,  false)),
    (1,  ResourceDef::new("firmware_uri",          ResourceType::String,  false)),
    (2,  ResourceDef::new("firmware_update",       ResourceType::None,    true)),
    (3,  ResourceDef::new("firmware_state",        ResourceType::Integer, true)),
    (4,  ResourceDef::new("update_supported_objects", ResourceType::Boolean, false)),
    (5,  ResourceDef::new("update_result",         ResourceType::Integer, true)),
    (6,  ResourceDef::new("pkg_name",              ResourceType::String,  false)),
    (7,  ResourceDef::new("pkg_version",           ResourceType::String,  false)),
    (8,  ResourceDef::new("firmware_update_delivery_method", ResourceType::Integer, true)),
];

// ── Object 6 — Location ───────────────────────────────────────────────────────

static OBJECT_6_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("latitude",   ResourceType::Float,   true)),
    (1,  ResourceDef::new("longitude",  ResourceType::Float,   true)),
    (2,  ResourceDef::new("altitude",   ResourceType::Float,   false)),
    (3,  ResourceDef::new("radius",     ResourceType::Float,   false)),
    (4,  ResourceDef::new("velocity",   ResourceType::Opaque,  false)),
    (5,  ResourceDef::new("timestamp",  ResourceType::Time,    true)),
    (6,  ResourceDef::new("speed",      ResourceType::Float,   false)),
];

// ── Object 7 — Connectivity Statistics ───────────────────────────────────────

static OBJECT_7_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("sms_tx_counter",        ResourceType::Integer, false)),
    (1,  ResourceDef::new("sms_rx_counter",        ResourceType::Integer, false)),
    (2,  ResourceDef::new("tx_data",               ResourceType::Integer, false)),
    (3,  ResourceDef::new("rx_data",               ResourceType::Integer, false)),
    (4,  ResourceDef::new("max_message_size",      ResourceType::Integer, false)),
    (5,  ResourceDef::new("average_message_size",  ResourceType::Integer, false)),
    (6,  ResourceDef::new("start_stats",           ResourceType::None,    true)),
    (7,  ResourceDef::new("stop_stats",            ResourceType::None,    true)),
    (8,  ResourceDef::new("collection_period",     ResourceType::Integer, false)),
];

// ── Object 8 — Lock & Wipe ────────────────────────────────────────────────────

static OBJECT_8_RESOURCES: &[(u16, ResourceDef)] = &[
    (0,  ResourceDef::new("state",             ResourceType::Integer, true)),
    (1,  ResourceDef::new("lock_target",       ResourceType::Opaque,  false)),
    (4,  ResourceDef::new("device_lock_code",  ResourceType::Opaque,  false)),
    (5,  ResourceDef::new("device_lock_action",ResourceType::None,    false)),
];

// ── Registry lookup ───────────────────────────────────────────────────────────

use std::sync::OnceLock;

static REGISTRY: OnceLock<HashMap<u16, &'static [(u16, ResourceDef)]>> = OnceLock::new();

fn registry() -> &'static HashMap<u16, &'static [(u16, ResourceDef)]> {
    REGISTRY.get_or_init(|| {
        let mut m: HashMap<u16, &'static [(u16, ResourceDef)]> = HashMap::new();
        m.insert(0, OBJECT_0_RESOURCES);
        m.insert(1, OBJECT_1_RESOURCES);
        m.insert(2, OBJECT_2_RESOURCES);
        m.insert(3, OBJECT_3_RESOURCES);
        m.insert(4, OBJECT_4_RESOURCES);
        m.insert(5, OBJECT_5_RESOURCES);
        m.insert(6, OBJECT_6_RESOURCES);
        m.insert(7, OBJECT_7_RESOURCES);
        m.insert(8, OBJECT_8_RESOURCES);
        m
    })
}

/// Look up a resource definition by object ID and resource ID.
pub fn lookup_resource(object_id: u16, resource_id: u16) -> Option<&'static ResourceDef> {
    registry()
        .get(&object_id)
        .and_then(|resources| {
            resources.iter().find(|(rid, _)| *rid == resource_id).map(|(_, def)| def)
        })
}

/// Resolve a resource ID to its telemetry key name.
/// Falls back to the raw resource ID string if not found.
pub fn resource_to_key(object_id: u16, resource_id: u16) -> String {
    lookup_resource(object_id, resource_id)
        .map(|def| def.name.to_string())
        .unwrap_or_else(|| resource_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_object_lookup() {
        let def = lookup_resource(3, 9).unwrap();
        assert_eq!(def.name, "battery_level");
        assert_eq!(def.res_type, ResourceType::Integer);
    }

    #[test]
    fn test_location_object_lookup() {
        let def = lookup_resource(6, 0).unwrap();
        assert_eq!(def.name, "latitude");
        assert_eq!(def.res_type, ResourceType::Float);
    }

    #[test]
    fn test_resource_to_key() {
        assert_eq!(resource_to_key(3, 9),  "battery_level");
        assert_eq!(resource_to_key(6, 0),  "latitude");
        assert_eq!(resource_to_key(6, 1),  "longitude");
        assert_eq!(resource_to_key(99, 99), "99"); // unknown → raw ID
    }

    #[test]
    fn test_security_object() {
        let def = lookup_resource(0, 2).unwrap();
        assert_eq!(def.name, "Security Mode");
    }

    #[test]
    fn test_firmware_update_object() {
        let def = lookup_resource(5, 3).unwrap();
        assert_eq!(def.name, "firmware_state");
    }
}
