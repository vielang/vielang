//! VieLang — 3D Digital Twin (Phase 20)
//!
//! Standalone Bevy application. NOT a server.
//!
//! Usage:
//!   cargo run -p vl-digital-twin
//!
//!   # Với token (bỏ qua login screen):
//!   TB_TOKEN=<jwt> cargo run -p vl-digital-twin
//!
//!   # Hoặc đặt trong config file:
//!   # ~/.config/vielang/twin.toml
//!
//! Không có backend → demo devices với dữ liệu giả lập.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod alarm;
mod analytics;
mod api;
mod asset_hierarchy;
mod assets;
mod auth;
mod dashboard;
mod playback;
mod components;
mod config;
mod events;
mod plugin;
mod scene;
mod systems;
mod telemetry;
mod ui;
mod ws;
mod xr;

use config::TwinConfig;
use plugin::{AppState, DigitalTwinPlugin};

fn main() {
    // Tạo default config file nếu chưa có (để user tham khảo)
    TwinConfig::write_default_if_missing();

    // Load config: file → env vars override
    let twin_cfg = TwinConfig::load();

    // Nếu đã có token → bỏ qua login, vào Running ngay
    let initial_state = if twin_cfg.auth.token.is_empty() {
        tracing::info!("No token found — showing login screen");
        AppState::Login
    } else {
        tracing::info!("Token found — starting in Running state");
        AppState::Running
    };

    let win_w = twin_cfg.ui.window_width;
    let win_h = twin_cfg.ui.window_height;

    // Build WsConfig và ApiConfig từ TwinConfig
    let ws_config  = ws::WsConfig::from_twin_config(&twin_cfg);
    let api_config = api::ApiConfig::from_twin_config(&twin_cfg);

    // Khởi tạo login form với giá trị từ config
    let mut login_state = ui::LoginFormState::default();
    login_state.url      = twin_cfg.server.base_url.clone();
    login_state.username = twin_cfg.auth.username.clone();
    login_state.password = twin_cfg.auth.password.clone();

    // In debug builds, resolve assets relative to this crate's source directory.
    // In release builds, assets are expected next to the executable.
    #[cfg(debug_assertions)]
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string();
    #[cfg(not(debug_assertions))]
    let asset_path = "assets".to_string();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "VieLang — 3D Digital Twin".into(),
                        resolution: bevy::window::WindowResolution::new(win_w, win_h),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    file_path: asset_path,
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        // Override resources với giá trị từ TwinConfig
        .insert_resource(ws_config)
        .insert_resource(api_config)
        .insert_resource(login_state)
        // Set initial state trước khi add plugin
        .insert_state(initial_state)
        .add_plugins(DigitalTwinPlugin)
        .run();
}
