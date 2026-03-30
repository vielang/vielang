use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use vl_dao::PageLink;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Enrich message metadata with details of the latest active alarm for the originator.
/// Java: TbGetAlarmDetailsNode (part of alarm enrichment suite)
/// Relations: Success (with or without alarm), Failure (DAO error)
/// Config:
/// ```json
/// {
///   "alarmTypes": ["Temperature Alarm"],   // empty = any type
///   "detailsList": ["type", "severity", "status", "startTs", "endTs"],
///   "fetchActiveOnly": true
/// }
/// ```
pub struct GetAlarmDetailsNode {
    alarm_types:       Vec<String>,
    details_list:      Vec<String>,
    fetch_active_only: bool,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "alarmTypes", default)]
    alarm_types: Vec<String>,
    #[serde(rename = "detailsList", default = "default_details")]
    details_list: Vec<String>,
    #[serde(rename = "fetchActiveOnly", default = "default_true")]
    fetch_active_only: bool,
}

fn default_true() -> bool { true }
fn default_details() -> Vec<String> {
    vec!["type".into(), "severity".into(), "status".into()]
}

impl GetAlarmDetailsNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetAlarmDetailsNode: {}", e)))?;
        Ok(Self {
            alarm_types: cfg.alarm_types,
            details_list: cfg.details_list,
            fetch_active_only: cfg.fetch_active_only,
        })
    }

    fn alarm_status_str(acknowledged: bool, cleared: bool) -> &'static str {
        match (acknowledged, cleared) {
            (false, false) => "ACTIVE_UNACK",
            (true,  false) => "ACTIVE_ACK",
            (false, true)  => "CLEARED_UNACK",
            (true,  true)  => "CLEARED_ACK",
        }
    }
}

#[async_trait]
impl RuleNode for GetAlarmDetailsNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let page = PageLink::new(0, 100);
        let alarms = ctx.dao.alarm
            .find_by_originator(ctx.tenant_id, msg.originator_id, &page)
            .await?;

        // Find the most recent matching alarm
        let alarm = alarms.data.into_iter()
            .filter(|a| {
                let type_ok = self.alarm_types.is_empty()
                    || self.alarm_types.contains(&a.alarm_type);
                let active_ok = !self.fetch_active_only || !a.cleared;
                type_ok && active_ok
            })
            .max_by_key(|a| a.start_ts);

        let mut out = msg;

        if let Some(a) = alarm {
            let status = Self::alarm_status_str(a.acknowledged, a.cleared);
            for detail in &self.details_list {
                let val = match detail.as_str() {
                    "type"     | "alarmType"   => a.alarm_type.clone(),
                    "severity"                 => format!("{:?}", a.severity),
                    "status"                   => status.to_string(),
                    "startTs"                  => a.start_ts.to_string(),
                    "endTs"                    => a.end_ts.to_string(),
                    "acknowledged"             => a.acknowledged.to_string(),
                    "cleared"                  => a.cleared.to_string(),
                    "id"                       => a.id.to_string(),
                    _                          => continue,
                };
                out.metadata.insert(format!("alarm_{}", detail), val);
            }
            out.metadata.insert("hasActiveAlarm".into(), "true".into());
        } else {
            out.metadata.insert("hasActiveAlarm".into(), "false".into());
        }

        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_config_ok() {
        let node = GetAlarmDetailsNode::new(&json!({})).unwrap();
        assert!(node.fetch_active_only);
        assert_eq!(node.details_list.len(), 3);
    }

    #[test]
    fn parses_alarm_types() {
        let node = GetAlarmDetailsNode::new(&json!({
            "alarmTypes": ["High Temp", "Low Battery"]
        })).unwrap();
        assert_eq!(node.alarm_types.len(), 2);
    }
}
