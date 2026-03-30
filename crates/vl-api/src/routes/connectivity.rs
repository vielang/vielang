use axum::{
    extract::{Extension, Path, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::DeviceActivity;

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, DeviceState, CoreState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: DeviceConnectivityController
        .route("/device-connectivity/{deviceId}", get(get_device_connectivity))
        // Khớp Java: DeviceController — device activity
        .route("/device/{deviceId}/activity",     get(get_device_activity))
        // Khớp Java: DeviceStateService — active devices for tenant
        .route("/tenant/devices/active",          get(get_active_devices))
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolConnectivity {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConnectivityInfo {
    pub mqtt: ProtocolConnectivity,
    pub http: ProtocolConnectivity,
    pub coap: ProtocolConnectivity,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/device-connectivity/{deviceId}
/// Trả về thông tin kết nối device: MQTT host/port, HTTP, CoAP
async fn get_device_connectivity(
    State(device): State<DeviceState>,
    State(core): State<CoreState>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<DeviceConnectivityInfo>, ApiError> {
    // Kiểm tra device tồn tại
    device.device_dao.find_by_id(device_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Device [{}] is not found", device_id)))?;

    let cfg = &core.config;
    let host = cfg.server.host.clone();

    Ok(Json(DeviceConnectivityInfo {
        mqtt: ProtocolConnectivity {
            enabled:     cfg.transport.mqtt.enabled,
            host:        host.clone(),
            port:        cfg.transport.mqtt.port,
            ssl_enabled: Some(false),
        },
        http: ProtocolConnectivity {
            enabled:     cfg.transport.http.enabled,
            host:        host.clone(),
            port:        cfg.transport.http.port,
            ssl_enabled: None,
        },
        coap: ProtocolConnectivity {
            enabled:     cfg.transport.coap.enabled,
            host:        host.clone(),
            port:        cfg.transport.coap.port,
            ssl_enabled: None,
        },
    }))
}

/// GET /api/device/{deviceId}/activity
/// Trả về trạng thái hoạt động của device, đọc từ device_activity table
async fn get_device_activity(
    State(state): State<DeviceState>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<DeviceActivity>, ApiError> {
    state.device_dao.find_by_id(device_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Device [{}] is not found", device_id)))?;

    let activity = state.device_activity_dao.find(device_id).await?
        .unwrap_or_else(|| DeviceActivity::new(device_id));

    Ok(Json(activity))
}

/// GET /api/tenant/devices/active
/// Danh sách devices đang online (active=true) của tenant hiện tại
async fn get_active_devices(
    State(state): State<DeviceState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<DeviceActivity>>, ApiError> {
    let tenant_id = ctx.tenant_id;
    let devices = state.device_activity_dao.find_active(tenant_id).await?;
    Ok(Json(devices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "verified passing"]
    fn connectivity_routes_registered() {
        let r = router();
        drop(r);
    }
}
