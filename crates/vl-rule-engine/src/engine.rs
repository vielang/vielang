use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{error, info};
use uuid::Uuid;

use vl_core::entities::TbMsg;
use vl_dao::{DbPool, postgres::{
    alarm::AlarmDao, asset::AssetDao, customer::CustomerDao,
    device::DeviceDao, device_profile::DeviceProfileDao,
    event::EventDao, geofence::GeofenceDao,
    kv::KvDao, relation::RelationDao, tenant::TenantDao,
}};
use crate::{
    chain::RuleChain,
    node::{DaoServices, RuleNodeCtx},
    tenant_registry::TenantChainRegistry,
};

const CHANNEL_CAPACITY: usize = 1024;

/// RuleEngine — receives TbMsg via mpsc channel and processes through rule chains.
/// Supports two modes:
///  - Single-tenant (legacy): chains loaded at startup, all messages use fixed tenant_id.
///  - Multi-tenant: TenantChainRegistry lazily loads per-tenant root chains from DB.
#[derive(Clone)]
pub struct RuleEngine {
    sender:   mpsc::Sender<TbMsg>,
    registry: Option<Arc<TenantChainRegistry>>,
}

impl RuleEngine {
    /// Start the rule engine with pre-loaded chains (single-tenant / legacy mode).
    /// `tenant_id` is used as context for all messages regardless of `msg.tenant_id`.
    pub fn start(pool: DbPool, chains: Vec<RuleChain>, tenant_id: Uuid) -> Self {
        let (tx, mut rx) = mpsc::channel::<TbMsg>(CHANNEL_CAPACITY);

        let dao_services = build_dao_services(pool);
        let chains = Arc::new(chains);

        tokio::spawn(async move {
            info!("RuleEngine worker started (chains={})", chains.len());
            while let Some(msg) = rx.recv().await {
                let start = Instant::now();
                let ctx = RuleNodeCtx {
                    node_id:     Uuid::nil(),
                    tenant_id,
                    dao:         dao_services.clone(),
                    edge_sender: None,
                };
                let mut had_error = false;
                for chain in chains.as_ref() {
                    if let Err(e) = chain.process_msg(&ctx, msg.clone()).await {
                        had_error = true;
                        metrics::counter!("vielang_rule_engine_errors_total").increment(1);
                        error!(chain_id = %chain.id, error = %e, "Rule chain processing error");
                    }
                }
                if !had_error {
                    metrics::counter!("vielang_rule_engine_messages_processed_total").increment(1);
                }
                metrics::histogram!("vielang_rule_engine_processing_time_seconds")
                    .record(start.elapsed().as_secs_f64());
            }
            info!("RuleEngine worker stopped");
        });

        Self { sender: tx, registry: None }
    }

    /// Start the multi-tenant rule engine backed by a `TenantChainRegistry`.
    ///
    /// Messages with `msg.tenant_id = Some(tid)` are routed to the tenant's root chain
    /// (loaded lazily from DB on first use). Messages without a tenant_id are silently
    /// discarded — callers must set `TbMsg::with_tenant()` before sending.
    pub fn start_with_registry(pool: DbPool, registry: Arc<TenantChainRegistry>) -> Self {
        let (tx, mut rx) = mpsc::channel::<TbMsg>(CHANNEL_CAPACITY);

        let dao_services = build_dao_services(pool);
        let reg = registry.clone();

        tokio::spawn(async move {
            info!("RuleEngine multi-tenant worker started");
            while let Some(msg) = rx.recv().await {
                let Some(tenant_id) = msg.tenant_id else {
                    // Message has no tenant context — skip silently.
                    // Transport handlers should call msg.with_tenant(tenant_id).
                    continue;
                };

                let chain = match reg.get_or_load(tenant_id).await {
                    Ok(Some(c)) => c,
                    Ok(None)    => continue, // No rule chain configured for this tenant
                    Err(e) => {
                        error!(tenant_id = %tenant_id, error = %e, "Failed to load rule chain");
                        continue;
                    }
                };

                let ctx = RuleNodeCtx {
                    node_id:     Uuid::nil(),
                    tenant_id,
                    dao:         dao_services.clone(),
                    edge_sender: None,
                };

                let start = Instant::now();
                if let Err(e) = chain.process_msg(&ctx, msg).await {
                    metrics::counter!("vielang_rule_engine_errors_total").increment(1);
                    error!(tenant_id = %tenant_id, error = %e, "Rule chain processing error");
                } else {
                    metrics::counter!("vielang_rule_engine_messages_processed_total").increment(1);
                }
                metrics::histogram!("vielang_rule_engine_processing_time_seconds")
                    .record(start.elapsed().as_secs_f64());
            }
            info!("RuleEngine multi-tenant worker stopped");
        });

        Self { sender: tx, registry: Some(registry) }
    }

    /// Start a no-op rule engine (for when no chains are configured).
    pub fn start_noop() -> Self {
        let (tx, mut rx) = mpsc::channel::<TbMsg>(CHANNEL_CAPACITY);
        tokio::spawn(async move {
            while rx.recv().await.is_some() {}
        });
        Self { sender: tx, registry: None }
    }

    /// Send a message to be processed by the rule engine.
    /// Non-blocking — drops the message if the channel is full.
    pub fn send(&self, msg: TbMsg) {
        if let Err(e) = self.sender.try_send(msg) {
            error!("RuleEngine channel full or closed: {}", e);
        }
    }

    /// Async version — waits if the channel is full.
    pub async fn send_async(&self, msg: TbMsg) {
        if let Err(e) = self.sender.send(msg).await {
            error!("RuleEngine channel closed: {}", e);
        }
    }

    /// Get a raw sender for passing to other systems (e.g. transport layer).
    pub fn sender(&self) -> mpsc::Sender<TbMsg> {
        self.sender.clone()
    }

    /// Returns true if the rule engine worker task is still running.
    pub fn is_running(&self) -> bool {
        !self.sender.is_closed()
    }

    /// Get the tenant chain registry if this engine is in multi-tenant mode.
    pub fn registry(&self) -> Option<Arc<TenantChainRegistry>> {
        self.registry.clone()
    }

    /// Evict a specific tenant's cached rule chain.
    /// The next message for this tenant will re-load from DB.
    pub fn invalidate_tenant(&self, tenant_id: Uuid) {
        if let Some(reg) = &self.registry {
            reg.invalidate(tenant_id);
        }
    }

    /// Evict all cached rule chains.
    pub fn invalidate_all(&self) {
        if let Some(reg) = &self.registry {
            reg.invalidate_all();
        }
    }
}

fn build_dao_services(pool: DbPool) -> Arc<DaoServices> {
    Arc::new(DaoServices {
        kv:             Arc::new(KvDao::new(pool.clone())),
        alarm:          Arc::new(AlarmDao::new(pool.clone())),
        device:         Arc::new(DeviceDao::new(pool.clone())),
        device_profile: Arc::new(DeviceProfileDao::new(pool.clone())),
        asset:          Arc::new(AssetDao::new(pool.clone())),
        relation:       Arc::new(RelationDao::new(pool.clone())),
        customer:       Arc::new(CustomerDao::new(pool.clone())),
        tenant:         Arc::new(TenantDao::new(pool.clone())),
        event:          Arc::new(EventDao::new(pool.clone())),
        geofence:       Arc::new(GeofenceDao::new(pool)),
    })
}
