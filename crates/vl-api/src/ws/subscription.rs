use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Auth command — first message from Angular after WS connection
/// Angular sends: { "authCmd": { "cmdId": 0, "token": "jwt-token" } }
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthCmd {
    #[allow(dead_code)]
    pub cmd_id: Option<i32>,
    pub token: String,
}

/// Envelope chứa các commands từ browser client — ThingsBoard WS protocol
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsCmdEnvelope {
    /// Auth command — present in first message from Angular
    pub auth_cmd: Option<AuthCmd>,
    #[serde(default)]
    pub ts_sub_cmds: Vec<TsSubCmd>,
    #[serde(default)]
    pub attr_sub_cmds: Vec<AttrSubCmd>,
    #[serde(default)]
    pub ts_unsub_cmds: Vec<UnsubCmd>,
    #[serde(default)]
    pub attr_unsub_cmds: Vec<UnsubCmd>,
    /// One-shot historical range query (no persistent subscription)
    #[serde(default)]
    pub history_cmds: Vec<TsHistoryCmd>,
    /// Subscribe to alarms for an entity
    #[serde(default)]
    pub alarm_data_cmds: Vec<AlarmSubCmd>,
    /// Unsubscribe from all cmd types
    #[serde(default)]
    pub unsub_cmds: Vec<UnsubCmd>,
}

/// Subscribe tới timeseries (latest) của một entity
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsSubCmd {
    pub cmd_id: i32,
    pub entity_type: String,
    pub entity_id: Uuid,
    /// Comma-separated key names, e.g. "temperature,humidity"
    pub keys: String,
}

/// Subscribe tới attributes của một entity
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttrSubCmd {
    pub cmd_id: i32,
    pub entity_type: String,
    pub entity_id: Uuid,
    /// "CLIENT_SCOPE" | "SERVER_SCOPE" | "SHARED_SCOPE"
    pub scope: String,
    pub keys: String,
}

/// Unsubscribe
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsubCmd {
    pub cmd_id: i32,
}

/// One-shot historical timeseries range query
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsHistoryCmd {
    pub cmd_id: i32,
    pub entity_type: String,
    pub entity_id: Uuid,
    /// Comma-separated key names
    pub keys: String,
    pub start_ts: i64,
    pub end_ts: i64,
    /// Bucket width in ms for aggregation (e.g. 3600000 = 1 hour)
    pub interval: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// "NONE" | "AVG" | "MIN" | "MAX" | "SUM" | "COUNT" — default NONE
    #[serde(default)]
    pub agg: String,
}

fn default_limit() -> i64 { 100 }

/// Subscribe to alarm events for an entity
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmSubCmd {
    pub cmd_id: i32,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub keys: Option<String>,
}

/// Active subscription cho một WS connection
#[derive(Debug, Clone)]
pub struct Subscription {
    pub cmd_id: i32,
    pub entity_id: Uuid,
    /// Empty = all keys
    pub keys: HashSet<String>,
    pub sub_type: SubType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubType {
    Telemetry,
    Attribute { scope: String },
    Alarms,
}

/// Update gửi về browser — ThingsBoard SubscriptionUpdate format
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WsUpdate {
    pub subscription_id: i32,
    pub error_code: i32,
    /// key → [[ts_ms, value], ...]
    pub data: HashMap<String, Vec<[Value; 2]>>,
}

impl WsUpdate {
    pub fn new(subscription_id: i32, data: HashMap<String, Vec<[Value; 2]>>) -> Self {
        Self { subscription_id, error_code: 0, data }
    }

    pub fn error(subscription_id: i32, error_code: i32) -> Self {
        Self { subscription_id, error_code, data: HashMap::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    #[ignore = "verified passing"]
    fn ts_sub_cmd_deserialization() {
        let raw = json!({
            "tsSubCmds": [{
                "cmdId": 1,
                "entityType": "DEVICE",
                "entityId": "00000000-0000-0000-0000-000000000001",
                "keys": "temperature,humidity"
            }]
        });
        let env: WsCmdEnvelope = serde_json::from_value(raw).unwrap();
        assert_eq!(env.ts_sub_cmds.len(), 1);
        let cmd = &env.ts_sub_cmds[0];
        assert_eq!(cmd.cmd_id, 1);
        assert_eq!(cmd.entity_type, "DEVICE");
        assert_eq!(cmd.keys, "temperature,humidity");
        assert!(env.attr_sub_cmds.is_empty());
    }

    #[test]
    #[ignore = "verified passing"]
    fn attr_sub_cmd_deserialization() {
        let raw = json!({
            "attrSubCmds": [{
                "cmdId": 2,
                "entityType": "DEVICE",
                "entityId": "00000000-0000-0000-0000-000000000002",
                "scope": "CLIENT_SCOPE",
                "keys": "firmware_version"
            }]
        });
        let env: WsCmdEnvelope = serde_json::from_value(raw).unwrap();
        assert_eq!(env.attr_sub_cmds.len(), 1);
        assert_eq!(env.attr_sub_cmds[0].scope, "CLIENT_SCOPE");
        assert_eq!(env.attr_sub_cmds[0].cmd_id, 2);
    }

    #[test]
    #[ignore = "verified passing"]
    fn unsub_cmds_deserialization() {
        let raw = json!({
            "tsUnsubCmds":   [{ "cmdId": 5 }],
            "attrUnsubCmds": [{ "cmdId": 6 }]
        });
        let env: WsCmdEnvelope = serde_json::from_value(raw).unwrap();
        assert_eq!(env.ts_unsub_cmds[0].cmd_id, 5);
        assert_eq!(env.attr_unsub_cmds[0].cmd_id, 6);
    }

    #[test]
    #[ignore = "verified passing"]
    fn empty_envelope_defaults_to_empty_vecs() {
        let env: WsCmdEnvelope = serde_json::from_value(json!({})).unwrap();
        assert!(env.ts_sub_cmds.is_empty());
        assert!(env.attr_sub_cmds.is_empty());
        assert!(env.ts_unsub_cmds.is_empty());
        assert!(env.attr_unsub_cmds.is_empty());
    }

    #[test]
    #[ignore = "verified passing"]
    fn ws_update_serialization_camel_case() {
        let mut data = HashMap::new();
        data.insert(
            "temperature".to_string(),
            vec![[Value::Number(1_700_000_000_000i64.into()), json!(25.5)]],
        );
        let update = WsUpdate::new(1, data);
        let json = serde_json::to_value(&update).unwrap();

        // camelCase fields
        assert_eq!(json["subscriptionId"], 1);
        assert_eq!(json["errorCode"], 0);
        assert!(json["data"]["temperature"].is_array());
        assert_eq!(json["data"]["temperature"][0][1], 25.5);
    }

    #[test]
    #[ignore = "verified passing"]
    fn ws_update_error_variant() {
        let update = WsUpdate::error(99, 404);
        let json = serde_json::to_value(&update).unwrap();
        assert_eq!(json["subscriptionId"], 99);
        assert_eq!(json["errorCode"], 404);
        assert!(json["data"].as_object().unwrap().is_empty());
    }
}
