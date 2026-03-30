//! vl-digital-twin library root.
//!
//! Exposes modules for both native and WASM builds.
//! The binary entry point (`main.rs`) is used for native desktop.
//! For WASM, `wasm_main()` is the entry point via `#[wasm_bindgen(start)]`.

pub mod alarm;
pub mod analytics;
pub mod api;
pub mod asset_hierarchy;
pub mod assets;
pub mod auth;
pub mod dashboard;
pub mod playback;
pub mod components;
pub mod config;
pub mod events;
pub mod plugin;
pub mod scene;
pub mod systems;
pub mod telemetry;
pub mod ui;
pub mod ws;
pub mod xr;

// ── Phase 36-41: Industrial-grade enhancements ──────────────────────────────
pub mod dtdl;
pub mod twin_graph;
pub mod aas;
pub mod automation;
pub mod annotations;
pub mod data_source;

// ── WASM entry point ─────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    use bevy::prelude::*;
    use bevy_egui::EguiPlugin;
    use plugin::{AppState, DigitalTwinPlugin};

    // WASM: dùng env-like approach (URL params) hoặc defaults
    let twin_cfg = config::TwinConfig::load();
    let initial_state = if twin_cfg.auth.token.is_empty() {
        AppState::Login
    } else {
        AppState::Running
    };

    let ws_cfg  = ws::WsConfig::from_twin_config(&twin_cfg);
    let api_cfg = api::ApiConfig::from_twin_config(&twin_cfg);

    let mut login_state = ui::LoginFormState::default();
    login_state.url      = twin_cfg.server.base_url.clone();
    login_state.username = twin_cfg.auth.username.clone();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "VieLang — 3D Digital Twin".into(),
                canvas: Some("#bevy-canvas".into()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(ws_cfg)
        .insert_resource(api_cfg)
        .insert_resource(login_state)
        .insert_state(initial_state)
        .add_plugins(DigitalTwinPlugin)
        .run();
}
