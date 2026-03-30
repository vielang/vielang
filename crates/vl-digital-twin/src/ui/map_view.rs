//! Phase 27 — Geospatial Map View.
//!
//! Renders an egui window showing device positions on a 2D map.
//! Coordinate math uses Web Mercator (EPSG:3857) — same projection as OSM tiles.
//! Press **M** to toggle.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

use crate::components::{AlarmIndicator, DeviceEntity};
use crate::ui::device_panel::SelectedDevice;

// ── Resource ─────────────────────────────────────────────────────────────────

/// State for the geospatial map panel.
#[derive(Resource, Debug, Clone)]
pub struct MapViewState {
    /// Whether the map window is open.
    pub visible:    bool,
    /// Map center latitude (WGS-84 degrees).
    pub center_lat: f64,
    /// Map center longitude (WGS-84 degrees).
    pub center_lon: f64,
    /// Zoom scale — pixels per world unit (world unit = full globe width).
    /// Default 512.0 ≈ zoom-level 9 at 512 px canvas width.
    pub zoom:       f64,
}

impl Default for MapViewState {
    fn default() -> Self {
        Self {
            visible:    false,
            center_lat: 21.028,   // Hà Nội
            center_lon: 105.834,
            zoom:       512.0,
        }
    }
}

// ── Coordinate helpers ────────────────────────────────────────────────────────

/// Convert WGS-84 lat/lon to Web Mercator world coordinates in [0, 1].
/// x=0 → 180°W, x=1 → 180°E; y=0 → north pole, y=1 → south pole.
fn lat_lon_to_world(lat_deg: f64, lon_deg: f64) -> (f64, f64) {
    let lat_rad = lat_deg.to_radians();
    let x = (lon_deg + 180.0) / 360.0;
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0;
    (x, y)
}

/// Project world coordinates to canvas-local pixel offset (relative to canvas top-left).
fn world_to_canvas(
    wx: f64, wy: f64,
    center_wx: f64, center_wy: f64,
    zoom: f64,
    canvas_size: egui::Vec2,
) -> egui::Pos2 {
    let px = (wx - center_wx) * zoom + canvas_size.x as f64 / 2.0;
    let py = (wy - center_wy) * zoom + canvas_size.y as f64 / 2.0;
    egui::pos2(px as f32, py as f32)
}

/// Convert a canvas drag delta back into world-space offset.
fn drag_to_world_delta(delta_px: egui::Vec2, zoom: f64) -> (f64, f64) {
    (-(delta_px.x as f64) / zoom, -(delta_px.y as f64) / zoom)
}

// ── System ────────────────────────────────────────────────────────────────────

