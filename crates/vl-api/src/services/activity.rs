use std::collections::HashMap;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, warn};
use uuid::Uuid;

use vl_core::entities::ActivityEvent;
use vl_dao::DeviceActivityDao;

/// Background service nhận ActivityEvent từ transport layer qua mpsc channel.
/// Connect/Disconnect → flush ngay (quan trọng cho online status).
/// Telemetry/Rpc → buffer 1s để tránh DB overload khi nhiều device active cùng lúc.
pub struct ActivityService {
    rx:               mpsc::Receiver<ActivityEvent>,
    dao:              DeviceActivityDao,
    telemetry_buffer: HashMap<Uuid, i64>,  // device_id → latest ts
    rpc_buffer:       HashMap<Uuid, i64>,
}

impl ActivityService {
    /// Khởi động service, trả về sender để các transport gửi events vào.
    pub fn start(dao: DeviceActivityDao) -> mpsc::Sender<ActivityEvent> {
        let (tx, rx) = mpsc::channel(10_000);
        let svc = Self {
            rx,
            dao,
            telemetry_buffer: HashMap::new(),
            rpc_buffer:       HashMap::new(),
        };
        tokio::spawn(async move { svc.run().await });
        tx
    }

    async fn run(mut self) {
        let mut flush_interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                Some(event) = self.rx.recv() => self.handle_event(event).await,
                _ = flush_interval.tick() => self.flush().await,
            }
        }
    }

    async fn handle_event(&mut self, event: ActivityEvent) {
        match event {
            ActivityEvent::Connected { device_id, ts } => {
                debug!(device_id = %device_id, "Activity: connected");
                if let Err(e) = self.dao.update_connect(device_id, ts).await {
                    warn!(device_id = %device_id, error = %e, "Failed to update connect activity");
                }
            }
            ActivityEvent::Disconnected { device_id, ts } => {
                debug!(device_id = %device_id, "Activity: disconnected");
                // Flush buffered telemetry for this device before marking inactive
                if let Some(ts_buffered) = self.telemetry_buffer.remove(&device_id) {
                    self.dao.update_telemetry(device_id, ts_buffered).await.ok();
                }
                self.rpc_buffer.remove(&device_id);
                if let Err(e) = self.dao.update_disconnect(device_id, ts).await {
                    warn!(device_id = %device_id, error = %e, "Failed to update disconnect activity");
                }
            }
            ActivityEvent::Telemetry { device_id, ts } => {
                // Buffer — keep the latest ts per device
                self.telemetry_buffer.entry(device_id)
                    .and_modify(|t| *t = (*t).max(ts))
                    .or_insert(ts);
            }
            ActivityEvent::Rpc { device_id, ts } => {
                self.rpc_buffer.entry(device_id)
                    .and_modify(|t| *t = (*t).max(ts))
                    .or_insert(ts);
            }
        }
    }

    async fn flush(&mut self) {
        let telemetry = std::mem::take(&mut self.telemetry_buffer);
        for (device_id, ts) in telemetry {
            if let Err(e) = self.dao.update_telemetry(device_id, ts).await {
                warn!(device_id = %device_id, error = %e, "Failed to flush telemetry activity");
            }
        }

        let rpcs = std::mem::take(&mut self.rpc_buffer);
        for (device_id, ts) in rpcs {
            if let Err(e) = self.dao.update_rpc(device_id, ts).await {
                warn!(device_id = %device_id, error = %e, "Failed to flush rpc activity");
            }
        }
    }
}
