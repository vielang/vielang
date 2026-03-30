//! Automation actions — what happens when a workflow triggers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// An action to execute when a workflow condition is met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Send an RPC command to a device.
    SendRpc {
        /// Target device ID. If None, applies to the triggering device.
        device_id: Option<Uuid>,
        method: String,
        params: serde_json::Value,
        is_twoway: bool,
    },

    /// Raise or update an alarm.
    SetAlarm {
        alarm_type: String,
        severity: String,
        message: String,
    },

    /// Clear an alarm by type.
    ClearAlarm {
        alarm_type: String,
    },

    /// Push a notification (toast).
    Notification {
        severity: String,
        message: String,
    },

    /// Update a twin property.
    UpdateProperty {
        twin_id: Option<Uuid>,
        property_name: String,
        value: serde_json::Value,
    },

    /// Log a message to the audit log.
    AuditLog {
        action: String,
        details: String,
    },

    /// Execute a webhook (HTTP POST to external URL).
    Webhook {
        url: String,
        headers: HashMap<String, String>,
        body: serde_json::Value,
    },

    /// Chain: trigger another workflow by ID.
    TriggerWorkflow {
        workflow_id: Uuid,
    },

    /// Delay before next action in the chain (ms).
    Delay {
        duration_ms: u64,
    },

    /// Set a device operating mode.
    SetOperatingMode {
        device_id: Option<Uuid>,
        mode: String,
    },
}

/// Result of executing an action.
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub action_type: String,
    pub success: bool,
    pub message: String,
    pub timestamp_ms: u64,
}

impl ActionResult {
    pub fn success(action_type: &str, message: &str) -> Self {
        Self {
            action_type: action_type.into(),
            success: true,
            message: message.into(),
            timestamp_ms: crate::components::device::current_time_ms() as u64,
        }
    }

    pub fn failure(action_type: &str, message: &str) -> Self {
        Self {
            action_type: action_type.into(),
            success: false,
            message: message.into(),
            timestamp_ms: crate::components::device::current_time_ms() as u64,
        }
    }
}

/// An execution log entry for workflow audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub device_id: Uuid,
    pub timestamp_ms: u64,
    pub actions_executed: usize,
    pub actions_succeeded: usize,
    pub actions_failed: usize,
}

/// Execution history — Bevy resource for tracking workflow runs.
#[derive(bevy::prelude::Resource, Default)]
pub struct WorkflowHistory {
    pub executions: std::collections::VecDeque<WorkflowExecution>,
    pub max_entries: usize,
}

impl WorkflowHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            executions: std::collections::VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }

    pub fn record(&mut self, execution: WorkflowExecution) {
        if self.executions.len() >= self.max_entries {
            self.executions.pop_front();
        }
        self.executions.push_back(execution);
    }

    pub fn recent(&self, n: usize) -> Vec<&WorkflowExecution> {
        self.executions.iter().rev().take(n).collect()
    }

    pub fn by_workflow(&self, workflow_id: Uuid) -> Vec<&WorkflowExecution> {
        self.executions.iter()
            .filter(|e| e.workflow_id == workflow_id)
            .collect()
    }
}

// ── Pre-built workflow templates ─────────────────────────────────────────────

/// Create an emergency stop workflow: when vibration exceeds limit, stop the machine.
pub fn emergency_stop_workflow(vibration_limit: f64) -> super::Workflow {
    super::Workflow::new(
        "Emergency Stop on High Vibration",
        super::Condition::ThresholdExceeded {
            device_type: None,
            key: "vibration".into(),
            operator: super::ComparisonOp::GreaterThan,
            value: vibration_limit,
        },
        vec![
            Action::SendRpc {
                device_id: None,
                method: "emergencyStop".into(),
                params: serde_json::json!({}),
                is_twoway: false,
            },
            Action::SetAlarm {
                alarm_type: "HIGH_VIBRATION_ESTOP".into(),
                severity: "CRITICAL".into(),
                message: format!("Emergency stop triggered: vibration exceeded {vibration_limit} mm/s"),
            },
            Action::Notification {
                severity: "CRITICAL".into(),
                message: "Emergency stop activated due to high vibration!".into(),
            },
            Action::AuditLog {
                action: "EMERGENCY_STOP".into(),
                details: "Automated emergency stop due to vibration threshold".into(),
            },
        ],
    ).with_cooldown(300_000).with_priority(1) // 5 min cooldown, highest priority
}

