//! Asset pipeline systems — GLTF spawning, billboard labels, LOD.

use bevy::prelude::*;

use crate::{assets::ModelRegistry, components::DeviceEntity};

// ── Marker components ─────────────────────────────────────────────────────────

/// Marks an entity whose GLTF model child has been spawned.
#[derive(Component)]
pub struct ModelSpawned;

/// Marks an entity whose billboard label child has been spawned.
#[derive(Component)]
pub struct LabelSpawned;

/// Marks a child Text2d entity that should always face the camera.
#[derive(Component)]
pub struct Billboard;

// ── LOD ───────────────────────────────────────────────────────────────────────

/// Current level-of-detail for a device entity.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LodLevel {
    /// < 10 units — full GLTF model visible
    #[default]
    High,
    /// 10–25 units — full GLTF, label hidden
    Medium,
    /// > 25 units — cube LED only, label hidden
    Low,
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Spawn a GLTF child SceneRoot for each device that doesn't have one yet.
///
/// The cube (0.2³) on the device entity remains as the alarm-status LED.
/// The GLTF scene is spawned as a child so they render together.
/// If the GLB file is missing, Bevy silently keeps the handle in loading state
/// and nothing extra renders — the cube LED remains the only visual.
pub fn spawn_device_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry:     Res<ModelRegistry>,
    query:        Query<(Entity, &DeviceEntity), Without<ModelSpawned>>,
) {
    for (entity, device) in query.iter() {
        let asset_path = registry.asset_path(&device.device_type).to_string();
        let path: Handle<Scene> = asset_server.load(asset_path.clone());

        commands
            .entity(entity)
            .insert(ModelSpawned)
            .with_children(|parent| {
                parent.spawn((SceneRoot(path), Transform::default()));
            });

        tracing::debug!(
            device = %device.name,
            dtype  = %device.device_type,
            path   = asset_path,
            "GLTF model requested"
        );
    }
}

/// Spawn a floating Text2d label above each device that doesn't have one yet.
pub fn spawn_device_labels(
    mut commands: Commands,
    query:        Query<(Entity, &DeviceEntity), Without<LabelSpawned>>,
) {
    for (entity, device) in query.iter() {
        commands
            .entity(entity)
            .insert(LabelSpawned)
            .with_children(|parent| {
                parent.spawn((
                    Text2d::new(device.name.clone()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 1.2, 0.0),
                    Billboard,
                ));
            });
    }
}

/// Orient Billboard label children so they always face the active Camera3d.
///
/// Device entities can rotate (e.g. wind turbine) so we correctly convert the
/// desired world-space rotation back to local-space using the parent's rotation.
pub fn update_billboards(
    camera_q:   Query<&GlobalTransform, With<Camera3d>>,
    parent_q:   Query<&GlobalTransform, Without<Billboard>>,
    mut bill_q: Query<(&ChildOf, &GlobalTransform, &mut Transform), With<Billboard>>,
) {
    let Ok(cam_gtf) = camera_q.single() else { return };
    let cam_pos: Vec3 = cam_gtf.translation();

    for (child_of, global_tf, mut local_tf) in bill_q.iter_mut() {
        let label_world_pos: Vec3 = global_tf.translation();
        let dir = cam_pos - label_world_pos;
        if dir.length_squared() < 1e-6 {
            continue;
        }
        let dir = dir.normalize();

        // Desired world rotation: Text2d faces -Z toward camera
        let world_rot = Quat::from_rotation_arc(Vec3::NEG_Z, dir);

        // Convert to local: local_rot = parent_world_rot.inverse() * world_rot
        if let Ok(parent_gtf) = parent_q.get(child_of.parent()) {
            let (_, parent_world_rot, _) = parent_gtf.to_scale_rotation_translation();
            local_tf.rotation = parent_world_rot.inverse() * world_rot;
        } else {
            local_tf.rotation = world_rot;
        }
    }
}

/// Update LodLevel for each device based on distance to camera.
pub fn update_lod(
    camera_q:  Query<&GlobalTransform, With<Camera3d>>,
    mut dev_q: Query<(&GlobalTransform, &mut LodLevel), With<DeviceEntity>>,
) {
    let Ok(cam_gtf) = camera_q.single() else { return };
    let cam_pos: Vec3 = cam_gtf.translation();

    for (device_gtf, mut lod) in dev_q.iter_mut() {
        let dist = cam_pos.distance(device_gtf.translation());
        let new_lod = if dist < 10.0 {
            LodLevel::High
        } else if dist < 25.0 {
            LodLevel::Medium
        } else {
            LodLevel::Low
        };

        if *lod != new_lod {
            *lod = new_lod;
        }
    }
}

/// Show/hide billboard labels based on current LodLevel.
pub fn apply_lod_visibility(
    dev_q:     Query<(&LodLevel, &Children), (With<DeviceEntity>, Changed<LodLevel>)>,
    bill_q:    Query<(), With<Billboard>>,
    mut vis_q: Query<&mut Visibility>,
) {
    for (lod, children) in dev_q.iter() {
        let visible = *lod == LodLevel::High;
        for child in children.iter() {
            if bill_q.contains(child) {
                if let Ok(mut vis) = vis_q.get_mut(child) {
                    *vis = if visible {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
        }
    }
}
