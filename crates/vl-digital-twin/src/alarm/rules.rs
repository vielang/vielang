//! Phase 35 — Client-side Alarm Rules Engine.
//!
//! Users define local threshold rules: "if temperature > 80 → MAJOR alarm".
//! Rules fire against every `TelemetryUpdate` event and inject results into
//! `AlarmRegistry` with `alarm_type = "CLIENT_RULE:<uuid>"`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AlarmRegistry;
use crate::components::{current_time_ms, AlarmSeverity};
use crate::events::TelemetryUpdate;

// ── Rule model ────────────────────────────────────────────────────────────────

/// Comparison operator for a rule threshold.
/// Uses struct variants for lossless TOML round-trip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum RuleCondition {
    GreaterThan    { value: f64 },
    LessThan       { value: f64 },
    GreaterOrEqual { value: f64 },
    LessOrEqual    { value: f64 },
    /// Fires when |reading − center| > margin.
    OutsideRange   { center: f64, margin: f64 },
}

impl RuleCondition {
    pub fn evaluate(&self, v: f64) -> bool {
        match self {
            Self::GreaterThan    { value }           => v > *value,
            Self::LessThan       { value }           => v < *value,
            Self::GreaterOrEqual { value }           => v >= *value,
            Self::LessOrEqual    { value }           => v <= *value,
            Self::OutsideRange   { center, margin }  => (v - center).abs() > *margin,
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::GreaterThan    { value }           => format!("> {}", value),
            Self::LessThan       { value }           => format!("< {}", value),
            Self::GreaterOrEqual { value }           => format!(">= {}", value),
            Self::LessOrEqual    { value }           => format!("<= {}", value),
            Self::OutsideRange   { center, margin }  => format!("outside {}±{}", center, margin),
        }
    }
}

/// A single client-side alarm rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmRule {
    pub id:        Uuid,
    pub name:      String,
    /// `None` = applies to all devices; `Some(id)` = scoped to one device.
    pub device_id: Option<Uuid>,
    /// Telemetry key to watch (e.g. `"temperature"`).
    pub key:       String,
    pub condition: RuleCondition,
    /// AlarmSeverity as uppercase string (e.g. `"MAJOR"`).
    pub severity:  String,
    pub message:   String,
    pub enabled:   bool,
}

impl AlarmRule {
    /// Unique alarm_type tag so rules show up distinctly in the alarm panel.
    pub fn alarm_type(&self) -> String {
        format!("CLIENT_RULE:{}", self.id)
    }
}

// ── Resource ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
struct RulesFile {
    #[serde(default)]
    rules: Vec<AlarmRule>,
}

/// Client-side rule set — persisted to `{data_dir}/vielang/alarm_rules.toml`.
#[derive(Resource, Default)]
pub struct AlarmRuleSet {
    pub rules: Vec<AlarmRule>,
}

