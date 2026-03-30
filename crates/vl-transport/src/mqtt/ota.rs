use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use vl_dao::{
    DbPool, OtaDeviceState, OtaStateDao, OtaUpdateStatus,
    postgres::ota_package::OtaPackageDao,
};

use super::codec::encode_publish;
use crate::error::TransportError;

/// OTA firmware chunk handler — xử lý MQTT v2/fw protocol
pub struct OtaHandler {
    ota_state_dao:   Arc<OtaStateDao>,
    ota_package_dao: Arc<OtaPackageDao>,
    chunk_size:      usize,
}

impl OtaHandler {
    pub fn new(pool: DbPool, chunk_size_kb: usize) -> Self {
        Self {
            ota_state_dao:   Arc::new(OtaStateDao::new(pool.clone())),
            ota_package_dao: Arc::new(OtaPackageDao::new(pool)),
            chunk_size:      chunk_size_kb * 1024,
        }
    }

    /// Xử lý topic: v2/fw/request/{requestId}/chunk/{chunkIndex}
    /// Server reply: v2/fw/response/{requestId}/chunk/{chunkIndex}
    pub async fn handle_chunk_request(
        &self,
        write_tx:    &mpsc::Sender<bytes::Bytes>,
        device_id:   Uuid,
        request_id:  &str,
        chunk_index: u32,
    ) {
        let pkg = match self.ota_package_dao.find_pending_for_device(device_id).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                warn!(device_id = %device_id, "OTA chunk request but no pending firmware");
                return;
            }
            Err(e) => {
                warn!(device_id = %device_id, error = %e, "OTA: failed to find pending firmware");
                return;
            }
        };

        if !pkg.has_data {
            warn!(device_id = %device_id, pkg_id = %pkg.id, "OTA chunk request but package has no data");
            return;
        }

        match self.ota_package_dao.get_chunk(pkg.id, chunk_index, self.chunk_size).await {
            Ok(Some(data)) => {
                debug!(
                    device_id = %device_id,
                    chunk_index = chunk_index,
                    bytes = data.len(),
                    "Sending OTA chunk"
                );
                let response_topic = format!("v2/fw/response/{}/chunk/{}", request_id, chunk_index);
                write_tx.send(encode_publish(&response_topic, &data)).await.ok();

                // Update state to DOWNLOADING on first chunk
                if chunk_index == 0 {
                    self.update_state(device_id, pkg.id, OtaUpdateStatus::Downloading, None).await;
                }
            }
            Ok(None) => {
                warn!(
                    device_id = %device_id,
                    chunk_index = chunk_index,
                    "OTA chunk out of bounds"
                );
            }
            Err(e) => {
                warn!(device_id = %device_id, error = %e, "OTA get_chunk failed");
            }
        }
    }

    /// Xử lý telemetry với current_fw_state — update state machine
    pub async fn handle_telemetry_fw_state(
        &self,
        device_id: Uuid,
        fw_state:  &str,
        error_msg: Option<&str>,
    ) {
        let pkg = match self.ota_package_dao.find_pending_for_device(device_id).await {
            Ok(Some(p)) => p,
            Ok(None) => return,
            Err(_)    => return,
        };

        let status = OtaUpdateStatus::from_str(fw_state);
        info!(
            device_id = %device_id,
            pkg_id = %pkg.id,
            status = fw_state,
            "OTA state update from device telemetry"
        );

        self.update_state(device_id, pkg.id, status, error_msg.map(|s| s.to_string())).await;
    }

    /// Ghi hoặc cập nhật OTA state record
    async fn update_state(
        &self,
        device_id:  Uuid,
        package_id: Uuid,
        status:     OtaUpdateStatus,
        error:      Option<String>,
    ) {
        let now = now_ms();
        let state = OtaDeviceState {
            id:             Uuid::new_v4(),
            device_id,
            ota_package_id: package_id,
            status,
            error,
            created_time:   now,
            updated_time:   now,
        };
        if let Err(e) = self.ota_state_dao.upsert(&state).await {
            warn!(device_id = %device_id, error = %e, "Failed to upsert OTA state");
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
