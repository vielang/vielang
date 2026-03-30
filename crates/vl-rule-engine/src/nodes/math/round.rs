use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Round numeric values in the message body to a configurable precision.
/// Can target specific keys or all numeric keys in the message.
/// Java: TbRoundNode
/// Relations: Success, Failure (data is not JSON)
/// Config:
/// ```json
/// {
///   "keys": ["temperature", "humidity"],   // empty = round ALL numeric keys
///   "precision": 2,                        // decimal places (default: 0)
///   "roundingStrategy": "HALF_UP"          // HALF_UP | HALF_DOWN | FLOOR | CEIL
/// }
/// ```
pub struct RoundNode {
    keys:      Vec<String>,   // empty = all keys
    precision: u32,
    strategy:  RoundStrategy,
}

#[derive(Debug, Clone, Copy)]
enum RoundStrategy {
    HalfUp,
    HalfDown,
    Floor,
    Ceil,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    keys: Vec<String>,
    #[serde(default)]
    precision: u32,
    #[serde(rename = "roundingStrategy", default = "default_strategy")]
    rounding_strategy: String,
}

fn default_strategy() -> String { "HALF_UP".into() }

impl RoundNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("RoundNode: {}", e)))?;
        let strategy = match cfg.rounding_strategy.to_uppercase().as_str() {
            "HALF_UP"   => RoundStrategy::HalfUp,
            "HALF_DOWN" => RoundStrategy::HalfDown,
            "FLOOR"     => RoundStrategy::Floor,
            "CEIL"      => RoundStrategy::Ceil,
            other       => return Err(RuleEngineError::Config(
                format!("RoundNode: unknown roundingStrategy '{}'", other))),
        };
        Ok(Self {
            keys: cfg.keys,
            precision: cfg.precision,
            strategy,
        })
    }

    fn round_value(&self, val: f64) -> f64 {
        let factor = 10f64.powi(self.precision as i32);
        match self.strategy {
            RoundStrategy::HalfUp   => (val * factor + 0.5).floor() / factor,
            RoundStrategy::HalfDown => (val * factor - 0.5).ceil() / factor,
            RoundStrategy::Floor    => (val * factor).floor() / factor,
            RoundStrategy::Ceil     => (val * factor).ceil() / factor,
        }
    }

    fn should_round(&self, key: &str) -> bool {
        self.keys.is_empty() || self.keys.iter().any(|k| k == key)
    }
}

