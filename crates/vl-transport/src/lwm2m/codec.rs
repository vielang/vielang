/// LwM2M TLV (Type-Length-Value) codec — OMA TS-DM spec section 6.4.3.
///
/// TLV is the binary wire format used by most LwM2M devices (Leshan, Wakaama, etc.)
/// when the server does not negotiate a content format. It is more compact than JSON
/// and supports nested structures (Object Instance → Resource Instance hierarchy).
///
/// Also provides JSON and CBOR dispatch utilities for content-format routing.

use serde_json::{Map, Value};

// ── TLV record types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TlvType {
    ObjectInstance,
    ResourceInstance,
    MultipleResource,
    ResourceValue,
}

/// A single parsed TLV record.
#[derive(Debug, Clone)]
pub struct TlvRecord {
    pub tlv_type: TlvType,
    /// Identifier: resource ID, instance ID, etc.
    pub id:       u16,
    /// Raw value bytes (for leaf records)
    pub value:    Vec<u8>,
    /// Child records (for MultipleResource containers)
    pub children: Vec<TlvRecord>,
}

impl TlvRecord {
    /// Interpret the raw value as an integer.
    pub fn as_i64(&self) -> Option<i64> {
        match self.value.len() {
            1 => Some(self.value[0] as i8 as i64),
            2 => Some(i16::from_be_bytes([self.value[0], self.value[1]]) as i64),
            4 => Some(i32::from_be_bytes([
                self.value[0], self.value[1], self.value[2], self.value[3],
            ]) as i64),
            8 => Some(i64::from_be_bytes(self.value.as_slice().try_into().ok()?)),
            _ => None,
        }
    }

    /// Interpret the raw value as an f64 (IEEE 754).
    pub fn as_f64(&self) -> Option<f64> {
        match self.value.len() {
            4 => {
                let bits = u32::from_be_bytes(self.value[..4].try_into().ok()?);
                Some(f32::from_bits(bits) as f64)
            }
            8 => {
                let bits = u64::from_be_bytes(self.value[..8].try_into().ok()?);
                Some(f64::from_bits(bits))
            }
            _ => None,
        }
    }

    /// Interpret the raw value as a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        self.value.first().map(|&b| b != 0)
    }

    /// Interpret the raw value as a UTF-8 string.
    pub fn as_str(&self) -> Option<String> {
        std::str::from_utf8(&self.value).ok().map(|s| s.to_string())
    }

    /// Convert value to the best-fit JSON value using heuristics
    /// (prefer string → try integer → try float → hex fallback).
    pub fn to_json_value(&self) -> Value {
        if self.value.is_empty() {
            return Value::Null;
        }
        // Boolean: single byte 0 or 1
        if self.value.len() == 1 && (self.value[0] == 0 || self.value[0] == 1) {
            // Ambiguous — could be integer. Prefer integer to avoid false boolean.
            return Value::Number(serde_json::Number::from(self.value[0] as i64));
        }
        // Try UTF-8 string first
        if let Some(s) = self.as_str() {
            if s.chars().all(|c| !c.is_control() || c == '\n' || c == '\r') {
                return Value::String(s);
            }
        }
        // Try float (4 or 8 bytes)
        if let Some(f) = self.as_f64() {
            if f.is_finite() {
                if let Some(n) = serde_json::Number::from_f64(f) {
                    return Value::Number(n);
                }
            }
        }
        // Try integer
        if let Some(i) = self.as_i64() {
            return Value::Number(serde_json::Number::from(i));
        }
        // Fallback: hex string
        Value::String(
            self.value.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(""),
        )
    }
}

// ── TLV parser ────────────────────────────────────────────────────────────────

/// Parse a byte slice into a list of top-level TLV records.
pub fn parse_tlv(data: &[u8]) -> Vec<TlvRecord> {
    let mut pos = 0;
    let mut records = Vec::new();
    while pos < data.len() {
        match parse_one_tlv(data, &mut pos) {
            Some(r) => records.push(r),
            None    => break,
        }
    }
    records
}

