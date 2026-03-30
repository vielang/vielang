//! RPC command log panel — shows all RPC calls made in this session.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::events::RpcResult;
use crate::components::current_time_ms;

// ── Resource ──────────────────────────────────────────────────────────────────

/// One entry in the in-session RPC log.
#[derive(Debug, Clone)]
pub struct RpcLogEntry {
    pub sent_at:     i64,
    pub device_name: String,
    pub method:      String,
    pub result:      Result<serde_json::Value, String>,
}

/// Holds the in-session RPC call history.
#[derive(Resource, Default)]
pub struct RpcLogState {
    pub entries: Vec<RpcLogEntry>,
    /// Whether to show the log panel.
    pub visible: bool,
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Append incoming RpcResult events to the in-session log.
pub fn collect_rpc_results(
    mut events:    MessageReader<RpcResult>,
    mut log_state: ResMut<RpcLogState>,
) {
    for ev in events.read() {
        log_state.entries.push(RpcLogEntry {
            sent_at:     ev.sent_at,
            device_name: ev.device_name.clone(),
            method:      ev.method.clone(),
            result:      ev.result.clone(),
        });
    }
}

/// Render the RPC log panel in a floating egui window.
pub fn render_rpc_log(
    mut ctx:       EguiContexts,
    mut log_state: ResMut<RpcLogState>,
) {
    if !log_state.visible {
        return;
    }

    let ctx = ctx.ctx_mut().expect("egui context");

    egui::Window::new("RPC Command Log")
        .default_width(500.0)
        .default_height(250.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    log_state.entries.clear();
                }
                if ui.button("Close").clicked() {
                    log_state.visible = false;
                }
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    egui::Grid::new("rpc_log_grid")
                        .num_columns(4)
                        .striped(true)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            // Header
                            ui.strong("Time");
                            ui.strong("Device");
                            ui.strong("Method");
                            ui.strong("Result");
                            ui.end_row();

                            let now = current_time_ms();
                            for entry in log_state.entries.iter().rev().take(100) {
                                let age_secs = (now - entry.sent_at) / 1_000;
                                let time_str = if age_secs < 60 {
                                    format!("{age_secs}s ago")
                                } else {
                                    format!("{}m ago", age_secs / 60)
                                };

                                ui.label(&time_str);
                                ui.label(if entry.device_name.is_empty() {
                                    "—"
                                } else {
                                    &entry.device_name
                                });
                                ui.label(&entry.method);

                                match &entry.result {
                                    Ok(v) => {
                                        let text = if v.is_null() {
                                            "✓ (sent)".into()
                                        } else {
                                            format!("✓ {v}")
                                        };
                                        ui.colored_label(egui::Color32::GREEN, text);
                                    }
                                    Err(e) => {
                                        ui.colored_label(
                                            egui::Color32::RED,
                                            format!("✗ {e}"),
                                        );
                                    }
                                }
                                ui.end_row();
                            }
                        });
                });
        });
}
