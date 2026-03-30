//! Phase 34 — Device Profiles & Telemetry Schema.
//!
//! Maps device types to their expected telemetry keys, units, and valid ranges.
//! Drives automatic heatmap configuration, chart axis scaling, and range-violation flagging.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Key definition ─────────────────────────────────────────────────────────────

/// Definition of a single telemetry key within a device profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryKeyDef {
    pub key:          String,
    pub display_name: String,
    pub unit:         String,
    /// Minimum valid reading — values below this are flagged as out-of-range.
    pub range_min:    Option<f64>,
    /// Maximum valid reading.
    pub range_max:    Option<f64>,
    /// RGBA display colour (each component 0.0–1.0).
    pub color:        Option<[f32; 4]>,
}

impl TelemetryKeyDef {
    /// Returns `true` when `value` falls within the defined valid range.
    pub fn is_in_range(&self, value: f64) -> bool {
        match (self.range_min, self.range_max) {
            (Some(min), Some(max)) => value >= min && value <= max,
            (Some(min), None)      => value >= min,
            (None, Some(max))      => value <= max,
            (None, None)           => true,
        }
    }

    /// Human-readable range string, e.g. "[-40 … 125 °C]".
    pub fn range_label(&self) -> String {
        match (self.range_min, self.range_max) {
            (Some(min), Some(max)) => format!("[{} … {} {}]", min, max, self.unit),
            (Some(min), None)      => format!("[{}… {}]", min, self.unit),
            (None, Some(max))      => format!("[…{} {}]", max, self.unit),
            (None, None)           => self.unit.clone(),
        }
    }
}

// ── Profile ───────────────────────────────────────────────────────────────────

/// Complete schema for one device type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub name:  String,
    pub keys:  Vec<TelemetryKeyDef>,
    /// Optional icon identifier (for future UI use).
    pub icon:  Option<String>,
}

impl DeviceProfile {
    /// Look up the key definition by key name.
    pub fn key_def(&self, key: &str) -> Option<&TelemetryKeyDef> {
        self.keys.iter().find(|k| k.key == key)
    }

    /// All key names defined by this profile.
    pub fn key_names(&self) -> Vec<&str> {
        self.keys.iter().map(|k| k.key.as_str()).collect()
    }
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// Global device-type → profile registry.
#[derive(Resource, Default)]
pub struct ProfileRegistry {
    /// device_type string → DeviceProfile
    pub profiles: HashMap<String, DeviceProfile>,
}

impl ProfileRegistry {
    /// Populate with built-in profiles for known device types.
    pub fn with_builtins(mut self) -> Self {
        self.profiles.insert(
            "temperature_sensor".into(),
            DeviceProfile {
                name: "Temperature Sensor".into(),
                icon: Some("thermometer".into()),
                keys: vec![
                    TelemetryKeyDef {
                        key:          "temperature".into(),
                        display_name: "Temperature".into(),
                        unit:         "°C".into(),
                        range_min:    Some(-40.0),
                        range_max:    Some(125.0),
                        color:        Some([1.0, 0.3, 0.1, 1.0]),
                    },
                    TelemetryKeyDef {
                        key:          "humidity".into(),
                        display_name: "Humidity".into(),
                        unit:         "%RH".into(),
                        range_min:    Some(0.0),
                        range_max:    Some(100.0),
                        color:        Some([0.1, 0.5, 1.0, 1.0]),
                    },
                    TelemetryKeyDef {
                        key:          "battery".into(),
                        display_name: "Battery".into(),
                        unit:         "%".into(),
                        range_min:    Some(0.0),
                        range_max:    Some(100.0),
                        color:        Some([0.2, 0.8, 0.2, 1.0]),
                    },
                ],
            },
        );

        self.profiles.insert(
            "wind_turbine".into(),
            DeviceProfile {
                name: "Wind Turbine".into(),
                icon: Some("wind".into()),
                keys: vec![
                    TelemetryKeyDef {
                        key:          "rpm".into(),
                        display_name: "Rotor Speed".into(),
                        unit:         "RPM".into(),
                        range_min:    Some(0.0),
                        range_max:    Some(200.0),
                        color:        Some([0.4, 0.8, 1.0, 1.0]),
                    },
                    TelemetryKeyDef {
                        key:          "power_kw".into(),
                        display_name: "Power Output".into(),
                        unit:         "kW".into(),
                        range_min:    Some(0.0),
                        range_max:    Some(5000.0),
                        color:        Some([1.0, 0.8, 0.0, 1.0]),
                    },
                    TelemetryKeyDef {
                        key:          "wind_speed".into(),
                        display_name: "Wind Speed".into(),
                        unit:         "m/s".into(),
                        range_min:    Some(0.0),
                        range_max:    Some(30.0),
                        color:        Some([0.6, 0.9, 0.6, 1.0]),
                    },
                    TelemetryKeyDef {
                        key:          "vibration".into(),
                        display_name: "Vibration".into(),
                        unit:         "mm/s".into(),
                        range_min:    Some(0.0),
                        range_max:    Some(10.0),
                        color:        Some([0.9, 0.5, 0.1, 1.0]),
                    },
                ],
            },
        );

        self
    }

