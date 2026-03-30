//! Phase 31 — Audit log for user actions.
//!
//! Records significant actions (alarm acks, layout saves, model imports)
//! with username, timestamp, and description.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::current_time_ms;

// ── Resource ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp_ms: i64,
    pub username:     String,
    pub action:       String,
}

/// Rolling audit log — keeps last 200 entries.
#[derive(Resource, Default)]
pub struct AuditLog {
    pub entries:  Vec<AuditEntry>,
    pub visible:  bool,
}

impl AuditLog {
    pub fn record(&mut self, username: &str, action: impl Into<String>) {
        let entry = AuditEntry {
            timestamp_ms: current_time_ms(),
            username:     username.to_string(),
            action:       action.into(),
        };
        tracing::info!(user = username, action = %entry.action, "Audit log entry");
        self.entries.push(entry);
        if self.entries.len() > 200 {
            self.entries.remove(0);
        }
    }
}

// ── System ────────────────────────────────────────────────────────────────────

/// Render the audit log floating panel.
pub fn render_audit_log(
    mut contexts: EguiContexts,
    mut log:      ResMut<AuditLog>,
) {
    if !log.visible { return; }

    let ctx = contexts.ctx_mut().expect("egui context");

    egui::Window::new("Audit Log")
        .default_size([500.0, 300.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{} entries", log.entries.len()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").clicked() {
                        log.visible = false;
                    }
                    if ui.small_button("Clear").clicked() {
                        log.entries.clear();
                    }
                });
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    egui::Grid::new("audit_grid").num_columns(3).striped(true).show(ui, |ui| {
                        ui.label(egui::RichText::new("Time").strong());
                        ui.label(egui::RichText::new("User").strong());
                        ui.label(egui::RichText::new("Action").strong());
                        ui.end_row();

                        for entry in log.entries.iter().rev().take(100) {
                            let secs = entry.timestamp_ms / 1000;
                            let h = (secs % 86400) / 3600;
                            let m = (secs % 3600) / 60;
                            let s = secs % 60;
                            ui.label(format!("{h:02}:{m:02}:{s:02}"));
                            ui.label(&entry.username);
                            ui.label(&entry.action);
                            ui.end_row();
                        }
                    });
                });
        });
}
