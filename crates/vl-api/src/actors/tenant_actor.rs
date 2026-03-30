//! Per-tenant actor — manages rule chains and device actors for a single tenant.
//!
//! Mirrors ThingsBoard's `TenantActor`:
//! - Creates `RuleChainActor` for each rule chain in the tenant.
//! - Creates `DeviceActor` lazily on first device message.
//! - Routes rule engine messages to root chain or specific chain.
//! - Handles device lifecycle events.
//! - Broadcasts session timeouts to device actors.

use async_trait::async_trait;
use std::collections::HashSet;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use vl_actor::{
    ActorError, ActorMsg, EntityType, LifecycleEvent, StopReason, TbActor, TbActorCtx, TbActorId,
};

use super::{ActorSystemCtx, DeviceActor, RuleChainActor};

pub struct TenantActor {
    tenant_id: Uuid,
    sys_ctx: ActorSystemCtx,
    ctx: Option<TbActorCtx>,
    /// Root rule chain ref for this tenant.
    root_chain_id: Option<Uuid>,
    /// Deleted devices — ignore messages for these.
    deleted_devices: HashSet<Uuid>,
}

impl TenantActor {
    pub fn new(tenant_id: Uuid, sys_ctx: ActorSystemCtx) -> Self {
        Self {
            tenant_id,
            sys_ctx,
            ctx: None,
            root_chain_id: None,
            deleted_devices: HashSet::new(),
        }
    }

    /// Initialize rule chain actors for this tenant.
    async fn init_rule_chains(&mut self) {
        let chains = match self
            .sys_ctx
            .rule_chain_dao
            .find_all_by_tenant(self.tenant_id)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                error!("tenant {}: failed to load rule chains: {e}", self.tenant_id);
                return;
            }
        };

        info!(
            "tenant {}: initializing {} rule chains",
            self.tenant_id,
            chains.len()
        );

        let ctx = match &self.ctx {
            Some(c) => c,
            None => return,
        };

        for chain in &chains {
            let chain_id = chain.id;
            let is_root = chain.root;
            let sys = self.sys_ctx.clone();
            let tid = self.tenant_id;

            let actor_id = TbActorId::entity(chain_id, EntityType::RuleChain);
            match ctx.get_or_create_child(actor_id, move || {
                Box::new(RuleChainActor::new(tid, chain_id, sys))
            }) {
                Ok(_) => {
                    if is_root {
                        self.root_chain_id = Some(chain_id);
                    }
                }
                Err(e) => {
                    error!(
                        "tenant {}: failed to create RuleChainActor {chain_id}: {e}",
                        self.tenant_id
                    );
                }
            }
        }
    }

    /// Get or create a DeviceActor for the given device ID.
    fn get_or_create_device_actor(&self, device_id: Uuid) -> Option<vl_actor::TbActorRef> {
        let ctx = self.ctx.as_ref()?;
        let actor_id = TbActorId::entity(device_id, EntityType::Device);
        let tid = self.tenant_id;
        let sys = self.sys_ctx.clone();
        match ctx.get_or_create_child(actor_id, move || {
            Box::new(DeviceActor::new(tid, device_id, sys))
        }) {
            Ok(r) => Some(r),
            Err(e) => {
                error!(
                    "tenant {}: failed to create DeviceActor {device_id}: {e}",
                    self.tenant_id
                );
                None
            }
        }
    }

    /// Forward message to a device actor.
    fn forward_to_device(&self, device_id: Uuid, msg: ActorMsg, high_priority: bool) {
        if self.deleted_devices.contains(&device_id) {
            debug!("ignoring message for deleted device {device_id}");
            return;
        }
        if let Some(actor_ref) = self.get_or_create_device_actor(device_id) {
            if high_priority {
                actor_ref.tell_high_priority(msg);
            } else {
                actor_ref.tell(msg);
            }
        }
    }

    /// Forward message to a rule chain actor.
    fn forward_to_rule_chain(&self, chain_id: Uuid, msg: ActorMsg) {
        let ctx = match &self.ctx {
            Some(c) => c,
            None => return,
        };
        let actor_id = TbActorId::entity(chain_id, EntityType::RuleChain);
        ctx.tell(&actor_id, msg);
    }
}

#[async_trait]
impl TbActor for TenantActor {
    async fn init(&mut self, ctx: TbActorCtx) -> Result<(), ActorError> {
        debug!("TenantActor {} initializing", self.tenant_id);
        self.ctx = Some(ctx);
        self.init_rule_chains().await;
        Ok(())
    }

    async fn destroy(&mut self, reason: StopReason) {
        info!("TenantActor {} destroyed: {reason:?}", self.tenant_id);
    }

