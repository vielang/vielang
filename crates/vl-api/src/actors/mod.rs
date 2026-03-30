//! Application-level actors — ThingsBoard-compatible actor hierarchy.
//!
//! ```text
//! AppActor
//!   └── TenantActor (per tenant)
//!       ├── RuleChainActor (per rule chain)
//!       │   └── RuleNodeActor (per rule node)
//!       ├── DeviceActor (per device, lazy-created on first message)
//!       └── CalculatedFieldManagerActor (optional)
//! ```

mod app_actor;
mod tenant_actor;
mod device_actor;
mod rule_chain_actor;
mod rule_node_actor;

pub use app_actor::AppActor;
pub use tenant_actor::TenantActor;
pub use device_actor::DeviceActor;
pub use rule_chain_actor::RuleChainActor;
pub use rule_node_actor::RuleNodeActor;

use std::sync::Arc;
use vl_actor::{TbActorId, TbActorSystem, TbActorSystemSettings};
use vl_dao::postgres::tenant::TenantDao;

/// Shared context that all application actors can access.
///
/// Contains references to DAOs and services needed by actors.
/// This is separate from `AppState` — actors get only what they need.
#[derive(Clone)]
pub struct ActorSystemCtx {
    pub tenant_dao: Arc<TenantDao>,
    pub device_dao: Arc<vl_dao::postgres::device::DeviceDao>,
    pub rule_chain_dao: Arc<vl_dao::postgres::rule_chain::RuleChainDao>,
    pub rule_node_dao: Arc<vl_dao::RuleNodeDao>,
    pub rule_node_state_dao: Arc<vl_dao::RuleNodeStateDao>,
    pub calc_field_dao: Arc<vl_dao::CalculatedFieldDao>,
    pub rule_engine: Arc<vl_rule_engine::RuleEngine>,
    pub queue_producer: Arc<dyn vl_queue::TbProducer>,
    pub re_registry: Arc<vl_rule_engine::TenantChainRegistry>,
}

/// Initialize the actor system and create the root AppActor.
///
/// Returns the `TbActorSystem` for sending messages and the root `TbActorRef`.
pub fn init_actor_system(
    ctx: ActorSystemCtx,
    settings: TbActorSystemSettings,
) -> TbActorSystem {
    let system = TbActorSystem::new(settings);

    let app_actor_id = TbActorId::named("APP");
    let ctx_clone = ctx.clone();
    let _ = system.create_root_actor(app_actor_id.clone(), move || {
        Box::new(AppActor::new(ctx_clone))
    });

    // Send AppInit to kick off tenant actor creation.
    system.tell(&app_actor_id, vl_actor::ActorMsg::AppInit);

    system
}