fn parse_one_tlv(data: &[u8], pos: &mut usize) -> Option<TlvRecord> {
    if *pos >= data.len() { return None; }

    let type_byte = data[*pos]; *pos += 1;

    // Bits 7-6: TLV type
    let tlv_type = match (type_byte >> 6) & 0x03 {
        0x00 => TlvType::ObjectInstance,
        0x01 => TlvType::ResourceInstance,
        0x02 => TlvType::MultipleResource,
        _    => TlvType::ResourceValue,
    };

    // Bit 5: ID length (0 = 1 byte, 1 = 2 bytes)
    let id_len = if (type_byte >> 5) & 0x01 == 0 { 1 } else { 2 };

    // Bits 4-3: length field type
    let len_type = (type_byte >> 3) & 0x03;

    // Bits 2-0: length bits (used when len_type == 0)
    let len_bits = (type_byte & 0x07) as usize;

    // Read identifier
    if *pos + id_len > data.len() { return None; }
    let id = if id_len == 1 {
        let v = data[*pos] as u16; *pos += 1; v
    } else {
        let v = u16::from_be_bytes([data[*pos], data[*pos + 1]]); *pos += 2; v
    };

    // Read length
    let length = match len_type {
        0 => len_bits,
        1 => { if *pos >= data.len() { return None; } let v = data[*pos] as usize; *pos += 1; v }
        2 => {
            if *pos + 2 > data.len() { return None; }
            let v = u16::from_be_bytes([data[*pos], data[*pos + 1]]) as usize;
            *pos += 2; v
        }
        _ => {
            if *pos + 3 > data.len() { return None; }
            let v = ((data[*pos] as usize) << 16)
                  | ((data[*pos + 1] as usize) << 8)
                  | (data[*pos + 2] as usize);
            *pos += 3; v
        }
    };

    if *pos + length > data.len() { return None; }
    let value = data[*pos..*pos + length].to_vec();
    *pos += length;

    // Recursively parse children for MultipleResource
    let children = if tlv_type == TlvType::MultipleResource {
        parse_tlv(&value)
    } else {
        Vec::new()
    };

    Some(TlvRecord { tlv_type, id, value, children })
}

// ── TLV → JSON ────────────────────────────────────────────────────────────────

/// Convert a flat list of ResourceValue TLV records to a JSON object.
/// Key is the resource ID as a string (e.g., "5700"), value is the parsed value.
pub fn tlv_to_json(records: &[TlvRecord]) -> Map<String, Value> {
    let mut map = Map::new();
    for rec in records {
        match &rec.tlv_type {
            TlvType::ResourceValue => {
                map.insert(rec.id.to_string(), rec.to_json_value());
            }
            TlvType::MultipleResource => {
                // Represent as an array of the children's values
                let arr: Vec<Value> = rec.children.iter().map(|c| c.to_json_value()).collect();
                map.insert(rec.id.to_string(), Value::Array(arr));
            }
            TlvType::ObjectInstance | TlvType::ResourceInstance => {
                // Recurse — flatten nested resources into the map
                let nested = tlv_to_json(&parse_tlv(&rec.value));
                map.extend(nested);
            }
        }
    }
    map
}

// ── Content-Format routing ─────────────────────────────────────────────────────

/// CoAP content-format codes relevant to LwM2M.
pub mod content_format {
    pub const TEXT_PLAIN:      u16 = 0;
    pub const LINK_FORMAT:     u16 = 40;
    pub const APPLICATION_JSON: u16 = 50;
    pub const APPLICATION_CBOR: u16 = 60;
    pub const APPLICATION_SENML_JSON: u16 = 110;
    pub const APPLICATION_LWM2M_TLV:  u16 = 11542;
    pub const APPLICATION_LWM2M_JSON: u16 = 11543;
    pub const APPLICATION_SENML_CBOR: u16 = 112;
}

/// Detect payload format from CoAP content-format option value.
#[derive(Debug, PartialEq)]
pub enum PayloadFormat {
    Tlv,
    SenMlJson,
    Json,
    TextPlain,
    Unknown(u16),
}

pub fn detect_format(content_format: Option<u16>) -> PayloadFormat {
    match content_format {
        None | Some(content_format::APPLICATION_LWM2M_TLV) => PayloadFormat::Tlv,
        Some(content_format::APPLICATION_SENML_JSON)       => PayloadFormat::SenMlJson,
        Some(content_format::APPLICATION_LWM2M_JSON)
        | Some(content_format::APPLICATION_JSON)           => PayloadFormat::Json,
        Some(content_format::TEXT_PLAIN)                   => PayloadFormat::TextPlain,
        Some(other)                                        => PayloadFormat::Unknown(other),
    }
}

// ── TLV encoder ───────────────────────────────────────────────────────────────

