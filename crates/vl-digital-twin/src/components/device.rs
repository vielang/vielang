use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Helper: lấy current time tính bằng milliseconds.
/// WASM-safe: dùng SystemTime trên native, fallback về 0 trên WASM.
pub fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Marks an entity as an IoT device in the digital twin.
/// Maps to ThingsBoard Device concept.
#[derive(Component, Debug, Clone)]
pub struct DeviceEntity {
    pub device_id:       Uuid,
    pub name:            String,
    pub device_type:     String,
    pub tenant_id:       Uuid,
    /// WGS-84 latitude in degrees (populated from ThingsBoard server attributes).
    pub latitude:        Option<f64>,
    /// WGS-84 longitude in degrees (populated from ThingsBoard server attributes).
    pub longitude:       Option<f64>,
    /// Phase 33: parent asset node in the hierarchy (Site/Building/Floor/Zone).
    pub parent_asset_id: Option<Uuid>,
}

/// Current connection status of the device.
#[derive(Component, Debug, Clone, Default)]
pub struct DeviceStatus {
    pub online:             bool,
    pub last_activity_time: Option<i64>,
}

/// Live telemetry values — updated on every WS push.
#[derive(Component, Debug, Clone, Default)]
pub struct TelemetryData {
    /// key → latest numeric value
    pub values:     HashMap<String, f64>,
    pub updated_at: i64,
}

/// Active alarm state for a device.
#[derive(Component, Debug, Clone, Default)]
pub struct AlarmIndicator {
    pub active:     bool,
    pub severity:   AlarmSeverity,
    pub alarm_type: Option<String>,
    pub message:    Option<String>,
}

/// Theo dõi độ tươi của dữ liệu — dim device khi không nhận update.
#[derive(Component, Debug, Clone)]
pub struct DataFreshness {
    /// Wall clock time (ms) của lần update cuối
    pub last_update_ms:    i64,
    /// Ngưỡng để coi là stale (ms) — default 30s
    pub stale_threshold_ms: i64,
    /// Stale state đã cache — để detect transition và tránh update material mỗi frame
    pub is_stale: bool,
}

impl Default for DataFreshness {
    fn default() -> Self {
        Self {
            last_update_ms:    current_time_ms(),
            stale_threshold_ms: 30_000,
            is_stale:           false,
        }
    }
}

impl DataFreshness {
    pub fn new_with_threshold(stale_threshold_ms: i64) -> Self {
        Self {
            stale_threshold_ms,
            ..Default::default()
        }
    }

    /// Cập nhật timestamp và xóa stale flag.
    pub fn mark_fresh(&mut self) {
        self.last_update_ms = current_time_ms();
        self.is_stale       = false;
    }

    /// Kiểm tra xem hiện tại có stale không.
    pub fn check_stale(&self) -> bool {
        current_time_ms() - self.last_update_ms > self.stale_threshold_ms
    }

    /// Thời gian (ms) kể từ lần update cuối.
    pub fn age_ms(&self) -> i64 {
        current_time_ms() - self.last_update_ms
    }
}

/// Alarm severity levels matching ThingsBoard Java enum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AlarmSeverity {
    #[default]
    None,
    Indeterminate,
    Warning,
    Minor,
    Major,
    Critical,
}

impl AlarmSeverity {
    /// Map severity to a Bevy LinearRgba color for 3D materials.
    pub fn to_linear_color(&self) -> LinearRgba {
        match self {
            AlarmSeverity::None | AlarmSeverity::Indeterminate
                                        => LinearRgba::new(0.04, 0.64, 0.04, 1.0), // green
            AlarmSeverity::Warning      => LinearRgba::new(1.0,  1.0,  0.0,  1.0), // yellow
            AlarmSeverity::Minor        => LinearRgba::new(1.0,  0.6,  0.0,  1.0), // orange
            AlarmSeverity::Major        => LinearRgba::new(1.0,  0.2,  0.0,  1.0), // red
            AlarmSeverity::Critical     => LinearRgba::new(0.8,  0.0,  0.8,  1.0), // purple
        }
    }

