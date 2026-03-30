/// EdgeUplinkHandlerImpl — concrete implementation of EdgeUplinkHandler for vl-api.
///
/// Injected into the Edge gRPC server so it can save telemetry and forward to rule engine
/// without vl-cluster depending on vl-dao/vl-rule-engine.
use std::sync::Arc;
use uuid::Uuid;
use tracing::{debug, info, warn};

use vl_cluster::{EdgeUplinkHandler, AuthError, TelemetryEntry, TelemetryValue};
use vl_dao::{TimeseriesDao, postgres::edge::EdgeDao};
use vl_dao::postgres::device::DeviceDao;
use vl_dao::postgres::device_profile::DeviceProfileDao;
use vl_dao::postgres::rule_chain::RuleChainDao;
use vl_core::entities::{ActivityEvent, TsRecord, TbMsg, msg_type};
use tokio::sync::mpsc;

/// Concrete uplink handler wiring vl-api resources into the Edge gRPC server.
pub struct EdgeUplinkHandlerImpl {
    pub edge_dao:        Arc<EdgeDao>,
    pub device_dao:      Arc<DeviceDao>,
    pub device_profile_dao: Arc<DeviceProfileDao>,
    pub rule_chain_dao:  Arc<RuleChainDao>,
    pub ts_dao:          Arc<dyn TimeseriesDao>,
    pub activity_tx:     mpsc::Sender<ActivityEvent>,
    pub rule_engine_tx:  Arc<Option<mpsc::Sender<TbMsg>>>,
}

#[async_trait::async_trait]
impl EdgeUplinkHandler for EdgeUplinkHandlerImpl {
    async fn authenticate_edge(
        &self,
        routing_key: &str,
        secret: &str,
    ) -> Result<(Uuid, Uuid), AuthError> {
        let edge = self.edge_dao
            .find_by_routing_key(routing_key)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?
            .ok_or(AuthError::NotFound)?;

        if edge.secret != secret {
            return Err(AuthError::InvalidSecret);
        }

        Ok((edge.id, edge.tenant_id))
    }

    async fn save_edge_telemetry(
        &self,
        _edge_id:   Uuid,
        _tenant_id: Uuid,
        device_id:  Uuid,
        entries:    Vec<TelemetryEntry>,
    ) -> Result<(), String> {
        let records: Vec<TsRecord> = entries.into_iter().map(|e| {
            let (bool_v, long_v, dbl_v, str_v, json_v) = match e.value {
                TelemetryValue::Bool(b)   => (Some(b), None,    None,    None,    None),
                TelemetryValue::Long(l)   => (None,    Some(l), None,    None,    None),
                TelemetryValue::Double(d) => (None,    None,    Some(d), None,    None),
                TelemetryValue::String(s) => (None,    None,    None,    Some(s), None),
                TelemetryValue::Json(v)   => (None,    None,    None,    None,    Some(v)),
            };
            TsRecord { entity_id: device_id, key: e.key, ts: e.ts, bool_v, long_v, dbl_v, str_v, json_v }
        }).collect();

        self.ts_dao.save_batch("DEVICE", &records).await
            .map_err(|e| e.to_string())
    }

    async fn handle_edge_rpc(
        &self,
        _edge_id:   Uuid,
        tenant_id:  Uuid,
        device_id:  Uuid,
        request_id: i64,
        method:     String,
        params:     String,
    ) -> Result<(), String> {
        debug!(device_id = %device_id, request_id, method = %method, "Edge RPC call");

        // Forward to rule engine as RPC_REQUEST message
        let data = serde_json::json!({
            "method":    method,
            "params":    params,
            "requestId": request_id,
        });
        let msg = TbMsg::new(msg_type::RPC_CALL_FROM_SERVER, device_id, "DEVICE", &data.to_string())
            .with_tenant(tenant_id);

        if let Some(tx) = self.rule_engine_tx.as_ref() {
            let _ = tx.try_send(msg);
        }
        Ok(())
    }

    async fn handle_device_connect(
        &self,
        _edge_id:  Uuid,
        _tenant_id: Uuid,
        device_id: Uuid,
    ) -> Result<(), String> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
            .as_millis() as i64;
        let _ = self.activity_tx.try_send(ActivityEvent::Connected { device_id, ts });
        Ok(())
    }

    async fn handle_device_disconnect(
        &self,
        _edge_id:  Uuid,
        _tenant_id: Uuid,
        device_id: Uuid,
    ) -> Result<(), String> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
            .as_millis() as i64;
        let _ = self.activity_tx.try_send(ActivityEvent::Disconnected { device_id, ts });
        Ok(())
    }

    async fn send_initial_sync(
        &self,
        _edge_id:   Uuid,
        tenant_id:  Uuid,
    ) -> Result<Vec<serde_json::Value>, String> {
        let mut payloads = Vec::new();

        // 1. Sync device profiles
        match self.device_profile_dao.find_by_tenant(tenant_id, &vl_dao::PageLink::new(0, 1000)).await {
            Ok(page) => {
                for dp in page.data {
                    payloads.push(serde_json::json!({
                        "entityType": "DEVICE_PROFILE",
                        "entityId":   dp.id.to_string(),
                        "tenantId":   dp.tenant_id.to_string(),
                        "body":       serde_json::to_value(&dp).unwrap_or_default(),
                    }));
                }
                info!(tenant_id = %tenant_id, count = payloads.len(), "Synced device profiles to edge");
            }
            Err(e) => warn!(tenant_id = %tenant_id, "Failed to fetch device profiles for edge sync: {}", e),
        }

        // 2. Sync devices
        match self.device_dao.find_all_by_tenant(tenant_id).await {
            Ok(devices) => {
                let count = devices.len();
                for d in devices {
                    payloads.push(serde_json::json!({
                        "entityType": "DEVICE",
                        "entityId":   d.id.to_string(),
                        "tenantId":   d.tenant_id.to_string(),
                        "body":       serde_json::to_value(&d).unwrap_or_default(),
                    }));
                }
                info!(tenant_id = %tenant_id, count, "Synced devices to edge");
            }
            Err(e) => warn!(tenant_id = %tenant_id, "Failed to fetch devices for edge sync: {}", e),
        }

        // 3. Sync rule chains
        match self.rule_chain_dao.find_all_by_tenant(tenant_id).await {
            Ok(chains) => {
                let count = chains.len();
                for rc in chains {
                    payloads.push(serde_json::json!({
                        "entityType": "RULE_CHAIN",
                        "entityId":   rc.id.to_string(),
                        "tenantId":   rc.tenant_id.to_string(),
                        "body":       serde_json::to_value(&rc).unwrap_or_default(),
                    }));
                }
                info!(tenant_id = %tenant_id, count, "Synced rule chains to edge");
            }
            Err(e) => warn!(tenant_id = %tenant_id, "Failed to fetch rule chains for edge sync: {}", e),
        }

        Ok(payloads)
    }
}
