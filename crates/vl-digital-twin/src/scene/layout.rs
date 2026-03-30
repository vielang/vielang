//! Scene layout persistence — save/load device positions as TOML.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::background::BackgroundSceneEntry;
use crate::asset_hierarchy::AssetNodeEntry;

// ── Data structures ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneLayout {
    pub meta: LayoutMeta,
    #[serde(default)]
    pub devices: Vec<DeviceLayoutEntry>,
    #[serde(default)]
    pub camera: CameraLayout,
    /// Background environment scenes (floor plans, building shells, etc.).
    #[serde(default)]
    pub backgrounds: Vec<BackgroundSceneEntry>,
    /// Phase 33: asset hierarchy nodes (Site / Building / Floor / Zone).
    #[serde(default)]
    pub assets: Vec<AssetNodeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutMeta {
    pub version:    u32,
    pub created_at: String,
    pub last_saved: String,
    pub name:       String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLayoutEntry {
    pub id:       Uuid,
    /// Stored for debugging only — not used as key
    pub name:     String,
    pub position: [f32; 3],
    /// Euler XYZ degrees
    pub rotation: [f32; 3],
    pub scale:    f32,
    /// WGS-84 latitude in degrees (optional — enables map view marker).
    #[serde(default)]
    pub latitude:  Option<f64>,
    /// WGS-84 longitude in degrees (optional — enables map view marker).
    #[serde(default)]
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraLayout {
    pub position: [f32; 3],
    pub look_at:  [f32; 3],
}

impl Default for CameraLayout {
    fn default() -> Self {
        Self { position: [0.0, 15.0, 20.0], look_at: [0.0, 0.0, 0.0] }
    }
}

// ── SceneLayout impl ─────────────────────────────────────────────────────────

impl SceneLayout {
    /// Create a blank layout with sane defaults.
    pub fn new(name: impl Into<String>) -> Self {
        let now = Self::now_iso();
        Self {
            meta: LayoutMeta {
                version:    1,
                created_at: now.clone(),
                last_saved: now,
                name:       name.into(),
            },
            devices:     Vec::new(),
            camera:      CameraLayout::default(),
            backgrounds: Vec::new(),
            assets:      Vec::new(),
        }
    }

    /// Load the `default` profile from disk. Returns `None` if not found or invalid.
    pub fn load_default() -> Option<Self> {
        Self::load("default")
    }

    /// Load a named profile from disk.
    pub fn load(name: &str) -> Option<Self> {
        let path = layout_path(name);
        if !path.exists() { return None; }
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Save this layout as the `default` profile.
    pub fn save_default(&self) -> Result<(), String> {
        self.save("default")
    }

    /// Save this layout under the given profile name.
    pub fn save(&self, name: &str) -> Result<(), String> {
        let path = layout_path(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, content).map_err(|e| e.to_string())
    }

    /// Delete a named profile from disk.
    pub fn delete(name: &str) -> Result<(), String> {
        let path = layout_path(name);
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// List all saved profile names (file stems in the layouts directory).
    pub fn list_profiles() -> Vec<String> {
        let dir = layouts_dir();
        std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension()? == "toml" {
                    path.file_stem()?.to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    fn now_iso() -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

// ── Path helpers ─────────────────────────────────────────────────────────────

fn layouts_dir() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vielang")
        .join("layouts")
}

fn layout_path(name: &str) -> std::path::PathBuf {
    layouts_dir().join(format!("{name}.toml"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_layout_new_has_version_1() {
        let layout = SceneLayout::new("Test");
        assert_eq!(layout.meta.version, 1);
        assert_eq!(layout.meta.name, "Test");
        assert!(layout.devices.is_empty());
    }

    #[test]
    fn scene_layout_roundtrip_toml() {
        let mut layout = SceneLayout::new("Roundtrip");
        layout.devices.push(DeviceLayoutEntry {
            id:        Uuid::nil(),
            name:      "Sensor A".into(),
            position:  [1.0, 2.0, 3.0],
            rotation:  [0.0, 45.0, 0.0],
            scale:     1.0,
            latitude:  None,
            longitude: None,
        });
        let toml_str  = toml::to_string_pretty(&layout).expect("serialize");
        let recovered: SceneLayout = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(recovered.devices.len(), 1);
        assert_eq!(recovered.devices[0].position, [1.0, 2.0, 3.0]);
        assert_eq!(recovered.meta.name, "Roundtrip");
    }

    #[test]
    fn camera_layout_default_position() {
        let cam = CameraLayout::default();
        assert_eq!(cam.position, [0.0, 15.0, 20.0]);
        assert_eq!(cam.look_at,  [0.0, 0.0, 0.0]);
    }

    // ── Assets field ──────────────────────────────────────────────────────────

    #[test]
    fn scene_layout_new_has_empty_assets() {
        let layout = SceneLayout::new("Test");
        assert!(layout.assets.is_empty(), "new layout should have no assets");
    }

    #[test]
    fn scene_layout_with_assets_roundtrip() {
        let mut layout = SceneLayout::new("WithAssets");
        layout.assets.push(crate::asset_hierarchy::AssetNodeEntry {
            id:         Uuid::nil().to_string(),
            name:       "Hà Nội Site".into(),
            asset_type: "Site".into(),
            parent_id:  None,
        });
        layout.assets.push(crate::asset_hierarchy::AssetNodeEntry {
            id:         Uuid::new_v4().to_string(),
            name:       "Nhà máy điện gió".into(),
            asset_type: "Building".into(),
            parent_id:  Some(Uuid::nil().to_string()),
        });
        let toml_str  = toml::to_string_pretty(&layout).expect("serialize");
        let recovered: SceneLayout = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(recovered.assets.len(), 2);
        assert_eq!(recovered.assets[0].name, "Hà Nội Site");
        assert_eq!(recovered.assets[1].asset_type, "Building");
        assert_eq!(recovered.assets[1].parent_id, Some(Uuid::nil().to_string()));
    }

    #[test]
    fn scene_layout_old_toml_without_assets_loads_defaults() {
        // Old TOML files without 'assets' key should load cleanly with empty Vec
        let toml_str = r#"
[meta]
version    = 1
created_at = "2026-01-01T00:00:00Z"
last_saved = "2026-01-01T00:00:00Z"
name       = "Legacy Layout"

[camera]
position = [0.0, 15.0, 20.0]
look_at  = [0.0, 0.0, 0.0]
"#;
        let layout: SceneLayout = toml::from_str(toml_str).expect("should parse legacy TOML");
        assert!(layout.assets.is_empty(),      "missing assets field → empty vec");
        assert!(layout.backgrounds.is_empty(), "missing backgrounds field → empty vec");
        assert!(layout.devices.is_empty(),     "missing devices field → empty vec");
        assert_eq!(layout.meta.name, "Legacy Layout");
    }

    #[test]
    fn device_layout_entry_missing_optional_fields_loads_ok() {
        // DeviceLayoutEntry with only required fields (no latitude, longitude)
        let toml_str = r#"
[meta]
version    = 1
created_at = "2026-01-01T00:00:00Z"
last_saved = "2026-01-01T00:00:00Z"
name       = "Minimal"

[camera]
position = [0.0, 15.0, 20.0]
look_at  = [0.0, 0.0, 0.0]

[[devices]]
id       = "00000000-0000-0000-0000-000000000000"
name     = "Sensor A"
position = [1.0, 0.0, 0.0]
rotation = [0.0, 0.0, 0.0]
scale    = 1.0
"#;
        let layout: SceneLayout = toml::from_str(toml_str).expect("should parse minimal device");
        assert_eq!(layout.devices.len(), 1);
        assert!(layout.devices[0].latitude.is_none());
        assert!(layout.devices[0].longitude.is_none());
    }
}