/// Render the geospatial map panel. No-ops when `map_state.visible == false`.
pub fn render_map_panel(
    mut contexts:  EguiContexts,
    mut map_state: ResMut<MapViewState>,
    device_query:  Query<(Entity, &DeviceEntity, &AlarmIndicator)>,
    mut selected:  ResMut<SelectedDevice>,
) {
    if !map_state.visible { return; }

    // Collect device data before entering egui closure to avoid borrow conflicts.
    #[derive(Clone)]
    struct DevMarker {
        entity:    Entity,
        device_id: Uuid,
        name:      String,
        lat:       f64,
        lon:       f64,
        dot_color: egui::Color32,
    }

    let markers: Vec<DevMarker> = device_query
        .iter()
        .filter_map(|(entity, dev, alarm)| {
            let lat = dev.latitude?;
            let lon = dev.longitude?;
            Some(DevMarker {
                entity,
                device_id: dev.device_id,
                name:      dev.name.clone(),
                lat,
                lon,
                dot_color: alarm.severity.to_egui_color(),
            })
        })
        .collect();

    let ctx = contexts.ctx_mut().expect("egui context");

    let (center_wx, center_wy) = lat_lon_to_world(map_state.center_lat, map_state.center_lon);
    let selected_id = selected.device_id;

    let mut new_selected: Option<DevMarker> = None;
    let mut new_zoom     = map_state.zoom;
    let mut new_center   = (map_state.center_lat, map_state.center_lon);
    let mut should_close = false;

    egui::Window::new("Map View")
        .default_size([620.0, 440.0])
        .min_size([320.0, 220.0])
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            // ── Controls bar ──────────────────────────────────────────────────
            ui.horizontal(|ui| {
                if ui.small_button("−").clicked() {
                    new_zoom = (new_zoom / 1.5).max(32.0);
                }
                ui.label(format!("Zoom ×{:.0}", new_zoom));
                if ui.small_button("+").clicked() {
                    new_zoom = (new_zoom * 1.5).min(2_000_000.0);
                }
                ui.separator();
                if ui.small_button("Reset").clicked() {
                    new_zoom = MapViewState::default().zoom;
                    new_center = (MapViewState::default().center_lat, MapViewState::default().center_lon);
                }
                ui.separator();
                ui.label(format!(
                    "{:.4}°N  {:.4}°E",
                    map_state.center_lat, map_state.center_lon
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").on_hover_text("Close (M)").clicked() {
                        should_close = true;
                    }
                });
            });

            ui.separator();

            // ── Map canvas ────────────────────────────────────────────────────
            let available = ui.available_size().max(egui::vec2(300.0, 200.0));
            let (response, painter) =
                ui.allocate_painter(available, egui::Sense::click_and_drag());
            let rect = response.rect;
            let size = rect.size();

            // Ocean background
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(168, 198, 230));

            // Subtle Mercator grid
            draw_grid(&painter, rect, center_wx, center_wy, new_zoom, size);

            // Pan
            if response.dragged() {
                let (dwx, dwy) = drag_to_world_delta(response.drag_delta(), new_zoom);
                let (new_cx, new_cy) = lat_lon_to_world(new_center.0, new_center.1);
                let (nwx, nwy) = (new_cx + dwx, new_cy + dwy);
                // Inverse Mercator: y → lat
                let lat = (std::f64::consts::PI * (1.0 - 2.0 * nwy)).sinh().atan().to_degrees();
                let lon = nwx * 360.0 - 180.0;
                new_center = (lat.clamp(-85.0, 85.0), lon.rem_euclid(360.0) - 180.0);
            }

            // Scroll zoom
            let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 && response.hovered() {
                let factor = if scroll > 0.0 { 1.1 } else { 1.0 / 1.1 };
                new_zoom = (new_zoom * factor).clamp(32.0, 2_000_000.0);
            }

            // ── Device markers ─────────────────────────────────────────────
            let click_pos = if response.clicked() {
                response.interact_pointer_pos()
            } else {
                None
            };

            for m in &markers {
                let (wx, wy) = lat_lon_to_world(m.lat, m.lon);
                let local = world_to_canvas(wx, wy, center_wx, center_wy, new_zoom, size);
                let screen = rect.min + local.to_vec2();

                if !rect.contains(screen) { continue; }

                let is_sel = selected_id == Some(m.device_id);
                let radius  = if is_sel { 10.0_f32 } else { 7.0_f32 };
                let outline = if is_sel {
                    egui::Stroke::new(3.0, egui::Color32::WHITE)
                } else {
                    egui::Stroke::new(1.5, egui::Color32::from_gray(60))
                };

                painter.circle_filled(screen, radius, m.dot_color);
                painter.circle_stroke(screen, radius, outline);

                // Label
                painter.text(
                    screen + egui::vec2(radius + 3.0, -6.0),
                    egui::Align2::LEFT_TOP,
                    &m.name,
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_gray(20),
                );

                // Click detection
                if let Some(pos) = click_pos {
                    let hit_r = radius + 5.0;
                    if (pos - screen).length() <= hit_r {
                        new_selected = Some(m.clone());
                    }
                }
            }

            // No-coordinate hint
            if markers.is_empty() {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No devices with coordinates.\nSet latitude/longitude on your devices.",
                    egui::FontId::proportional(13.0),
                    egui::Color32::from_gray(100),
                );
            }
        });

    // ── Apply mutations outside closure ───────────────────────────────────────
    if should_close {
        map_state.visible = false;
    }
    map_state.zoom       = new_zoom;
    map_state.center_lat = new_center.0;
    map_state.center_lon = new_center.1;

    if let Some(m) = new_selected {
        selected.entity    = Some(m.entity);
        selected.device_id = Some(m.device_id);
        selected.name      = m.name;
        tracing::info!(device = %selected.name, "Device selected via map");
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Draw a subtle Mercator grid over the map canvas.
fn draw_grid(
    painter:   &egui::Painter,
    rect:      egui::Rect,
    center_wx: f64,
    center_wy: f64,
    zoom:      f64,
    size:      egui::Vec2,
) {
    let grid_color = egui::Color32::from_rgba_premultiplied(140, 170, 200, 80);
    let stroke = egui::Stroke::new(1.0, grid_color);

    // Longitude lines every 10°
    for lon_deg in (-180..=180).step_by(10) {
        let wx = (lon_deg as f64 + 180.0) / 360.0;
        let local = world_to_canvas(wx, center_wy, center_wx, center_wy, zoom, size);
        let x = rect.min.x + local.x;
        if x >= rect.min.x && x <= rect.max.x {
            painter.line_segment(
                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                stroke,
            );
        }
    }

    // Latitude lines every 10°
    for lat_deg in (-80..=80).step_by(10) {
        let (_, wy) = lat_lon_to_world(lat_deg as f64, 0.0);
        let local = world_to_canvas(center_wx, wy, center_wx, center_wy, zoom, size);
        let y = rect.min.y + local.y;
        if y >= rect.min.y && y <= rect.max.y {
            painter.line_segment(
                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                stroke,
            );
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lat_lon_world_bounds() {
        // Equator / prime meridian → 0.5, 0.5
        let (x, y) = lat_lon_to_world(0.0, 0.0);
        assert!((x - 0.5).abs() < 1e-9);
        assert!((y - 0.5).abs() < 1e-9);
    }

    #[test]
    fn lat_lon_world_antimeridian() {
        let (x, _) = lat_lon_to_world(0.0, 180.0);
        assert!((x - 1.0).abs() < 1e-9);
        let (x2, _) = lat_lon_to_world(0.0, -180.0);
        assert!((x2 - 0.0).abs() < 1e-9);
    }

    #[test]
    fn map_view_state_default() {
        let s = MapViewState::default();
        assert!(!s.visible);
        assert!(s.zoom > 0.0);
    }
}
