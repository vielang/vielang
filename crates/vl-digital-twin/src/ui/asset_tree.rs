//! Phase 33 — Asset hierarchy tree panel.
//!
//! Renders the left-side panel as a collapsible tree:
//!   🌐 Site → 🏢 Building → 🟢 Device rows
//!
//! Replaces the flat `render_device_list` panel in the Running state.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use uuid::Uuid;

use crate::asset_hierarchy::{AssetNodeData, AssetTree};
use crate::components::{AlarmIndicator, DeviceEntity, DeviceStatus};
use crate::ui::{DeviceListFilter, LayoutMode, SelectedDevice};

// ── System ────────────────────────────────────────────────────────────────────

/// Render the left-side asset hierarchy tree panel.
pub fn render_asset_tree(
    mut contexts: EguiContexts,
    mut selected: ResMut<SelectedDevice>,
    layout_mode:  Res<LayoutMode>,
    tree:         Res<AssetTree>,
    mut filter:   ResMut<DeviceListFilter>,
    query:        Query<(Entity, &DeviceEntity, &DeviceStatus, &AlarmIndicator)>,
) {
    if *layout_mode == LayoutMode::FullscreenScene { return; }

    // Pre-collect device data to avoid borrow conflict inside the egui closure.
    // Tuple: (entity, device_id, name, online, alarmed, parent_asset_id)
    let devices: Vec<(Entity, Uuid, String, bool, bool, Option<Uuid>)> = query.iter()
        .map(|(e, dev, st, al)| (
            e,
            dev.device_id,
            dev.name.clone(),
            st.online,
            al.active,
            dev.parent_asset_id,
        ))
        .collect();

    // Clone tree nodes for immutable use inside the egui closure.
    let nodes = tree.nodes.clone();

    let total   = devices.len();
    let online  = devices.iter().filter(|d| d.3).count();
    let alarmed = devices.iter().filter(|d| d.4).count();

    // Staged mutations — applied after the egui closure to satisfy borrow rules.
    let mut new_entity    = selected.entity;
    let mut new_device_id = selected.device_id;
    let mut new_name      = selected.name.clone();

    let ctx = contexts.ctx_mut().expect("egui context");

    egui::SidePanel::left("asset_hierarchy")
        .min_width(230.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Assets & Devices");
            ui.separator();

            // Summary row
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::LIGHT_GRAY, format!("{} devices", total));
                ui.separator();
                ui.colored_label(egui::Color32::GREEN, format!("{} online", online));
                if alarmed > 0 {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, format!("{} alarms", alarmed));
                }
            });

            // Search bar
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut filter.query)
                    .on_hover_text("Filter devices by name");
                if ui.small_button("✕").clicked() {
                    filter.query.clear();
                }
            });
            ui.separator();

            let filter_lower = filter.query.to_lowercase();

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Root asset nodes (nodes with no parent)
                let roots: Vec<_> = nodes.iter()
                    .filter(|n| n.parent_id.is_none())
                    .collect();

                for root in roots {
                    render_asset_node(
                        ui, root, &nodes, &devices, &filter_lower,
                        &mut new_entity, &mut new_device_id, &mut new_name,
                    );
                }

                // Devices with no parent_asset_id ("unassigned")
                let orphans: Vec<_> = devices.iter()
                    .filter(|d| d.5.is_none())
                    .filter(|d| {
                        filter_lower.is_empty()
                            || d.2.to_lowercase().contains(&filter_lower)
                    })
                    .collect();

                if !orphans.is_empty() {
                    if !nodes.is_empty() {
                        ui.separator();
                        ui.weak("Unassigned devices");
                    }
                    let mut sorted = orphans;
                    sorted.sort_by(|a, b| a.2.cmp(&b.2));
                    for (entity, dev_id, name, online, alarmed, _) in sorted {
                        render_device_row(
                            ui, *entity, *dev_id, name, *online, *alarmed,
                            &mut new_entity, &mut new_device_id, &mut new_name,
                        );
                    }
                }
            });
        });

    // Apply staged mutations
    selected.entity    = new_entity;
    selected.device_id = new_device_id;
    selected.name      = new_name;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Recursively render one asset node and its children via egui CollapsingHeader.
fn render_asset_node(
    ui:            &mut egui::Ui,
    node:          &AssetNodeData,
    all_nodes:     &[AssetNodeData],
    devices:       &[(Entity, Uuid, String, bool, bool, Option<Uuid>)],
    filter_lower:  &str,
    sel_entity:    &mut Option<Entity>,
    sel_device_id: &mut Option<Uuid>,
    sel_name:      &mut String,
) {
    let icon = match node.asset_type.as_str() {
        "Site"     => "🌐",
        "Building" => "🏢",
        "Floor"    => "📋",
        "Zone"     => "📦",
        _          => "📁",
    };

    // Badge: count direct-child device alarms
    let alarm_count = devices.iter()
        .filter(|(_, _, _, _, alarmed, parent)| *alarmed && *parent == Some(node.asset_id))
        .count();
    let badge = if alarm_count > 0 {
        format!(" 🔴 {}", alarm_count)
    } else {
        String::new()
    };

    let header_label = format!("{} {}{}", icon, node.name, badge);

    egui::CollapsingHeader::new(header_label)
        .id_salt(node.asset_id)
        .default_open(true)
        .show(ui, |ui| {
            // Child asset nodes first (depth-first)
            let children: Vec<_> = all_nodes.iter()
                .filter(|n| n.parent_id == Some(node.asset_id))
                .collect();
            for child in children {
                render_asset_node(
                    ui, child, all_nodes, devices, filter_lower,
                    sel_entity, sel_device_id, sel_name,
                );
            }

            // Device rows whose parent == this asset node
            let mut child_devs: Vec<_> = devices.iter()
                .filter(|(_, _, name, _, _, parent)| {
                    *parent == Some(node.asset_id)
                        && (filter_lower.is_empty()
                            || name.to_lowercase().contains(filter_lower))
                })
                .collect();
            child_devs.sort_by(|a, b| a.2.cmp(&b.2));

            for (entity, dev_id, name, online, alarmed, _) in child_devs {
                render_device_row(
                    ui, *entity, *dev_id, name, *online, *alarmed,
                    sel_entity, sel_device_id, sel_name,
                );
            }
        });
}

fn render_device_row(
    ui:            &mut egui::Ui,
    entity:        Entity,
    device_id:     Uuid,
    name:          &str,
    online:        bool,
    alarmed:       bool,
    sel_entity:    &mut Option<Entity>,
    sel_device_id: &mut Option<Uuid>,
    sel_name:      &mut String,
) {
    let icon = if alarmed { "🔴" } else if online { "🟢" } else { "⚫" };
    let label = format!("  {} {}", icon, name);
    let is_selected = *sel_entity == Some(entity);
    if ui.selectable_label(is_selected, &label).clicked() {
        *sel_entity    = Some(entity);
        *sel_device_id = Some(device_id);
        *sel_name      = name.to_string();
    }
}