#[async_trait]
impl RuleNode for RoundNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut data: serde_json::Value = match serde_json::from_str(&msg.data) {
            Ok(v) => v,
            Err(_) => {
                let mut m = msg;
                m.metadata.insert("error".into(), "RoundNode: message data is not valid JSON".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        if let Some(obj) = data.as_object_mut() {
            for (key, val) in obj.iter_mut() {
                if self.should_round(key) {
                    if let Some(n) = val.as_f64() {
                        *val = serde_json::json!(self.round_value(n));
                    }
                }
            }
        }

        let mut out = msg;
        out.data = serde_json::to_string(&data)
            .map_err(|e| RuleEngineError::Config(format!("RoundNode: {}", e)))?;
        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;
    use uuid::Uuid;
    use vl_core::entities::TbMsg;
    use vl_dao::postgres::{
        alarm::AlarmDao, asset::AssetDao, customer::CustomerDao, device::DeviceDao,
        device_profile::DeviceProfileDao, event::EventDao, geofence::GeofenceDao,
        kv::KvDao, relation::RelationDao, tenant::TenantDao,
    };
    use crate::node::{DaoServices, RuleNode, RuleNodeCtx};

    fn make_ctx() -> RuleNodeCtx {
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").unwrap();
        RuleNodeCtx {
            node_id:     Uuid::nil(),
            tenant_id:   Uuid::nil(),
            edge_sender: None,
            dao: Arc::new(DaoServices {
                kv:             Arc::new(KvDao::new(pool.clone())),
                alarm:          Arc::new(AlarmDao::new(pool.clone())),
                device:         Arc::new(DeviceDao::new(pool.clone())),
                device_profile: Arc::new(DeviceProfileDao::new(pool.clone())),
                asset:          Arc::new(AssetDao::new(pool.clone())),
                relation:       Arc::new(RelationDao::new(pool.clone())),
                customer:       Arc::new(CustomerDao::new(pool.clone())),
                tenant:         Arc::new(TenantDao::new(pool.clone())),
                event:          Arc::new(EventDao::new(pool.clone())),
                geofence:       Arc::new(GeofenceDao::new(pool)),
            }),
        }
    }

    fn msg_with_data(data: &str) -> TbMsg {
        TbMsg::new("POST_TELEMETRY_REQUEST", Uuid::new_v4(), "DEVICE", data)
    }

    #[test]
    fn parses_config() {
        let node = RoundNode::new(&json!({
            "keys": ["temperature"],
            "precision": 2,
            "roundingStrategy": "HALF_UP"
        })).unwrap();
        assert_eq!(node.precision, 2);
        assert_eq!(node.keys, vec!["temperature"]);
    }

    #[test]
    fn empty_config_allowed() {
        assert!(RoundNode::new(&json!({})).is_ok());
    }

    #[test]
    fn unknown_strategy_is_error() {
        assert!(RoundNode::new(&json!({ "roundingStrategy": "BANKER" })).is_err());
    }

    #[tokio::test]
    async fn rounds_specific_key() {
        let node = RoundNode::new(&json!({
            "keys": ["temperature"],
            "precision": 1
        })).unwrap();
        let msg = msg_with_data(r#"{"temperature": 22.567, "humidity": 60.123}"#);
        let result = node.process(&make_ctx(), msg).await.unwrap();
        let data: serde_json::Value = serde_json::from_str(&result[0].1.data).unwrap();
        // temperature rounded to 1 decimal
        assert!((data["temperature"].as_f64().unwrap() - 22.6).abs() < 0.01);
        // humidity unchanged
        assert!((data["humidity"].as_f64().unwrap() - 60.123).abs() < 0.001);
    }

    #[tokio::test]
    async fn rounds_all_numeric_keys_when_keys_empty() {
        let node = RoundNode::new(&json!({ "precision": 0 })).unwrap();
        let msg = msg_with_data(r#"{"temp": 22.7, "pressure": 1013.4}"#);
        let result = node.process(&make_ctx(), msg).await.unwrap();
        let data: serde_json::Value = serde_json::from_str(&result[0].1.data).unwrap();
        assert_eq!(data["temp"].as_f64().unwrap(), 23.0);
        assert_eq!(data["pressure"].as_f64().unwrap(), 1013.0);
    }

    #[tokio::test]
    async fn floor_strategy() {
        let node = RoundNode::new(&json!({ "precision": 0, "roundingStrategy": "FLOOR" })).unwrap();
        let msg = msg_with_data(r#"{"temp": 22.9}"#);
        let result = node.process(&make_ctx(), msg).await.unwrap();
        let data: serde_json::Value = serde_json::from_str(&result[0].1.data).unwrap();
        assert_eq!(data["temp"].as_f64().unwrap(), 22.0);
    }

    #[tokio::test]
    async fn ceil_strategy() {
        let node = RoundNode::new(&json!({ "precision": 0, "roundingStrategy": "CEIL" })).unwrap();
        let msg = msg_with_data(r#"{"temp": 22.1}"#);
        let result = node.process(&make_ctx(), msg).await.unwrap();
        let data: serde_json::Value = serde_json::from_str(&result[0].1.data).unwrap();
        assert_eq!(data["temp"].as_f64().unwrap(), 23.0);
    }

    #[tokio::test]
    async fn non_json_data_routes_failure() {
        let node = RoundNode::new(&json!({})).unwrap();
        let msg = msg_with_data("not json");
        let result = node.process(&make_ctx(), msg).await.unwrap();
        assert_eq!(result[0].0, crate::node::RelationType::Failure);
    }
}
