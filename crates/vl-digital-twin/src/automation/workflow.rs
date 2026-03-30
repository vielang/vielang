//! Workflow engine — evaluates conditions and triggers actions.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::action::Action;

/// A condition that triggers a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Telemetry value exceeds threshold.
    ThresholdExceeded {
        device_type: Option<String>,
        key: String,
        operator: ComparisonOp,
        value: f64,
    },
    /// Alarm of a specific severity or higher is active.
    AlarmActive {
        min_severity: String,
        alarm_type: Option<String>,
    },
    /// Device has been offline for longer than duration_ms.
    DeviceOffline {
        duration_ms: u64,
    },
    /// Anomaly detected by the analytics engine.
    AnomalyDetected {
        anomaly_kind: Option<String>,
    },
    /// CUSUM change-point detected.
    ChangePointDetected {
        key: String,
        direction: Option<String>,
    },
    /// Data staleness — no telemetry for duration_ms.
    DataStale {
        key: String,
        duration_ms: u64,
    },
    /// Composite AND: all sub-conditions must be true.
    All(Vec<Condition>),
    /// Composite OR: at least one sub-condition must be true.
    Any(Vec<Condition>),
    /// Negate a condition.
    Not(Box<Condition>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOp {
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = "<=")]
    LessThanOrEqual,
    #[serde(rename = "==")]
    Equal,
    #[serde(rename = "!=")]
    NotEqual,
}

impl ComparisonOp {
    pub fn evaluate(&self, actual: f64, expected: f64) -> bool {
        match self {
            ComparisonOp::GreaterThan        => actual > expected,
            ComparisonOp::GreaterThanOrEqual => actual >= expected,
            ComparisonOp::LessThan           => actual < expected,
            ComparisonOp::LessThanOrEqual    => actual <= expected,
            ComparisonOp::Equal              => (actual - expected).abs() < 1e-9,
            ComparisonOp::NotEqual           => (actual - expected).abs() >= 1e-9,
        }
    }
}

/// A complete automation workflow: condition → action(s).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub enabled: bool,
    pub condition: Condition,
    pub actions: Vec<Action>,
    /// Minimum time between trigger executions (ms) to prevent flapping.
    pub cooldown_ms: u64,
    /// Last time this workflow was triggered (internal state).
    #[serde(skip)]
    pub last_triggered_ms: u64,
    /// Priority (lower = higher priority).
    #[serde(default)]
    pub priority: u32,
}

impl Workflow {
    pub fn new(name: &str, condition: Condition, actions: Vec<Action>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            enabled: true,
            condition,
            actions,
            cooldown_ms: 60_000,
            last_triggered_ms: 0,
            priority: 100,
        }
    }

    pub fn with_cooldown(mut self, ms: u64) -> Self {
        self.cooldown_ms = ms;
        self
    }

    pub fn with_priority(mut self, p: u32) -> Self {
        self.priority = p;
        self
    }

    /// Check if the cooldown period has elapsed.
    pub fn is_cooldown_elapsed(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.last_triggered_ms) >= self.cooldown_ms
    }
}

/// Context available during condition evaluation.
pub struct EvalContext {
    /// Current telemetry values per device: device_id → (key → value).
    pub telemetry: HashMap<Uuid, HashMap<String, f64>>,
    /// Active alarm severities per device: device_id → (alarm_type → severity).
    pub alarms: HashMap<Uuid, HashMap<String, String>>,
    /// Device online status: device_id → (is_online, last_activity_ms).
    pub device_status: HashMap<Uuid, (bool, u64)>,
    /// Active anomalies: device_id → anomaly_kind.
    pub anomalies: HashMap<Uuid, Vec<String>>,
    /// Active CUSUM alerts: (device_id, key) → direction.
    pub change_points: HashMap<(Uuid, String), String>,
    /// Telemetry freshness: (device_id, key) → last_update_ms.
    pub freshness: HashMap<(Uuid, String), u64>,
    /// Current time (ms).
    pub now_ms: u64,
}

