use std::sync::Arc;
use std::time::Duration;
use vl_config::HousekeeperConfig;
use vl_dao::{CleanupStats, HousekeeperDao};
use tracing::{error, info};

pub struct HousekeeperService {
    dao: Arc<HousekeeperDao>,
    config: HousekeeperConfig,
}

impl HousekeeperService {
    pub fn new(dao: Arc<HousekeeperDao>, config: HousekeeperConfig) -> Self {
        Self { dao, config }
    }

    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(self.config.interval_secs));
            interval.tick().await; // skip first immediate tick
            loop {
                interval.tick().await;
                if let Err(e) = self.run_cycle().await {
                    error!("Housekeeper cycle failed: {}", e);
                }
            }
        })
    }

    pub async fn run_cycle(&self) -> anyhow::Result<()> {
        let exec_id = self.dao.start_execution().await?;
        info!("Housekeeper starting execution {}", exec_id);

        let now_ms = chrono::Utc::now().timestamp_millis();
        let ms_per_day = 86_400_000i64;
        let batch = self.config.batch_size;

        let cleaned_telemetry = self.dao
            .delete_old_telemetry(now_ms - self.config.ts_ttl_days * ms_per_day, batch)
            .await
            .unwrap_or_else(|e| { error!("Housekeeper telemetry cleanup error: {}", e); 0 });

        let cleaned_events = self.dao
            .delete_old_events(now_ms - self.config.events_ttl_days * ms_per_day, batch)
            .await
            .unwrap_or_else(|e| { error!("Housekeeper events cleanup error: {}", e); 0 });

        let cleaned_alarms = self.dao
            .delete_old_alarms(now_ms - self.config.alarms_ttl_days * ms_per_day, batch)
            .await
            .unwrap_or_else(|e| { error!("Housekeeper alarms cleanup error: {}", e); 0 });

        let cleaned_rpc = self.dao
            .delete_old_rpc(now_ms - self.config.rpc_ttl_days * ms_per_day, batch)
            .await
            .unwrap_or_else(|e| { error!("Housekeeper rpc cleanup error: {}", e); 0 });

        info!(
            "Housekeeper done: ts={} events={} alarms={} rpc={}",
            cleaned_telemetry, cleaned_events, cleaned_alarms, cleaned_rpc
        );

        let stats = CleanupStats {
            cleaned_telemetry,
            cleaned_events,
            cleaned_alarms,
            cleaned_rpc,
        };
        self.dao.finish_execution(exec_id, stats, "DONE").await?;
        Ok(())
    }
}
