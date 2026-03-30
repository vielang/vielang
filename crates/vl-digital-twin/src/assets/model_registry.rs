//! ModelRegistry — maps device_type string to GLTF asset path.

use std::collections::HashMap;

use bevy::prelude::*;

/// Maps device_type strings to their GLTF/GLB asset paths.
///
/// Asset paths are relative to the `assets/` folder.
/// If a device_type has no entry, `default_path` is used as fallback.
#[derive(Resource, Debug)]
pub struct ModelRegistry {
    entries:      HashMap<String, String>,
    default_path: String,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        let mut entries = HashMap::new();
        entries.insert(
            "temperature_sensor".into(),
            "models/temperature_sensor.glb#Scene0".into(),
        );
        entries.insert(
            "wind_turbine".into(),
            "models/wind_turbine.glb#Scene0".into(),
        );
        entries.insert("pump".into(), "models/pump.glb#Scene0".into());
        entries.insert("valve".into(), "models/valve.glb#Scene0".into());
        entries.insert("camera".into(), "models/camera.glb#Scene0".into());
        Self {
            entries,
            default_path: "models/default_device.glb#Scene0".into(),
        }
    }
}

impl ModelRegistry {
    /// Return the GLTF asset path for the given device_type.
    /// Falls back to `default_path` if the type is unknown.
    pub fn asset_path(&self, device_type: &str) -> &str {
        self.entries
            .get(device_type)
            .map(|s| s.as_str())
            .unwrap_or(&self.default_path)
    }

    /// Register or override a device_type → asset path mapping at runtime.
    pub fn register(&mut self, device_type: impl Into<String>, asset_path: impl Into<String>) {
        self.entries.insert(device_type.into(), asset_path.into());
    }
}
