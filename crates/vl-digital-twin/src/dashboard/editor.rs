//! Dashboard layout editor — toggled with Ctrl+E.
//!
//! Shows a floating editor panel with:
//! - Grid overlay over the viewport
//! - Widget list with remove buttons
//! - "Add Widget" picker
//! - Save / Discard buttons

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

use super::layout::{ActiveDashboard, DashboardLayout, GRID_COLS, GRID_ROWS};
use super::renderer::grid_cell_to_px;
use super::widget::{GridRect, WidgetKind};

// ── Resource ─────────────────────────────────────────────────────────────────

/// State for the dashboard layout editor (Ctrl+E).
#[derive(Resource, Default)]
pub struct DashboardEditorState {
    pub editing:        bool,
    pub show_add_panel: bool,
    /// Kind index selected in the "Add Widget" picker.
    pub add_kind_idx:   usize,
    /// Grid position for next added widget.
    pub add_col:        u16,
    pub add_row:        u16,
    pub add_cols:       u16,
    pub add_rows:       u16,
}

// ── System ────────────────────────────────────────────────────────────────────

/// Render the dashboard editor overlay when editing == true.
pub fn render_dashboard_editor(
    mut contexts:  EguiContexts,
    mut editor:    ResMut<DashboardEditorState>,
    mut dashboard: ResMut<ActiveDashboard>,
) {
    if !editor.editing { return; }

    let ctx = contexts.ctx_mut().expect("egui context");
    let viewport = ctx.screen_rect();

    // ── Grid overlay ──────────────────────────────────────────────────────────
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("dash_grid_overlay"),
    ));
    let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(80, 130, 200, 60));
    let cell_w = viewport.width()  / GRID_COLS as f32;
    let cell_h = viewport.height() / GRID_ROWS as f32;

    for col in 0..=GRID_COLS {
        let x = viewport.left() + col as f32 * cell_w;
        painter.line_segment(
            [egui::pos2(x, viewport.top()), egui::pos2(x, viewport.bottom())],
            grid_stroke,
        );
    }
    for row in 0..=GRID_ROWS {
        let y = viewport.top() + row as f32 * cell_h;
        painter.line_segment(
            [egui::pos2(viewport.left(), y), egui::pos2(viewport.right(), y)],
            grid_stroke,
        );
    }

    // Highlight each widget rect
    for widget in &dashboard.0.widgets {
        let px = grid_cell_to_px(&widget.rect, viewport);
        painter.rect_stroke(
            px.shrink(2.0),
            3.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgba_premultiplied(100, 180, 255, 120)),
            egui::StrokeKind::Middle,
        );
        painter.text(
            px.center_top() + egui::vec2(0.0, 4.0),
            egui::Align2::CENTER_TOP,
            widget.kind.display_name(),
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgba_premultiplied(200, 220, 255, 200),
        );
    }

    // ── Editor window ─────────────────────────────────────────────────────────
    let mut close = false;
    let mut widgets_to_remove: Vec<u32> = Vec::new();
    let mut widget_to_add: Option<(WidgetKind, GridRect)> = None;

    egui::Window::new("Dashboard Editor")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
        .default_width(280.0)
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Edit Mode (Ctrl+E to exit)").strong());
            ui.separator();

            // ── Widget list ───────────────────────────────────────────────────
            ui.label("Widgets:");
            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                egui::Grid::new("widget_list").num_columns(3).show(ui, |ui| {
                    for widget in &dashboard.0.widgets {
                        ui.label(format!("#{}", widget.id));
                        ui.label(widget.kind.display_name());
                        if widget.kind != WidgetKind::Scene3D {
                            if ui.small_button("✕").clicked() {
                                widgets_to_remove.push(widget.id);
                            }
                        } else {
                            ui.label(""); // Scene3D non-removable
                        }
                        ui.end_row();
                    }
                });
            });

            ui.separator();

            // ── Add widget ────────────────────────────────────────────────────
            ui.collapsing("Add Widget", |ui| {
                let kinds = available_widget_kinds();
                egui::ComboBox::from_label("Type")
                    .selected_text(kinds[editor.add_kind_idx].0)
                    .show_ui(ui, |ui| {
                        for (i, (label, _)) in kinds.iter().enumerate() {
                            ui.selectable_value(&mut editor.add_kind_idx, i, *label);
                        }
                    });

                ui.horizontal(|ui| {
                    ui.label("Col:");
                    ui.add(egui::DragValue::new(&mut editor.add_col).range(0..=GRID_COLS - 1));
                    ui.label("Row:");
                    ui.add(egui::DragValue::new(&mut editor.add_row).range(0..=GRID_ROWS - 1));
                });
                ui.horizontal(|ui| {
                    ui.label("W:");
                    ui.add(egui::DragValue::new(&mut editor.add_cols).range(1..=GRID_COLS));
                    ui.label("H:");
                    ui.add(egui::DragValue::new(&mut editor.add_rows).range(1..=GRID_ROWS));
                });

                if ui.button("Add").clicked() {
                    let rect = GridRect {
                        col:  editor.add_col.min(GRID_COLS - 1),
                        row:  editor.add_row.min(GRID_ROWS - 1),
                        cols: editor.add_cols.max(1),
                        rows: editor.add_rows.max(1),
                    };
                    let kind = (kinds[editor.add_kind_idx].1)();
                    widget_to_add = Some((kind, rect));
                }
            });

            ui.separator();

            // ── Save / discard ────────────────────────────────────────────────
            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    if let Err(e) = dashboard.0.save() {
                        tracing::warn!("Dashboard save failed: {e}");
                    } else {
                        tracing::info!("Dashboard saved: {}", dashboard.0.name);
                    }
                    close = true;
                }
                if ui.button("✕ Discard").clicked() {
                    // Reload from disk — if not found, use default
                    dashboard.0 = DashboardLayout::load(&dashboard.0.name).unwrap_or_default();
                    close = true;
                }
            });
        });

    // Apply deferred mutations
    for id in widgets_to_remove {
        dashboard.0.remove(id);
    }
    if let Some((kind, rect)) = widget_to_add {
        dashboard.0.add(kind, rect);
    }
    if close {
        editor.editing = false;
    }
}

// ── Available widget kinds for the picker ────────────────────────────────────

type KindFactory = (&'static str, fn() -> WidgetKind);

fn available_widget_kinds() -> Vec<KindFactory> {
    vec![
        ("Alarm Table",     || WidgetKind::AlarmTable { device_id: None, max_rows: 20 }),
        ("Heatmap Control", || WidgetKind::HeatmapControl),
        ("Map Widget",      || WidgetKind::MapWidget { center: [21.028, 105.834], zoom: 512.0 }),
        ("Stat Card",       || WidgetKind::StatCard {
            device_id: Uuid::nil(),
            key:   "temperature".into(),
            unit:  "°C".into(),
            title: "Temperature".into(),
        }),
        ("Device Info",     || WidgetKind::DeviceInfo { device_id: Uuid::nil() }),
        ("Telemetry Chart", || WidgetKind::TelemetryChart {
            device_id: None,
            keys:  vec!["temperature".into()],
            title: "Temperature".into(),
        }),
    ]
}

// ── Keyboard shortcut helper ──────────────────────────────────────────────────

/// Toggle editor when Ctrl+E is pressed.
pub fn toggle_dashboard_editor(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<DashboardEditorState>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && keyboard.just_pressed(KeyCode::KeyE) {
        editor.editing = !editor.editing;
        tracing::debug!("Dashboard editor: {}", if editor.editing { "open" } else { "closed" });
    }
}
