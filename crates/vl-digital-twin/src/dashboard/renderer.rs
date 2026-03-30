//! Dashboard grid renderer — draws each widget at its grid position.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

use crate::alarm::AlarmRegistry;
use crate::api::HistoricalDataCache;
use crate::components::{DeviceEntity, TelemetryData};
use crate::systems::visual_update::HeatmapConfig;
use crate::telemetry::DeviceKeyRegistry;
use crate::ui::device_panel::SelectedDevice;

use super::layout::{ActiveDashboard, GRID_COLS, GRID_ROWS};
use super::widget::{GridRect, WidgetKind};

// ── Grid math ─────────────────────────────────────────────────────────────────

/// Convert a grid rect to pixel coordinates within the viewport.
pub fn grid_cell_to_px(rect: &GridRect, viewport: egui::Rect) -> egui::Rect {
    let cell_w = viewport.width()  / GRID_COLS as f32;
    let cell_h = viewport.height() / GRID_ROWS as f32;
    egui::Rect::from_min_max(
        egui::pos2(
            viewport.left() + rect.col as f32 * cell_w,
            viewport.top()  + rect.row as f32 * cell_h,
        ),
        egui::pos2(
            viewport.left() + (rect.col + rect.cols) as f32 * cell_w,
            viewport.top()  + (rect.row + rect.rows) as f32 * cell_h,
        ),
    )
}

// ── System ────────────────────────────────────────────────────────────────────

/// Render all dashboard widgets each frame.
pub fn render_dashboard(
    mut contexts:  EguiContexts,
    dashboard:     Res<ActiveDashboard>,
    telemetry_q:   Query<(&DeviceEntity, &TelemetryData)>,
    alarms:        Res<AlarmRegistry>,
    hist_cache:    Res<HistoricalDataCache>,
    key_registry:  Res<DeviceKeyRegistry>,
    selected:      Res<SelectedDevice>,
    heatmap_cfg:   Res<HeatmapConfig>,
) {
    let ctx = contexts.ctx_mut().expect("egui context");
    let viewport = ctx.viewport_rect();

    // Collect telemetry data to avoid borrow conflicts inside closures
    let telem_data: Vec<(Uuid, String, std::collections::HashMap<String, f64>)> = telemetry_q
        .iter()
        .map(|(dev, td)| (dev.device_id, dev.name.clone(), td.values.clone()))
        .collect();

    for widget in &dashboard.0.widgets {
        let px = grid_cell_to_px(&widget.rect, viewport);

        match &widget.kind {
            WidgetKind::Scene3D => {
                // 3D scene renders behind egui — just draw a light border as placeholder
                render_scene3d_border(ctx, px);
            }

            WidgetKind::TelemetryChart { device_id, keys, title } => {
                render_telemetry_chart(ctx, px, widget.id, *device_id, keys, title, &telem_data);
            }

            WidgetKind::AlarmTable { device_id, max_rows } => {
                render_alarm_table(ctx, px, widget.id, *device_id, *max_rows, &alarms, &telem_data);
            }

            WidgetKind::DeviceInfo { device_id } => {
                render_device_info(ctx, px, widget.id, *device_id, &telem_data, &key_registry);
            }

            WidgetKind::StatCard { device_id, key, unit, title } => {
                render_stat_card(ctx, px, widget.id, *device_id, key, unit, title, &telem_data);
            }

            WidgetKind::HeatmapControl => {
                render_heatmap_control(ctx, px, widget.id, &heatmap_cfg);
            }

            WidgetKind::MapWidget { center, zoom } => {
                render_map_info(ctx, px, widget.id, center, *zoom);
            }

            // Phase 40: new industrial widgets — render placeholders
            WidgetKind::Gauge { title, .. }
            | WidgetKind::KpiIndicator { title, .. } => {
                render_placeholder(ctx, px, widget.id, title);
            }
            WidgetKind::PieChart { title, .. }
            | WidgetKind::ScatterPlot { title, .. }
            | WidgetKind::DataTable { title, .. }
            | WidgetKind::StatusGrid { title, .. }
            | WidgetKind::RulDisplay { title, .. } => {
                render_placeholder(ctx, px, widget.id, title);
            }
            WidgetKind::Sparkline { title, .. } => {
                render_placeholder(ctx, px, widget.id, title);
            }
        }
    }

    // Ignore unused warnings from system params when disabled
    let _ = (hist_cache, selected);
}

// ── Widget renderers ──────────────────────────────────────────────────────────

fn render_scene3d_border(ctx: &egui::Context, px: egui::Rect) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    painter.rect_stroke(px, 2.0, egui::Stroke::new(1.0, egui::Color32::from_gray(80)), egui::StrokeKind::Middle);
}

fn render_telemetry_chart(
    ctx:       &egui::Context,
    px:        egui::Rect,
    id:        u32,
    device_id: Option<Uuid>,
    keys:      &[String],
    title:     &str,
    telem:     &[(Uuid, String, std::collections::HashMap<String, f64>)],
) {
    egui::Window::new(format!("##tc_{id}"))
        .title_bar(true)
        .fixed_pos(px.min)
        .default_size(px.size())
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(title).strong());
            ui.separator();
            for row in telem.iter().filter(|(did, _, _)| device_id.map_or(true, |d| d == *did)) {
                for key in keys {
                    if let Some(v) = row.2.get(key) {
                        ui.label(format!("{} — {}: {:.2}", row.1, key, v));
                    }
                }
            }
        });
}

