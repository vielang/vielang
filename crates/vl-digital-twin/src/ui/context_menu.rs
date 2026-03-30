//! Right-click context menu — quick RPC commands for selected devices.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

use crate::{
    components::{DeviceEntity, DeviceRpcPresets, RpcPreset},
    events::SendRpcRequest,
    components::current_time_ms,
};

// ── Resource ──────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ContextMenuState {
    pub visible:         bool,
    pub device_id:       Option<Uuid>,
    pub device_name:     String,
    /// Custom RPC input fields
    pub custom_method:   String,
    pub custom_params:   String,
    pub custom_twoway:   bool,
    /// A preset awaiting the user's confirmation before being sent.
    pub pending_confirm: Option<RpcPreset>,
}

// ── System: open menu on right-click ──────────────────────────────────────────

/// Detect right-click on device entities → populate and open context menu.
pub fn handle_entity_right_click(
    mut click_events: MessageReader<
        bevy::picking::events::Pointer<bevy::picking::events::Click>,
    >,
    query:        Query<&DeviceEntity>,
    mut menu:     ResMut<ContextMenuState>,
) {
    for event in click_events.read() {
        if event.button != bevy::picking::pointer::PointerButton::Secondary {
            continue;
        }
        if let Ok(device) = query.get(event.entity) {
            menu.visible     = true;
            menu.device_id   = Some(device.device_id);
            menu.device_name = device.name.clone();
            menu.pending_confirm = None;
            tracing::debug!(device = %device.name, "Context menu opened");
        }
    }
}

// ── System: render context menu ───────────────────────────────────────────────

pub fn render_context_menu(
    mut ctx:        EguiContexts,
    mut menu:       ResMut<ContextMenuState>,
    device_query:   Query<(&DeviceEntity, &DeviceRpcPresets)>,
    mut rpc_writer: MessageWriter<SendRpcRequest>,
) {
    if !menu.visible {
        return;
    }
    let Some(device_id) = menu.device_id else { return };

    // Look up presets for the selected device
    let presets: Vec<RpcPreset> = device_query
        .iter()
        .find(|(d, _)| d.device_id == device_id)
        .map(|(_, p)| p.0.clone())
        .unwrap_or_default();

    let ctx = ctx.ctx_mut().expect("egui context");

    egui::Window::new(format!("⚙ {}", menu.device_name))
        .collapsible(false)
        .resizable(true)
        .default_width(260.0)
        .show(ctx, |ui| {
            // ── Quick commands ─────────────────────────────────────────────
            ui.label("Quick Commands:");
            ui.add_space(4.0);

            let mut to_send: Option<SendRpcRequest> = None;
            let mut to_confirm: Option<RpcPreset>   = None;

            for preset in &presets {
                let label = if preset.is_twoway {
                    format!("{} ↩", preset.label)
                } else {
                    preset.label.clone()
                };
                if ui.button(&label).clicked() {
                    if preset.confirm {
                        to_confirm = Some(preset.clone());
                    } else {
                        to_send = Some(SendRpcRequest {
                            device_id,
                            method:    preset.method.clone(),
                            params:    preset.params.clone(),
                            is_twoway: preset.is_twoway,
                            sent_at:   current_time_ms(),
                        });
                    }
                }
            }

            if let Some(p) = to_confirm { menu.pending_confirm = Some(p); }
            if let Some(req) = to_send  { rpc_writer.write(req); }

            // ── Custom RPC ────────────────────────────────────────────────
            ui.add_space(6.0);
            ui.separator();
            ui.collapsing("Custom RPC", |ui| {
                egui::Grid::new("custom_rpc_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Method:");
                        ui.text_edit_singleline(&mut menu.custom_method);
                        ui.end_row();

                        ui.label("Params (JSON):");
                        ui.text_edit_singleline(&mut menu.custom_params);
                        ui.end_row();

                        ui.label("Two-way:");
                        ui.checkbox(&mut menu.custom_twoway, "");
                        ui.end_row();
                    });

                if ui.button("Send").clicked() && !menu.custom_method.is_empty() {
                    let params = serde_json::from_str::<serde_json::Value>(&menu.custom_params)
                        .unwrap_or(serde_json::Value::Object(Default::default()));
                    rpc_writer.write(SendRpcRequest {
                        device_id,
                        method:    menu.custom_method.clone(),
                        params,
                        is_twoway: menu.custom_twoway,
                        sent_at:   current_time_ms(),
                    });
                }
            });

            ui.add_space(6.0);
            if ui.button("Close").clicked() {
                menu.visible = false;
            }
        });

    // ── Confirm dialog ────────────────────────────────────────────────────────
    if let Some(preset) = menu.pending_confirm.clone() {
        egui::Window::new("Confirm Action")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Execute: {}?", preset.label));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
                        rpc_writer.write(SendRpcRequest {
                            device_id,
                            method:    preset.method.clone(),
                            params:    preset.params.clone(),
                            is_twoway: preset.is_twoway,
                            sent_at:   current_time_ms(),
                        });
                        menu.pending_confirm = None;
                    }
                    if ui.button("Cancel").clicked() {
                        menu.pending_confirm = None;
                    }
                });
            });
    }
}
