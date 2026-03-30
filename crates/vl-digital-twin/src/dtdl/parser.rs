//! DTDL JSON parser — loads interface definitions from JSON files.

use super::model::DtdlInterface;

/// Parse a single DTDL interface from a JSON string.
pub fn parse_interface(json: &str) -> Result<DtdlInterface, DtdlParseError> {
    let iface: DtdlInterface =
        serde_json::from_str(json).map_err(|e| DtdlParseError::Json(e.to_string()))?;

    if !DtdlInterface::validate_dtmi(&iface.id) {
        return Err(DtdlParseError::InvalidDtmi(iface.id.clone()));
    }

    Ok(iface)
}

/// Parse multiple DTDL interfaces from a JSON array string.
pub fn parse_interfaces(json: &str) -> Result<Vec<DtdlInterface>, DtdlParseError> {
    // Try as array first, then as single object
    if let Ok(interfaces) = serde_json::from_str::<Vec<DtdlInterface>>(json) {
        for iface in &interfaces {
            if !DtdlInterface::validate_dtmi(&iface.id) {
                return Err(DtdlParseError::InvalidDtmi(iface.id.clone()));
            }
        }
        return Ok(interfaces);
    }

    // Single interface
    parse_interface(json).map(|i| vec![i])
}

#[derive(Debug, Clone)]
pub enum DtdlParseError {
    Json(String),
    InvalidDtmi(String),
    UnsupportedVersion(String),
}

impl std::fmt::Display for DtdlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(e) => write!(f, "JSON parse error: {e}"),
            Self::InvalidDtmi(id) => write!(f, "Invalid DTMI: {id}"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported DTDL version: {v}"),
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_interface() {
        let json = r#"{
            "@id": "dtmi:test:Minimal;1",
            "@type": "Interface",
            "@context": "dtmi:dtdl:context;3"
        }"#;
        let iface = parse_interface(json).expect("should parse");
        assert_eq!(iface.id, "dtmi:test:Minimal;1");
        assert_eq!(iface.version(), 1);
    }

    #[test]
    fn parse_invalid_dtmi() {
        let json = r#"{
            "@id": "not_valid",
            "@type": "Interface"
        }"#;
        assert!(matches!(
            parse_interface(json),
            Err(DtdlParseError::InvalidDtmi(_))
        ));
    }

    #[test]
    fn parse_array_of_interfaces() {
        let json = r#"[
            { "@id": "dtmi:test:A;1", "@type": "Interface" },
            { "@id": "dtmi:test:B;2", "@type": "Interface" }
        ]"#;
        let interfaces = parse_interfaces(json).expect("should parse");
        assert_eq!(interfaces.len(), 2);
    }

    #[test]
    fn parse_single_as_array() {
        let json = r#"{ "@id": "dtmi:test:Single;1", "@type": "Interface" }"#;
        let interfaces = parse_interfaces(json).expect("should parse");
        assert_eq!(interfaces.len(), 1);
    }

    #[test]
    fn roundtrip_builtin_interface() {
        let original = DtdlInterface::temperature_sensor();
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed = parse_interface(&json).expect("parse");
        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.telemetry.len(), original.telemetry.len());
    }
}
