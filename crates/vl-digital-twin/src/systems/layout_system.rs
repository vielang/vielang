//! Layout persistence — auto-save scene layout and drag-to-reposition.

use bevy::prelude::*;

use crate::components::DeviceEntity;
use crate::scene::layout::{CameraLayout, DeviceLayoutEntry, LayoutMeta, SceneLayout};

// ── Resources ─────────────────────────────────────────────────────────────────

/// Holds the in-memory scene layout that gets periodically saved to disk.
#[derive(Resource)]
pub struct CurrentLayout {
    pub scene: SceneLayout,
}

impl Default for CurrentLayout {
    fn default() -> Self {
        let scene = SceneLayout::load_default()
            .unwrap_or_else(|| SceneLayout::new("Main Dashboard"));
        Self { scene }
    }
}

/// Repeating timer that triggers auto-save every 60 seconds.
#[derive(Resource)]
pub struct AutoSaveTimer(pub Timer);

impl Default for AutoSaveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(60.0, TimerMode::Repeating))
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Save device positions and camera state to disk every 60 s.
pub fn auto_save_layout(
    time:        Res<Time>,
    mut timer:   ResMut<AutoSaveTimer>,
    query:       Query<(&DeviceEntity, &Transform)>,
    camera:      Query<&Transform, With<Camera3d>>,
    mut layout:  ResMut<CurrentLayout>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() { return; }

    save_layout_now(&time, &query, &camera, &mut layout);
}

/// Flush layout to disk immediately (called from keyboard Ctrl+S).
pub fn save_layout_now(
    _time:      &Time,
    query:      &Query<(&DeviceEntity, &Transform)>,
    camera:     &Query<&Transform, With<Camera3d>>,
    layout:     &mut CurrentLayout,
) {
    let entries: Vec<DeviceLayoutEntry> = query.iter()
        .map(|(device, tf)| {
            let (x, y, z) = tf.rotation.to_euler(EulerRot::XYZ);
            DeviceLayoutEntry {
                id:        device.device_id,
                name:      device.name.clone(),
                position:  tf.translation.into(),
                rotation:  [x.to_degrees(), y.to_degrees(), z.to_degrees()],
                scale:     tf.scale.x,
                latitude:  device.latitude,
                longitude: device.longitude,
            }
        })
        .collect();

    let camera_layout = camera.single().ok()
        .map(|tf| CameraLayout {
            position: tf.translation.into(),
            look_at:  [0.0, 0.0, 0.0],
        })
        .unwrap_or_default();

    let prev_meta        = &layout.scene.meta;
    let prev_backgrounds = layout.scene.backgrounds.clone();
    let prev_assets      = layout.scene.assets.clone();
    layout.scene = SceneLayout {
        meta: LayoutMeta {
            version:    1,
            created_at: prev_meta.created_at.clone(),
            last_saved: chrono::Utc::now().to_rfc3339(),
            name:       prev_meta.name.clone(),
        },
        devices:     entries,
        camera:      camera_layout,
        backgrounds: prev_backgrounds,
        assets:      prev_assets,
    };

    match layout.scene.save_default() {
        Ok(())   => tracing::debug!("Layout auto-saved"),
        Err(e)   => tracing::warn!("Failed to save layout: {e}"),
    }
}

/// Allow dragging devices in the 3D view to reposition them.
/// Uses bevy_picking `Pointer<Drag>` events.
/// Drag delta is in screen pixels; we scale by ~0.03 to approximate world units.
pub fn handle_device_drag(
    mut drag_events: MessageReader<
        bevy::picking::events::Pointer<bevy::picking::events::Drag>,
    >,
    mut transforms: Query<&mut Transform, With<DeviceEntity>>,
) {
    const DRAG_SCALE: f32 = 0.03;

    for event in drag_events.read() {
        let Ok(mut tf) = transforms.get_mut(event.entity) else { continue };
        let delta = event.event.delta;
        tf.translation.x += delta.x * DRAG_SCALE;
        tf.translation.z += delta.y * DRAG_SCALE;  // Screen Y → World Z
    }
}

/// Apply saved layout positions to devices when they are first spawned.
/// Runs every frame but is a no-op once the layout has been applied.
pub fn apply_saved_layout(
    layout:    Res<CurrentLayout>,
    mut query: Query<(&DeviceEntity, &mut Transform)>,
) {
    if layout.scene.devices.is_empty() { return; }

    for (device, mut tf) in query.iter_mut() {
        if let Some(entry) = layout.scene.devices.iter().find(|e| e.id == device.device_id) {
            let [px, py, pz] = entry.position;
            let [rx, ry, rz] = entry.rotation;
            tf.translation = Vec3::new(px, py, pz);
            tf.rotation    = Quat::from_euler(
                EulerRot::XYZ,
                rx.to_radians(),
                ry.to_radians(),
                rz.to_radians(),
            );
            tf.scale       = Vec3::splat(entry.scale);
        }
    }
}
