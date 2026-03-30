use std::sync::Arc;

use tracing::{info, warn};
use uuid::Uuid;

use vl_dao::{
    OtaDeviceState, OtaStateDao, OtaUpdateStatus,
    postgres::{
        device::DeviceDao,
        ota_package::OtaPackageDao,
    },
};
use vl_transport::DeviceWriteRegistry;
use vl_transport::mqtt::codec::encode_publish;

use crate::error::ApiError;

/// OTA State Service — assign firmware, track state machine, notify devices
pub struct OtaService {
    ota_state_dao:   Arc<OtaStateDao>,
    ota_package_dao: Arc<OtaPackageDao>,
    device_dao:      Arc<DeviceDao>,
    device_registry: Arc<DeviceWriteRegistry>,
}

impl OtaService {
    pub fn new(
        ota_state_dao:   Arc<OtaStateDao>,
        ota_package_dao: Arc<OtaPackageDao>,
        device_dao:      Arc<DeviceDao>,
        device_registry: Arc<DeviceWriteRegistry>,
    ) -> Self {
        Self { ota_state_dao, ota_package_dao, device_dao, device_registry }
    }

    /// Assign firmware package → device, tạo QUEUED state, gửi MQTT notification
    pub async fn assign_firmware(
        &self,
        device_id:  Uuid,
        package_id: Uuid,
    ) -> Result<OtaDeviceState, ApiError> {
        // Verify package tồn tại
        let pkg = self.ota_package_dao.find_by_id(package_id).await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound(format!("OtaPackage [{}] not found", package_id)))?;

        // Verify device tồn tại
        self.device_dao.find_by_id(device_id).await
            .map_err(|e| ApiError::Internal(e.to_string()))?
            .ok_or_else(|| ApiError::NotFound(format!("Device [{}] not found", device_id)))?;

        let now = now_ms();
        let state = OtaDeviceState {
            id:             Uuid::new_v4(),
            device_id,
            ota_package_id: package_id,
            status:         OtaUpdateStatus::Queued,
            error:          None,
            created_time:   now,
            updated_time:   now,
        };

        self.ota_state_dao.upsert(&state).await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        info!(device_id = %device_id, pkg_id = %package_id, "OTA firmware assigned — QUEUED");

        // Gửi MQTT shared attributes notification nếu device đang connected
        self.notify_device_mqtt(device_id, &pkg.title, &pkg.version, pkg.data_size, pkg.checksum.as_deref(), pkg.checksum_algorithm.map(|a| a.as_str())).await;

        Ok(state)
    }

    /// Cập nhật OTA state từ device telemetry/report
    pub async fn process_status_update(
        &self,
        device_id: Uuid,
        status:    &str,
        error_msg: Option<&str>,
    ) -> Result<(), ApiError> {
        // Tìm package đang pending cho device
        let pkg = match self.ota_package_dao.find_pending_for_device(device_id).await
            .map_err(|e| ApiError::Internal(e.to_string()))? {
            Some(p) => p,
            None => {
                warn!(device_id = %device_id, "OTA status update but no pending firmware");
                return Ok(());
            }
        };

        let now = now_ms();
        let state = OtaDeviceState {
            id:             Uuid::new_v4(),
            device_id,
            ota_package_id: pkg.id,
            status:         OtaUpdateStatus::from_str(status),
            error:          error_msg.map(|s| s.to_string()),
            created_time:   now,
            updated_time:   now,
        };

        self.ota_state_dao.upsert(&state).await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        info!(device_id = %device_id, status = status, "OTA state updated");
        Ok(())
    }

    /// Lấy OTA state hiện tại của device
    pub async fn get_device_ota_status(
        &self,
        device_id: Uuid,
    ) -> Result<Option<OtaDeviceState>, ApiError> {
        self.ota_state_dao.find_latest_by_device(device_id).await
            .map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Gửi MQTT shared attributes notification cho device về firmware mới
    /// Topic: v1/devices/me/attributes
    /// Payload: {"shared": {"fw_title": ..., "fw_version": ..., ...}}
    async fn notify_device_mqtt(
        &self,
        device_id:          Uuid,
        title:              &str,
        version:            &str,
        size:               Option<i64>,
        checksum:           Option<&str>,
        checksum_algorithm: Option<&str>,
    ) {
        let mut shared = serde_json::json!({
            "fw_title":   title,
            "fw_version": version,
        });

        if let Some(s) = size {
            shared["fw_size"] = serde_json::json!(s);
        }
        if let Some(cs) = checksum {
            shared["fw_checksum"] = serde_json::json!(cs);
        }
        if let Some(alg) = checksum_algorithm {
            shared["fw_checksum_algorithm"] = serde_json::json!(alg);
        }

        let payload = serde_json::json!({ "shared": shared });
        let payload_bytes = payload.to_string();

        if let Some(write_tx) = self.device_registry.get(&device_id) {
            let packet = encode_publish("v1/devices/me/attributes", payload_bytes.as_bytes());
            if let Err(e) = write_tx.send(packet).await {
                warn!(device_id = %device_id, error = %e, "Failed to send OTA MQTT notification");
            } else {
                info!(device_id = %device_id, title = title, version = version, "OTA MQTT notification sent");
            }
        } else {
            info!(device_id = %device_id, "Device not connected — OTA notification will be delivered on next connect (QUEUED)");
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