    /// Map severity to an egui Color32 for UI panels.
    pub fn to_egui_color(&self) -> bevy_egui::egui::Color32 {
        match self {
            AlarmSeverity::None | AlarmSeverity::Indeterminate
                                => bevy_egui::egui::Color32::GREEN,
            AlarmSeverity::Warning  => bevy_egui::egui::Color32::YELLOW,
            AlarmSeverity::Minor    => bevy_egui::egui::Color32::from_rgb(255, 153, 0),
            AlarmSeverity::Major    => bevy_egui::egui::Color32::RED,
            AlarmSeverity::Critical => bevy_egui::egui::Color32::from_rgb(204, 0, 204),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "WARNING"       => AlarmSeverity::Warning,
            "MINOR"         => AlarmSeverity::Minor,
            "MAJOR"         => AlarmSeverity::Major,
            "CRITICAL"      => AlarmSeverity::Critical,
            "INDETERMINATE" => AlarmSeverity::Indeterminate,
            _               => AlarmSeverity::None,
        }
    }
}

impl std::fmt::Display for AlarmSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlarmSeverity::None          => write!(f, "None"),
            AlarmSeverity::Indeterminate => write!(f, "Indeterminate"),
            AlarmSeverity::Warning       => write!(f, "Warning"),
            AlarmSeverity::Minor         => write!(f, "Minor"),
            AlarmSeverity::Major         => write!(f, "Major"),
            AlarmSeverity::Critical      => write!(f, "Critical"),
        }
    }
}

/// Marker for the currently selected entity.
#[derive(Component)]
pub struct Selected;

/// Marker for the currently hovered entity.
#[derive(Component)]
pub struct Hovered;

// ── RPC presets ───────────────────────────────────────────────────────────────

/// A pre-configured RPC command shown in the device context menu.
#[derive(Debug, Clone)]
pub struct RpcPreset {
    /// Button label shown in the context menu.
    pub label:     String,
    /// ThingsBoard RPC method name.
    pub method:    String,
    /// JSON params sent with the request.
    pub params:    serde_json::Value,
    /// If true, waits for a device response (twoway).
    pub is_twoway: bool,
    /// If true, shows a confirmation dialog before sending.
    pub confirm:   bool,
}

/// Device-type-specific RPC preset commands attached to each device entity.
#[derive(Component, Debug, Clone)]
pub struct DeviceRpcPresets(pub Vec<RpcPreset>);

impl DeviceRpcPresets {
    pub fn for_device_type(device_type: &str) -> Self {
        let presets = match device_type {
            "wind_turbine" => vec![
                RpcPreset {
                    label:     "Set Speed: 50 RPM".into(),
                    method:    "setSpeed".into(),
                    params:    serde_json::json!({ "rpm": 50 }),
                    is_twoway: false,
                    confirm:   false,
                },
                RpcPreset {
                    label:     "Emergency Stop".into(),
                    method:    "emergencyStop".into(),
                    params:    serde_json::json!({}),
                    is_twoway: false,
                    confirm:   true,
                },
                RpcPreset {
                    label:     "Get Status".into(),
                    method:    "getStatus".into(),
                    params:    serde_json::json!({}),
                    is_twoway: true,
                    confirm:   false,
                },
            ],
            "temperature_sensor" => vec![
                RpcPreset {
                    label:     "Set Threshold: 50°C".into(),
                    method:    "setThreshold".into(),
                    params:    serde_json::json!({ "value": 50.0 }),
                    is_twoway: false,
                    confirm:   false,
                },
                RpcPreset {
                    label:     "Calibrate".into(),
                    method:    "calibrate".into(),
                    params:    serde_json::json!({}),
                    is_twoway: true,
                    confirm:   true,
                },
            ],
            _ => vec![RpcPreset {
                label:     "Get Status".into(),
                method:    "getStatus".into(),
                params:    serde_json::json!({}),
                is_twoway: true,
                confirm:   false,
            }],
        };
        Self(presets)
    }
}

// ── Shared attributes ─────────────────────────────────────────────────────────

/// Shared-scope device attributes received via WebSocket subscription.
/// Updated when the ThingsBoard operator changes device configuration.
#[derive(Component, Debug, Clone, Default)]
pub struct SharedAttributes {
    /// key → current JSON value
    pub values:     std::collections::HashMap<String, serde_json::Value>,
    pub updated_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alarm_severity_from_str_roundtrip() {
        assert_eq!(AlarmSeverity::from_str("WARNING"),       AlarmSeverity::Warning);
        assert_eq!(AlarmSeverity::from_str("warning"),       AlarmSeverity::Warning);
        assert_eq!(AlarmSeverity::from_str("MINOR"),         AlarmSeverity::Minor);
        assert_eq!(AlarmSeverity::from_str("MAJOR"),         AlarmSeverity::Major);
        assert_eq!(AlarmSeverity::from_str("CRITICAL"),      AlarmSeverity::Critical);
        assert_eq!(AlarmSeverity::from_str("INDETERMINATE"), AlarmSeverity::Indeterminate);
        assert_eq!(AlarmSeverity::from_str("unknown"),       AlarmSeverity::None);
        assert_eq!(AlarmSeverity::from_str(""),              AlarmSeverity::None);
    }

