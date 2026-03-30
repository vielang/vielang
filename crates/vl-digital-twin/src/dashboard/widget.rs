//! Widget type system for the configurable dashboard (Phase 28).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique widget instance identifier.
pub type WidgetId = u32;

/// Serializable widget descriptor — defines what a widget shows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WidgetKind {
    /// Line chart of telemetry keys over time.
    TelemetryChart {
        device_id: Option<Uuid>,
        keys:      Vec<String>,
        title:     String,
    },
    /// Table of active alarms.
    AlarmTable {
        /// None = all devices.
        device_id: Option<Uuid>,
        max_rows:  usize,
    },
    /// Device attribute info card.
    DeviceInfo {
        device_id: Uuid,
    },
    /// Numeric stat card — shows current value of one telemetry key.
    StatCard {
        device_id: Uuid,
        key:       String,
        unit:      String,
        title:     String,
    },
    /// 2D heatmap configuration shortcut widget.
    HeatmapControl,
    /// Embedded map view (Phase 27).
    MapWidget {
        /// [lat, lon] of map center.
        center: [f64; 2],
        zoom:   f64,
    },
    /// The Bevy 3D scene viewport (always present, non-removable).
    Scene3D,

    // ── Phase 40: Industrial dashboard widgets ───────────────────────────────

    /// Radial gauge — shows a single value within a defined range.
    Gauge {
        device_id: Uuid,
        key:       String,
        unit:      String,
        title:     String,
        min_value: f64,
        max_value: f64,
        /// Warning threshold (yellow zone starts here).
        warning_threshold: Option<f64>,
        /// Critical threshold (red zone starts here).
        critical_threshold: Option<f64>,
    },

    /// Pie / donut chart — shows distribution of values across devices or keys.
    PieChart {
        title: String,
        /// device_id → key to read from each device.
        sources: Vec<PieSource>,
        /// If true, render as a donut instead of full pie.
        donut: bool,
    },

    /// Scatter plot — correlate two telemetry keys.
    ScatterPlot {
        title:    String,
        x_key:    String,
        y_key:    String,
        x_unit:   String,
        y_unit:   String,
        /// If set, only show data from these devices.
        device_ids: Vec<Uuid>,
    },

    /// Data table — tabular view of device telemetry with sorting/filtering.
    DataTable {
        title:    String,
        /// Columns to display (key names).
        columns:  Vec<TableColumn>,
        /// If set, only show these devices.
        device_ids: Vec<Uuid>,
        max_rows: usize,
        /// Enable sorting.
        sortable: bool,
    },

    /// KPI indicator — shows a metric with trend arrow and comparison.
    KpiIndicator {
        device_id: Uuid,
        key:       String,
        title:     String,
        unit:      String,
        /// Target/goal value for comparison.
        target:    Option<f64>,
        /// Time range for trend calculation (ms).
        trend_window_ms: u64,
    },

    /// Sparkline — compact inline chart without axes.
    Sparkline {
        device_id: Uuid,
        key:       String,
        title:     String,
        /// Number of recent data points to show.
        points:    usize,
    },

    /// Status grid — overview of multiple devices in compact colored cells.
    StatusGrid {
        title: String,
        /// Device IDs to show. Empty = all.
        device_ids: Vec<Uuid>,
        /// Telemetry key to base color on.
        color_key: Option<String>,
    },

    /// RUL (Remaining Useful Life) display — shows predictive maintenance info.
    RulDisplay {
        device_id:         Uuid,
        degradation_key:   String,
        failure_threshold: f64,
        title:             String,
    },
}

/// Source for a pie chart slice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PieSource {
    pub device_id: Uuid,
    pub key:       String,
    pub label:     String,
}

/// Column definition for the data table widget.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableColumn {
    pub key:          String,
    pub display_name: String,
    pub unit:         Option<String>,
    /// Format: "%.2f", "%d", etc.
    pub format:       Option<String>,
}

impl WidgetKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            WidgetKind::TelemetryChart { .. } => "Telemetry Chart",
            WidgetKind::AlarmTable { .. }     => "Alarm Table",
            WidgetKind::DeviceInfo { .. }     => "Device Info",
            WidgetKind::StatCard { .. }       => "Stat Card",
            WidgetKind::HeatmapControl        => "Heatmap Control",
            WidgetKind::MapWidget { .. }      => "Map",
            WidgetKind::Scene3D               => "3D Scene",
            WidgetKind::Gauge { .. }          => "Gauge",
            WidgetKind::PieChart { .. }       => "Pie Chart",
            WidgetKind::ScatterPlot { .. }    => "Scatter Plot",
            WidgetKind::DataTable { .. }      => "Data Table",
            WidgetKind::KpiIndicator { .. }   => "KPI Indicator",
            WidgetKind::Sparkline { .. }      => "Sparkline",
            WidgetKind::StatusGrid { .. }     => "Status Grid",
            WidgetKind::RulDisplay { .. }     => "RUL Display",
        }
    }
}

/// Grid cell position and size within a GRID_COLS × GRID_ROWS grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridRect {
    /// 0-based starting column.
    pub col:  u16,
    /// 0-based starting row.
    pub row:  u16,
    /// Width in grid columns.
    pub cols: u16,
    /// Height in grid rows.
    pub rows: u16,
}

/// One widget instance in the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id:   WidgetId,
    pub rect: GridRect,
    pub kind: WidgetKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_kind_toml_roundtrip() {
        let kind = WidgetKind::StatCard {
            device_id: Uuid::nil(),
            key:   "temperature".into(),
            unit:  "°C".into(),
            title: "Temp".into(),
        };
        let w = Widget { id: 1, rect: GridRect { col: 0, row: 0, cols: 4, rows: 2 }, kind };
        let s = toml::to_string_pretty(&w).expect("serialize");
        let w2: Widget = toml::from_str(&s).expect("deserialize");
        assert_eq!(w2.id, 1);
        assert!(matches!(w2.kind, WidgetKind::StatCard { .. }));
    }
}
