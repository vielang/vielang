//! Alarm management — AlarmRegistry, lifecycle (ack/clear), action queue.

pub mod rules;
pub use rules::{evaluate_alarm_rules, AlarmRule, AlarmRuleSet, RuleCondition};

use std::collections::HashMap;

use bevy::prelude::*;
use uuid::Uuid;

use crate::components::{current_time_ms, AlarmSeverity};
use crate::events::AlarmUpdate;

// ── Data model ────────────────────────────────────────────────────────────────

/// Lifecycle status of an alarm (mirrors ThingsBoard `AlarmStatus`).
#[derive(Debug, Clone, PartialEq)]
pub enum AlarmStatus {
    ActiveUnack,
    ActiveAck,
    ClearedUnack,
    ClearedAck,
}

impl AlarmStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "ACTIVE_ACK"    => Self::ActiveAck,
            "CLEARED_UNACK" => Self::ClearedUnack,
            "CLEARED_ACK"   => Self::ClearedAck,
            _               => Self::ActiveUnack,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::ActiveUnack | Self::ActiveAck)
    }
}

impl std::fmt::Display for AlarmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ActiveUnack  => write!(f, "ACTIVE"),
            Self::ActiveAck    => write!(f, "ACK'd"),
            Self::ClearedUnack => write!(f, "CLEARED"),
            Self::ClearedAck   => write!(f, "DONE"),
        }
    }
}

/// A single alarm record — may come from WS push or REST fetch.
#[derive(Debug, Clone)]
pub struct AlarmRecord {
    /// REST alarm UUID (Some when fetched from backend, None for WS-only).
    pub id:           Option<Uuid>,
    pub device_id:    Uuid,
    pub alarm_type:   String,
    pub severity:     AlarmSeverity,
    pub status:       AlarmStatus,
    pub created_ts:   i64,
    pub acknowledged: bool,
    pub cleared:      bool,
}

// ── Resource ──────────────────────────────────────────────────────────────────

/// Central alarm store — updated from WS events and REST fetches.
#[derive(Resource, Default)]
pub struct AlarmRegistry {
    pub alarms: Vec<AlarmRecord>,
}

impl AlarmRegistry {
    /// Create or update an alarm record from a WS AlarmUpdate event.
    pub fn upsert_from_ws(
        &mut self,
        device_id:  Uuid,
        alarm_type: &str,
        severity:   AlarmSeverity,
        active:     bool,
        ts:         i64,
    ) {
        if let Some(alarm) = self.alarms.iter_mut()
            .find(|a| a.device_id == device_id && a.alarm_type == alarm_type)
        {
            alarm.severity = severity;
            alarm.cleared  = !active;
            // Don't downgrade from ACK'd to UNACK when we just get a refresh
            if active && alarm.status == AlarmStatus::ClearedAck {
                alarm.status = AlarmStatus::ActiveAck;
            } else if active && alarm.status == AlarmStatus::ClearedUnack {
                alarm.status = AlarmStatus::ActiveUnack;
            } else if !active && alarm.status == AlarmStatus::ActiveUnack {
                alarm.status = AlarmStatus::ClearedUnack;
            } else if !active && alarm.status == AlarmStatus::ActiveAck {
                alarm.status = AlarmStatus::ClearedAck;
            }
        } else {
            self.alarms.push(AlarmRecord {
                id:           None,
                device_id,
                alarm_type:   alarm_type.to_string(),
                severity,
                status:       if active { AlarmStatus::ActiveUnack } else { AlarmStatus::ClearedUnack },
                created_ts:   ts,
                acknowledged: false,
                cleared:      !active,
            });
        }

        // Cap at 200 entries (FIFO eviction of oldest)
        if self.alarms.len() > 200 {
            self.alarms.remove(0);
        }
    }

    /// Optimistically acknowledge an alarm (no REST call — use process_alarm_actions for REST).
    pub fn acknowledge(&mut self, device_id: Uuid, alarm_type: &str) {
        for alarm in &mut self.alarms {
            if alarm.device_id == device_id && alarm.alarm_type == alarm_type {
                alarm.acknowledged = true;
                alarm.status = if alarm.cleared {
                    AlarmStatus::ClearedAck
                } else {
                    AlarmStatus::ActiveAck
                };
            }
        }
    }

