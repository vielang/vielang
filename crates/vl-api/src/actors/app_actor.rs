//! Root application actor — manages tenant actors.
//!
//! Mirrors ThingsBoard's `AppActor`:
//! - On `AppInit`: loads all tenants and creates a `TenantActor` per tenant.
//! - Routes tenant-scoped messages to the correct `TenantActor`.
//! - Handles tenant creation/deletion lifecycle events.
//! - Broadcasts partition changes and session timeouts.

use async_trait::async_trait;
use std::collections::HashSet;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use vl_actor::{
    ActorError, ActorMsg, EntityType, LifecycleEvent, StopReason, TbActor, TbActorCtx, TbActorId,
};

use super::{ActorSystemCtx, TenantActor};

pub struct AppActor {
    sys_ctx: ActorSystemCtx,
    ctx: Option<TbActorCtx>,
    deleted_tenants: HashSet<Uuid>,
}

impl AppActor {
    pub fn new(sys_ctx: ActorSystemCtx) -> Self {
        Self {
            sys_ctx,
            ctx: None,
            deleted_tenants: HashSet::new(),
        }
    }

    /// Get or create a TenantActor for the given tenant ID.
    fn get_or_create_tenant_actor(
        &self,
        tenant_id: Uuid,
    ) -> Option<vl_actor::TbActorRef> {
        let ctx = self.ctx.as_ref()?;
        let actor_id = TbActorId::entity(tenant_id, EntityType::Tenant);
        let sys_ctx = self.sys_ctx.clone();
        match ctx.get_or_create_child(actor_id, move || {
            Box::new(TenantActor::new(tenant_id, sys_ctx))
        }) {
            Ok(r) => Some(r),
            Err(e) => {
                error!("failed to create TenantActor for {tenant_id}: {e}");
                None
            }
        }
    }

    /// Forward a message to the appropriate tenant actor.
    fn forward_to_tenant(&self, tenant_id: Uuid, msg: ActorMsg, high_priority: bool) {
        if self.deleted_tenants.contains(&tenant_id) {
            debug!("ignoring message for deleted tenant {tenant_id}");
            return;
        }
        if let Some(actor_ref) = self.get_or_create_tenant_actor(tenant_id) {
            if high_priority {
                actor_ref.tell_high_priority(msg);
            } else {
                actor_ref.tell(msg);
            }
        }
    }

    /// Initialize all tenant actors from the database.
    async fn init_all_tenants(&self) {
        match self.sys_ctx.tenant_dao.find_all_ids().await {
            Ok(tenant_ids) => {
                info!("initializing {} tenant actors", tenant_ids.len());
                for tenant_id in tenant_ids {
                    self.get_or_create_tenant_actor(tenant_id);
                }
            }
            Err(e) => {
                error!("failed to load tenant IDs: {e}");
            }
        }
    }

    /// Handle component lifecycle events (tenant creation/deletion).
    fn handle_lifecycle(
        &mut self,
        tenant_id: Uuid,
        entity_id: Uuid,
        entity_type: &str,
        event: LifecycleEvent,
    ) {
        if entity_type == "TENANT" {
            match event {
                LifecycleEvent::Created | LifecycleEvent::Activated => {
                    self.deleted_tenants.remove(&entity_id);
                    self.get_or_create_tenant_actor(entity_id);
                }
                LifecycleEvent::Deleted => {
                    let actor_id = TbActorId::entity(entity_id, EntityType::Tenant);
                    if let Some(ctx) = &self.ctx {
                        ctx.stop(&actor_id);
                    }
                    self.deleted_tenants.insert(entity_id);
                    info!("tenant {entity_id} deleted, actor stopped");
                }
                _ => {
                    // Forward lifecycle to tenant actor.
                    self.forward_to_tenant(
                        tenant_id,
                        ActorMsg::ComponentLifecycle {
                            tenant_id,
                            entity_id,
                            entity_type: entity_type.to_string(),
                            event,
                        },
                        true,
                    );
                }
            }
        } else {
            // Non-tenant lifecycle → forward to tenant actor.
            self.forward_to_tenant(
                tenant_id,
                ActorMsg::ComponentLifecycle {
                    tenant_id,
                    entity_id,
                    entity_type: entity_type.to_string(),
                    event,
                },
                true,
            );
        }
    }
}

#[async_trait]
impl TbActor for AppActor {
    async fn init(&mut self, ctx: TbActorCtx) -> Result<(), ActorError> {
        info!("AppActor initializing");
        self.ctx = Some(ctx);
        Ok(())
    }

    async fn destroy(&mut self, reason: StopReason) {
        info!("AppActor destroyed: {reason:?}");
    }

    async fn process(&mut self, msg: ActorMsg) -> bool {
        match msg {
            ActorMsg::AppInit => {
                self.init_all_tenants().await;
                true
            }

            ActorMsg::PartitionChange { ref service_type } => {
                let ctx = match &self.ctx {
                    Some(c) => c,
                    None => return false,
                };
                let svc = service_type.clone();
                ctx.broadcast_to_children_by_type(
                    EntityType::Tenant,
                    &|| ActorMsg::PartitionChange {
                        service_type: svc.clone(),
                    },
                    true,
                );
                true
            }

            ActorMsg::SessionTimeout => {
                if let Some(ctx) = &self.ctx {
                    ctx.broadcast_to_children_by_type(
                        EntityType::Tenant,
                        &|| ActorMsg::SessionTimeout,
                        false,
                    );
                }
                true
            }

            ActorMsg::ComponentLifecycle {
                tenant_id,
                entity_id,
                ref entity_type,
                event,
            } => {
                self.handle_lifecycle(tenant_id, entity_id, entity_type, event);
                true
            }

            // ── Tenant-scoped messages → forward to tenant actor ──────
            ActorMsg::QueueToRuleEngine { tenant_id, .. } => {
                self.forward_to_tenant(tenant_id, msg, false);
                true
            }

            ActorMsg::TransportToDevice { tenant_id, .. } => {
                self.forward_to_tenant(tenant_id, msg, false);
                true
            }

            // Device notifications → high priority.
            ActorMsg::DeviceAttributesUpdate { tenant_id, .. }
            | ActorMsg::DeviceCredentialsUpdate { tenant_id, .. }
            | ActorMsg::DeviceNameOrTypeUpdate { tenant_id, .. }
            | ActorMsg::DeviceDelete { tenant_id, .. }
            | ActorMsg::DeviceEdgeUpdate { tenant_id, .. }
            | ActorMsg::DeviceRpcRequest { tenant_id, .. }
            | ActorMsg::DeviceRpcResponse { tenant_id, .. }
            | ActorMsg::RemoveRpc { tenant_id, .. } => {
                self.forward_to_tenant(tenant_id, msg, true);
                true
            }

            // Calculated field messages.
            ActorMsg::CfCacheInit { tenant_id, .. }
            | ActorMsg::CfStateRestore { tenant_id, .. }
            | ActorMsg::CfPartitionsChange { tenant_id, .. }
            | ActorMsg::CfEntityLifecycle { tenant_id, .. }
            | ActorMsg::CfTelemetry { tenant_id, .. }
            | ActorMsg::CfLinkedTelemetry { tenant_id, .. }
            | ActorMsg::CfEntityAction { tenant_id, .. } => {
                self.forward_to_tenant(tenant_id, msg, true);
                true
            }

            _ => {
                debug!("AppActor: unhandled message {:?}", std::mem::discriminant(&msg));
                false
            }
        }
    }
}