    /// Return a profile by device type (None if unknown).
    pub fn get(&self, device_type: &str) -> Option<&DeviceProfile> {
        self.profiles.get(device_type)
    }

    /// Return valid range min/max for a specific key on a device type.
    pub fn range_for(&self, device_type: &str, key: &str) -> (Option<f64>, Option<f64>) {
        self.profiles
            .get(device_type)
            .and_then(|p| p.key_def(key))
            .map(|k| (k.range_min, k.range_max))
            .unwrap_or((None, None))
    }

    /// Check if a reading is within the valid range for this device type + key.
    pub fn is_in_range(&self, device_type: &str, key: &str, value: f64) -> bool {
        self.profiles
            .get(device_type)
            .and_then(|p| p.key_def(key))
            .map(|k| k.is_in_range(value))
            .unwrap_or(true) // unknown type/key → no violation
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_profiles_registered() {
        let reg = ProfileRegistry::default().with_builtins();
        assert!(reg.get("temperature_sensor").is_some());
        assert!(reg.get("wind_turbine").is_some());
        assert!(reg.get("unknown_type").is_none());
    }

    #[test]
    fn key_def_lookup() {
        let reg = ProfileRegistry::default().with_builtins();
        let prof = reg.get("wind_turbine").expect("profile");
        let rpm = prof.key_def("rpm").expect("key def");
        assert_eq!(rpm.unit, "RPM");
        assert_eq!(rpm.range_max, Some(200.0));
    }

    #[test]
    fn is_in_range_valid() {
        let def = TelemetryKeyDef {
            key: "t".into(), display_name: "T".into(), unit: "°C".into(),
            range_min: Some(-40.0), range_max: Some(125.0), color: None,
        };
        assert!(def.is_in_range(25.0));
        assert!(!def.is_in_range(130.0));
        assert!(!def.is_in_range(-50.0));
    }

    #[test]
    fn is_in_range_unbounded() {
        let def = TelemetryKeyDef {
            key: "k".into(), display_name: "K".into(), unit: "".into(),
            range_min: None, range_max: None, color: None,
        };
        assert!(def.is_in_range(f64::MAX));
        assert!(def.is_in_range(f64::MIN));
    }

    #[test]
    fn temperature_sensor_has_3_keys() {
        let reg = ProfileRegistry::default().with_builtins();
        let prof = reg.get("temperature_sensor").expect("profile");
        assert_eq!(prof.keys.len(), 3);
    }

    #[test]
    fn wind_turbine_has_4_keys() {
        let reg = ProfileRegistry::default().with_builtins();
        let prof = reg.get("wind_turbine").expect("profile");
        assert_eq!(prof.keys.len(), 4);
    }

    #[test]
    fn registry_is_in_range() {
        let reg = ProfileRegistry::default().with_builtins();
        assert!(reg.is_in_range("temperature_sensor", "temperature", 25.0));
        assert!(!reg.is_in_range("temperature_sensor", "temperature", 200.0));
    }

    #[test]
    fn range_label_format() {
        let def = TelemetryKeyDef {
            key: "t".into(), display_name: "T".into(), unit: "°C".into(),
            range_min: Some(-40.0), range_max: Some(125.0), color: None,
        };
        assert!(def.range_label().contains("-40"));
        assert!(def.range_label().contains("125"));
    }

    // ── key_names ─────────────────────────────────────────────────────────────

    #[test]
    fn profile_key_names_returns_all() {
        let reg  = ProfileRegistry::default().with_builtins();
        let prof = reg.get("temperature_sensor").expect("profile");
        let names = prof.key_names();
        assert!(names.contains(&"temperature"), "should include temperature");
        assert!(names.contains(&"humidity"),    "should include humidity");
        assert!(names.contains(&"battery"),     "should include battery");
        assert_eq!(names.len(), 3);
    }

    #[test]
    fn profile_wind_turbine_key_names() {
        let reg  = ProfileRegistry::default().with_builtins();
        let prof = reg.get("wind_turbine").expect("profile");
        let names = prof.key_names();
        assert!(names.contains(&"rpm"));
        assert!(names.contains(&"power_kw"));
        assert!(names.contains(&"wind_speed"));
        assert!(names.contains(&"vibration"));
    }

    // ── range_for ─────────────────────────────────────────────────────────────

    #[test]
    fn range_for_known_key() {
        let reg = ProfileRegistry::default().with_builtins();
        let (min, max) = reg.range_for("wind_turbine", "rpm");
        assert_eq!(min, Some(0.0));
        assert_eq!(max, Some(200.0));
    }

    #[test]
    fn range_for_unknown_device_type() {
        let reg = ProfileRegistry::default().with_builtins();
        let (min, max) = reg.range_for("nonexistent", "rpm");
        assert!(min.is_none());
        assert!(max.is_none());
    }

    #[test]
    fn range_for_unknown_key_on_known_type() {
        let reg = ProfileRegistry::default().with_builtins();
        let (min, max) = reg.range_for("temperature_sensor", "nonexistent_key");
        assert!(min.is_none());
        assert!(max.is_none());
    }

    // ── Color field ───────────────────────────────────────────────────────────

    #[test]
    fn temperature_key_has_color() {
        let reg  = ProfileRegistry::default().with_builtins();
        let prof = reg.get("temperature_sensor").unwrap();
        let key  = prof.key_def("temperature").unwrap();
        let [r, g, b, a] = key.color.expect("color should be set");
        // Warm reddish color
        assert!(r > g, "red component should dominate for temperature");
        assert_eq!(a, 1.0);
    }

    // ── Custom profile registration ───────────────────────────────────────────

    #[test]
    fn custom_profile_can_be_registered() {
        let mut reg = ProfileRegistry::default();
        reg.profiles.insert(
            "pressure_sensor".into(),
            DeviceProfile {
                name: "Pressure Sensor".into(),
                icon: None,
                keys: vec![TelemetryKeyDef {
                    key:          "pressure".into(),
                    display_name: "Pressure".into(),
                    unit:         "Pa".into(),
                    range_min:    Some(80000.0),
                    range_max:    Some(110000.0),
                    color:        None,
                }],
            },
        );
        assert!(reg.get("pressure_sensor").is_some());
        assert_eq!(reg.get("pressure_sensor").unwrap().keys.len(), 1);
        assert!(reg.is_in_range("pressure_sensor", "pressure", 101325.0));
        assert!(!reg.is_in_range("pressure_sensor", "pressure", 50000.0));
    }

    // ── TelemetryKeyDef range variants ────────────────────────────────────────

    #[test]
    fn range_only_min_bounded() {
        let def = TelemetryKeyDef {
            key: "k".into(), display_name: "K".into(), unit: "".into(),
            range_min: Some(0.0), range_max: None, color: None,
        };
        assert!(!def.is_in_range(-1.0)); // below min
        assert!(def.is_in_range(0.0));   // at min
        assert!(def.is_in_range(1e9));   // no upper bound
    }

    #[test]
    fn range_only_max_bounded() {
        let def = TelemetryKeyDef {
            key: "k".into(), display_name: "K".into(), unit: "".into(),
            range_min: None, range_max: Some(100.0), color: None,
        };
        assert!(def.is_in_range(f64::NEG_INFINITY)); // no lower bound
        assert!(def.is_in_range(100.0));              // at max
        assert!(!def.is_in_range(100.01));            // above max
    }

    #[test]
    fn range_label_only_min() {
        let def = TelemetryKeyDef {
            key: "k".into(), display_name: "".into(), unit: "m/s".into(),
            range_min: Some(0.0), range_max: None, color: None,
        };
        let label = def.range_label();
        assert!(label.contains("0"), "should show min value");
        assert!(label.contains("m/s"));
    }

    // ── Profile TOML roundtrip ────────────────────────────────────────────────

    #[test]
    fn telemetry_key_def_toml_roundtrip() {
        let def = TelemetryKeyDef {
            key:          "temperature".into(),
            display_name: "Temperature".into(),
            unit:         "°C".into(),
            range_min:    Some(-40.0),
            range_max:    Some(125.0),
            color:        Some([1.0, 0.3, 0.1, 1.0]),
        };
        let toml_str  = toml::to_string_pretty(&def).expect("serialize");
        let recovered: TelemetryKeyDef = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(recovered.key,       "temperature");
        assert_eq!(recovered.unit,      "°C");
        assert_eq!(recovered.range_min, Some(-40.0));
        assert_eq!(recovered.range_max, Some(125.0));
        assert!(recovered.color.is_some());
    }
}
