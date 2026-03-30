//! Phase 32 — WebXR / VR mode state.
//!
//! Tracks whether a WebXR immersive-vr session is active.
//! Actual stereoscopic rendering requires a Bevy WebXR plugin (future work);
//! this module provides the session state infrastructure and the VR HUD overlay.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

// ── Resource ─────────────────────────────────────────────────────────────────

#[derive(Resource, Default, PartialEq, Eq)]
pub enum VrModeState {
    #[default]
    /// Standard desktop/browser rendering.
    Desktop,
    /// WebXR session requested, waiting for browser confirmation.
    Requesting,
    /// WebXR immersive-vr session active.
    Active,
    /// Not available on this platform/browser.
    Unsupported,
}

impl VrModeState {
    pub fn is_active(&self) -> bool { matches!(self, VrModeState::Active) }
}

// ── System ────────────────────────────────────────────────────────────────────

/// Render a minimal VR HUD — active alarm count overlay when VR is running.
pub fn render_vr_hud(
    mut contexts: EguiContexts,
    vr_state:     Res<VrModeState>,
    alarm_reg:    Res<crate::alarm::AlarmRegistry>,
) {
    if !vr_state.is_active() { return; }

    let ctx = contexts.ctx_mut().expect("egui context");
    let active = alarm_reg.active_count();

    egui::Area::new(egui::Id::new("vr_hud"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 20.0))
        .show(ctx, |ui| {
            egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
                .show(ui, |ui| {
                    if active > 0 {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("⚠ {active} active alarm(s)"),
                        );
                    } else {
                        ui.colored_label(egui::Color32::GREEN, "All clear");
                    }
                    ui.label("VR Mode — press F to exit");
                });
        });
}

/// Render the "Enter VR" button in the desktop UI (WASM only).
pub fn render_vr_enter_button(
    mut contexts: EguiContexts,
    mut vr_state: ResMut<VrModeState>,
) {
    // Only show on WASM — native desktop uses FPS mode instead
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (contexts, vr_state); // suppress unused warnings

    #[cfg(target_arch = "wasm32")]
    {
        if *vr_state == VrModeState::Unsupported { return; }

        let ctx = contexts.ctx_mut().expect("egui context");
        egui::Area::new(egui::Id::new("vr_btn"))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
            .show(ctx, |ui| {
                let label = match *vr_state {
                    VrModeState::Desktop    => "Enter VR",
                    VrModeState::Requesting => "Starting VR...",
                    VrModeState::Active     => "Exit VR",
                    VrModeState::Unsupported => unreachable!(),
                };
                if ui.button(label).clicked() {
                    match *vr_state {
                        VrModeState::Desktop => {
                            *vr_state = VrModeState::Requesting;
                            tracing::info!("WebXR session requested");
                            // Actual XR session init via wasm_bindgen_futures::spawn_local
                            // would go here once a Bevy WebXR plugin is available.
                        }
                        VrModeState::Active => {
                            *vr_state = VrModeState::Desktop;
                            tracing::info!("WebXR session ended");
                        }
                        _ => {}
                    }
                }
            });
    }
}
