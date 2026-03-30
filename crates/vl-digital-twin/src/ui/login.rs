//! Login screen — hiển thị khi không có JWT token trong config.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::api::ApiConfig;
use crate::plugin::AppState;
use crate::ws::WsConfig;

// ── Resources ─────────────────────────────────────────────────────────────────

/// State của login form — lưu input và kết quả async login.
#[derive(Resource)]
pub struct LoginFormState {
    pub url:      String,
    pub username: String,
    pub password: String,
    /// Lỗi hiển thị cho user
    pub error:    Option<String>,
    /// Đang submit — disable form
    pub submitting: bool,
    /// Kết quả từ async login task (None = chưa có)
    pub task_result: Arc<Mutex<Option<Result<String, String>>>>,
}

impl Default for LoginFormState {
    fn default() -> Self {
        Self {
            url:         std::env::var("TB_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
            username:    "tenant@thingsboard.org".into(),
            password:    "tenant".into(),
            error:       None,
            submitting:  false,
            task_result: Arc::new(Mutex::new(None)),
        }
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Render login form với egui.
pub fn render_login_screen(
    mut ctx:         EguiContexts,
    mut login_state: ResMut<LoginFormState>,
) {
    let ctx = ctx.ctx_mut().expect("egui context");

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.heading("VieLang — 3D Digital Twin");
            ui.add_space(8.0);
            ui.label("Connect to VieLang / ThingsBoard backend");
            ui.add_space(24.0);

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_max_width(380.0);

                egui::Grid::new("login_form")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Server URL:");
                        ui.add_enabled(
                            !login_state.submitting,
                            egui::TextEdit::singleline(&mut login_state.url)
                                .hint_text("http://localhost:8080")
                                .desired_width(240.0),
                        );
                        ui.end_row();

                        ui.label("Username:");
                        ui.add_enabled(
                            !login_state.submitting,
                            egui::TextEdit::singleline(&mut login_state.username)
                                .desired_width(240.0),
                        );
                        ui.end_row();

                        ui.label("Password:");
                        ui.add_enabled(
                            !login_state.submitting,
                            egui::TextEdit::singleline(&mut login_state.password)
                                .password(true)
                                .desired_width(240.0),
                        );
                        ui.end_row();
                    });

                ui.add_space(8.0);

                if let Some(err) = &login_state.error.clone() {
                    ui.colored_label(egui::Color32::RED, format!("✗ {err}"));
                    ui.add_space(4.0);
                }

                ui.horizontal(|ui| {
                    let btn = egui::Button::new(
                        if login_state.submitting { "Logging in..." } else { "Login" }
                    );
                    if ui.add_enabled(!login_state.submitting, btn).clicked() {
                        submit_login(&mut login_state);
                    }

                    if login_state.submitting {
                        ui.spinner();
                    }
                });
            });

            ui.add_space(16.0);
            ui.weak(format!("Config file: {}", crate::config::TwinConfig::config_path().display()));
        });
    });
}

/// Kiểm tra kết quả async login mỗi frame.
/// Khi có token → cập nhật config resources → chuyển sang Running state.
pub fn handle_login_task(
    mut login_state: ResMut<LoginFormState>,
    mut api_config:  ResMut<ApiConfig>,
    mut ws_config:   ResMut<WsConfig>,
    mut next_state:  ResMut<NextState<AppState>>,
) {
    if !login_state.submitting {
        return;
    }

    let result = login_state.task_result.lock().ok().and_then(|mut g| g.take());

    if let Some(outcome) = result {
        login_state.submitting = false;
        match outcome {
            Ok(token) => {
                tracing::info!("Login successful, transitioning to Running");
                api_config.base_url  = login_state.url.clone();
                api_config.jwt_token = token.clone();
                ws_config.server_url = login_state.url
                    .replace("http://", "ws://")
                    .replace("https://", "wss://");
                ws_config.jwt_token  = token;
                next_state.set(AppState::Running);
            }
            Err(e) => {
                tracing::warn!(error = %e, "Login failed");
                login_state.error = Some(e);
            }
        }
    }
}

// ── Async login helper ────────────────────────────────────────────────────────

fn submit_login(state: &mut LoginFormState) {
    state.error      = None;
    state.submitting = true;

    let url      = state.url.clone();
    let username = state.username.clone();
    let password = state.password.clone();
    let result   = state.task_result.clone();

    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        let outcome = rt.block_on(do_login(url, username, password));
        if let Ok(mut guard) = result.lock() {
            *guard = Some(outcome);
        }
    });

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let outcome = do_login(url, username, password).await;
        if let Ok(mut guard) = result.lock() {
            *guard = Some(outcome);
        }
    });
}

/// Async: gọi POST /api/auth/login và trả về JWT token.
async fn do_login(base_url: String, username: String, password: String) -> Result<String, String> {
    use crate::api::{ApiConfig, client::ApiClient};

    let config = ApiConfig {
        base_url:  base_url.trim_end_matches('/').to_string(),
        jwt_token: String::new(),
    };
    let client = ApiClient::new(config);
    client.login(&username, &password).await
}
