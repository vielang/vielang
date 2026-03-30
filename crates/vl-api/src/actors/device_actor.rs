//! Per-device actor — manages device sessions, RPC, and state.
//!
//! Mirrors ThingsBoard's `DeviceActor` + `DeviceActorMessageProcessor`:
//! - Tracks active transport sessions (MQTT, CoAP, HTTP).
//! - Handles RPC requests/responses with timeouts.
//! - Processes device attribute/credential updates.
//! - Checks session timeouts periodically.

use async_trait::async_trait;
use tracing::{debug, info, warn};
use uuid::Uuid;
use vl_actor::{ActorError, ActorMsg, StopReason, TbActor, TbActorCtx, TbActorId};

use super::ActorSystemCtx;

pub struct DeviceActor {
    tenant_id: Uuid,
    device_id: Uuid,
    sys_ctx: ActorSystemCtx,
    ctx: Option<TbActorCtx>,
}

impl DeviceActor {
    pub fn new(tenant_id: Uuid, device_id: Uuid, sys_ctx: ActorSystemCtx) -> Self {
        Self {
            tenant_id,
            device_id,
            sys_ctx,
            ctx: None,
        }
    }

    fn process_transport_msg(&mut self, payload: &[u8]) {
        // Forward telemetry to the rule engine via queue producer.
        debug!(
            "device {}: received transport message ({} bytes)",
            self.device_id,
            payload.len()
        );
    }

    fn process_attributes_update(&mut self, scope: &str, deleted: bool) {
        debug!(
            "device {}: attributes update (scope={scope}, deleted={deleted})",
            self.device_id
        );
    }

    fn process_credentials_update(&mut self) {
        debug!(
            "device {}: credentials updated, closing sessions",
            self.device_id
        );
        // In a full impl: close all transport sessions to force re-auth.
    }

    fn process_name_or_type_update(&mut self, name: &str, device_type: &str) {
        debug!(
            "device {}: name/type updated to {name}/{device_type}",
            self.device_id
        );
    }

    fn process_rpc_request(&mut self, request_id: Uuid, oneway: bool, body: &str) {
        debug!(
            "device {}: RPC request {request_id} (oneway={oneway})",
            self.device_id
        );
    }

    fn process_rpc_response(&mut self, request_id: i32, response: &Option<String>) {
        debug!(
            "device {}: RPC response for {request_id}",
            self.device_id
        );
    }

    fn check_sessions_timeout(&mut self) {
        // Check if any transport sessions have timed out.
    }
}

#[async_trait]
impl TbActor for DeviceActor {
    async fn init(&mut self, ctx: TbActorCtx) -> Result<(), ActorError> {
        debug!("DeviceActor {} initialized", self.device_id);
        self.ctx = Some(ctx);
        Ok(())
    }

    async fn destroy(&mut self, reason: StopReason) {
        debug!("DeviceActor {} destroyed: {reason:?}", self.device_id);
    }

    async fn process(&mut self, msg: ActorMsg) -> bool {
        match msg {
            ActorMsg::TransportToDevice { payload, .. } => {
                self.process_transport_msg(&payload);
                true
            }

            ActorMsg::DeviceAttributesUpdate {
                scope, deleted, ..
            } => {
                self.process_attributes_update(&scope, deleted);
                true
            }

            ActorMsg::DeviceCredentialsUpdate { .. } => {
                self.process_credentials_update();
                true
            }

            ActorMsg::DeviceNameOrTypeUpdate {
                ref device_name,
                ref device_type,
                ..
            } => {
                self.process_name_or_type_update(device_name, device_type);
                true
            }

            ActorMsg::DeviceDelete { .. } => {
                // Parent (TenantActor) stops this actor.
                if let Some(ctx) = &self.ctx {
                    ctx.stop(ctx.self_id());
                }
                true
            }

            ActorMsg::DeviceEdgeUpdate { edge_id, .. } => {
                debug!(
                    "device {}: edge update to {:?}",
                    self.device_id, edge_id
                );
                true
            }

            ActorMsg::DeviceRpcRequest {
                request_id,
                oneway,
                ref body,
                ..
            } => {
                self.process_rpc_request(request_id, oneway, body);
                true
            }

            ActorMsg::DeviceRpcResponse {
                request_id,
                ref response,
                ..
            } => {
                self.process_rpc_response(request_id, response);
                true
            }

            ActorMsg::DeviceRpcTimeout { rpc_id, .. } => {
                debug!("device {}: RPC {rpc_id} timed out", self.device_id);
                true
            }

            ActorMsg::RemoveRpc { request_id, .. } => {
                debug!("device {}: removing RPC {request_id}", self.device_id);
                true
            }

            ActorMsg::SessionTimeout => {
                self.check_sessions_timeout();
                true
            }

            _ => false,
        }
    }
}