impl EvalContext {
    /// Evaluate a condition against this context for a specific device.
    pub fn evaluate(&self, condition: &Condition, device_id: Uuid) -> bool {
        match condition {
            Condition::ThresholdExceeded { key, operator, value, .. } => {
                self.telemetry.get(&device_id)
                    .and_then(|m| m.get(key))
                    .map(|v| operator.evaluate(*v, *value))
                    .unwrap_or(false)
            }
            Condition::AlarmActive { min_severity, alarm_type } => {
                self.alarms.get(&device_id)
                    .map(|alarms| {
                        alarms.iter().any(|(atype, severity)| {
                            let severity_match = severity_ge(severity, min_severity);
                            let type_match = alarm_type.as_ref()
                                .map(|t| atype == t)
                                .unwrap_or(true);
                            severity_match && type_match
                        })
                    })
                    .unwrap_or(false)
            }
            Condition::DeviceOffline { duration_ms } => {
                self.device_status.get(&device_id)
                    .map(|(online, last_activity)| {
                        !online && self.now_ms.saturating_sub(*last_activity) >= *duration_ms
                    })
                    .unwrap_or(false)
            }
            Condition::AnomalyDetected { anomaly_kind } => {
                self.anomalies.get(&device_id)
                    .map(|kinds| {
                        anomaly_kind.as_ref()
                            .map(|k| kinds.contains(k))
                            .unwrap_or(!kinds.is_empty())
                    })
                    .unwrap_or(false)
            }
            Condition::ChangePointDetected { key, direction } => {
                self.change_points.get(&(device_id, key.clone()))
                    .map(|d| direction.as_ref().map(|dir| d == dir).unwrap_or(true))
                    .unwrap_or(false)
            }
            Condition::DataStale { key, duration_ms } => {
                self.freshness.get(&(device_id, key.clone()))
                    .map(|ts| self.now_ms.saturating_sub(*ts) >= *duration_ms)
                    .unwrap_or(true) // No data = stale
            }
            Condition::All(conditions) => {
                conditions.iter().all(|c| self.evaluate(c, device_id))
            }
            Condition::Any(conditions) => {
                conditions.iter().any(|c| self.evaluate(c, device_id))
            }
            Condition::Not(condition) => {
                !self.evaluate(condition, device_id)
            }
        }
    }
}

/// Compare alarm severity strings (higher = more severe).
fn severity_ge(actual: &str, min: &str) -> bool {
    let order = |s: &str| match s.to_uppercase().as_str() {
        "CRITICAL" => 5,
        "MAJOR" => 4,
        "MINOR" => 3,
        "WARNING" => 2,
        "INDETERMINATE" => 1,
        _ => 0,
    };
    order(actual) >= order(min)
}

/// Central workflow engine — Bevy resource.
#[derive(Resource, Default)]
pub struct WorkflowEngine {
    pub workflows: Vec<Workflow>,
}

impl WorkflowEngine {
    pub fn add(&mut self, workflow: Workflow) {
        self.workflows.push(workflow);
        self.workflows.sort_by_key(|w| w.priority);
    }

    pub fn remove(&mut self, workflow_id: Uuid) {
        self.workflows.retain(|w| w.id != workflow_id);
    }

    /// Evaluate all workflows for a device and return triggered actions.
    pub fn evaluate(
        &mut self,
        device_id: Uuid,
        ctx: &EvalContext,
    ) -> Vec<(Uuid, Vec<Action>)> {
        let mut triggered = Vec::new();

        for workflow in &mut self.workflows {
            if !workflow.enabled {
                continue;
            }
            if !workflow.is_cooldown_elapsed(ctx.now_ms) {
                continue;
            }
            if ctx.evaluate(&workflow.condition, device_id) {
                workflow.last_triggered_ms = ctx.now_ms;
                triggered.push((workflow.id, workflow.actions.clone()));
            }
        }

        triggered
    }

