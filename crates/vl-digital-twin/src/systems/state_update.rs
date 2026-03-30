//! ECS state update systems — apply incoming events to components.

use bevy::prelude::*;

use crate::components::{AlarmIndicator, AlarmSeverity, DataFreshness, DeviceEntity, SharedAttributes, TelemetryData};
use crate::events::{AlarmUpdate, AttributeUpdate, TelemetryUpdate};
use crate::ui::TelemetryHistory;

/// Apply TelemetryUpdate events to TelemetryData components.
/// Đồng thời cập nhật DataFreshness nếu có.
pub fn apply_telemetry_updates(
    mut events: MessageReader<TelemetryUpdate>,
    mut query:  Query<(&DeviceEntity, &mut TelemetryData, Option<&mut DataFreshness>)>,
) {
    for event in events.read() {
        for (device, mut telemetry, freshness) in query.iter_mut() {
            if device.device_id == event.device_id {
                telemetry.values.insert(event.key.clone(), event.value);
                telemetry.updated_at = event.ts;
                if let Some(mut f) = freshness {
                    f.mark_fresh();
                }
            }
        }
    }
}

/// Apply AlarmUpdate events to AlarmIndicator components.
pub fn apply_alarm_updates(
    mut events: MessageReader<AlarmUpdate>,
    mut query:  Query<(&DeviceEntity, &mut AlarmIndicator)>,
) {
    for event in events.read() {
        for (device, mut alarm) in query.iter_mut() {
            if device.device_id == event.device_id {
                alarm.active     = event.active;
                alarm.alarm_type = Some(event.alarm_type.clone());
                alarm.severity   = AlarmSeverity::from_str(&event.severity);
            }
        }
    }
}

/// Apply AttributeUpdate events to SharedAttributes components.
pub fn apply_attribute_updates(
    mut events: MessageReader<AttributeUpdate>,
    mut query:  Query<(&DeviceEntity, &mut SharedAttributes)>,
) {
    for event in events.read() {
        for (device, mut attrs) in query.iter_mut() {
            if device.device_id == event.device_id {
                attrs.values.insert(event.key.clone(), event.value.clone());
                attrs.updated_at = crate::components::current_time_ms();
            }
        }
    }
}

/// Keep rolling telemetry history for charts (last 100 points per key).
pub fn update_telemetry_history(
    mut events:  MessageReader<TelemetryUpdate>,
    mut history: ResMut<TelemetryHistory>,
) {
    for event in events.read() {
        let entry = history
            .data
            .entry((event.device_id, event.key.clone()))
            .or_insert_with(Vec::new);

        entry.push((event.ts, event.value));

        // Keep last 100 data points to avoid unbounded growth
        if entry.len() > 100 {
            entry.drain(..entry.len() - 100);
        }
    }
}