fn render_alarm_table(
    ctx:       &egui::Context,
    px:        egui::Rect,
    id:        u32,
    device_id: Option<Uuid>,
    max_rows:  usize,
    alarms:    &AlarmRegistry,
    telem:     &[(Uuid, String, std::collections::HashMap<String, f64>)],
) {
    egui::Window::new(format!("##at_{id}"))
        .title_bar(true)
        .fixed_pos(px.min)
        .default_size(px.size())
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Active Alarms").strong());
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let records: Vec<_> = alarms.alarms.iter()
                    .filter(|r| device_id.map_or(true, |d| d == r.device_id))
                    .filter(|r| r.status.is_active())
                    .take(max_rows)
                    .collect();

                if records.is_empty() {
                    ui.colored_label(egui::Color32::GREEN, "No active alarms");
                } else {
                    for rec in records {
                        // look up device name
                        let name = telem.iter()
                            .find(|(did, _, _)| *did == rec.device_id)
                            .map(|(_, n, _)| n.as_str())
                            .unwrap_or("?");
                        ui.label(format!(
                            "[{}] {} — {} ({})",
                            rec.severity, name, rec.alarm_type, rec.status
                        ));
                    }
                }
            });
        });
}

fn render_device_info(
    ctx:          &egui::Context,
    px:           egui::Rect,
    id:           u32,
    device_id:    Uuid,
    telem:        &[(Uuid, String, std::collections::HashMap<String, f64>)],
    key_registry: &DeviceKeyRegistry,
) {
    egui::Window::new(format!("##di_{id}"))
        .title_bar(true)
        .fixed_pos(px.min)
        .default_size(px.size())
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            if let Some((_, name, values)) = telem.iter().find(|(did, _, _)| *did == device_id) {
                ui.label(egui::RichText::new(name).strong().size(14.0));
                ui.separator();
                let keys = key_registry.keys_for(device_id);
                if keys.is_empty() {
                    ui.label("No telemetry received yet.");
                } else {
                    egui::Grid::new(format!("di_grid_{id}")).show(ui, |ui| {
                        for key in &keys {
                            ui.label(key);
                            if let Some(v) = values.get(key) {
                                ui.label(format!("{v:.2}"));
                            } else {
                                ui.label("—");
                            }
                            ui.end_row();
                        }
                    });
                }
            } else {
                ui.label("Select a device to view info.");
            }
        });
}

fn render_stat_card(
    ctx:       &egui::Context,
    px:        egui::Rect,
    id:        u32,
    device_id: Uuid,
    key:       &str,
    unit:      &str,
    title:     &str,
    telem:     &[(Uuid, String, std::collections::HashMap<String, f64>)],
) {
    let value = telem.iter()
        .find(|(did, _, _)| *did == device_id)
        .and_then(|(_, _, values)| values.get(key).copied());

    egui::Window::new(format!("##sc_{id}"))
        .title_bar(false)
        .fixed_pos(px.min)
        .default_size(px.size())
        .resizable(false)
        .collapsible(false)
        .frame(egui::Frame::window(&ctx.style()))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(title).size(12.0).color(egui::Color32::GRAY));
                ui.add_space(4.0);
                match value {
                    Some(v) => {
                        ui.label(
                            egui::RichText::new(format!("{v:.1} {unit}"))
                                .size(28.0)
                                .strong(),
                        );
                    }
                    None => {
                        ui.label(egui::RichText::new("—").size(28.0).color(egui::Color32::GRAY));
                    }
                }
            });
        });
}

fn render_heatmap_control(
    ctx:        &egui::Context,
    px:         egui::Rect,
    id:         u32,
    heatmap:    &HeatmapConfig,
) {
    egui::Window::new(format!("##hm_{id}"))
        .title_bar(true)
        .fixed_pos(px.min)
        .default_size(px.size())
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Heatmap").strong());
            ui.separator();
            ui.label(format!("Key: {}", heatmap.active_key));
            ui.label(format!("Range: {:.1} – {:.1}", heatmap.range_min, heatmap.range_max));
            ui.label(format!("Enabled: {}", heatmap.enabled));
            ui.label("(Edit via HeatmapConfig resource)");
        });
}

fn render_map_info(
    ctx:    &egui::Context,
    px:     egui::Rect,
    id:     u32,
    center: &[f64; 2],
    zoom:   f64,
) {
    egui::Window::new(format!("##mw_{id}"))
        .title_bar(true)
        .fixed_pos(px.min)
        .default_size(px.size())
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Map").strong());
            ui.separator();
            ui.label(format!("Center: {:.4}°N {:.4}°E", center[0], center[1]));
            ui.label(format!("Zoom: {:.0}", zoom));
            ui.label("Press M to open full map view.");
        });
}

/// Render a placeholder widget for new widget types (Phase 40+).
fn render_placeholder(
    ctx:   &egui::Context,
    px:    egui::Rect,
    id:    u32,
    title: &str,
) {
    egui::Window::new(format!("##ph_{id}"))
        .title_bar(true)
        .fixed_pos(px.min)
        .default_size(px.size())
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(title).strong());
            ui.separator();
            ui.label("Widget rendering coming soon.");
        });
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_cell_to_px_basic() {
        let viewport = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1200.0, 800.0));
        let rect = GridRect { col: 0, row: 0, cols: GRID_COLS, rows: GRID_ROWS };
        let px = grid_cell_to_px(&rect, viewport);
        assert!((px.width() - 1200.0).abs() < 0.01);
        assert!((px.height() - 800.0).abs() < 0.01);
    }

    #[test]
    fn grid_cell_to_px_partial() {
        let viewport = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1200.0, 800.0));
        let rect = GridRect { col: 0, row: 0, cols: 6, rows: 4 };
        let px = grid_cell_to_px(&rect, viewport);
        // Width should be half the viewport
        assert!((px.width() - 600.0).abs() < 0.01);
        assert!((px.height() - 400.0).abs() < 0.01);
    }
}
