use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// SenML record as defined in RFC 8428.
/// Only the fields commonly used by LwM2M devices are included.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SenmlRecord {
    /// Name (relative or absolute resource path, e.g. "/3303/0/5700")
    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    /// Base name — prepended to `n` for all subsequent records until overridden
    #[serde(rename = "bn", skip_serializing_if = "Option::is_none")]
    pub bn: Option<String>,
    /// Numeric value
    #[serde(rename = "v", skip_serializing_if = "Option::is_none")]
    pub v: Option<f64>,
    /// String value
    #[serde(rename = "vs", skip_serializing_if = "Option::is_none")]
    pub vs: Option<String>,
    /// Boolean value
    #[serde(rename = "vb", skip_serializing_if = "Option::is_none")]
    pub vb: Option<bool>,
    /// Data value (base64-encoded binary)
    #[serde(rename = "vd", skip_serializing_if = "Option::is_none")]
    pub vd: Option<String>,
    /// Time (seconds since Unix epoch, may be fractional or negative offset)
    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
    pub t: Option<f64>,
    /// Unit
    #[serde(rename = "u", skip_serializing_if = "Option::is_none")]
    pub u: Option<String>,
}

/// Parse a SenML JSON array from raw bytes.
/// Returns `None` if the bytes are not valid SenML JSON.
pub fn parse_senml(data: &[u8]) -> Option<Vec<SenmlRecord>> {
    serde_json::from_slice(data).ok()
}

/// Convert a slice of SenML records into a flat ThingsBoard-style key-value map.
///
/// - Base name (`bn`) is resolved per RFC 8428 §4.5.1 — it accumulates across records.
/// - The resolved name is used as the JSON key.
/// - Value priority: numeric (`v`) > string (`vs`) > boolean (`vb`) > data (`vd`).
/// - Records with no resolved name are skipped.
pub fn senml_to_kv(records: &[SenmlRecord]) -> Value {
    let mut map = serde_json::Map::new();
    let mut current_base_name = String::new();

    for record in records {
        // Update accumulated base name if present in this record
        if let Some(bn) = &record.bn {
            current_base_name = bn.clone();
        }

        // Resolve full name: base_name + relative name
        let full_name = match &record.n {
            Some(rel) => format!("{}{}", current_base_name, rel),
            None => {
                if current_base_name.is_empty() {
                    continue; // no usable name
                }
                current_base_name.clone()
            }
        };

        if full_name.is_empty() {
            continue;
        }

        // Pick value with priority: v > vs > vb > vd
        let value = if let Some(num) = record.v {
            json!(num)
        } else if let Some(s) = &record.vs {
            json!(s)
        } else if let Some(b) = record.vb {
            json!(b)
        } else if let Some(d) = &record.vd {
            json!(d)
        } else {
            continue; // no value — skip
        };

        map.insert(full_name, value);
    }

    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_array() {
        let result = parse_senml(b"[]");
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn parse_invalid_json_returns_none() {
        assert!(parse_senml(b"not json").is_none());
    }

    #[test]
    fn parse_single_numeric_value() {
        let data = br#"[{"n":"temperature","v":25.5}]"#;
        let records = parse_senml(data).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].n.as_deref(), Some("temperature"));
        assert_eq!(records[0].v, Some(25.5));
    }

    #[test]
    fn parse_multiple_value_types() {
        let data = br#"[
            {"n":"temp","v":22.1},
            {"n":"label","vs":"kitchen"},
            {"n":"active","vb":true},
            {"n":"raw","vd":"AQID"}
        ]"#;
        let records = parse_senml(data).unwrap();
        assert_eq!(records.len(), 4);
        assert_eq!(records[1].vs.as_deref(), Some("kitchen"));
        assert_eq!(records[2].vb, Some(true));
        assert_eq!(records[3].vd.as_deref(), Some("AQID"));
    }

    #[test]
    fn parse_with_timestamp() {
        let data = br#"[{"n":"voltage","v":3.3,"t":1700000000.0}]"#;
        let records = parse_senml(data).unwrap();
        assert_eq!(records[0].t, Some(1700000000.0));
    }

    #[test]
    fn senml_to_kv_empty() {
        let kv = senml_to_kv(&[]);
        assert!(kv.as_object().unwrap().is_empty());
    }

    #[test]
    fn senml_to_kv_single_numeric() {
        let records = parse_senml(br#"[{"n":"temp","v":25.5}]"#).unwrap();
        let kv = senml_to_kv(&records);
        assert_eq!(kv["temp"], json!(25.5));
    }

    #[test]
    fn senml_to_kv_multiple_types() {
        let data = br#"[
            {"n":"temperature","v":22.1},
            {"n":"label","vs":"kitchen"},
            {"n":"active","vb":true}
        ]"#;
        let records = parse_senml(data).unwrap();
        let kv = senml_to_kv(&records);
        assert_eq!(kv["temperature"], json!(22.1));
        assert_eq!(kv["label"], json!("kitchen"));
        assert_eq!(kv["active"], json!(true));
    }

    #[test]
    fn senml_to_kv_with_base_name_prefix() {
        let data = br#"[
            {"bn":"/3303/0/","n":"5700","v":25.5},
            {"n":"5701","vs":"Cel"}
        ]"#;
        let records = parse_senml(data).unwrap();
        let kv = senml_to_kv(&records);
        assert!(kv.get("/3303/0/5700").is_some());
        assert_eq!(kv["/3303/0/5700"], json!(25.5));
        assert_eq!(kv["/3303/0/5701"], json!("Cel"));
    }

    #[test]
    fn senml_to_kv_value_priority_numeric_over_string() {
        // When both v and vs are present, v takes priority
        let data = br#"[{"n":"x","v":1.0,"vs":"ignored"}]"#;
        let records = parse_senml(data).unwrap();
        let kv = senml_to_kv(&records);
        assert_eq!(kv["x"], json!(1.0));
    }
}
