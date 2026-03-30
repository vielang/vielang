//! Phase 30 — Client-side AI/ML anomaly detection.
//!
//! Runs entirely in-process; no backend changes required.
//! Feeds `TelemetryUpdate` events through `AnomalyDetector` and raises
//! anomaly alarms via `AlarmRegistry` + `NotificationQueue`.

pub mod detector;
pub mod stats;
pub mod isolation_forest;
pub mod cusum;

pub use detector::{AnomalyConfig, AnomalyDetector, AnomalyKind, AnomalyResult};

use bevy::prelude::*;

use crate::alarm::AlarmRegistry;
use crate::components::AlarmSeverity;
use crate::events::TelemetryUpdate;
use crate::ui::notifications::NotificationQueue;

/// System: feed every incoming telemetry value through the anomaly detector.
///
/// When anomalies are found, they are raised as Minor/Major alarms in the
/// `AlarmRegistry` and pushed as toast notifications.
pub fn run_anomaly_detection(
    mut telem_events: MessageReader<TelemetryUpdate>,
    mut detector:     ResMut<AnomalyDetector>,
    mut alarm_reg:    ResMut<AlarmRegistry>,
    mut notif_queue:  ResMut<NotificationQueue>,
) {
    for ev in telem_events.read() {
        let results = detector.feed(
            ev.device_id,
            &ev.key,
            ev.value as f32,
            ev.ts as u64,
        );

        for anomaly in results {
            let severity = if anomaly.z_score.map(|z| z.abs() >= 5.0).unwrap_or(false) {
                AlarmSeverity::Major
            } else {
                AlarmSeverity::Minor
            };

            let alarm_type = format!("ANOMALY_{}", anomaly.kind);
            alarm_reg.upsert_from_ws(
                anomaly.device_id,
                &alarm_type,
                severity.clone(),
                true,
                anomaly.timestamp_ms as i64,
            );

            notif_queue.push(
                severity,
                "Anomaly Detector".to_string(),
                anomaly.description.clone(),
            );

            tracing::warn!(
                device = %anomaly.device_id,
                key    = %anomaly.key,
                kind   = %anomaly.kind,
                desc   = %anomaly.description,
                "Anomaly detected"
            );
        }
    }
}