    /// Optimistically clear an alarm.
    pub fn clear_alarm(&mut self, device_id: Uuid, alarm_type: &str) {
        for alarm in &mut self.alarms {
            if alarm.device_id == device_id && alarm.alarm_type == alarm_type {
                alarm.cleared = true;
                alarm.status  = if alarm.acknowledged {
                    AlarmStatus::ClearedAck
                } else {
                    AlarmStatus::ClearedUnack
                };
            }
        }
    }

    /// Active alarm count by severity string (for badge display).
    pub fn active_count_by_severity(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for alarm in &self.alarms {
            if alarm.status.is_active() {
                *counts.entry(alarm.severity.to_string()).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Total number of active (unresolved) alarms.
    pub fn active_count(&self) -> usize {
        self.alarms.iter().filter(|a| a.status.is_active()).count()
    }
}

// ── System ────────────────────────────────────────────────────────────────────

/// Populate AlarmRegistry from incoming AlarmUpdate WS events.
pub fn update_alarm_registry(
    mut events:   MessageReader<AlarmUpdate>,
    mut registry: ResMut<AlarmRegistry>,
) {
    for ev in events.read() {
        let severity = AlarmSeverity::from_str(&ev.severity);
        let ts       = current_time_ms();
        registry.upsert_from_ws(ev.device_id, &ev.alarm_type, severity, ev.active, ts);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alarm_status_from_str() {
        assert_eq!(AlarmStatus::from_str("ACTIVE_ACK"),    AlarmStatus::ActiveAck);
        assert_eq!(AlarmStatus::from_str("CLEARED_UNACK"), AlarmStatus::ClearedUnack);
        assert_eq!(AlarmStatus::from_str("CLEARED_ACK"),   AlarmStatus::ClearedAck);
        assert_eq!(AlarmStatus::from_str("UNKNOWN"),        AlarmStatus::ActiveUnack);
    }

    #[test]
    fn alarm_registry_upsert_new() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        reg.upsert_from_ws(id, "HIGH_TEMP", AlarmSeverity::Critical, true, 0);
        assert_eq!(reg.alarms.len(), 1);
        assert_eq!(reg.alarms[0].status, AlarmStatus::ActiveUnack);
        assert!(!reg.alarms[0].acknowledged);
    }

    #[test]
    fn alarm_registry_upsert_update_existing() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        reg.upsert_from_ws(id, "HIGH_TEMP", AlarmSeverity::Critical, true, 0);
        // Update — alarm cleared
        reg.upsert_from_ws(id, "HIGH_TEMP", AlarmSeverity::Critical, false, 1000);
        assert_eq!(reg.alarms.len(), 1);
        assert_eq!(reg.alarms[0].status, AlarmStatus::ClearedUnack);
    }

    #[test]
    fn alarm_registry_acknowledge() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        reg.upsert_from_ws(id, "HIGH_TEMP", AlarmSeverity::Major, true, 0);
        reg.acknowledge(id, "HIGH_TEMP");
        assert_eq!(reg.alarms[0].status, AlarmStatus::ActiveAck);
        assert!(reg.alarms[0].acknowledged);
    }

    #[test]
    fn alarm_registry_clear() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        reg.upsert_from_ws(id, "HIGH_TEMP", AlarmSeverity::Warning, true, 0);
        reg.clear_alarm(id, "HIGH_TEMP");
        assert_eq!(reg.alarms[0].status, AlarmStatus::ClearedUnack);
        assert!(reg.alarms[0].cleared);
    }

    #[test]
    fn alarm_registry_active_count() {
        let mut reg = AlarmRegistry::default();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        reg.upsert_from_ws(id1, "A", AlarmSeverity::Critical, true, 0);
        reg.upsert_from_ws(id2, "B", AlarmSeverity::Warning, true, 0);
        reg.upsert_from_ws(id1, "C", AlarmSeverity::Minor, false, 0); // cleared
        assert_eq!(reg.active_count(), 2);
    }

