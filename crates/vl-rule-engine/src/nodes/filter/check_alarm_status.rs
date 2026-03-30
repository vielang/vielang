use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use vl_dao::PageLink;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Filter messages based on whether originator has active alarms matching given criteria.
/// Java: TbCheckAlarmStatusNode
/// Relations:
///   - True  — at least one matching alarm exists
///   - False — no matching alarm found
/// Config:
/// ```json
/// {
///   "alarmStatusList": ["ACTIVE_UNACK", "ACTIVE_ACK"],
///   "alarmTypes": ["Temperature Alarm"],   // optional, empty = any type
///   "operation": "AND"                     // AND = all statuses must match, OR = any (default OR)
/// }
/// ```
pub struct CheckAlarmStatusNode {
    status_filter: Vec<String>,
    alarm_types:   Vec<String>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "alarmStatusList", default)]
    alarm_status_list: Vec<String>,
    #[serde(rename = "alarmTypes", default)]
    alarm_types: Vec<String>,
}

impl CheckAlarmStatusNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CheckAlarmStatusNode: {}", e)))?;
        Ok(Self {
            status_filter: cfg.alarm_status_list,
            alarm_types: cfg.alarm_types,
        })
    }

    fn alarm_matches_status(alarm: &vl_core::entities::Alarm, statuses: &[String]) -> bool {
        if statuses.is_empty() {
            return true;
        }
        // Compute ThingsBoard status string from acknowledged/cleared flags
        let status = match (alarm.acknowledged, alarm.cleared) {
            (false, false) => "ACTIVE_UNACK",
            (true,  false) => "ACTIVE_ACK",
            (false, true)  => "CLEARED_UNACK",
            (true,  true)  => "CLEARED_ACK",
        };
        statuses.iter().any(|s| s == status)
    }
}

#[async_trait]
impl RuleNode for CheckAlarmStatusNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let page = PageLink::new(0, 100);
        let alarms = ctx.dao.alarm
            .find_by_originator(ctx.tenant_id, msg.originator_id, &page)
            .await?;

        let matched = alarms.data.iter().any(|alarm| {
            let type_ok = self.alarm_types.is_empty()
                || self.alarm_types.contains(&alarm.alarm_type);
            let status_ok = Self::alarm_matches_status(alarm, &self.status_filter);
            type_ok && status_ok
        });

        if matched {
            Ok(vec![(RelationType::True, msg)])
        } else {
            Ok(vec![(RelationType::False, msg)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_config() {
        let cfg = json!({
            "alarmStatusList": ["ACTIVE_UNACK", "ACTIVE_ACK"],
            "alarmTypes": ["High Temp"]
        });
        let node = CheckAlarmStatusNode::new(&cfg).unwrap();
        assert_eq!(node.status_filter.len(), 2);
        assert_eq!(node.alarm_types.len(), 1);
    }

    #[test]
    fn empty_config_allowed() {
        let node = CheckAlarmStatusNode::new(&json!({})).unwrap();
        assert!(node.status_filter.is_empty());
        assert!(node.alarm_types.is_empty());
    }
}
