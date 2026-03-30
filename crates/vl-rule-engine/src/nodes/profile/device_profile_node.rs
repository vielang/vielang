use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use vl_core::entities::{Alarm, TbMsg};
use vl_dao::PageLink;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};
use super::alarm_state_machine::{
    build_alarm, evaluate_alarm_rules, AlarmAction, ProfileAlarmRule,
};

/// Main device profile rule node — evaluates alarm rules from device profile
/// and creates/clears alarms accordingly.
/// Java: TbDeviceProfileNode
///
/// Message types processed: POST_TELEMETRY_REQUEST, POST_ATTRIBUTES_REQUEST,
///                          ACTIVITY_EVENT, INACTIVITY_EVENT
///
/// Relations:
///   - "Alarm Created"  — a new alarm was created
///   - "Alarm Updated"  — existing alarm severity changed
///   - "Alarm Severity Updated" — severity changed but alarm already existed
///   - "Alarm Cleared"  — alarm was cleared
///   - Success          — message processed, no alarm changes
///   - Failure          — device/profile not found or error
///
/// Config: `{}` — all config comes from the device profile in the DB
pub struct DeviceProfileRuleNode {
    // Cache: device_id → device_profile_id (avoid DB lookup on every message)
    profile_id_cache: Arc<DashMap<Uuid, Uuid>>,
    // Cache: profile_id → parsed alarm rules
    rule_cache: Arc<DashMap<Uuid, Vec<ProfileAlarmRule>>>,
}

impl DeviceProfileRuleNode {
    pub fn new(_config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        Ok(Self {
            profile_id_cache: Arc::new(DashMap::new()),
            rule_cache: Arc::new(DashMap::new()),
        })
    }

    async fn get_alarm_rules(
        &self,
        ctx: &RuleNodeCtx,
        device_id: Uuid,
    ) -> Result<Option<Vec<ProfileAlarmRule>>, RuleEngineError> {
        // Step 1: get device profile id
        let profile_id = if let Some(pid) = self.profile_id_cache.get(&device_id) {
            *pid
        } else {
            let device = ctx.dao.device.find_by_id(device_id).await?;
            match device {
                None => return Ok(None),
                Some(d) => {
                    let pid = d.device_profile_id;
                    self.profile_id_cache.insert(device_id, pid);
                    pid
                }
            }
        };

        // Step 2: get alarm rules from profile
        if let Some(rules) = self.rule_cache.get(&profile_id) {
            return Ok(Some(rules.clone()));
        }

        let profile = ctx.dao.device_profile.find_by_id(profile_id).await?;
        let rules = match profile.and_then(|p| p.profile_data) {
            None => vec![],
            Some(data) => {
                data.get("alarms")
                    .and_then(|a| serde_json::from_value::<Vec<ProfileAlarmRule>>(a.clone()).ok())
                    .unwrap_or_default()
            }
        };
        self.rule_cache.insert(profile_id, rules.clone());
        Ok(Some(rules))
    }
}

#[async_trait]
impl RuleNode for DeviceProfileRuleNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Only process DEVICE originator
        if msg.originator_type.to_uppercase() != "DEVICE" {
            return Ok(vec![(RelationType::Success, msg)]);
        }

        let rules = match self.get_alarm_rules(ctx, msg.originator_id).await? {
            None => return Ok(vec![(RelationType::Failure, msg)]),
            Some(r) if r.is_empty() => return Ok(vec![(RelationType::Success, msg)]),
            Some(r) => r,
        };

        let data: serde_json::Value = serde_json::from_str(&msg.data)
            .unwrap_or(serde_json::json!({}));

        // Load active alarms for this device
        let page = PageLink::new(0, 100);
        let existing_alarms = ctx.dao.alarm
            .find_by_originator(ctx.tenant_id, msg.originator_id, &page)
            .await?;

        let active_alarms: HashMap<String, Alarm> = existing_alarms.data
            .into_iter()
            .filter(|a| !a.cleared)
            .map(|a| (a.alarm_type.clone(), a))
            .collect();

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let actions = evaluate_alarm_rules(&rules, &data, &active_alarms, now_ms);

        let mut out_msgs: Vec<(RelationType, TbMsg)> = Vec::new();

        for action in actions {
            match action {
                AlarmAction::Create { alarm_type, severity, propagate, propagate_to_owner, propagate_to_tenant, details } => {
                    let is_new = !active_alarms.contains_key(&alarm_type);

                    let alarm = if let Some(existing) = active_alarms.get(&alarm_type) {
                        // Update existing alarm severity
                        let mut updated = existing.clone();
                        updated.severity = severity;
                        updated.end_ts = now_ms;
                        updated.details = details;
                        updated
                    } else {
                        build_alarm(
                            ctx.tenant_id, msg.originator_id,
                            alarm_type.clone(), severity,
                            propagate, propagate_to_owner, propagate_to_tenant,
                            details, now_ms,
                        )
                    };

                    ctx.dao.alarm.save(&alarm).await?;

                    let relation = if is_new {
                        RelationType::Other("Alarm Created".into())
                    } else {
                        RelationType::Other("Alarm Updated".into())
                    };

                    let mut alarm_msg = msg.clone();
                    alarm_msg.metadata.insert("alarmType".into(), alarm_type);
                    alarm_msg.metadata.insert("alarmSeverity".into(), format!("{:?}", alarm.severity));
                    alarm_msg.metadata.insert("alarmId".into(), alarm.id.to_string());
                    alarm_msg.data = serde_json::to_string(&alarm).unwrap_or(alarm_msg.data);
                    out_msgs.push((relation, alarm_msg));
                }

                AlarmAction::Clear { alarm_type } => {
                    if let Some(existing) = active_alarms.get(&alarm_type) {
                        ctx.dao.alarm.clear(existing.id, now_ms).await?;

                        let mut alarm_msg = msg.clone();
                        alarm_msg.metadata.insert("alarmType".into(), alarm_type.clone());
                        alarm_msg.metadata.insert("alarmId".into(), existing.id.to_string());
                        out_msgs.push((RelationType::Other("Alarm Cleared".into()), alarm_msg));
                    }
                }

                AlarmAction::NoChange => {}
            }
        }

        if out_msgs.is_empty() {
            out_msgs.push((RelationType::Success, msg));
        }

        Ok(out_msgs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn constructs_with_empty_config() {
        let node = DeviceProfileRuleNode::new(&json!({})).unwrap();
        assert!(node.profile_id_cache.is_empty());
    }
}
