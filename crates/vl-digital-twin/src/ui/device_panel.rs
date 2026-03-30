//! Right-side egui panel — shows selected device info + telemetry charts.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_plot::{Line, Plot, PlotPoints};
use uuid::Uuid;

use crate::api::HistoricalDataCache;
use crate::components::{current_time_ms, AlarmIndicator, DataFreshness, DeviceEntity, TelemetryData};
use crate::playback::PlaybackState;
use crate::telemetry::DeviceKeyRegistry;
use crate::ws::WsConnectionStatus;

/// Currently selected device (set by click or device list).
#[derive(Resource, Default)]
pub struct SelectedDevice {
    pub entity:    Option<Entity>,
    pub device_id: Option<Uuid>,
    pub name:      String,
}

/// Rolling telemetry history for charts — key: (device_id, telemetry_key) → [(ts_ms, value)].
#[derive(Resource, Default)]
pub struct TelemetryHistory {
    pub data: HashMap<(Uuid, String), Vec<(i64, f64)>>,
}

/// Per-device selected chart key (Phase 26 — user can switch between keys).
#[derive(Resource, Default)]
pub struct DevicePanelState {
    /// device_id → currently selected telemetry key for the chart
    pub chart_keys: HashMap<Uuid, String>,
}

/// Render the right-side device info panel.
pub fn render_device_panel(
    mut contexts:   EguiContexts,
    selected:       Res<SelectedDevice>,
    ws_status:      Res<WsConnectionStatus>,
    history:        Res<TelemetryHistory>,
    hist_cache:     Res<HistoricalDataCache>,
    playback:       Res<PlaybackState>,
    mut panel_state: ResMut<DevicePanelState>,
    key_registry:   Res<DeviceKeyRegistry>,
    query: Query<(&DeviceEntity, &TelemetryData, &AlarmIndicator, Option<&DataFreshness>)>,
) {
    let ctx = contexts.ctx_mut().expect("egui context");

    // Top status bar
    egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ws_status.connected {
                ui.colored_label(egui::Color32::GREEN, "● Connected");
            } else if ws_status.reconnecting {
                let retry_s = ws_status.next_retry_ms.unwrap_or(0) / 1000;
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("↻ Reconnecting... (attempt {}, retry in {}s)",
                        ws_status.reconnect_attempts, retry_s),
                );
            } else {
                ui.colored_label(egui::Color32::RED, "● Disconnected");
                if let Some(err) = &ws_status.error {
                    ui.separator();
                    ui.colored_label(egui::Color32::LIGHT_RED, format!("⚠ {err}"));
                }
            }
        });
    });

    // Right panel — device details
    let Some(device_id) = selected.device_id else { return };

    egui::SidePanel::right("device_panel")
        .min_width(320.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading(&selected.name);
            ui.small(format!("ID: {device_id}"));
            ui.separator();

            for (device, telemetry, alarm, freshness) in query.iter() {
                if device.device_id != device_id {
                    continue;
                }

                // ── Stale data warning ────────────────────────────────────────
                if let Some(f) = freshness {
                    if f.is_stale {
                        ui.colored_label(
                            egui::Color32::from_rgb(180, 180, 50),
                            format!("⚠ Data stale ({} s ago)", f.age_ms() / 1_000),
                        );
                        ui.separator();
                    }
                }

                // ── Alarm badge ───────────────────────────────────────────────
                if alarm.active {
                    let color = alarm.severity.to_egui_color();
                    ui.colored_label(
                        color,
                        format!(
                            "⚠ {} ({})",
                            alarm.alarm_type.as_deref().unwrap_or("Unknown alarm"),
                            alarm.severity
                        ),
                    );
                    ui.separator();
                }

                // ── Live telemetry grid ───────────────────────────────────────
                ui.label("Live Telemetry");
                if telemetry.values.is_empty() {
                    ui.weak("No telemetry data yet...");
                } else {
                    egui::Grid::new("telemetry_grid")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            // Sort keys for stable display
                            let mut pairs: Vec<_> = telemetry.values.iter().collect();
                            pairs.sort_by_key(|(k, _)| k.as_str());
                            for (key, value) in pairs {
                                ui.label(format!("{}:", key));
                                ui.strong(format!("{:.3}", value));
                                ui.end_row();
                            }
                        });
                }

                ui.separator();

                // ── Multi-key chart ───────────────────────────────────────────
                let available_keys = key_registry.keys_for(device_id);

                if !available_keys.is_empty() {
                    // Get or initialize the selected key for this device
                    let active_key = panel_state
                        .chart_keys
                        .entry(device_id)
                        .or_insert_with(|| available_keys[0].clone())
                        .clone();

                    // Key selector tabs
                    ui.horizontal_wrapped(|ui| {
                        for key in &available_keys {
                            let selected = *key == active_key;
                            if ui.selectable_label(selected, key).clicked() {
                                panel_state.chart_keys.insert(device_id, key.clone());
                            }
                        }
                    });

                    let chart_label = if playback.is_live() {
                        format!("{active_key} (live)")
                    } else {
                        format!("{active_key} (historical)")
                    };

                    let chart_points: PlotPoints = if playback.is_live() {
                        history.data
                            .get(&(device_id, active_key.clone()))
                            .map(|pts| {
                                let min_ts = pts.first().map(|(t, _)| *t).unwrap_or(0);
                                pts.iter()
                                    .map(|(ts, val)| [(ts - min_ts) as f64 / 1000.0, *val])
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default()
                            .into()
                    } else {
                        let at_ts  = playback.current_ts();
                        let start  = at_ts - 1_800_000; // 30 min window
                        let pts    = hist_cache.get_range(device_id, &active_key, start, at_ts);
                        let min_ts = pts.first().map(|p| p.ts).unwrap_or(0);
                        pts.iter()
                            .map(|p| [(p.ts - min_ts) as f64 / 1000.0, p.value])
                            .collect::<Vec<_>>()
                            .into()
                    };

                    if !matches!(&chart_points, PlotPoints::Owned(v) if v.is_empty()) {
                        ui.label(&chart_label);
                        Plot::new(format!("chart_{device_id}_{active_key}"))
                            .height(140.0)
                            .x_axis_label("time (s)")
                            .show(ui, |plot_ui| {
                                plot_ui.line(
                                    Line::new(active_key.as_str(), chart_points)
                                        .color(egui::Color32::LIGHT_BLUE),
                                );
                            });

                        if !playback.is_live() {
                            if let Some(val) = hist_cache.get_at(device_id, &active_key, playback.current_ts()) {
                                ui.small(format!("Value at cursor: {val:.3}"));
                            }
                        }
                    }
                }

                // ── Last updated ──────────────────────────────────────────────
                if telemetry.updated_at > 0 {
                    let age_ms = current_time_ms() - telemetry.updated_at;
                    let age_str = if age_ms < 1_000 {
                        format!("{age_ms} ms ago")
                    } else if age_ms < 60_000 {
                        format!("{} s ago", age_ms / 1_000)
                    } else {
                        format!("{} min ago", age_ms / 60_000)
                    };
                    ui.small(format!("Last update: {age_str}"));
                }

                break; // only one device matches
            }
        });
}