impl AlarmRuleSet {
    /// Load rules from disk (native only; returns empty set on WASM).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Self {
        let path = rules_path();
        if !path.exists() {
            return Self::default();
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let file: RulesFile = toml::from_str(&content).unwrap_or_default();
        Self { rules: file.rules }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Self {
        Self::default()
    }

    /// Persist rules to disk (native only; no-op on WASM).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        let path = rules_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = RulesFile { rules: self.rules.clone() };
        if let Ok(content) = toml::to_string_pretty(&file) {
            let _ = std::fs::write(path, content);
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {}

    pub fn add(&mut self, rule: AlarmRule) {
        self.rules.push(rule);
    }

    pub fn remove(&mut self, rule_id: Uuid) {
        self.rules.retain(|r| r.id != rule_id);
    }

    pub fn toggle(&mut self, rule_id: Uuid) {
        if let Some(r) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            r.enabled = !r.enabled;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn rules_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vielang")
        .join("alarm_rules.toml")
}

// ── System ────────────────────────────────────────────────────────────────────

/// Evaluate all enabled alarm rules against incoming telemetry.
/// Runs after `drain_ws_events` so events are already in the queue.
pub fn evaluate_alarm_rules(
    mut events:   MessageReader<TelemetryUpdate>,
    rules:        Res<AlarmRuleSet>,
    mut registry: ResMut<AlarmRegistry>,
) {
    for ev in events.read() {
        for rule in rules.rules.iter().filter(|r| r.enabled && r.key == ev.key) {
            if let Some(target) = rule.device_id {
                if target != ev.device_id { continue; }
            }
            let fired      = rule.condition.evaluate(ev.value);
            let alarm_type = rule.alarm_type();
            let severity   = AlarmSeverity::from_str(&rule.severity);
            registry.upsert_from_ws(ev.device_id, &alarm_type, severity, fired, current_time_ms());
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_condition_greater_than() {
        assert!(RuleCondition::GreaterThan { value: 80.0 }.evaluate(90.0));
        assert!(!RuleCondition::GreaterThan { value: 80.0 }.evaluate(70.0));
        assert!(!RuleCondition::GreaterThan { value: 80.0 }.evaluate(80.0));
    }

    #[test]
    fn rule_condition_less_than() {
        assert!(RuleCondition::LessThan { value: 5.0 }.evaluate(3.0));
        assert!(!RuleCondition::LessThan { value: 5.0 }.evaluate(5.0));
    }

    #[test]
    fn rule_condition_greater_or_equal() {
        assert!(RuleCondition::GreaterOrEqual { value: 80.0 }.evaluate(80.0));
        assert!(RuleCondition::GreaterOrEqual { value: 80.0 }.evaluate(81.0));
        assert!(!RuleCondition::GreaterOrEqual { value: 80.0 }.evaluate(79.9));
    }

    #[test]
    fn rule_condition_outside_range() {
        let cond = RuleCondition::OutsideRange { center: 50.0, margin: 10.0 };
        assert!(cond.evaluate(70.0));  // |70-50| = 20 > 10
        assert!(!cond.evaluate(55.0)); // |55-50| = 5 < 10
        assert!(cond.evaluate(30.0));  // |30-50| = 20 > 10
    }

    #[test]
    fn alarm_rule_type_prefix() {
        let rule = AlarmRule {
            id:        Uuid::nil(),
            name:      "Test".into(),
            device_id: None,
            key:       "temperature".into(),
            condition: RuleCondition::GreaterThan { value: 80.0 },
            severity:  "MAJOR".into(),
            message:   "Too hot!".into(),
            enabled:   true,
        };
        assert!(rule.alarm_type().starts_with("CLIENT_RULE:"));
    }

    #[test]
    fn alarm_rule_set_add_remove() {
        let mut rset = AlarmRuleSet::default();
        let id = Uuid::new_v4();
        rset.add(AlarmRule {
            id,
            name:      "R1".into(),
            device_id: None,
            key:       "t".into(),
            condition: RuleCondition::GreaterThan { value: 80.0 },
            severity:  "MAJOR".into(),
            message:   "hot".into(),
            enabled:   true,
        });
        assert_eq!(rset.rules.len(), 1);
        rset.remove(id);
        assert!(rset.rules.is_empty());
    }

    #[test]
    fn alarm_rule_set_toggle() {
        let mut rset = AlarmRuleSet::default();
        let id = Uuid::new_v4();
        rset.add(AlarmRule {
            id,
            name: "R1".into(), device_id: None, key: "t".into(),
            condition: RuleCondition::GreaterThan { value: 1.0 },
            severity: "WARNING".into(), message: "".into(), enabled: true,
        });
        rset.toggle(id);
        assert!(!rset.rules[0].enabled);
        rset.toggle(id);
        assert!(rset.rules[0].enabled);
    }

    // ── Condition edge cases ──────────────────────────────────────────────────

    #[test]
    fn rule_condition_less_or_equal_boundary_inclusive() {
        let cond = RuleCondition::LessOrEqual { value: 5.0 };
        assert!(cond.evaluate(5.0));   // inclusive boundary
        assert!(cond.evaluate(4.99));
        assert!(!cond.evaluate(5.01));
    }

    #[test]
    fn rule_condition_greater_or_equal_boundary_inclusive() {
        let cond = RuleCondition::GreaterOrEqual { value: 10.0 };
        assert!(cond.evaluate(10.0));  // inclusive boundary
        assert!(!cond.evaluate(9.99));
    }

    #[test]
    fn rule_condition_outside_range_symmetric() {
        let cond = RuleCondition::OutsideRange { center: 0.0, margin: 5.0 };
        assert!(!cond.evaluate(0.0));   // center = not outside
        assert!(!cond.evaluate(4.9));   // inside margin
        assert!(cond.evaluate(5.01));   // just outside
        assert!(cond.evaluate(-5.01));  // symmetric negative
    }

    #[test]
    fn rule_condition_description_format() {
        assert!(RuleCondition::GreaterThan    { value: 80.0 }                   .description().contains("80"));
        assert!(RuleCondition::LessThan       { value: 5.0 }                    .description().contains("5"));
        assert!(RuleCondition::OutsideRange   { center: 50.0, margin: 10.0 }    .description().contains("50"));
    }

    // ── TOML serialization roundtrip ─────────────────────────────────────────

    #[test]
    fn alarm_rule_condition_toml_roundtrip_greater_than() {
        // Use a local wrapper because RulesFile is private
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Wrap { rules: Vec<AlarmRule> }

        let rule = AlarmRule {
            id:        Uuid::nil(),
            name:      "HotAlert".into(),
            device_id: None,
            key:       "temperature".into(),
            condition: RuleCondition::GreaterThan { value: 80.0 },
            severity:  "MAJOR".into(),
            message:   "Overheating".into(),
            enabled:   true,
        };
        let wrap = Wrap { rules: vec![rule] };
        let toml_str  = toml::to_string_pretty(&wrap).expect("serialize");
        let recovered: Wrap = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(recovered.rules.len(), 1);
        assert_eq!(recovered.rules[0].name, "HotAlert");
        assert_eq!(
            recovered.rules[0].condition,
            RuleCondition::GreaterThan { value: 80.0 }
        );
    }

    #[test]
    fn alarm_rule_condition_toml_roundtrip_outside_range() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Wrap { rules: Vec<AlarmRule> }

        let rule = AlarmRule {
            id:        Uuid::new_v4(),
            name:      "RangeCheck".into(),
            device_id: Some(Uuid::nil()),
            key:       "pressure".into(),
            condition: RuleCondition::OutsideRange { center: 101.3, margin: 5.0 },
            severity:  "WARNING".into(),
            message:   "Pressure deviation".into(),
            enabled:   false,
        };
        let toml_str = toml::to_string_pretty(&Wrap { rules: vec![rule.clone()] }).expect("serialize");
        let recovered: Wrap = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(
            recovered.rules[0].condition,
            RuleCondition::OutsideRange { center: 101.3, margin: 5.0 }
        );
        assert!(!recovered.rules[0].enabled);
        assert_eq!(recovered.rules[0].device_id, Some(Uuid::nil()));
    }

    // ── Device-scoped rules ───────────────────────────────────────────────────

    #[test]
    fn alarm_rule_global_scope_has_none_device_id() {
        let rule = AlarmRule {
            id: Uuid::new_v4(), name: "Global".into(),
            device_id: None,   // applies to all devices
            key: "t".into(),
            condition: RuleCondition::GreaterThan { value: 50.0 },
            severity: "WARNING".into(), message: "".into(), enabled: true,
        };
        assert!(rule.device_id.is_none());
    }

    #[test]
    fn alarm_rule_scoped_to_specific_device() {
        let target_id = Uuid::new_v4();
        let rule = AlarmRule {
            id: Uuid::new_v4(), name: "Scoped".into(),
            device_id: Some(target_id),
            key: "t".into(),
            condition: RuleCondition::GreaterThan { value: 50.0 },
            severity: "MAJOR".into(), message: "".into(), enabled: true,
        };
        assert_eq!(rule.device_id, Some(target_id));
    }

    // ── Multiple rules in set ─────────────────────────────────────────────────

    #[test]
    fn alarm_rule_set_multiple_rules_independent() {
        let mut rset = AlarmRuleSet::default();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        rset.add(AlarmRule { id: id1, name: "R1".into(), device_id: None, key: "temp".into(),
            condition: RuleCondition::GreaterThan { value: 80.0 },
            severity: "MAJOR".into(), message: "".into(), enabled: true });
        rset.add(AlarmRule { id: id2, name: "R2".into(), device_id: None, key: "humidity".into(),
            condition: RuleCondition::LessThan { value: 10.0 },
            severity: "WARNING".into(), message: "".into(), enabled: true });
        assert_eq!(rset.rules.len(), 2);
        // Remove only the first
        rset.remove(id1);
        assert_eq!(rset.rules.len(), 1);
        assert_eq!(rset.rules[0].key, "humidity");
    }

    #[test]
    fn alarm_rule_set_remove_nonexistent_is_noop() {
        let mut rset = AlarmRuleSet::default();
        let id = Uuid::new_v4();
        rset.add(AlarmRule { id, name: "R1".into(), device_id: None, key: "t".into(),
            condition: RuleCondition::GreaterThan { value: 1.0 },
            severity: "WARNING".into(), message: "".into(), enabled: true });
        rset.remove(Uuid::new_v4()); // different UUID
        assert_eq!(rset.rules.len(), 1, "removing nonexistent rule should be a no-op");
    }
}
