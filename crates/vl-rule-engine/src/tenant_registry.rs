use std::sync::Arc;

use dashmap::DashMap;
use tracing::{info, warn};
use uuid::Uuid;

use vl_dao::postgres::rule_chain::RuleChainDao;

use crate::{
    chain::RuleChain,
    error::RuleEngineError,
    registry::NodeRegistry,
};

/// Per-tenant rule chain cache.
///
/// Lazy-loads the root rule chain for each tenant from DB on first message.
/// The caller (rule engine worker) calls `get_or_load(tenant_id)` per message;
/// subsequent messages for the same tenant get the cached compiled chain — O(1).
///
/// Call `invalidate(tenant_id)` after updating a tenant's rule chain via API
/// so the next message triggers a fresh DB load.
pub struct TenantChainRegistry {
    /// tenant_id → compiled root chain executor
    chains:           DashMap<Uuid, Arc<RuleChain>>,
    rule_chain_dao:   Arc<RuleChainDao>,
    node_registry:    NodeRegistry,
}

impl TenantChainRegistry {
    pub fn new(rule_chain_dao: Arc<RuleChainDao>) -> Self {
        Self {
            chains:         DashMap::new(),
            rule_chain_dao,
            node_registry:  NodeRegistry,
        }
    }

    /// Return the compiled root chain for `tenant_id`, loading from DB if not cached.
    /// Returns `Ok(None)` when the tenant has no root rule chain configured.
    pub async fn get_or_load(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<Arc<RuleChain>>, RuleEngineError> {
        // Fast path — already compiled and cached
        if let Some(chain) = self.chains.get(&tenant_id) {
            return Ok(Some(chain.value().clone()));
        }

        // Slow path — fetch root chain from DB
        let db_chain = self.rule_chain_dao
            .find_root_by_tenant(tenant_id)
            .await
            .map_err(|e| RuleEngineError::Config(format!("DB load failed for tenant {tenant_id}: {e}")))?;

        let Some(db_chain) = db_chain else {
            return Ok(None);
        };

        let Some(config_str) = &db_chain.configuration else {
            warn!(tenant_id = %tenant_id, chain_id = %db_chain.id, "Root rule chain has no configuration — skipping");
            return Ok(None);
        };

        match RuleChain::from_config_str(db_chain.id, config_str, &self.node_registry) {
            Ok(chain) => {
                let chain = Arc::new(chain);
                self.chains.insert(tenant_id, chain.clone());
                info!(tenant_id = %tenant_id, chain_id = %db_chain.id, "Rule chain loaded and compiled for tenant");
                Ok(Some(chain))
            }
            Err(e) => {
                warn!(tenant_id = %tenant_id, error = %e, "Failed to compile rule chain — not caching");
                Err(e)
            }
        }
    }

    /// Evict a single tenant's cached chain.
    /// The next message for this tenant will trigger a fresh DB load.
    pub fn invalidate(&self, tenant_id: Uuid) {
        if self.chains.remove(&tenant_id).is_some() {
            info!(tenant_id = %tenant_id, "Rule chain cache invalidated for tenant");
        }
    }

    /// Evict all cached chains (e.g. after bulk import or global config change).
    pub fn invalidate_all(&self) {
        let count = self.chains.len();
        self.chains.clear();
        info!(count, "All rule chain caches invalidated");
    }

    /// Number of tenants with a currently-compiled chain in cache.
    pub fn cached_count(&self) -> usize {
        self.chains.len()
    }
}