    #[test]
    fn alarm_registry_capped_at_200() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        for i in 0..210u32 {
            reg.upsert_from_ws(id, &format!("ALARM_{i}"), AlarmSeverity::Warning, true, i as i64);
        }
        assert_eq!(reg.alarms.len(), 200);
    }

    // ── AlarmStatus tests ─────────────────────────────────────────────────────

    #[test]
    fn alarm_status_is_active_variants() {
        assert!(AlarmStatus::ActiveUnack.is_active());
        assert!(AlarmStatus::ActiveAck.is_active());
        assert!(!AlarmStatus::ClearedUnack.is_active());
        assert!(!AlarmStatus::ClearedAck.is_active());
    }

    #[test]
    fn alarm_status_display() {
        assert_eq!(AlarmStatus::ActiveUnack.to_string(),  "ACTIVE");
        assert_eq!(AlarmStatus::ActiveAck.to_string(),    "ACK'd");
        assert_eq!(AlarmStatus::ClearedUnack.to_string(), "CLEARED");
        assert_eq!(AlarmStatus::ClearedAck.to_string(),   "DONE");
    }

    // ── State machine transitions ─────────────────────────────────────────────

    #[test]
    fn alarm_ack_then_clear_is_cleared_ack() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        reg.upsert_from_ws(id, "TEMP", AlarmSeverity::Major, true, 0);
        // ActiveUnack → ack → ActiveAck
        reg.acknowledge(id, "TEMP");
        assert_eq!(reg.alarms[0].status, AlarmStatus::ActiveAck);
        // ActiveAck → clear → ClearedAck
        reg.clear_alarm(id, "TEMP");
        assert_eq!(reg.alarms[0].status, AlarmStatus::ClearedAck);
        assert!(reg.alarms[0].cleared);
        assert!(reg.alarms[0].acknowledged);
    }

    #[test]
    fn alarm_clear_then_ack_is_cleared_ack() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        reg.upsert_from_ws(id, "TEMP", AlarmSeverity::Major, true, 0);
        // ActiveUnack → clear → ClearedUnack
        reg.clear_alarm(id, "TEMP");
        assert_eq!(reg.alarms[0].status, AlarmStatus::ClearedUnack);
        // ClearedUnack → ack → ClearedAck
        reg.acknowledge(id, "TEMP");
        assert_eq!(reg.alarms[0].status, AlarmStatus::ClearedAck);
    }

    #[test]
    fn alarm_re_active_after_cleared_ack() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        reg.upsert_from_ws(id, "TEMP", AlarmSeverity::Major, true, 0);
        reg.acknowledge(id, "TEMP");
        reg.clear_alarm(id, "TEMP");
        assert_eq!(reg.alarms[0].status, AlarmStatus::ClearedAck);
        // WS fires active again → transitions back to ActiveAck (already acked)
        reg.upsert_from_ws(id, "TEMP", AlarmSeverity::Major, true, 1000);
        assert_eq!(reg.alarms[0].status, AlarmStatus::ActiveAck);
    }

    // ── active_count_by_severity ──────────────────────────────────────────────

    #[test]
    fn alarm_registry_active_count_by_severity_mixed() {
        let mut reg = AlarmRegistry::default();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        reg.upsert_from_ws(id1, "A", AlarmSeverity::Critical, true,  0);
        reg.upsert_from_ws(id2, "B", AlarmSeverity::Critical, true,  0);
        reg.upsert_from_ws(id3, "C", AlarmSeverity::Warning,  true,  0);
        reg.upsert_from_ws(id1, "D", AlarmSeverity::Major,    false, 0); // cleared → not active
        let counts = reg.active_count_by_severity();
        assert_eq!(counts.get("Critical"), Some(&2));
        assert_eq!(counts.get("Warning"),  Some(&1));
        assert_eq!(counts.get("Major"),    None,    "cleared alarm must not appear");
    }

    #[test]
    fn alarm_registry_active_count_excludes_cleared() {
        let mut reg = AlarmRegistry::default();
        let id = Uuid::new_v4();
        reg.upsert_from_ws(id, "X", AlarmSeverity::Major, true,  0);
        reg.upsert_from_ws(id, "X", AlarmSeverity::Major, false, 1); // clear
        assert_eq!(reg.active_count(), 0);
    }

    // ── Multiple devices, same alarm_type ─────────────────────────────────────

    #[test]
    fn alarm_registry_separate_records_per_device() {
        let mut reg = AlarmRegistry::default();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        reg.upsert_from_ws(id1, "HIGH_TEMP", AlarmSeverity::Major,    true, 0);
        reg.upsert_from_ws(id2, "HIGH_TEMP", AlarmSeverity::Critical, true, 0);
        assert_eq!(reg.alarms.len(), 2);
        assert_ne!(reg.alarms[0].severity, reg.alarms[1].severity);
    }

    // ── Severity display ──────────────────────────────────────────────────────

    #[test]
    fn alarm_severity_none_and_indeterminate_share_color() {
        use crate::components::AlarmSeverity;
        // Both map to green — verify they don't panic
        let _ = AlarmSeverity::None.to_linear_color();
        let _ = AlarmSeverity::Indeterminate.to_linear_color();
    }
}
