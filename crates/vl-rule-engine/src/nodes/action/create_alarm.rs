use async_trait::async_trait;
use uuid::Uuid;
use vl_core::entities::{Alarm, AlarmSeverity, EntityType, TbMsg, msg_type};
use crate::{
    error::RuleEngineError,
    node::{RelationType, RuleNode, RuleNodeCtx},
};

/// Create or update an alarm.
/// Config:
/// ```json
/// {
///   "alarmType": "HighTemperature",
///   "alarmSeverity": "CRITICAL",
///   "clearRule": false
/// }
/// ```
pub struct CreateAlarmNode {
    alarm_type: String,
    severity:   AlarmSeverity,
}

impl CreateAlarmNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let alarm_type = config["alarmType"]
            .as_str()
            .ok_or_else(|| RuleEngineError::Config("alarmType required".into()))?
            .to_string();
        let severity = parse_severity(config["alarmSeverity"].as_str().unwrap_or("INDETERMINATE"));
        Ok(Self { alarm_type, severity })
    }
}

#[async_trait]
impl RuleNode for CreateAlarmNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let now = chrono::Utc::now().timestamp_millis();
        let alarm = Alarm {
            id:                       Uuid::new_v4(),
            created_time:             now,
            tenant_id:                ctx.tenant_id,
            customer_id:              None,
            alarm_type:               self.alarm_type.clone(),
            originator_id:            msg.originator_id,
            originator_type:          parse_entity_type(&msg.originator_type),
            severity:                 self.severity.clone(),
            acknowledged:             false,
            cleared:                  false,
            assignee_id:              None,
            start_ts:                 now,
            end_ts:                   now,
            ack_ts:                   None,
            clear_ts:                 None,
            assign_ts:                0,
            propagate:                false,
            propagate_to_owner:       false,
            propagate_to_tenant:      false,
            propagate_relation_types: None,
            details:                  None,
        };

        ctx.dao.alarm.save(&alarm).await?;

        let mut out = msg;
        out.msg_type = msg_type::ALARM.to_string();
        out.metadata.insert("alarmType".into(), self.alarm_type.clone());
        out.metadata.insert("alarmSeverity".into(), severity_str(&self.severity).to_string());
        out.metadata.insert("alarmStatus".into(), "ACTIVE_UNACK".into());

        Ok(vec![(RelationType::Success, out)])
    }
}

fn parse_severity(s: &str) -> AlarmSeverity {
    match s.to_uppercase().as_str() {
        "CRITICAL"      => AlarmSeverity::Critical,
        "MAJOR"         => AlarmSeverity::Major,
        "MINOR"         => AlarmSeverity::Minor,
        "WARNING"       => AlarmSeverity::Warning,
        _               => AlarmSeverity::Indeterminate,
    }
}

fn severity_str(s: &AlarmSeverity) -> &'static str {
    match s {
        AlarmSeverity::Critical      => "CRITICAL",
        AlarmSeverity::Major         => "MAJOR",
        AlarmSeverity::Minor         => "MINOR",
        AlarmSeverity::Warning       => "WARNING",
        AlarmSeverity::Indeterminate => "INDETERMINATE",
    }
}

fn parse_entity_type(s: &str) -> EntityType {
    match s.to_uppercase().as_str() {
        "TENANT"   => EntityType::Tenant,
        "CUSTOMER" => EntityType::Customer,
        "USER"     => EntityType::User,
        "ASSET"    => EntityType::Asset,
        _          => EntityType::Device,
    }
}
