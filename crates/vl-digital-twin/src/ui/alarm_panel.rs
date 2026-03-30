//! Alarm Management Panel — table with Ack/Clear buttons, severity filter.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

use crate::alarm::{AlarmRegistry, AlarmStatus};
use crate::components::DeviceEntity;

// ── State resource ─────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct AlarmPanelState {
    pub visible:       bool,
    pub filter_active: bool,
}

// ── System ─────────────────────────────────────────────────────────────────────

pub fn render_alarm_panel(
    mut contexts: EguiContexts,
    mut state:    ResMut<AlarmPanelState>,
    mut registry: ResMut<AlarmRegistry>,
    device_q:     Query<&DeviceEntity>,
) {
    if !state.visible { return; }
    let ctx = match contexts.ctx_mut() { Ok(c) => c, Err(_) => return };

    let mut open = state.visible;
    egui::Window::new("Alarm Management")
        .default_size([680.0, 420.0])
        .open(&mut open)
        .show(ctx, |ui| {
            // ── Toolbar ───────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.checkbox(&mut state.filter_active, "Active only");
                ui.separator();

                // Severity badge counts
                let counts = registry.active_count_by_severity();
                for sev in &["Critical", "Major", "Minor", "Warning"] {
                    if let Some(&n) = counts.get(*sev) {
                        let color = severity_egui_color(sev);
                        ui.colored_label(color, format!("{sev}: {n}"));
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Remove resolved").clicked() {
                        registry.alarms.retain(|a| a.status.is_active());
                    }
                });
            });

            ui.separator();

            // ── Alarm table ───────────────────────────────────────────────────
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    egui::Grid::new("alarm_grid")
                        .num_columns(6)
                        .striped(true)
                        .min_col_width(80.0)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            // Header row
                            ui.strong("Severity");
                            ui.strong("Device");
                            ui.strong("Alarm Type");
                            ui.strong("Status");
                            ui.strong("Ack");
                            ui.strong("Clear");
                            ui.end_row();

                            // Collect pending actions (can't borrow registry mutably inside iter)
                            let mut ack_key:   Option<(Uuid, String)> = None;
                            let mut clear_key: Option<(Uuid, String)> = None;

                            let alarms: Vec<_> = registry.alarms.iter()
                                .filter(|a| !state.filter_active || a.status.is_active())
                                .cloned()
                                .collect();

                            for alarm in &alarms {
                                let color = severity_egui_color(&alarm.severity.to_string());

                                ui.colored_label(color, alarm.severity.to_string());

                                let device_name = device_q.iter()
                                    .find(|d| d.device_id == alarm.device_id)
                                    .map(|d| d.name.as_str())
                                    .unwrap_or("Unknown");
                                ui.label(device_name);
                                ui.label(&alarm.alarm_type);

                                // Status cell
                                let status_text = alarm.status.to_string();
                                let status_color = match alarm.status {
                                    AlarmStatus::ActiveUnack  => egui::Color32::RED,
                                    AlarmStatus::ActiveAck    => egui::Color32::YELLOW,
                                    AlarmStatus::ClearedUnack => egui::Color32::LIGHT_GREEN,
                                    AlarmStatus::ClearedAck   => egui::Color32::GRAY,
                                };
                                ui.colored_label(status_color, status_text);

                                // Ack button
                                let can_ack = !alarm.acknowledged && alarm.status.is_active();
                                if ui.add_enabled(can_ack, egui::Button::new("Ack"))
                                    .on_disabled_hover_text("Already acknowledged or not active")
                                    .clicked()
                                {
                                    ack_key = Some((alarm.device_id, alarm.alarm_type.clone()));
                                }

                                // Clear button
                                let can_clear = !alarm.cleared;
                                if ui.add_enabled(can_clear, egui::Button::new("Clear"))
                                    .on_disabled_hover_text("Already cleared")
                                    .clicked()
                                {
                                    clear_key = Some((alarm.device_id, alarm.alarm_type.clone()));
                                }

                                ui.end_row();
                            }

                            if alarms.is_empty() {
                                ui.label(if state.filter_active {
                                    "No active alarms."
                                } else {
                                    "No alarms."
                                });
                                ui.end_row();
                            }

                            // Apply deferred mutations
                            if let Some((did, atype)) = ack_key {
                                registry.acknowledge(did, &atype);
                            }
                            if let Some((did, atype)) = clear_key {
                                registry.clear_alarm(did, &atype);
                            }
                        });
                });
        });

    state.visible = open;
}

fn severity_egui_color(sev: &str) -> egui::Color32 {
    match sev {
        "Critical"      => egui::Color32::from_rgb(204, 0, 204),
        "Major"         => egui::Color32::RED,
        "Minor"         => egui::Color32::from_rgb(255, 153, 0),
        "Warning"       => egui::Color32::YELLOW,
        _               => egui::Color32::GREEN,
    }
}
