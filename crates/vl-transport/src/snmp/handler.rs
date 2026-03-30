/// SNMP trap handler — OID → telemetry key mapping and telemetry extraction.
///
/// Converts raw SNMP varbinds (OID → JSON value) into telemetry records
/// suitable for saving to the ThingsBoard timeseries store.
///
/// Mapping priority:
/// 1. OID registry (MIB-II friendly names: sysName, ifInOctets, etc.)
/// 2. Raw OID dotted-decimal (fallback, preserves all data)

use serde_json::{Map, Value};
use tracing::debug;

use super::oid_registry::oid_to_key;

/// Process SNMP varbinds into a telemetry JSON object ready for saving.
///
/// Filters out meta-varbinds (sysUpTime from trap header, snmpTrapOID) when
/// `filter_meta` is true, since those are not device telemetry values.
pub fn varbinds_to_telemetry(
    varbinds:    &[(String, Value)],
    filter_meta: bool,
) -> Map<String, Value> {
    // Meta OIDs that appear in every SNMPv2c trap — not device metrics
    const META_OIDS: &[&str] = &[
        "1.3.6.1.2.1.1.3.0",    // sysUpTime (trap header timestamp)
        "1.3.6.1.6.3.1.1.4.1.0", // snmpTrapOID
    ];

    let mut telemetry = Map::new();
    for (oid, value) in varbinds {
        if filter_meta && META_OIDS.contains(&oid.as_str()) {
            debug!(oid = %oid, "Skipping meta varbind");
            continue;
        }
        let key = oid_to_key(oid);
        telemetry.insert(key, value.clone());
    }
    telemetry
}

/// Extract a single telemetry value by OID or friendly name.
pub fn get_varbind_value<'a>(
    varbinds: &'a [(String, Value)],
    oid_or_name: &str,
) -> Option<&'a Value> {
    for (oid, value) in varbinds {
        if oid == oid_or_name || oid_to_key(oid) == oid_or_name {
            return Some(value);
        }
    }
    None
}

/// Determine SNMP version from a parsed PDU tag byte.
/// TAG_TRAPV1 (0xa4) = v1, TAG_TRAPV2 (0xa7) = v2c.
pub fn pdu_tag_to_version(pdu_tag: u8) -> u8 {
    if pdu_tag == 0xa4 { 1 } else { 2 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_varbinds() -> Vec<(String, Value)> {
        vec![
            ("1.3.6.1.2.1.1.3.0".to_string(),    json!(98765u64)),    // sysUpTime (meta)
            ("1.3.6.1.6.3.1.1.4.1.0".to_string(), json!("1.3.6.1.6.3.1.1.5.4")), // snmpTrapOID (meta)
            ("1.3.6.1.2.1.1.5.0".to_string(),     json!("router-01")), // sysName
            ("1.3.6.1.2.1.2.2.1.10".to_string(),  json!(1234567u64)),  // ifInOctets
            ("1.3.6.1.9.9.9.0".to_string(),        json!(42)),          // unknown OID
        ]
    }

    #[test]
    fn test_filter_meta() {
        let vb = make_varbinds();
        let t = varbinds_to_telemetry(&vb, true);
        assert!(!t.contains_key("sysUpTime"));
        assert!(!t.contains_key("snmpTrapOID"));
        assert_eq!(t.get("sysName").and_then(|v| v.as_str()), Some("router-01"));
        assert_eq!(t.get("ifInOctets").and_then(|v| v.as_u64()), Some(1234567));
        // Unknown OID used as key
        assert!(t.contains_key("1.3.6.1.9.9.9.0"));
    }

    #[test]
    fn test_include_meta() {
        let vb = make_varbinds();
        let t = varbinds_to_telemetry(&vb, false);
        // sysUpTime should now be included under friendly name
        assert!(t.contains_key("sysUpTime") || t.contains_key("1.3.6.1.2.1.1.3.0"));
    }

    #[test]
    fn test_get_varbind_by_oid() {
        let vb = make_varbinds();
        let v = get_varbind_value(&vb, "1.3.6.1.2.1.1.5.0");
        assert_eq!(v.and_then(|v| v.as_str()), Some("router-01"));
    }

    #[test]
    fn test_get_varbind_by_name() {
        let vb = make_varbinds();
        let v = get_varbind_value(&vb, "sysName");
        assert_eq!(v.and_then(|v| v.as_str()), Some("router-01"));
    }

    #[test]
    fn test_pdu_tag_to_version() {
        assert_eq!(pdu_tag_to_version(0xa4), 1);
        assert_eq!(pdu_tag_to_version(0xa7), 2);
    }
}
