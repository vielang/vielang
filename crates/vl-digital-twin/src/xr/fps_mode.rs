//! Phase 32 — Desktop first-person (FPS) camera mode.
//!
//! Toggle with **F** key. In FPS mode:
//! - WASD moves the camera
//! - Mouse look (drag) rotates yaw/pitch
//! - ESC exits FPS mode back to orbit camera
//!
//! This runs on both native and WASM. WebXR integration is in `vr_state.rs`.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

// ── Resource ─────────────────────────────────────────────────────────────────

/// State for desktop FPS navigation mode.
#[derive(Resource, Default)]
pub struct FpsModeState {
    pub active:   bool,
    pub yaw:      f32,   // degrees
    pub pitch:    f32,   // degrees, clamped ±89°
    pub position: Vec3,
}

impl FpsModeState {
    pub fn enter(&mut self, current_transform: &Transform) {
        self.active   = true;
        self.position = current_transform.translation;
        let (yaw_r, pitch_r, _) = current_transform.rotation.to_euler(EulerRot::YXZ);
        self.yaw   =  yaw_r.to_degrees();
        self.pitch = pitch_r.to_degrees();
    }

    pub fn exit(&mut self) {
        self.active = false;
    }

    fn rotation(&self) -> Quat {
        Quat::from_euler(
            EulerRot::YXZ,
            self.yaw.to_radians(),
            self.pitch.to_radians(),
            0.0,
        )
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Toggle FPS mode on F key; exit on Escape (handled by keyboard_system too).
pub fn toggle_fps_mode(
    keyboard:  Res<ButtonInput<KeyCode>>,
    mut state: ResMut<FpsModeState>,
    cam_query: Query<&Transform, With<Camera3d>>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        if state.active {
            state.exit();
            tracing::info!("FPS mode: off");
        } else {
            if let Ok(tf) = cam_query.single() {
                state.enter(tf);
                tracing::info!("FPS mode: on");
            }
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) && state.active {
        state.exit();
    }
}

/// Drive the camera when FPS mode is active.
pub fn update_fps_camera(
    time:       Res<Time>,
    keyboard:   Res<ButtonInput<KeyCode>>,
    mouse:      Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut contexts: EguiContexts,
    mut state:  ResMut<FpsModeState>,
    mut cam_q:  Query<&mut Transform, With<Camera3d>>,
) {
    if !state.active { return; }

    // Don't process mouse look when egui is capturing
    let egui_capturing = contexts.ctx_mut()
        .map(|ctx| ctx.is_pointer_over_area())
        .unwrap_or(false);

    let dt    = time.delta_secs();
    let speed = 5.0_f32;
    let sens  = 0.3_f32;

    // ── Mouse look ────────────────────────────────────────────────────────────
    if mouse.pressed(MouseButton::Right) && !egui_capturing {
        for ev in motion.read() {
            state.yaw   -= ev.delta.x * sens;
            state.pitch -= ev.delta.y * sens;
            state.pitch  = state.pitch.clamp(-89.0, 89.0);
        }
    } else {
        motion.clear();
    }

    // ── WASD movement ─────────────────────────────────────────────────────────
    let rot       = state.rotation();
    let forward   = rot * Vec3::NEG_Z;
    let right_dir = rot * Vec3::X;
    let mut delta = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) { delta += forward; }
    if keyboard.pressed(KeyCode::KeyS) { delta -= forward; }
    if keyboard.pressed(KeyCode::KeyA) { delta -= right_dir; }
    if keyboard.pressed(KeyCode::KeyD) { delta += right_dir; }
    if keyboard.pressed(KeyCode::KeyE) || keyboard.pressed(KeyCode::Space) {
        delta += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::KeyQ) { delta -= Vec3::Y; }

    if delta != Vec3::ZERO {
        state.position += delta.normalize() * speed * dt;
    }

    // ── Apply to camera ───────────────────────────────────────────────────────
    if let Ok(mut tf) = cam_q.single_mut() {
        tf.translation = state.position;
        tf.rotation    = rot;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_state_default_inactive() {
        let s = FpsModeState::default();
        assert!(!s.active);
    }

    #[test]
    fn fps_state_exit_clears_active() {
        let mut s = FpsModeState { active: true, ..Default::default() };
        s.exit();
        assert!(!s.active);
    }

    #[test]
    fn fps_rotation_identity_at_zero() {
        let s = FpsModeState::default();
        // yaw=0, pitch=0 → camera looks along -Z (Bevy default)
        let rot = s.rotation();
        let fwd = rot * Vec3::NEG_Z;
        assert!((fwd - Vec3::NEG_Z).length() < 1e-5);
    }
}
