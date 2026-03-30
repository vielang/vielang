//! Layout profile manager — save, load, and delete named scene layouts.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::scene::SceneLayout;
use crate::systems::layout_system::CurrentLayout;

// ── Resource ──────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct LayoutManager {
    /// Names of profiles that exist on disk.
    pub profiles:     Vec<String>,
    /// Currently active profile name.
    pub active:       String,
    /// Show the profile manager window.
    pub show_manager: bool,
    /// Scratch buffer for creating a new profile.
    new_profile_name: String,
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self {
            profiles:         SceneLayout::list_profiles(),
            active:           "default".to_string(),
            show_manager:     false,
            new_profile_name: String::new(),
        }
    }
}

impl LayoutManager {
    /// Reload the profiles list from disk.
    pub fn refresh(&mut self) {
        self.profiles = SceneLayout::list_profiles();
        self.profiles.sort();
    }
}

// ── System ────────────────────────────────────────────────────────────────────

/// Render the layout profile manager window (opened by toggling `show_manager`).
pub fn render_layout_manager(
    mut contexts: EguiContexts,
    mut manager:  ResMut<LayoutManager>,
    mut layout:   ResMut<CurrentLayout>,
) {
    if !manager.show_manager { return; }

    let ctx = match contexts.ctx_mut() {
        Ok(c)  => c,
        Err(_) => return,
    };

    let mut open = manager.show_manager;

    egui::Window::new("Layout Profiles")
        .open(&mut open)
        .resizable(true)
        .min_width(320.0)
        .show(ctx, |ui| {
            ui.heading("Saved Profiles");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let profiles: Vec<String> = manager.profiles.clone();
                    let active   = manager.active.clone();

                    for name in &profiles {
                        ui.horizontal(|ui| {
                            let is_active = *name == active;
                            if is_active {
                                ui.colored_label(egui::Color32::GREEN, format!("● {name}"));
                            } else {
                                ui.label(format!("  {name}"));
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Delete
                                if !is_active && ui.small_button("🗑").on_hover_text("Delete").clicked() {
                                    let _ = SceneLayout::delete(name);
                                    manager.refresh();
                                }
                                // Load
                                if ui.small_button("📂").on_hover_text("Load").clicked() {
                                    if let Some(loaded) = SceneLayout::load(name) {
                                        layout.scene    = loaded;
                                        manager.active  = name.clone();
                                        tracing::info!("Layout profile loaded: {name}");
                                    }
                                }
                            });
                        });
                    }
                });

            ui.separator();

            // ── New profile ──────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("New profile:");
                ui.text_edit_singleline(&mut manager.new_profile_name);
                if ui.button("+ Create").clicked() && !manager.new_profile_name.is_empty() {
                    let new_name = manager.new_profile_name.trim().to_string();
                    let new_layout = SceneLayout::new(&new_name);
                    if let Err(e) = new_layout.save(&new_name) {
                        tracing::warn!("Failed to create profile '{new_name}': {e}");
                    } else {
                        layout.scene   = new_layout;
                        manager.active = new_name;
                        manager.new_profile_name.clear();
                        manager.refresh();
                    }
                }
            });

            ui.separator();

            // ── Save current → active profile ────────────────────────────────
            ui.horizontal(|ui| {
                let label = format!("Save → '{}'", manager.active);
                if ui.button(label).clicked() {
                    let name = manager.active.clone();
                    match layout.scene.save(&name) {
                        Ok(())  => tracing::info!("Profile '{name}' saved"),
                        Err(e)  => tracing::warn!("Save failed: {e}"),
                    }
                    manager.refresh();
                }
            });
        });

    manager.show_manager = open;
}
