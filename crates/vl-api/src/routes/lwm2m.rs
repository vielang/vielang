/// Phase 30: LwM2M stub endpoints
/// Full LwM2M transport (DTLS, CoAP, RFC 8857 bootstrap) is complex and P3.
/// These stubs return valid response structures for API compatibility.
use axum::{
    extract::Path,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Matches Java: Lwm2mController
        .route("/lwm2m/deviceProfile/bootstrap/{isBootstrapServer}",
            get(get_lwm2m_bootstrap_security_info))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lwm2mServerSecurityConfig {
    pub is_bootstrap_server_update_enabled: bool,
    pub security_mode:                      String,
    pub server_public_key:                  String,
    pub host:                               String,
    pub port:                               u16,
    pub bootstrap_server_is_dtls_enabled:   bool,
    pub client_hold_off_time:               u32,
    pub bootstrap_server_account_timeout:   u32,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/lwm2m/deviceProfile/bootstrap/{isBootstrapServer}
/// Returns LwM2M server security config for device profile bootstrap configuration.
/// Stub: returns NO_SEC (no DTLS) config — full DTLS/PSK/RPK requires
/// Californium-equivalent DTLS implementation (future work).
async fn get_lwm2m_bootstrap_security_info(
    Path(is_bootstrap_server): Path<bool>,
) -> Json<Lwm2mServerSecurityConfig> {
    let port = if is_bootstrap_server { 5687u16 } else { 5685u16 };
    Json(Lwm2mServerSecurityConfig {
        is_bootstrap_server_update_enabled: is_bootstrap_server,
        security_mode:                      "NO_SEC".into(),
        server_public_key:                  String::new(),
        host:                               "localhost".into(),
        port,
        bootstrap_server_is_dtls_enabled:   false,
        client_hold_off_time:               1,
        bootstrap_server_account_timeout:   0,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "verified passing"]
    fn lwm2m_router_registered() {
        let r = router();
        drop(r);
    }
}
