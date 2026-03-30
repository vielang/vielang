use std::collections::HashMap;
use uuid::Uuid;
use super::condition_evaluator::{AlarmConditionSpec, ConditionType};

/// Per-rule, per-device state used to implement Duration and Repeating alarm conditions.
///
/// **Duration**: condition must hold continuously for at least `duration_ms`.
///   - On first true evaluation: record `condition_true_since`.
///   - On subsequent true evaluations: fire if `(now - condition_true_since) >= duration_ms`.
///   - On any false evaluation: reset `condition_true_since`.
///
/// **Repeating**: condition must be true at least `required_count` times (consecutive or total per Java).
///   - Increment `repeat_count` on each true evaluation.
///   - Fire when `repeat_count >= required_count`.
///   - Reset on any false evaluation.
///
/// Java: AlarmState in TbDeviceProfileNode + AlarmConditionState
#[derive(Debug, Clone, Default)]
pub struct AlarmRuleState {
    /// (device_id, alarm_type) → per-type state
    states: HashMap<(Uuid, String), RuleTypeState>,
}

#[derive(Debug, Clone)]
struct RuleTypeState {
    /// Timestamp (ms) when Duration condition became continuously true; None = not started
    condition_true_since: Option<i64>,
    /// Count of consecutive true evaluations for Repeating conditions
    repeat_count:         u32,
}

impl Default for RuleTypeState {
    fn default() -> Self {
        Self { condition_true_since: None, repeat_count: 0 }
    }
}

/// Result of evaluating a condition with hysteresis state
#[derive(Debug, PartialEq)]
pub enum HysteresisResult {
    /// Condition fires now (Simple always fires, Duration elapsed, Repeating count reached)
    Fire,
    /// Condition is true but hasn't elapsed/repeated enough yet
    Pending,
    /// Condition is false — reset state
    False,
}

impl AlarmRuleState {
    pub fn new() -> Self { Self::default() }

    /// Evaluate a condition spec considering hysteresis state.
    ///
    /// `condition_true` — whether the underlying predicate evaluated to true for this message.
    /// `spec`           — the full condition spec (type + duration/count params).
    /// `device_id`      — used as state key.
    /// `alarm_type`     — used as state key.
    /// `now_ms`         — current timestamp in milliseconds.
    pub fn evaluate(
        &mut self,
        condition_true: bool,
        spec:           &AlarmConditionSpec,
        device_id:      Uuid,
        alarm_type:     &str,
        now_ms:         i64,
    ) -> HysteresisResult {
        let key = (device_id, alarm_type.to_string());
        let state = self.states.entry(key).or_default();

        if !condition_true {
            state.condition_true_since = None;
            state.repeat_count = 0;
            return HysteresisResult::False;
        }

        match spec.condition_type {
            ConditionType::Simple => {
                // Simple: always fire on true
                HysteresisResult::Fire
            }

            ConditionType::Duration => {
                let duration_ms = duration_to_ms(
                    spec.duration_value.unwrap_or(0),
                    spec.duration_time_unit.as_deref().unwrap_or("SECONDS"),
                );

                match state.condition_true_since {
                    None => {
                        state.condition_true_since = Some(now_ms);
                        HysteresisResult::Pending
                    }
                    Some(since) => {
                        if (now_ms - since) >= duration_ms as i64 {
                            HysteresisResult::Fire
                        } else {
                            HysteresisResult::Pending
                        }
                    }
                }
            }

            ConditionType::Repeating => {
                let required = spec.duration_value.unwrap_or(1) as u32;
                state.repeat_count += 1;
                if state.repeat_count >= required {
                    HysteresisResult::Fire
                } else {
                    HysteresisResult::Pending
                }
            }
        }
    }

    /// Reset state for a specific (device, alarm_type) — called when alarm is cleared.
    pub fn reset(&mut self, device_id: Uuid, alarm_type: &str) {
        self.states.remove(&(device_id, alarm_type.to_string()));
    }
}

