//! DashboardLayout — serializable grid layout with TOML persistence.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use super::widget::{GridRect, Widget, WidgetId, WidgetKind};

/// Number of columns in the dashboard grid.
pub const GRID_COLS: u16 = 12;
/// Number of rows in the dashboard grid.
pub const GRID_ROWS: u16 = 8;

// ── DashboardLayout ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub version: u32,
    pub name:    String,
    pub widgets: Vec<Widget>,
    next_id:     WidgetId,
}

impl Default for DashboardLayout {
    fn default() -> Self {
        let mut layout = Self {
            version: 1,
            name:    "Default".into(),
            widgets: Vec::new(),
            next_id: 1,
        };
        // Sensible default: 3D scene fills left ¾, alarm table top-right, heatmap bottom-right
        layout.add(WidgetKind::Scene3D, GridRect { col: 0, row: 0, cols: 9, rows: 8 });
        layout.add(
            WidgetKind::AlarmTable { device_id: None, max_rows: 20 },
            GridRect { col: 9, row: 0, cols: 3, rows: 4 },
        );
        layout.add(WidgetKind::HeatmapControl, GridRect { col: 9, row: 4, cols: 3, rows: 4 });
        layout
    }
}

impl DashboardLayout {
    pub fn add(&mut self, kind: WidgetKind, rect: GridRect) -> WidgetId {
        let id = self.next_id;
        self.next_id += 1;
        self.widgets.push(Widget { id, rect, kind });
        id
    }

    pub fn remove(&mut self, id: WidgetId) {
        self.widgets.retain(|w| w.id != id);
    }

    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut Widget> {
        self.widgets.iter_mut().find(|w| w.id == id)
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Save to `~/.local/share/vielang/dashboards/{name}.toml`.
    pub fn save(&self) -> std::io::Result<()> {
        let dir = dashboard_dir();
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.toml", self.name));
        let text = self.to_toml().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
        std::fs::write(path, text)
    }

    pub fn load(name: &str) -> std::io::Result<Self> {
        let path = dashboard_dir().join(format!("{name}.toml"));
        let text = std::fs::read_to_string(path)?;
        Self::from_toml(&text).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub fn list() -> Vec<String> {
        let dir = dashboard_dir();
        let Ok(entries) = std::fs::read_dir(&dir) else { return vec![] };
        entries
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| name.ends_with(".toml"))
            .map(|name| name.trim_end_matches(".toml").to_string())
            .collect()
    }
}

fn dashboard_dir() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vielang")
        .join("dashboards")
}

// ── Resource wrapper ──────────────────────────────────────────────────────────

/// Active dashboard — loaded at startup, editable via Ctrl+E.
#[derive(Resource)]
pub struct ActiveDashboard(pub DashboardLayout);

impl Default for ActiveDashboard {
    fn default() -> Self {
        let layout = DashboardLayout::load("Default").unwrap_or_default();
        Self(layout)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_has_three_widgets() {
        let layout = DashboardLayout::default();
        assert_eq!(layout.widgets.len(), 3);
        assert!(layout.widgets.iter().any(|w| matches!(w.kind, WidgetKind::Scene3D)));
    }

    #[test]
    fn add_and_remove_widget() {
        let mut layout = DashboardLayout::default();
        let id = layout.add(
            WidgetKind::HeatmapControl,
            GridRect { col: 0, row: 0, cols: 4, rows: 4 },
        );
        assert!(layout.widgets.iter().any(|w| w.id == id));
        layout.remove(id);
        assert!(!layout.widgets.iter().any(|w| w.id == id));
    }

    #[test]
    fn toml_roundtrip() {
        let layout = DashboardLayout::default();
        let s = layout.to_toml().expect("serialize");
        let recovered = DashboardLayout::from_toml(&s).expect("deserialize");
        assert_eq!(recovered.widgets.len(), layout.widgets.len());
        assert_eq!(recovered.name, layout.name);
    }

    #[test]
    fn grid_cols_rows_constants() {
        assert_eq!(GRID_COLS, 12);
        assert_eq!(GRID_ROWS, 8);
    }
}
