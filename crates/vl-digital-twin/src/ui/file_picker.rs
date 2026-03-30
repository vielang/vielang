//! Phase 29 — File picker for BIM/CAD import.
//!
//! Native: opens a system file dialog via `rfd`.
//! WASM:   shows an egui text-input fallback (no native dialog available).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::scene::background::{BackgroundLayer, LoadBackgroundRequest};

// ── Resource ─────────────────────────────────────────────────────────────────

/// Pending file selection — set by the file picker, consumed by the load system.
#[derive(Resource, Default)]
pub struct FilePicker {
    pub pending_path:  Option<std::path::PathBuf>,
    pub pending_layer: BackgroundLayer,
    /// Whether to show the egui fallback input (WASM / when rfd unavailable).
    pub show_input:    bool,
    /// Text field content for the manual path input.
    pub path_input:    String,
}

// ── Native system ─────────────────────────────────────────────────────────────

/// Open a native file picker dialog on Ctrl+O.
#[cfg(not(target_arch = "wasm32"))]
pub fn open_file_picker_native(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut picker: ResMut<FilePicker>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && keyboard.just_pressed(KeyCode::KeyO) {
        // rfd::FileDialog::pick_file() blocks the calling thread.
        // We call it directly here; on Windows/macOS it's fast.
        let result = rfd::FileDialog::new()
            .add_filter("3D Models", &["glb", "gltf"])
            .set_title("Import Background Scene")
            .pick_file();
        match result {
            Some(path) => {
                tracing::info!(path = %path.display(), "File selected via picker");
                picker.pending_path  = Some(path);
                picker.pending_layer = BackgroundLayer::Building;
            }
            None => tracing::debug!("File picker cancelled"),
        }
    }
}

/// WASM stub for open_file_picker — shows the text input panel on Ctrl+O.
#[cfg(target_arch = "wasm32")]
pub fn open_file_picker_native(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut picker: ResMut<FilePicker>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && keyboard.just_pressed(KeyCode::KeyO) {
        picker.show_input = true;
    }
}

// ── WASM manual input UI ──────────────────────────────────────────────────────

/// Render a path-input fallback panel (shown on WASM or when picker.show_input == true).
pub fn render_file_picker_input(
    mut contexts: bevy_egui::EguiContexts,
    mut picker:   ResMut<FilePicker>,
) {
    if !picker.show_input { return; }

    let ctx = contexts.ctx_mut().expect("egui context");

    let mut close = false;
    egui::Window::new("Import 3D Model")
        .anchor(bevy_egui::egui::Align2::CENTER_CENTER, bevy_egui::egui::vec2(0.0, 0.0))
        .collapsible(false)
        .resizable(false)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.label("Enter path to a .glb or .gltf file:");
            ui.text_edit_singleline(&mut picker.path_input);
            ui.horizontal(|ui| {
                if ui.button("Load").clicked() && !picker.path_input.is_empty() {
                    picker.pending_path  = Some(std::path::PathBuf::from(&picker.path_input));
                    picker.pending_layer = BackgroundLayer::Building;
                    close = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });

    if close {
        picker.show_input  = false;
        picker.path_input.clear();
    }
}

// ── Emit event from pending path ──────────────────────────────────────────────

/// Convert a pending file path into a LoadBackgroundRequest event.
pub fn emit_load_from_picker(
    mut picker: ResMut<FilePicker>,
    mut writer: MessageWriter<LoadBackgroundRequest>,
) {
    if let Some(path) = picker.pending_path.take() {
        writer.write(LoadBackgroundRequest {
            layer: picker.pending_layer,
            path,
        });
    }
}