/// Convert a duration value + time unit string to milliseconds.
/// Java: TimeUnit equivalents used in ThingsBoard alarm rules.
fn duration_to_ms(value: u64, unit: &str) -> u64 {
    match unit.to_uppercase().as_str() {
        "MILLISECONDS" | "MS" => value,
        "SECONDS"      | "S"  => value * 1_000,
        "MINUTES"      | "M"  => value * 60_000,
        "HOURS"        | "H"  => value * 3_600_000,
        "DAYS"         | "D"  => value * 86_400_000,
        _                     => value * 1_000, // default to seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::profile::condition_evaluator::ConditionType;

    fn simple_spec() -> AlarmConditionSpec {
        AlarmConditionSpec {
            condition_type:     ConditionType::Simple,
            key:                None,
            predicate:          None,
            duration_time_unit: None,
            duration_value:     None,
        }
    }

    fn duration_spec(value: u64, unit: &str) -> AlarmConditionSpec {
        AlarmConditionSpec {
            condition_type:     ConditionType::Duration,
            key:                None,
            predicate:          None,
            duration_time_unit: Some(unit.to_string()),
            duration_value:     Some(value),
        }
    }

    fn repeating_spec(count: u64) -> AlarmConditionSpec {
        AlarmConditionSpec {
            condition_type:     ConditionType::Repeating,
            key:                None,
            predicate:          None,
            duration_time_unit: None,
            duration_value:     Some(count),
        }
    }

    #[test]
    fn simple_fires_immediately_on_true() {
        let mut s = AlarmRuleState::new();
        let id = Uuid::new_v4();
        let r = s.evaluate(true, &simple_spec(), id, "alarm", 0);
        assert_eq!(r, HysteresisResult::Fire);
    }

    #[test]
    fn simple_false_returns_false() {
        let mut s = AlarmRuleState::new();
        let id = Uuid::new_v4();
        let r = s.evaluate(false, &simple_spec(), id, "alarm", 0);
        assert_eq!(r, HysteresisResult::False);
    }

    #[test]
    fn duration_pending_then_fire() {
        let mut s = AlarmRuleState::new();
        let id = Uuid::new_v4();
        let spec = duration_spec(10, "SECONDS"); // 10s = 10_000ms

        // First true evaluation: pending
        let r1 = s.evaluate(true, &spec, id, "alarm", 0);
        assert_eq!(r1, HysteresisResult::Pending);

        // Still within 10s: pending
        let r2 = s.evaluate(true, &spec, id, "alarm", 5_000);
        assert_eq!(r2, HysteresisResult::Pending);

        // After 10s elapsed: fire
        let r3 = s.evaluate(true, &spec, id, "alarm", 10_000);
        assert_eq!(r3, HysteresisResult::Fire);
    }

    #[test]
    fn duration_resets_on_false() {
        let mut s = AlarmRuleState::new();
        let id = Uuid::new_v4();
        let spec = duration_spec(10, "SECONDS");

        s.evaluate(true, &spec, id, "alarm", 0);
        s.evaluate(false, &spec, id, "alarm", 5_000); // reset

        // After reset: starts pending again even at t=15_000
        let r = s.evaluate(true, &spec, id, "alarm", 15_000);
        assert_eq!(r, HysteresisResult::Pending);
    }

    #[test]
    fn repeating_fires_after_n_times() {
        let mut s = AlarmRuleState::new();
        let id = Uuid::new_v4();
        let spec = repeating_spec(3);

        assert_eq!(s.evaluate(true, &spec, id, "alarm", 0), HysteresisResult::Pending);
        assert_eq!(s.evaluate(true, &spec, id, "alarm", 1), HysteresisResult::Pending);
        assert_eq!(s.evaluate(true, &spec, id, "alarm", 2), HysteresisResult::Fire);
    }

    #[test]
    fn repeating_resets_on_false() {
        let mut s = AlarmRuleState::new();
        let id = Uuid::new_v4();
        let spec = repeating_spec(3);

        s.evaluate(true, &spec, id, "alarm", 0);
        s.evaluate(false, &spec, id, "alarm", 1); // reset count

        // Count restarts from 1
        assert_eq!(s.evaluate(true, &spec, id, "alarm", 2), HysteresisResult::Pending);
        assert_eq!(s.evaluate(true, &spec, id, "alarm", 3), HysteresisResult::Pending);
        assert_eq!(s.evaluate(true, &spec, id, "alarm", 4), HysteresisResult::Fire);
    }

    #[test]
    fn reset_clears_duration_state() {
        let mut s = AlarmRuleState::new();
        let id = Uuid::new_v4();
        let spec = duration_spec(10, "SECONDS");

        s.evaluate(true, &spec, id, "alarm", 0);
        s.reset(id, "alarm");

        // After reset: starts fresh
        let r = s.evaluate(true, &spec, id, "alarm", 100);
        assert_eq!(r, HysteresisResult::Pending);
    }

    #[test]
    fn duration_to_ms_conversions() {
        assert_eq!(duration_to_ms(5, "SECONDS"), 5_000);
        assert_eq!(duration_to_ms(2, "MINUTES"), 120_000);
        assert_eq!(duration_to_ms(1, "HOURS"), 3_600_000);
        assert_eq!(duration_to_ms(500, "MILLISECONDS"), 500);
    }
}