    async fn process(&mut self, msg: ActorMsg) -> bool {
        match msg {
            // ── Rule engine messages ──────────────────────────────
            ActorMsg::QueueToRuleEngine {
                rule_chain_id,
                msg: re_msg,
                tenant_id,
            } => {
                let target = rule_chain_id.or(self.root_chain_id);
                if let Some(chain_id) = target {
                    self.forward_to_rule_chain(
                        chain_id,
                        ActorMsg::QueueToRuleEngine {
                            tenant_id,
                            rule_chain_id: Some(chain_id),
                            msg: re_msg,
                        },
                    );
                } else {
                    warn!(
                        "tenant {}: no root rule chain, dropping message",
                        self.tenant_id
                    );
                }
                true
            }

            // ── Device messages ───────────────────────────────────
            ActorMsg::TransportToDevice { device_id, .. } => {
                self.forward_to_device(device_id, msg, false);
                true
            }

            ActorMsg::DeviceAttributesUpdate { device_id, .. }
            | ActorMsg::DeviceCredentialsUpdate { device_id, .. }
            | ActorMsg::DeviceNameOrTypeUpdate { device_id, .. }
            | ActorMsg::DeviceEdgeUpdate { device_id, .. }
            | ActorMsg::DeviceRpcRequest { device_id, .. }
            | ActorMsg::DeviceRpcResponse { device_id, .. }
            | ActorMsg::RemoveRpc { device_id, .. } => {
                self.forward_to_device(device_id, msg, true);
                true
            }

            ActorMsg::DeviceDelete { device_id, .. } => {
                // Stop device actor and mark as deleted.
                let actor_id = TbActorId::entity(device_id, EntityType::Device);
                if let Some(ctx) = &self.ctx {
                    ctx.stop(&actor_id);
                }
                self.deleted_devices.insert(device_id);
                true
            }

            // ── Session timeout → broadcast to device actors ─────
            ActorMsg::SessionTimeout => {
                if let Some(ctx) = &self.ctx {
                    ctx.broadcast_to_children_by_type(
                        EntityType::Device,
                        &|| ActorMsg::SessionTimeout,
                        false,
                    );
                }
                true
            }

            // ── Partition change → broadcast to all children ─────
            ActorMsg::PartitionChange { ref service_type } => {
                if let Some(ctx) = &self.ctx {
                    let svc = service_type.clone();
                    ctx.broadcast_to_children(
                        &|| ActorMsg::PartitionChange {
                            service_type: svc.clone(),
                        },
                        true,
                    );
                }
                true
            }

            // ── Component lifecycle ──────────────────────────────
            ActorMsg::ComponentLifecycle {
                entity_id,
                ref entity_type,
                event,
                ..
            } => {
                match entity_type.as_str() {
                    "DEVICE" => {
                        if event == LifecycleEvent::Deleted {
                            let actor_id = TbActorId::entity(entity_id, EntityType::Device);
                            if let Some(ctx) = &self.ctx {
                                ctx.stop(&actor_id);
                            }
                            self.deleted_devices.insert(entity_id);
                        }
                    }
                    "RULE_CHAIN" => {
                        let actor_id = TbActorId::entity(entity_id, EntityType::RuleChain);
                        match event {
                            LifecycleEvent::Deleted => {
                                if let Some(ctx) = &self.ctx {
                                    ctx.stop(&actor_id);
                                }
                                if self.root_chain_id == Some(entity_id) {
                                    self.root_chain_id = None;
                                }
                            }
                            LifecycleEvent::Created | LifecycleEvent::Updated => {
                                // Re-init rule chains.
                                self.init_rule_chains().await;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                true
            }

            // ── Calculated field messages → forward to CF manager ─
            ActorMsg::CfCacheInit { .. }
            | ActorMsg::CfStateRestore { .. }
            | ActorMsg::CfPartitionsChange { .. }
            | ActorMsg::CfEntityLifecycle { .. }
            | ActorMsg::CfTelemetry { .. }
            | ActorMsg::CfLinkedTelemetry { .. }
            | ActorMsg::CfEntityAction { .. } => {
                // CF actor management — forward to CF manager if exists.
                let cf_id = TbActorId::named(format!("CFM|{}", self.tenant_id));
                if let Some(ctx) = &self.ctx {
                    ctx.tell(&cf_id, msg);
                }
                true
            }

            _ => {
                debug!(
                    "TenantActor {}: unhandled message {:?}",
                    self.tenant_id,
                    std::mem::discriminant(&msg)
                );
                false
            }
        }
    }
}