    pub fn enabled_count(&self) -> usize {
        self.workflows.iter().filter(|w| w.enabled).count()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_context(now_ms: u64) -> EvalContext {
        EvalContext {
            telemetry: HashMap::new(),
            alarms: HashMap::new(),
            device_status: HashMap::new(),
            anomalies: HashMap::new(),
            change_points: HashMap::new(),
            freshness: HashMap::new(),
            now_ms,
        }
    }

    #[test]
    fn threshold_exceeded_true() {
        let device = Uuid::nil();
        let mut ctx = empty_context(1000);
        let mut telem = HashMap::new();
        telem.insert("temperature".into(), 85.0);
        ctx.telemetry.insert(device, telem);

        let cond = Condition::ThresholdExceeded {
            device_type: None,
            key: "temperature".into(),
            operator: ComparisonOp::GreaterThan,
            value: 80.0,
        };

        assert!(ctx.evaluate(&cond, device));
    }

    #[test]
    fn threshold_exceeded_false() {
        let device = Uuid::nil();
        let mut ctx = empty_context(1000);
        let mut telem = HashMap::new();
        telem.insert("temperature".into(), 75.0);
        ctx.telemetry.insert(device, telem);

        let cond = Condition::ThresholdExceeded {
            device_type: None,
            key: "temperature".into(),
            operator: ComparisonOp::GreaterThan,
            value: 80.0,
        };

        assert!(!ctx.evaluate(&cond, device));
    }

    #[test]
    fn composite_all() {
        let device = Uuid::nil();
        let mut ctx = empty_context(1000);
        let mut telem = HashMap::new();
        telem.insert("temp".into(), 90.0);
        telem.insert("pressure".into(), 150.0);
        ctx.telemetry.insert(device, telem);

        let cond = Condition::All(vec![
            Condition::ThresholdExceeded {
                device_type: None, key: "temp".into(),
                operator: ComparisonOp::GreaterThan, value: 80.0,
            },
            Condition::ThresholdExceeded {
                device_type: None, key: "pressure".into(),
                operator: ComparisonOp::GreaterThan, value: 100.0,
            },
        ]);

        assert!(ctx.evaluate(&cond, device));
    }

    #[test]
    fn composite_not() {
        let device = Uuid::nil();
        let ctx = empty_context(1000);
        let cond = Condition::Not(Box::new(Condition::AnomalyDetected { anomaly_kind: None }));
        assert!(ctx.evaluate(&cond, device)); // no anomalies = NOT triggered = true
    }

    #[test]
    fn workflow_engine_cooldown() {
        let mut engine = WorkflowEngine::default();
        let device = Uuid::nil();

        engine.add(Workflow::new(
            "Test",
            Condition::ThresholdExceeded {
                device_type: None, key: "temp".into(),
                operator: ComparisonOp::GreaterThan, value: 80.0,
            },
            vec![Action::Notification { severity: "MAJOR".into(), message: "Hot!".into() }],
        ).with_cooldown(60_000));

        let mut ctx = empty_context(100_000);
        let mut telem = HashMap::new();
        telem.insert("temp".into(), 90.0);
        ctx.telemetry.insert(device, telem);

        // First evaluation: should trigger
        let result = engine.evaluate(device, &ctx);
        assert_eq!(result.len(), 1);

        // Second evaluation within cooldown: should not trigger
        ctx.now_ms = 110_000; // only 10s later
        let result = engine.evaluate(device, &ctx);
        assert_eq!(result.len(), 0);

        // Third evaluation after cooldown: should trigger
        ctx.now_ms = 200_000; // 100s later
        let result = engine.evaluate(device, &ctx);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn comparison_ops() {
        assert!(ComparisonOp::GreaterThan.evaluate(5.0, 3.0));
        assert!(!ComparisonOp::GreaterThan.evaluate(3.0, 5.0));
        assert!(ComparisonOp::LessThan.evaluate(3.0, 5.0));
        assert!(ComparisonOp::Equal.evaluate(5.0, 5.0));
        assert!(ComparisonOp::NotEqual.evaluate(3.0, 5.0));
        assert!(ComparisonOp::GreaterThanOrEqual.evaluate(5.0, 5.0));
        assert!(ComparisonOp::LessThanOrEqual.evaluate(5.0, 5.0));
    }

    #[test]
    fn severity_ordering() {
        assert!(severity_ge("CRITICAL", "MAJOR"));
        assert!(severity_ge("MAJOR", "MAJOR"));
        assert!(!severity_ge("MINOR", "MAJOR"));
        assert!(severity_ge("WARNING", "WARNING"));
    }
}
