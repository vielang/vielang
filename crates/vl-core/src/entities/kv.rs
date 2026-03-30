use serde::{Deserialize, Serialize};

/// Khớp Java: DataType enum (protoNumber)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataType {
    Boolean = 0,
    Long = 1,
    Double = 2,
    String = 3,
    Json = 4,
}

/// Một telemetry entry — khớp Java: TsKvEntry
/// Bảng: ts_kv_latest, ts_kv (timescale)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsKvEntry {
    pub entity_id: uuid::Uuid,
    /// FK tới key_dictionary.key_id
    pub key: i32,
    pub ts: i64,
    pub bool_v: Option<bool>,
    pub str_v: Option<String>,
    pub long_v: Option<i64>,
    pub dbl_v: Option<f64>,
    pub json_v: Option<serde_json::Value>,
    pub version: i64,
}

impl TsKvEntry {
    pub fn data_type(&self) -> DataType {
        if self.bool_v.is_some() { return DataType::Boolean; }
        if self.long_v.is_some() { return DataType::Long; }
        if self.dbl_v.is_some()  { return DataType::Double; }
        if self.json_v.is_some() { return DataType::Json; }
        DataType::String
    }

    pub fn value_as_string(&self) -> String {
        match self.data_type() {
            DataType::Boolean => self.bool_v.map(|v| v.to_string()).unwrap_or_default(),
            DataType::Long    => self.long_v.map(|v| v.to_string()).unwrap_or_default(),
            DataType::Double  => self.dbl_v.map(|v| v.to_string()).unwrap_or_default(),
            DataType::String  => self.str_v.clone().unwrap_or_default(),
            DataType::Json    => self.json_v.as_ref().map(|v| v.to_string()).unwrap_or_default(),
        }
    }
}

/// Attribute entry — khớp Java: AttributeKvEntry
/// Bảng: attribute_kv
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeKvEntry {
    pub entity_id: uuid::Uuid,
    /// 1=CLIENT_SCOPE, 2=SERVER_SCOPE, 3=SHARED_SCOPE
    pub attribute_type: AttributeScope,
    /// FK tới key_dictionary.key_id
    pub attribute_key: i32,
    pub last_update_ts: i64,
    pub bool_v: Option<bool>,
    pub str_v: Option<String>,
    pub long_v: Option<i64>,
    pub dbl_v: Option<f64>,
    pub json_v: Option<serde_json::Value>,
    pub version: i64,
}

/// Khớp Java: AttributeScope / attribute_type integer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttributeScope {
    ClientScope  = 1,
    ServerScope  = 2,
    SharedScope  = 3,
}

impl TryFrom<i32> for AttributeScope {
    type Error = ();
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(Self::ClientScope),
            2 => Ok(Self::ServerScope),
            3 => Ok(Self::SharedScope),
            _ => Err(()),
        }
    }
}

/// key_dictionary — ánh xạ string key → integer ID
/// Dùng để tiết kiệm không gian trong ts_kv_latest và attribute_kv
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDictionary {
    pub key: String,
    pub key_id: i32,
}

/// Telemetry record với string key — dùng cho TimeseriesDao trait.
/// Không cần key_dictionary integer mapping, phù hợp cả PostgreSQL và Cassandra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsRecord {
    pub entity_id: uuid::Uuid,
    pub key: String,
    pub ts: i64,
    pub bool_v: Option<bool>,
    pub str_v: Option<String>,
    pub long_v: Option<i64>,
    pub dbl_v: Option<f64>,
    pub json_v: Option<serde_json::Value>,
}

impl TsRecord {
    pub fn value_as_string(&self) -> String {
        if let Some(v) = self.bool_v        { return v.to_string(); }
        if let Some(v) = self.long_v        { return v.to_string(); }
        if let Some(v) = self.dbl_v         { return v.to_string(); }
        if let Some(ref v) = self.json_v    { return v.to_string(); }
        if let Some(ref v) = self.str_v     { return v.clone(); }
        String::new()
    }
}

/// Payload telemetry từ device (JSON format ThingsBoard standard)
/// Ví dụ: {"temperature": 25.5, "humidity": 60}
/// Hoặc dạng mảng có timestamp: [{"ts": 1234567890000, "values": {"temperature": 25.5}}]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TelemetryPayload {
    /// {"key": value, ...}
    Simple(std::collections::HashMap<String, serde_json::Value>),
    /// [{"ts": i64, "values": {...}}]
    WithTimestamps(Vec<TsDataPoint>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsDataPoint {
    pub ts: i64,
    pub values: std::collections::HashMap<String, serde_json::Value>,
}
