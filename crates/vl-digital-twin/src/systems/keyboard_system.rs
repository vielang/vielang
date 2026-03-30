//! Global keyboard shortcuts for layout switching, playback control, etc.

use bevy::prelude::*;

use crate::components::current_time_ms;
use crate::playback::PlaybackState;
use crate::systems::layout_system::{AutoSaveTimer, CurrentLayout, save_layout_now};
use crate::ui::LayoutMode;

/// Handle global keyboard shortcuts.
///
/// | Key       | Action                              |
/// |-----------|-------------------------------------|
/// | F1        | Side-by-side layout                 |
/// | F2        | Bottom dashboard                    |
/// | F11       | Fullscreen 3D                       |
/// | M         | Toggle geospatial map view          |
/// | Space     | Toggle playback play/pause          |
/// | Escape    | Return to Live mode                 |
/// | Ctrl+S    | Save layout immediately             |
pub fn handle_keyboard_shortcuts(
    keyboard:    Res<ButtonInput<KeyCode>>,
    mut layout_mode: ResMut<LayoutMode>,
    mut playback:    ResMut<PlaybackState>,
    time:            Res<Time>,
    device_query:    Query<(&crate::components::DeviceEntity, &Transform)>,
    camera_query:    Query<&Transform, With<Camera3d>>,
    mut current_layout: ResMut<CurrentLayout>,
    mut save_timer:     ResMut<AutoSaveTimer>,
    mut map_state:      ResMut<crate::ui::MapViewState>,
) {
    // ── Map view toggle (M) ───────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::KeyM) {
        map_state.visible = !map_state.visible;
        tracing::debug!("Map view: {}", if map_state.visible { "open" } else { "closed" });
    }

    // ── Layout mode shortcuts ──────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::F1) {
        *layout_mode = LayoutMode::SideBySide;
        tracing::debug!("Layout: SideBySide");
    }
    if keyboard.just_pressed(KeyCode::F2) {
        *layout_mode = LayoutMode::BottomDashboard;
        tracing::debug!("Layout: BottomDashboard");
    }
    if keyboard.just_pressed(KeyCode::F11) {
        *layout_mode = if *layout_mode == LayoutMode::FullscreenScene {
            LayoutMode::SideBySide
        } else {
            LayoutMode::FullscreenScene
        };
        tracing::debug!("Layout: {:?}", *layout_mode);
    }

    // ── Playback shortcuts ────────────────────────────────────────────────────
    if keyboard.just_pressed(KeyCode::Space) {
        *playback = match &*playback {
            PlaybackState::Live => PlaybackState::Paused {
                at_ts: current_time_ms(),
            },
            PlaybackState::Paused { at_ts } => PlaybackState::Playing {
                current_ts:      *at_ts,
                speed:           1.0,
                last_frame_secs: time.elapsed_secs_f64(),
            },
            PlaybackState::Playing { current_ts, .. } => PlaybackState::Paused {
                at_ts: *current_ts,
            },
        };
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        *playback = PlaybackState::Live;
        tracing::info!("Playback: returned to Live");
    }

    // ── Ctrl+S — immediate save ───────────────────────────────────────────────
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && keyboard.just_pressed(KeyCode::KeyS) {
        save_layout_now(&time, &device_query, &camera_query, &mut current_layout);
        // Reset auto-save timer so it doesn't double-save shortly after
        save_timer.0.reset();
        tracing::info!("Layout saved via Ctrl+S");
    }
}