/// Encode a resource value (string) as a TLV ResourceValue record.
pub fn encode_resource_string(resource_id: u16, value: &str) -> Vec<u8> {
    encode_tlv_record(TlvType::ResourceValue, resource_id, value.as_bytes())
}

/// Encode a resource value (integer) as a TLV ResourceValue record.
pub fn encode_resource_i64(resource_id: u16, value: i64) -> Vec<u8> {
    let bytes = if value >= i8::MIN as i64 && value <= i8::MAX as i64 {
        vec![value as i8 as u8]
    } else if value >= i16::MIN as i64 && value <= i16::MAX as i64 {
        (value as i16).to_be_bytes().to_vec()
    } else if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
        (value as i32).to_be_bytes().to_vec()
    } else {
        value.to_be_bytes().to_vec()
    };
    encode_tlv_record(TlvType::ResourceValue, resource_id, &bytes)
}

fn encode_tlv_record(tlv_type: TlvType, id: u16, value: &[u8]) -> Vec<u8> {
    let type_bits: u8 = match tlv_type {
        TlvType::ObjectInstance   => 0x00,
        TlvType::ResourceInstance => 0x40,
        TlvType::MultipleResource => 0x80,
        TlvType::ResourceValue    => 0xc0,
    };

    let id_flag: u8 = if id > 0xFF { 0x20 } else { 0x00 };
    let length = value.len();

    let (len_flag, len_bytes): (u8, Vec<u8>) = if length < 8 {
        (length as u8, vec![])
    } else if length <= 0xFF {
        (0x08, vec![length as u8])
    } else if length <= 0xFFFF {
        (0x10, (length as u16).to_be_bytes().to_vec())
    } else {
        (0x18, vec![(length >> 16) as u8, (length >> 8) as u8, length as u8])
    };

    let type_byte = type_bits | id_flag | len_flag;
    let mut out = vec![type_byte];
    if id > 0xFF {
        out.extend_from_slice(&id.to_be_bytes());
    } else {
        out.push(id as u8);
    }
    out.extend(len_bytes);
    out.extend_from_slice(value);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_resource_value() {
        // Resource 5700 (0x1644), value = 4-byte float 25.5°C
        // type_byte = 0b11_0_01_000 = 0xC8 (ResourceValue, 1-byte ID, 3-bit len=4)
        // Wait — 5700 > 255, so ID is 2 bytes: id_flag = 0x20
        // type_byte = 0b11_1_00_100 = 0xE4 (ResourceValue, 2-byte ID, 3-bit len=4)
        let float_val: f32 = 25.5;
        let float_bytes = float_val.to_bits().to_be_bytes();
        let mut data = vec![0xE4u8, 0x16, 0x44]; // type=ResourceValue, id=5700, len=4
        data.extend_from_slice(&float_bytes);

        let records = parse_tlv(&data);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, 5700);
        assert_eq!(records[0].tlv_type, TlvType::ResourceValue);
        let v = records[0].as_f64().unwrap();
        assert!((v - 25.5).abs() < 0.001);
    }

    #[test]
    fn test_parse_string_resource() {
        // Resource 0 (sysDescr-like), value = "hello"
        // type_byte = 0b11_0_00_101 = 0xC5 (ResourceValue, 1-byte ID, len=5)
        let data = b"\xC5\x00hello";
        let records = parse_tlv(data);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, 0);
        assert_eq!(records[0].as_str().unwrap(), "hello");
    }

    #[test]
    fn test_roundtrip_encode_decode() {
        let encoded = encode_resource_i64(1, 42);
        let records  = parse_tlv(&encoded);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].as_i64(), Some(42));
    }

    #[test]
    fn test_roundtrip_string() {
        let encoded = encode_resource_string(3, "SensorV2");
        let records  = parse_tlv(&encoded);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, 3);
        assert_eq!(records[0].as_str().unwrap(), "SensorV2");
    }

    #[test]
    fn test_tlv_to_json() {
        let encoded = encode_resource_string(5, "firmware-1.0");
        let records  = parse_tlv(&encoded);
        let map      = tlv_to_json(&records);
        assert_eq!(map["5"].as_str(), Some("firmware-1.0"));
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(detect_format(Some(11542)), PayloadFormat::Tlv);
        assert_eq!(detect_format(None),        PayloadFormat::Tlv);
        assert_eq!(detect_format(Some(110)),   PayloadFormat::SenMlJson);
        assert_eq!(detect_format(Some(50)),    PayloadFormat::Json);
    }
}
