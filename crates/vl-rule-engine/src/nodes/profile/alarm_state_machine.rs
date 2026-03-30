use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use vl_core::entities::{Alarm, AlarmSeverity, EntityType};
use super::condition_evaluator::{evaluate_condition, AlarmConditionSpec};

/// Alarm rule definition — parsed from DeviceProfile.profile_data.alarms[]
/// Java: DeviceProfileAlarm
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileAlarmRule {
    pub id: Option<Uuid>,
    #[serde(rename = "alarmType")]
    pub alarm_type: String,
    /// Map of severity → condition spec to CREATE this alarm
    #[serde(rename = "createRules", default)]
    pub create_rules: HashMap<String, AlarmCreateSpec>,
    /// Condition to CLEAR the alarm
    #[serde(rename = "clearRule")]
    pub clear_rule: Option<AlarmClearSpec>,
    #[serde(default)]
    pub propagate: bool,
    #[serde(rename = "propagateToOwner", default)]
    pub propagate_to_owner: bool,
    #[serde(rename = "propagateToTenant", default)]
    pub propagate_to_tenant: bool,
    #[serde(rename = "propagateRelationTypes")]
    pub propagate_relation_types: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmCreateSpec {
    pub condition: AlarmRuleCondition,
    #[serde(rename = "alarmDetails")]
    pub alarm_details: Option<String>,
    #[serde(rename = "dashboardId")]
    pub dashboard_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmClearSpec {
    pub condition: AlarmRuleCondition,
    #[serde(rename = "alarmDetails")]
    pub alarm_details: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmRuleCondition {
    pub spec: AlarmConditionSpec,
}

/// Severity ordering: CRITICAL > MAJOR > MINOR > WARNING > INDETERMINATE
fn severity_rank(s: &str) -> u8 {
    match s.to_uppercase().as_str() {
        "CRITICAL"      => 5,
        "MAJOR"         => 4,
        "MINOR"         => 3,
        "WARNING"       => 2,
        "INDETERMINATE" => 1,
        _               => 0,
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

/// Result of evaluating a single alarm rule against incoming telemetry
#[derive(Debug)]
pub enum AlarmAction {
    /// Create a new alarm (or escalate if severity increased)
    Create {
        alarm_type: String,
        severity:   AlarmSeverity,
        propagate:  bool,
        propagate_to_owner: bool,
        propagate_to_tenant: bool,
        details:    Option<serde_json::Value>,
    },
    /// Clear an active alarm
    Clear {
        alarm_type: String,
    },
    /// No change needed
    NoChange,
}

/// Evaluate all alarm rules in a device profile against incoming telemetry.
/// Returns a list of actions to execute.
/// Java: TbDeviceProfileNode.processAlarmRules()
pub fn evaluate_alarm_rules(
    rules:           &[ProfileAlarmRule],
    data:            &serde_json::Value,
    active_alarms:   &HashMap<String, Alarm>, // alarm_type → active alarm
    now_ms:          i64,
) -> Vec<AlarmAction> {
    let _ = now_ms; // reserved for duration conditions
    let mut actions = Vec::new();

    for rule in rules {
        let active = active_alarms.get(&rule.alarm_type);

        // Evaluate create rules — find highest matching severity
        let mut best_severity: Option<(u8, &str, &AlarmCreateSpec)> = None;
        for (severity_str, create_spec) in &rule.create_rules {
            if evaluate_condition(&create_spec.condition.spec, data) {
                let rank = severity_rank(severity_str);
                if best_severity.map_or(true, |(r, _, _)| rank > r) {
                    best_severity = Some((rank, severity_str.as_str(), create_spec));
                }
            }
        }

        if let Some((_, severity_str, spec)) = best_severity {
            let sev = parse_severity(severity_str);

            // Check if we need to create or escalate
            let needs_create = match active {
                None => true, // no active alarm → create
                Some(existing) => {
                    // Escalate if new severity is higher
                    let existing_rank = match existing.severity {
                        AlarmSeverity::Critical      => 5u8,
                        AlarmSeverity::Major         => 4,
                        AlarmSeverity::Minor         => 3,
                        AlarmSeverity::Warning       => 2,
                        AlarmSeverity::Indeterminate => 1,
                    };
                    severity_rank(severity_str) > existing_rank
                }
            };

            if needs_create {
                // Render alarm details template (simple ${key} substitution)
                let details = spec.alarm_details.as_ref().map(|tmpl| {
                    render_details_template(tmpl, data)
                });

                actions.push(AlarmAction::Create {
                    alarm_type: rule.alarm_type.clone(),
                    severity: sev,
                    propagate: rule.propagate,
                    propagate_to_owner: rule.propagate_to_owner,
                    propagate_to_tenant: rule.propagate_to_tenant,
                    details,
                });
            }
            // condition matched → skip clear check
            continue;
        }

        // No create condition matched — check clear rule
        if active.is_some() {
            if let Some(clear_rule) = &rule.clear_rule {
                if evaluate_condition(&clear_rule.condition.spec, data) {
                    actions.push(AlarmAction::Clear {
                        alarm_type: rule.alarm_type.clone(),
                    });
                    continue;
                }
            }
        }

        actions.push(AlarmAction::NoChange);
    }

    actions
}

/// Simple ${key} template substitution from data JSON
fn render_details_template(tmpl: &str, data: &serde_json::Value) -> serde_json::Value {
    let mut result = tmpl.to_string();
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            let placeholder = format!("${{{}}}", k);
            let val_str = match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            result = result.replace(&placeholder, &val_str);
        }
    }
    serde_json::json!({ "description": result })
}

/// Build an Alarm struct for saving to DB
pub fn build_alarm(
    tenant_id:       Uuid,
    device_id:       Uuid,
    alarm_type:      String,
    severity:        AlarmSeverity,
    propagate:       bool,
    propagate_to_owner: bool,
    propagate_to_tenant: bool,
    details:         Option<serde_json::Value>,
    now_ms:          i64,
) -> Alarm {
    Alarm {
        id: Uuid::new_v4(),
        created_time: now_ms,
        tenant_id,
        customer_id: None,
        alarm_type,
        originator_id: device_id,
        originator_type: EntityType::Device,
        severity,
        acknowledged: false,
        cleared: false,
        assignee_id: None,
        start_ts: now_ms,
        end_ts: now_ms,
        ack_ts: None,
        clear_ts: None,
        assign_ts: 0,
        details,
        propagate,
        propagate_to_owner,
        propagate_to_tenant,
        propagate_relation_types: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::profile::condition_evaluator::{
        AlarmConditionKey, KeyFilterPredicate, NumericOperation, PredicateValue,
        ConditionType,
    };
    use serde_json::json;

    fn make_rule(alarm_type: &str, threshold: f64, clear_threshold: f64) -> ProfileAlarmRule {
        let create_spec = AlarmConditionSpec {
            condition_type: ConditionType::Simple,
            key: Some(AlarmConditionKey { key_type: "TIME_SERIES".into(), key: "temperature".into() }),
            predicate: Some(KeyFilterPredicate::Numeric {
                operation: NumericOperation::Greater,
                value: PredicateValue { default_value: json!(threshold), dynamic_value: None },
            }),
            duration_time_unit: None,
            duration_value: None,
        };
        let clear_spec = AlarmConditionSpec {
            condition_type: ConditionType::Simple,
            key: Some(AlarmConditionKey { key_type: "TIME_SERIES".into(), key: "temperature".into() }),
            predicate: Some(KeyFilterPredicate::Numeric {
                operation: NumericOperation::LessOrEqual,
                value: PredicateValue { default_value: json!(clear_threshold), dynamic_value: None },
            }),
            duration_time_unit: None,
            duration_value: None,
        };

        let mut create_rules = HashMap::new();
        create_rules.insert("CRITICAL".to_string(), AlarmCreateSpec {
            condition: AlarmRuleCondition { spec: create_spec },
            alarm_details: Some("Temperature is ${temperature}°C".into()),
            dashboard_id: None,
        });

        ProfileAlarmRule {
            id: None,
            alarm_type: alarm_type.to_string(),
            create_rules,
            clear_rule: Some(AlarmClearSpec {
                condition: AlarmRuleCondition { spec: clear_spec },
                alarm_details: None,
            }),
            propagate: false,
            propagate_to_owner: false,
            propagate_to_tenant: false,
            propagate_relation_types: None,
        }
    }

    #[test]
    fn creates_alarm_when_threshold_exceeded() {
        let rules = vec![make_rule("High Temp", 50.0, 40.0)];
        let data = json!({ "temperature": 55.0 });
        let active: HashMap<String, Alarm> = HashMap::new();
        let actions = evaluate_alarm_rules(&rules, &data, &active, 0);
        assert!(matches!(actions[0], AlarmAction::Create { .. }));
    }

    #[test]
    fn no_action_when_below_threshold() {
        let rules = vec![make_rule("High Temp", 50.0, 40.0)];
        let data = json!({ "temperature": 30.0 });
        let active: HashMap<String, Alarm> = HashMap::new();
        let actions = evaluate_alarm_rules(&rules, &data, &active, 0);
        assert!(matches!(actions[0], AlarmAction::NoChange));
    }

    #[test]
    fn template_substitution() {
        let result = render_details_template("Temp: ${temperature}°C", &json!({ "temperature": 55 }));
        assert_eq!(result["description"].as_str().unwrap(), "Temp: 55°C");
    }

    #[test]
    fn severity_ordering() {
        assert!(severity_rank("CRITICAL") > severity_rank("MAJOR"));
        assert!(severity_rank("MAJOR") > severity_rank("MINOR"));
        assert!(severity_rank("MINOR") > severity_rank("WARNING"));
    }
}