/// Create a predictive maintenance workflow: when bearing temp trends high, alert.
pub fn predictive_maintenance_workflow(temp_threshold: f64) -> super::Workflow {
    super::Workflow::new(
        "Predictive Maintenance: Bearing Temperature",
        super::Condition::All(vec![
            super::Condition::ThresholdExceeded {
                device_type: None,
                key: "bearing_temp".into(),
                operator: super::ComparisonOp::GreaterThan,
                value: temp_threshold,
            },
            super::Condition::ChangePointDetected {
                key: "bearing_temp".into(),
                direction: Some("Upward".into()),
            },
        ]),
        vec![
            Action::SetAlarm {
                alarm_type: "BEARING_DEGRADATION".into(),
                severity: "MAJOR".into(),
                message: format!("Bearing temperature exceeds {temp_threshold}°C with upward trend"),
            },
            Action::Notification {
                severity: "MAJOR".into(),
                message: "Schedule bearing inspection — predictive maintenance alert".into(),
            },
        ],
    ).with_cooldown(3_600_000).with_priority(10) // 1 hour cooldown
}

/// Create a data staleness workflow: alert when device stops reporting.
pub fn data_staleness_workflow(key: &str, stale_duration_ms: u64) -> super::Workflow {
    super::Workflow::new(
        &format!("Data Staleness: {key}"),
        super::Condition::DataStale {
            key: key.into(),
            duration_ms: stale_duration_ms,
        },
        vec![
            Action::SetAlarm {
                alarm_type: "DATA_STALE".into(),
                severity: "WARNING".into(),
                message: format!("No {key} data received for {}s", stale_duration_ms / 1000),
            },
        ],
    ).with_cooldown(stale_duration_ms).with_priority(50)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_result_success() {
        let r = ActionResult::success("SendRpc", "Command sent");
        assert!(r.success);
        assert_eq!(r.action_type, "SendRpc");
    }

    #[test]
    fn action_result_failure() {
        let r = ActionResult::failure("Webhook", "HTTP 500");
        assert!(!r.success);
    }

    #[test]
    fn workflow_history_bounded() {
        let mut history = WorkflowHistory::new(3);
        for i in 0..5 {
            history.record(WorkflowExecution {
                workflow_id: Uuid::nil(),
                workflow_name: format!("WF-{i}"),
                device_id: Uuid::nil(),
                timestamp_ms: i * 1000,
                actions_executed: 1,
                actions_succeeded: 1,
                actions_failed: 0,
            });
        }
        assert_eq!(history.executions.len(), 3); // bounded
    }

    #[test]
    fn emergency_stop_template() {
        let wf = emergency_stop_workflow(8.0);
        assert_eq!(wf.priority, 1);
        assert_eq!(wf.actions.len(), 4);
        assert!(matches!(wf.actions[0], Action::SendRpc { .. }));
    }

    #[test]
    fn predictive_maintenance_template() {
        let wf = predictive_maintenance_workflow(80.0);
        assert_eq!(wf.cooldown_ms, 3_600_000);
    }

    #[test]
    fn data_staleness_template() {
        let wf = data_staleness_workflow("temperature", 30_000);
        assert!(matches!(wf.condition, super::super::Condition::DataStale { .. }));
    }

    #[test]
    fn action_json_roundtrip() {
        let action = Action::SendRpc {
            device_id: Some(Uuid::nil()),
            method: "setSpeed".into(),
            params: serde_json::json!({"rpm": 50}),
            is_twoway: false,
        };
        let json = serde_json::to_string(&action).expect("serialize");
        let recovered: Action = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(recovered, Action::SendRpc { method, .. } if method == "setSpeed"));
    }
}
