use std::sync::Arc;
use tracing::info;
use vl_config::VieLangConfig;
use vl_dao::DbPool;

use crate::AppState;
use super::tasks::BackgroundTasks;

/// Start all background services: housekeeper, job scheduler, monitors, LDAP sync, token cleanup.
pub fn start_background_services(
    state: &AppState,
    config: &VieLangConfig,
    pool: &DbPool,
    tasks: &mut BackgroundTasks,
) {
    // Housekeeper (Phase 32)
    if config.housekeeper.enabled {
        state.housekeeper_service.clone().start();
        info!("Housekeeper enabled (interval={}s)", config.housekeeper.interval_secs);
    }

    // Job Scheduler (Phase 35)
    state.job_scheduler_service.clone().start();
    info!("Job scheduler enabled (tick=10s)");

    // Queue Monitor (Phase 36)
    state.queue_monitor_service.clone().start();
    info!("Queue monitor enabled (interval=60s)");

    // Cluster Monitor (Phase 39)
    state.cluster_monitor_service.clone().start();
    info!("Cluster monitor enabled (interval=15s)");

    // Usage Tracker flush loop (Phase 71)
    state.usage_tracker.clone().start_flush_loop();
    info!("Usage tracker flush loop started (interval=60s)");

    // IoT Simulator — start all enabled simulations
    if config.simulator.enabled {
        let sim_svc = state.simulator_service.clone();
        tokio::spawn(async move {
            sim_svc.start_all().await;
        });
        info!("IoT Simulator service enabled");
    }

    // LDAP periodic sync (P4)
    if config.auth.ldap.sync_enabled {
        let ldap_config_dao = Arc::new(vl_dao::LdapConfigDao::new(pool.clone()));
        let ldap_svc = Arc::new(crate::services::ldap_sync::LdapSyncService::new(
            ldap_config_dao,
            state.user_dao.clone(),
        ));
        let interval = config.auth.ldap.sync_interval_secs;
        tasks.spawn("ldap-sync", async move {
            ldap_svc.run_sync_loop(interval).await;
        });
        info!("LDAP sync enabled (interval={}s)", config.auth.ldap.sync_interval_secs);
    }

    // Activation token cleanup (P4)
    {
        let token_dao = Arc::new(vl_dao::ActivationTokenDao::new(pool.clone()));
        let cleanup_interval = config.auth.session_cleanup_interval_secs;
        tasks.spawn("token-cleanup", async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(cleanup_interval),
            );
            interval.tick().await;
            loop {
                interval.tick().await;
                match token_dao.cleanup_expired().await {
                    Ok(n) if n > 0 => tracing::debug!("Cleaned up {n} expired activation tokens"),
                    _ => {}
                }
            }
        });
    }
}
