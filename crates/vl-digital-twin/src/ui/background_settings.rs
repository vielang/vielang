//! Phase 29 — Background scene settings panel.
//!
//! Shows a floating egui window listing loaded background scenes
//! with per-entry transform sliders and remove buttons.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::scene::background::{BackgroundSceneRegistry, RemoveBackgroundRequest};

/// Render the background scene management panel.
pub fn render_background_settings(
    mut contexts:  EguiContexts,
    mut registry:  ResMut<BackgroundSceneRegistry>,
    mut rm_writer: MessageWriter<RemoveBackgroundRequest>,
    // Import button — delegates to FilePicker (handled in file_picker.rs)
    mut picker:    ResMut<crate::ui::file_picker::FilePicker>,
) {
    let ctx = contexts.ctx_mut().expect("egui context");

    let mut ids_to_remove: Vec<u32> = Vec::new();

    egui::Window::new("Background Scenes")
        .default_open(false)
        .default_width(320.0)
        .show(ctx, |ui| {
            if ui.button("📁 Import GLB/GLTF  (Ctrl+O)").clicked() {
                picker.show_input = true;
            }

            ui.separator();

            if registry.scenes.is_empty() {
                ui.label("No background scenes loaded.");
                return;
            }

            for entry in registry.scenes.iter_mut() {
                let file_name = entry.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");

                ui.collapsing(format!("[{}] {}", entry.layer, file_name), |ui| {
                    ui.checkbox(&mut entry.visible, "Visible");
                    ui.add(
                        egui::Slider::new(&mut entry.opacity, 0.0..=1.0)
                            .text("Opacity"),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Offset:");
                        ui.add(egui::DragValue::new(&mut entry.offset[0]).speed(0.1).prefix("X:"));
                        ui.add(egui::DragValue::new(&mut entry.offset[1]).speed(0.1).prefix("Y:"));
                        ui.add(egui::DragValue::new(&mut entry.offset[2]).speed(0.1).prefix("Z:"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Rotation:");
                        ui.add(egui::DragValue::new(&mut entry.rotation[0]).speed(1.0).suffix("°"));
                        ui.add(egui::DragValue::new(&mut entry.rotation[1]).speed(1.0).suffix("°"));
                        ui.add(egui::DragValue::new(&mut entry.rotation[2]).speed(1.0).suffix("°"));
                    });
                    ui.add(
                        egui::Slider::new(&mut entry.scale, 0.01..=100.0)
                            .logarithmic(true)
                            .text("Scale"),
                    );

                    if ui.button("🗑 Remove").clicked() {
                        ids_to_remove.push(entry.id);
                    }
                });
            }
        });

    for id in ids_to_remove {
        rm_writer.write(RemoveBackgroundRequest { id });
    }

}
