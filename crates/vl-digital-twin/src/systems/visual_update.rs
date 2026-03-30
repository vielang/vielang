//! Visual update systems — update 3D materials and transforms from ECS state.

use bevy::prelude::*;

use crate::api::HistoricalDataCache;
use crate::components::{AlarmIndicator, AlarmSeverity, DataFreshness, DeviceEntity, TelemetryData};
use crate::playback::PlaybackState;

// ── HeatmapConfig ─────────────────────────────────────────────────────────────

/// Controls which telemetry key drives the 3D color heatmap and its value range.
///
/// Defaults to `temperature` (0–50 °C) for backwards compatibility with demo devices.
/// Phase 26: key is now runtime-selectable from any observed telemetry key.
#[derive(Resource)]
pub struct HeatmapConfig {
    /// Telemetry key used for heatmap coloring (e.g. "temperature", "humidity").
    pub active_key: String,
    /// Minimum value mapped to blue (cold).
    pub range_min:  f32,
    /// Maximum value mapped to red (hot).
    pub range_max:  f32,
    pub enabled:    bool,
}

impl Default for HeatmapConfig {
    fn default() -> Self {
        Self {
            active_key: "temperature".into(),
            range_min:  0.0,
            range_max:  50.0,
            enabled:    true,
        }
    }
}

impl HeatmapConfig {
    /// Normalize a raw value to [0.0, 1.0] using the configured range.
    pub fn normalize(&self, value: f32) -> f32 {
        let range = self.range_max - self.range_min;
        if range.abs() < f32::EPSILON { return 0.5; }
        ((value - self.range_min) / range).clamp(0.0, 1.0)
    }

    /// Blue → red gradient color for a normalized value.
    pub fn color_for_t(t: f32) -> Color {
        Color::srgb(t, 0.2, 1.0 - t)
    }
}

/// Update material color/emissive based on AlarmIndicator state.
/// Runs when AlarmIndicator component changes (via Changed<> filter).
pub fn update_alarm_visuals(
    mut materials: ResMut<Assets<StandardMaterial>>,
    time:          Res<Time>,
    query:         Query<(&AlarmIndicator, &MeshMaterial3d<StandardMaterial>), Changed<AlarmIndicator>>,
) {
    for (alarm, mat_handle) in query.iter() {
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            let base_color = alarm.severity.to_linear_color();

            if alarm.active && alarm.severity == AlarmSeverity::Critical {
                // Pulsing effect for critical alarms
                let pulse = (time.elapsed_secs() * 4.0).sin() * 0.5 + 0.5;
                material.base_color = Color::from(base_color.with_alpha(0.5 + pulse * 0.5));
                material.emissive   = LinearRgba::new(0.5, 0.0, 0.5, 1.0) * pulse;
            } else if alarm.active {
                material.base_color = Color::from(base_color);
                material.emissive   = base_color * 0.3;
            } else {
                // No alarm — green
                material.base_color = Color::from(AlarmSeverity::None.to_linear_color());
                material.emissive   = LinearRgba::BLACK;
            }
        }
    }
}

/// Multi-metric heatmap — map any configured telemetry key to material color (blue→red).
///
/// Phase 26: key and range are now controlled by `HeatmapConfig` instead of hardcoded.
pub fn update_heatmap(
    config:        Res<HeatmapConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query:         Query<
        (&TelemetryData, &MeshMaterial3d<StandardMaterial>),
        Changed<TelemetryData>,
    >,
) {
    if !config.enabled { return; }

    for (telemetry, mat_handle) in query.iter() {
        let Some(&raw_value) = telemetry.values.get(&config.active_key) else { continue };
        let Some(material)   = materials.get_mut(&mat_handle.0) else { continue };

        let t = config.normalize(raw_value as f32);
        material.base_color = HeatmapConfig::color_for_t(t);
    }
}

/// Dim devices khi không nhận được telemetry update quá lâu (stale data).
/// Chạy mỗi frame nhưng chỉ cập nhật material khi stale state thay đổi.
pub fn update_stale_visuals(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut DataFreshness, &MeshMaterial3d<StandardMaterial>)>,
) {
    for (mut freshness, mat_handle) in query.iter_mut() {
        let now_stale = freshness.check_stale();

        // Chỉ update material khi state thay đổi — tránh redundant write mỗi frame
        if now_stale == freshness.is_stale {
            continue;
        }
        freshness.is_stale = now_stale;

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            if now_stale {
                // Dim alpha xuống 35% để báo hiệu data đã cũ
                let c = mat.base_color.to_srgba();
                mat.base_color = Color::srgba(c.red, c.green, c.blue, 0.35);
                mat.emissive   = LinearRgba::BLACK;
            } else {
                // Restore full opacity
                let c = mat.base_color.to_srgba();
                mat.base_color = Color::srgba(c.red, c.green, c.blue, 1.0);
            }
        }
    }
}

/// In Paused/Playing mode, update device material colors from historical cache.
///
/// Uses `HeatmapConfig.active_key` — same key as the live heatmap.
/// In Live mode, `update_heatmap` handles colors — this system is a no-op.
pub fn update_device_color_by_playback(
    playback:   Res<PlaybackState>,
    config:     Res<HeatmapConfig>,
    hist_cache: Res<HistoricalDataCache>,
    mut mats:   ResMut<Assets<StandardMaterial>>,
    query:      Query<(&DeviceEntity, &MeshMaterial3d<StandardMaterial>)>,
) {
    if playback.is_live() || !config.enabled { return; }
    let at_ts = playback.current_ts();

    for (device, mat_handle) in query.iter() {
        let Some(value) = hist_cache.get_at(device.device_id, &config.active_key, at_ts) else {
            continue;
        };
        let Some(mat) = mats.get_mut(&mat_handle.0) else { continue };

        let t = config.normalize(value as f32);
        mat.base_color = HeatmapConfig::color_for_t(t);
    }
}

/// Animate mesh transform based on telemetry (e.g. wind turbine rotation).
pub fn animate_by_telemetry(
    time:  Res<Time>,
    mut query: Query<(&DeviceEntity, &TelemetryData, &mut Transform)>,
) {
    for (device, telemetry, mut transform) in query.iter_mut() {
        if device.device_type == "wind_turbine" {
            if let Some(&wind_speed) = telemetry.values.get("wind_speed") {
                // RPM proportional to wind speed; convert to radians/sec
                let rpm = wind_speed as f32 * 2.0;
                let rad_per_sec = rpm * std::f32::consts::TAU / 60.0;
                transform.rotate_local_y(rad_per_sec * time.delta_secs());
            }
        }
    }
}
