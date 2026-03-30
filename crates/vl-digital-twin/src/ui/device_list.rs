//! Left-side egui panel — device tree with status indicators and search.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::{AlarmIndicator, DeviceEntity, DeviceStatus};
use crate::ui::{LayoutMode, SelectedDevice};

/// Search / filter state for the device list panel.
#[derive(Resource, Default)]
pub struct DeviceListFilter {
    pub query: String,
}

/// Render the left-side device list panel.
pub fn render_device_list(
    mut contexts: EguiContexts,
    mut selected: ResMut<SelectedDevice>,
    layout_mode:  Res<LayoutMode>,
    mut filter:   ResMut<DeviceListFilter>,
    query:        Query<(Entity, &DeviceEntity, &DeviceStatus, &AlarmIndicator)>,
) {
    // Hidden in fullscreen mode
    if *layout_mode == LayoutMode::FullscreenScene { return; }

    egui::SidePanel::left("device_list")
        .min_width(220.0)
        .resizable(true)
        .show(contexts.ctx_mut().expect("egui context"), |ui| {
            ui.heading("Devices");
            ui.separator();

            // Summary counters
            let total   = query.iter().count();
            let online  = query.iter().filter(|(_, _, s, _)| s.online).count();
            let alarmed = query.iter().filter(|(_, _, _, a)| a.active).count();

            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::LIGHT_GRAY, format!("{} devices", total));
                ui.separator();
                ui.colored_label(egui::Color32::GREEN, format!("{} online", online));
                if alarmed > 0 {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, format!("{} alarms", alarmed));
                }
            });

            // Search input
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut filter.query)
                    .on_hover_text("Filter devices by name or type");
                if ui.small_button("✕").clicked() {
                    filter.query.clear();
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Sort by name for a stable list
                let mut entries: Vec<_> = query.iter().collect();
                entries.sort_by_key(|(_, d, _, _)| d.name.as_str());

                let filter_lower = filter.query.to_lowercase();

                for (entity, device, status, alarm) in entries {
                    // Apply search filter (case-insensitive name or type match)
                    if !filter_lower.is_empty()
                        && !device.name.to_lowercase().contains(&filter_lower)
                        && !device.device_type.to_lowercase().contains(&filter_lower)
                    {
                        continue;
                    }

                    let is_selected = selected.entity == Some(entity);

                    let icon = if alarm.active {
                        "🔴"
                    } else if status.online {
                        "🟢"
                    } else {
                        "⚫"
                    };

                    let label = format!("{} {}", icon, device.name);

                    if ui.selectable_label(is_selected, &label).clicked() {
                        selected.entity    = Some(entity);
                        selected.device_id = Some(device.device_id);
                        selected.name      = device.name.clone();
                    }

                    ui.weak(format!("  {}", device.device_type));
                }
            });
        });
}
