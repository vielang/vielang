use anyhow::Context;
use tracing::info;
use vl_config::VieLangConfig;
use vl_dao::DbPool;

use crate::telemetry;

use super::tasks::BackgroundTasks;

/// Infrastructure state returned by [`init_infra`].
pub struct InfraState {
    pub config: VieLangConfig,
    pub pool: DbPool,
    pub prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
}

/// Initialize core infrastructure: config, tracing, metrics, database, migrations, KV warmup.
///
/// Returns the infrastructure state and a tracing guard (caller must keep alive).
pub async fn init_infra(tasks: &mut BackgroundTasks) -> anyhow::Result<(InfraState, impl Drop + use<>)> {
    // Rustls CryptoProvider — must be called before any TLS/JWT
    rustls::crypto::ring::default_provider()
        .install_default()
        .ok();

    // Config
    let config = VieLangConfig::load()
        .context("Failed to load configuration")?;

    // Tracing (structured logging + optional OTLP)
    let tracing_guard = telemetry::init_tracing(&config.observability);

    info!("Starting VieLang v{}", env!("CARGO_PKG_VERSION"));

    // Prometheus metrics
    let prometheus_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .context("Failed to install Prometheus recorder")?;
    crate::metrics::init_metrics();

    info!("Server: {}:{}", config.server.host, config.server.port);

    // Database pool
    let pool = vl_dao::postgres::init_pool(&config.database.postgres)
        .await
        .context("Failed to connect to database")?;

    // Migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;
    info!("Database migrations applied successfully");

    // KV cache warm-up (Phase 55.1)
    {
        let kv = vl_dao::postgres::kv::KvDao::new(pool.clone());
        if let Err(e) = kv.warm_up().await {
            tracing::warn!("KvDao warm-up failed (non-fatal): {}", e);
        }
    }

    // Partition manager (Phase 55.2)
    {
        let pm = vl_dao::PartitionManagerDao::new(pool.clone());
        if let Err(e) = pm.ensure_future_partitions().await {
            tracing::warn!("Partition manager startup check failed (non-fatal): {}", e);
        }
        let pm_pool = pool.clone();
        tasks.spawn("partition-manager", async move {
            let pm = vl_dao::PartitionManagerDao::new(pm_pool);
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(86_400));
            interval.tick().await;
            loop {
                interval.tick().await;
                if let Err(e) = pm.ensure_future_partitions().await {
                    tracing::warn!("Partition manager periodic check failed: {}", e);
                }
            }
        });
        info!("Partition manager: ts_kv partitions ensured (daily check enabled)");
    }

    Ok((
        InfraState { config, pool, prometheus_handle },
        tracing_guard,
    ))
}
