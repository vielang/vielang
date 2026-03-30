use std::sync::Arc;
use anyhow::Context;
use tracing::info;

use super::infra::InfraState;
use super::tasks::BackgroundTasks;
use crate::AppState;

/// Core services state returned by [`init_core_services`].
pub struct CoreServices {
    pub state: AppState,
    pub rule_engine_sender: tokio::sync::mpsc::Sender<vl_core::entities::TbMsg>,
    pub ts_dao: Arc<dyn vl_dao::TimeseriesDao>,
    pub activity_tx: tokio::sync::mpsc::Sender<vl_core::entities::ActivityEvent>,
    pub actor_system: vl_actor::TbActorSystem,
}

/// Initialize cache, rule engine, cluster, timeseries, queue, activity service, and build AppState.
pub async fn init_core_services(
    infra: &InfraState,
    tasks: &mut BackgroundTasks,
) -> anyhow::Result<CoreServices> {
    let config = &infra.config;
    let pool = &infra.pool;

    // Cache
    let cache = vl_cache::create_cache_async(&config.cache)
        .await
        .context("Failed to initialize cache")?;
    info!("Cache backend: {:?}", config.cache.cache_type);

    // Rule Engine (multi-tenant mode)
    let rule_chain_dao = Arc::new(vl_dao::postgres::rule_chain::RuleChainDao::new(pool.clone()));
    let re_registry = Arc::new(vl_rule_engine::TenantChainRegistry::new(rule_chain_dao));
    let rule_engine = vl_rule_engine::RuleEngine::start_with_registry(pool.clone(), re_registry.clone());
    info!("Rule engine started (multi-tenant mode)");
    let rule_engine_sender = rule_engine.sender();

    // Queue (requires rule_engine_sender for consumers)
    let queue_state = super::queue::init_queue(config, pool, rule_engine_sender.clone(), tasks).await?;

    // Cluster
    let cluster = vl_cluster::ClusterManager::new(&config.cluster)
        .await
        .context("Failed to initialize cluster manager")?;
    info!(
        "Cluster: {} mode (node_id={})",
        if config.cluster.enabled { "distributed" } else { "single-node" },
        cluster.local_node_id(),
    );

    // Timeseries DAO — PostgreSQL or Cassandra
    let ts_dao: Arc<dyn vl_dao::TimeseriesDao> = match config.database.timeseries.ts_type {
        vl_config::TsBackendType::Cassandra => {
            let cass_cfg = config.database.timeseries.cassandra.as_ref()
                .context("database.timeseries.cassandra config required when ts_type = cassandra")?;
            let cass_cluster = vl_cassandra::CassandraCluster::connect(cass_cfg)
                .await
                .context("Failed to connect to Cassandra")?;
            let granularity = cass_cfg.partition_granularity.parse()
                .unwrap_or(vl_cassandra::PartitionGranularity::Months);
            let dao = vl_cassandra::CassandraTs::new(
                cass_cluster.session(),
                cass_cluster.keyspace(),
                granularity,
                cass_cfg.ttl_seconds,
                cass_cfg.partition_cache_size,
            )
            .await
            .context("Failed to initialize Cassandra timeseries DAO")?;
            info!("Timeseries backend: Cassandra ({})", cass_cfg.url);
            Arc::new(dao)
        }
        vl_config::TsBackendType::Sql => {
            info!("Timeseries backend: PostgreSQL");
            Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()))
        }
    };

    // Activity Service (Phase 31)
    let activity_dao = vl_dao::DeviceActivityDao::new(pool.clone());
    let activity_tx = crate::services::activity::ActivityService::start(activity_dao);

    // AppState
    let state = AppState::new(
        pool.clone(),
        config.clone(),
        ts_dao.clone(),
        rule_engine,
        queue_state.queue_producer.clone(),
        cache.clone(),
        cluster,
        activity_tx.clone(),
    );

    // Actor System (Phase 7)
    let actor_system = {
        let actor_ctx = crate::actors::ActorSystemCtx {
            tenant_dao: state.tenant_dao.clone(),
            device_dao: state.device_dao.clone(),
            rule_chain_dao: state.rule_chain_dao.clone(),
            rule_node_dao: state.rule_node_dao.clone(),
            rule_node_state_dao: state.rule_node_state_dao.clone(),
            calc_field_dao: state.calc_field_dao.clone(),
            rule_engine: state.rule_engine.clone(),
            queue_producer: state.queue_producer.clone(),
            re_registry: state.re_registry.clone(),
        };
        let settings = vl_actor::TbActorSystemSettings {
            actor_throughput: config.server.actor_throughput.unwrap_or(30),
            max_init_attempts: config.server.max_actor_init_attempts.unwrap_or(10),
        };
        crate::actors::init_actor_system(actor_ctx, settings)
    };
    info!("Actor system started");

    // Cluster RPC handler (Phase 55.3)
    {
        let handler = crate::services::cluster_handler::TbRpcHandler::new(
            state.rule_engine.sender(),
            state.ws_tx.clone(),
        );
        state.cluster.start_rpc_with_handler(handler);
        info!("Cluster RPC handler registered");
    }

    Ok(CoreServices { state, rule_engine_sender, ts_dao, activity_tx, actor_system })
}
