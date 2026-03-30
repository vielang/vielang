//! TwinConfig — cấu hình toàn bộ ứng dụng digital twin.
//!
//! Thứ tự ưu tiên (cao → thấp):
//!   1. Env vars: TB_BASE_URL, TB_SERVER_URL, TB_TOKEN
//!   2. File config: ~/.config/vielang/twin.toml
//!   3. Default values

use serde::{Deserialize, Serialize};

// ── Top-level config ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TwinConfig {
    pub server:    ServerConfig,
    pub auth:      AuthConfig,
    pub reconnect: ReconnectConfig,
    pub ui:        UiConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// REST base URL, e.g. "http://localhost:8080"
    pub base_url: String,
    /// WebSocket base URL, e.g. "ws://localhost:8080"
    pub ws_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// JWT token — nếu có sẽ bỏ qua login screen
    pub token:    String,
    /// Dùng cho login form (nếu không có token)
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReconnectConfig {
    pub initial_delay_ms: u64,
    pub backoff_factor:   f64,
    pub max_delay_ms:     u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    pub window_width:  u32,
    pub window_height: u32,
}

// ── Default values ────────────────────────────────────────────────────────────

impl Default for TwinConfig {
    fn default() -> Self {
        Self {
            server:    ServerConfig::default(),
            auth:      AuthConfig::default(),
            reconnect: ReconnectConfig::default(),
            ui:        UiConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".into(),
            ws_url:   "ws://localhost:8080".into(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token:    String::new(),
            username: "tenant@thingsboard.org".into(),
            password: "tenant".into(),
        }
    }
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1_000,
            backoff_factor:   2.0,
            max_delay_ms:     60_000,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            window_width:  1600,
            window_height: 900,
        }
    }
}

// ── Load logic ────────────────────────────────────────────────────────────────

impl TwinConfig {
    /// Load config với thứ tự ưu tiên: env vars > file > defaults.
    pub fn load() -> Self {
        let mut cfg = Self::load_from_file().unwrap_or_default();

        // Env vars override file và defaults
        if let Ok(v) = std::env::var("TB_BASE_URL")    { cfg.server.base_url = v; }
        if let Ok(v) = std::env::var("TB_SERVER_URL")  { cfg.server.ws_url   = v; }
        if let Ok(v) = std::env::var("TB_TOKEN")       { cfg.auth.token      = v; }

        cfg
    }

    /// Đọc file config nếu tồn tại. Trả về None nếu không có file hoặc parse lỗi.
    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_file() -> Option<Self> {
        let path = Self::config_path();
        if !path.exists() {
            tracing::debug!(path = %path.display(), "No config file found, using defaults");
            return None;
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| tracing::warn!(path = %path.display(), error = %e, "Failed to read config"))
            .ok()?;
        toml::from_str(&content)
            .map_err(|e| tracing::warn!(error = %e, "Failed to parse config, using defaults"))
            .ok()
    }

    #[cfg(target_arch = "wasm32")]
    fn load_from_file() -> Option<Self> {
        None // WASM không có filesystem
    }

    /// Đường dẫn file config theo platform.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("vielang")
            .join("twin.toml")
    }

    #[cfg(target_arch = "wasm32")]
    pub fn config_path() -> std::path::PathBuf {
        std::path::PathBuf::from("twin.toml")
    }

    /// Ghi default config ra file để người dùng tham khảo.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_default_if_missing() {
        let path = Self::config_path();
        if path.exists() { return; }
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = DEFAULT_CONFIG_TOML;
        let _ = std::fs::write(&path, content);
        tracing::info!(path = %path.display(), "Created default config file");
    }
}

// ── Default config template ───────────────────────────────────────────────────

const DEFAULT_CONFIG_TOML: &str = r#"# VieLang Digital Twin Configuration
# Đường dẫn: ~/.config/vielang/twin.toml

[server]
base_url = "http://localhost:8080"
ws_url   = "ws://localhost:8080"

[auth]
# Để trống để dùng login screen.
# Hoặc set token trực tiếp để bỏ qua login:
token    = ""
username = "tenant@thingsboard.org"
password = "tenant"

[reconnect]
initial_delay_ms = 1000
backoff_factor   = 2.0
max_delay_ms     = 60000

[ui]
window_width  = 1600
window_height = 900
"#;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = TwinConfig::default();
        assert_eq!(cfg.server.base_url, "http://localhost:8080");
        assert_eq!(cfg.server.ws_url,   "ws://localhost:8080");
        assert!(cfg.auth.token.is_empty());
        assert_eq!(cfg.reconnect.initial_delay_ms, 1_000);
        assert_eq!(cfg.reconnect.max_delay_ms, 60_000);
    }

    #[test]
    fn default_config_toml_parses() {
        let result: Result<TwinConfig, _> = toml::from_str(DEFAULT_CONFIG_TOML);
        assert!(result.is_ok(), "DEFAULT_CONFIG_TOML failed to parse: {:?}", result.err());
    }

    #[test]
    fn backoff_cap() {
        let cfg = ReconnectConfig::default();
        let mut delay = cfg.initial_delay_ms;
        for _ in 0..20 {
            delay = ((delay as f64 * cfg.backoff_factor) as u64).min(cfg.max_delay_ms);
        }
        assert_eq!(delay, cfg.max_delay_ms, "Backoff should cap at max_delay_ms");
    }
}