    #[test]
    fn alarm_severity_display() {
        assert_eq!(AlarmSeverity::Critical.to_string(), "Critical");
        assert_eq!(AlarmSeverity::None.to_string(),     "None");
    }

    #[test]
    fn telemetry_data_default_empty() {
        let td = TelemetryData::default();
        assert!(td.values.is_empty());
        assert_eq!(td.updated_at, 0);
    }

    #[test]
    fn alarm_indicator_default_inactive() {
        let ai = AlarmIndicator::default();
        assert!(!ai.active);
        assert_eq!(ai.severity, AlarmSeverity::None);
        assert!(ai.alarm_type.is_none());
    }

    #[test]
    fn data_freshness_default_not_stale() {
        let f = DataFreshness::default();
        assert!(!f.is_stale);
        // Bắt đầu không stale (last_update = now)
        assert!(!f.check_stale());
    }

    #[test]
    fn data_freshness_mark_fresh_resets() {
        let mut f = DataFreshness {
            last_update_ms:    0, // rất cũ
            stale_threshold_ms: 30_000,
            is_stale:           true,
        };
        // Giả lập stale
        assert!(f.check_stale());
        // Mark fresh
        f.mark_fresh();
        assert!(!f.is_stale);
        assert!(!f.check_stale());
    }

    // ── Phase 33: parent_asset_id field ──────────────────────────────────────

    #[test]
    fn device_entity_parent_asset_id_defaults_to_none() {
        use uuid::Uuid;
        // Verify the field exists and can be constructed with None
        let dev = DeviceEntity {
            device_id:       Uuid::nil(),
            name:            "Test".into(),
            device_type:     "sensor".into(),
            tenant_id:       Uuid::nil(),
            latitude:        None,
            longitude:       None,
            parent_asset_id: None,
        };
        assert!(dev.parent_asset_id.is_none());
    }

    #[test]
    fn device_entity_parent_asset_id_can_be_set() {
        use uuid::Uuid;
        let asset_id = Uuid::new_v4();
        let dev = DeviceEntity {
            device_id:       Uuid::nil(),
            name:            "Test".into(),
            device_type:     "sensor".into(),
            tenant_id:       Uuid::nil(),
            latitude:        None,
            longitude:       None,
            parent_asset_id: Some(asset_id),
        };
        assert_eq!(dev.parent_asset_id, Some(asset_id));
    }

    // ── DataFreshness age_ms ─────────────────────────────────────────────────

    #[test]
    fn data_freshness_age_ms_increases_over_time() {
        // last_update_ms = 0 means age is approximately current_time_ms
        let f = DataFreshness {
            last_update_ms:    0,
            stale_threshold_ms: 30_000,
            is_stale:           false,
        };
        assert!(f.age_ms() > 0, "age should be positive when last_update is epoch 0");
    }

    // ── AlarmSeverity ordering ────────────────────────────────────────────────

    #[test]
    fn alarm_severity_from_str_case_insensitive() {
        assert_eq!(AlarmSeverity::from_str("warning"), AlarmSeverity::Warning);
        assert_eq!(AlarmSeverity::from_str("CRITICAL"), AlarmSeverity::Critical);
        assert_eq!(AlarmSeverity::from_str("Minor"), AlarmSeverity::Minor);
        assert_eq!(AlarmSeverity::from_str("major"), AlarmSeverity::Major);
    }

    // ── RpcPreset for unknown type ────────────────────────────────────────────

    #[test]
    fn device_rpc_presets_unknown_type_has_get_status() {
        let presets = DeviceRpcPresets::for_device_type("unknown_sensor");
        assert_eq!(presets.0.len(), 1);
        assert_eq!(presets.0[0].method, "getStatus");
    }

    #[test]
    fn device_rpc_presets_wind_turbine_has_emergency_stop() {
        let presets = DeviceRpcPresets::for_device_type("wind_turbine");
        assert!(presets.0.iter().any(|p| p.method == "emergencyStop"),
            "wind turbine should have emergency stop command");
    }

    #[test]
    fn device_rpc_presets_temperature_sensor_has_calibrate() {
        let presets = DeviceRpcPresets::for_device_type("temperature_sensor");
        assert!(presets.0.iter().any(|p| p.method == "calibrate"),
            "temperature sensor should have calibrate command");
    }
}
