//! Phase 29 — Background scene system.
//!
//! Allows loading GLB/GLTF floor plans or building shells as an immovable
//! background environment behind the device models.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Components ────────────────────────────────────────────────────────────────

/// Marker component for background environment entities.
#[derive(Component, Debug, Clone)]
pub struct BackgroundScene {
    pub id:    u32,
    pub layer: BackgroundLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BackgroundLayer {
    FloorPlan,
    #[default]
    Building,
    Interior,
    Custom(u8),
}

impl std::fmt::Display for BackgroundLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackgroundLayer::FloorPlan => write!(f, "Floor Plan"),
            BackgroundLayer::Building  => write!(f, "Building"),
            BackgroundLayer::Interior  => write!(f, "Interior"),
            BackgroundLayer::Custom(n) => write!(f, "Custom({n})"),
        }
    }
}

// ── Resources ─────────────────────────────────────────────────────────────────

/// Registry of all loaded/loading background scenes.
#[derive(Resource, Default)]
pub struct BackgroundSceneRegistry {
    pub scenes: Vec<BackgroundSceneEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSceneEntry {
    pub id:       u32,
    pub layer:    BackgroundLayer,
    pub path:     PathBuf,
    pub offset:   [f32; 3],
    pub rotation: [f32; 3],
    pub scale:    f32,
    pub visible:  bool,
    pub opacity:  f32,
}

impl BackgroundSceneEntry {
    pub fn new(id: u32, layer: BackgroundLayer, path: PathBuf) -> Self {
        Self {
            id,
            layer,
            path,
            offset:   [0.0; 3],
            rotation: [0.0; 3],
            scale:    1.0,
            visible:  true,
            opacity:  1.0,
        }
    }

    pub fn make_transform(&self) -> Transform {
        Transform {
            translation: Vec3::from(self.offset),
            rotation:    Quat::from_euler(
                EulerRot::XYZ,
                self.rotation[0].to_radians(),
                self.rotation[1].to_radians(),
                self.rotation[2].to_radians(),
            ),
            scale: Vec3::splat(self.scale),
        }
    }
}

/// Tracks in-progress GLTF asset loads.
#[derive(Resource, Default)]
pub struct BackgroundLoadState {
    /// (handle, entry_id, layer)
    pub pending: Vec<(Handle<bevy::gltf::Gltf>, u32, BackgroundLayer)>,
    next_id: u32,
}

impl BackgroundLoadState {
    pub fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

// ── Events ────────────────────────────────────────────────────────────────────

/// Request to load a GLB/GLTF file as a background layer.
#[derive(Message, Debug, Clone)]
pub struct LoadBackgroundRequest {
    pub layer: BackgroundLayer,
    pub path:  PathBuf,
}

/// Request to remove a background layer by id.
#[derive(Message, Debug, Clone)]
pub struct RemoveBackgroundRequest {
    pub id: u32,
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Handle LoadBackgroundRequest — registers the entry and starts loading the asset.
pub fn handle_load_background(
    mut events:    MessageReader<LoadBackgroundRequest>,
    mut load_state: ResMut<BackgroundLoadState>,
    mut registry:  ResMut<BackgroundSceneRegistry>,
    asset_server:  Res<AssetServer>,
) {
    for ev in events.read() {
        let id    = load_state.next_id();
        let entry = BackgroundSceneEntry::new(id, ev.layer, ev.path.clone());
        registry.scenes.push(entry);

        let path_str = ev.path.to_string_lossy().into_owned();
        let handle: Handle<bevy::gltf::Gltf> = asset_server.load(path_str);
        load_state.pending.push((handle, id, ev.layer));

        tracing::info!(id, layer = %ev.layer, path = %ev.path.display(), "Loading background scene");
    }
}

/// Poll pending GLTF loads; once ready, spawn the scene entity.
pub fn spawn_loaded_backgrounds(
    mut commands:   Commands,
    mut load_state: ResMut<BackgroundLoadState>,
    gltf_assets:    Res<Assets<bevy::gltf::Gltf>>,
    registry:       Res<BackgroundSceneRegistry>,
) {
    load_state.pending.retain(|(handle, id, layer)| {
        let Some(gltf) = gltf_assets.get(handle) else { return true };

        let transform = registry.scenes.iter()
            .find(|e| e.id == *id)
            .map(|e| e.make_transform())
            .unwrap_or_default();

        let scene_handle = gltf.scenes.first().cloned();
        if let Some(scene) = scene_handle {
            commands.spawn((
                SceneRoot(scene),
                transform,
                BackgroundScene { id: *id, layer: *layer },
            ));
            tracing::info!(id, "Background scene spawned");
        }

        false // remove from pending
    });
}

/// Handle RemoveBackgroundRequest — despawn entity and remove registry entry.
pub fn handle_remove_background(
    mut commands: Commands,
    mut events:   MessageReader<RemoveBackgroundRequest>,
    mut registry: ResMut<BackgroundSceneRegistry>,
    bg_query:     Query<(Entity, &BackgroundScene)>,
) {
    for ev in events.read() {
        registry.scenes.retain(|s| s.id != ev.id);
        for (entity, bg) in bg_query.iter() {
            if bg.id == ev.id {
                commands.entity(entity).despawn();
                tracing::info!(id = ev.id, "Background scene removed");
            }
        }
    }
}

/// Apply live transform changes from registry edits (called when user adjusts sliders).
pub fn sync_background_transforms(
    registry: Res<BackgroundSceneRegistry>,
    mut bg_query: Query<(&BackgroundScene, &mut Transform)>,
) {
    if !registry.is_changed() { return; }
    for (bg, mut tf) in bg_query.iter_mut() {
        if let Some(entry) = registry.scenes.iter().find(|e| e.id == bg.id) {
            *tf = entry.make_transform();
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_scene_entry_default_transform() {
        let entry = BackgroundSceneEntry::new(1, BackgroundLayer::Building, PathBuf::from("test.glb"));
        let tf = entry.make_transform();
        assert_eq!(tf.translation, Vec3::ZERO);
        assert_eq!(tf.scale, Vec3::ONE);
    }

    #[test]
    fn background_layer_display() {
        assert_eq!(BackgroundLayer::FloorPlan.to_string(), "Floor Plan");
        assert_eq!(BackgroundLayer::Custom(3).to_string(), "Custom(3)");
    }

    #[test]
    fn background_load_state_next_id_increments() {
        let mut state = BackgroundLoadState::default();
        assert_eq!(state.next_id(), 0);
        assert_eq!(state.next_id(), 1);
        assert_eq!(state.next_id(), 2);
    }
}
