use std::sync::Arc;
use tracing::info;
use vl_config::VieLangConfig;
use vl_dao::DbPool;

use super::services::CoreServices;
use super::tasks::BackgroundTasks;

/// Start all transport protocols based on config.
pub fn start_transports(
    core: &CoreServices,
    config: &VieLangConfig,
    pool: &DbPool,
    tasks: &mut BackgroundTasks,
) {
    let state = &core.state;
    let ts_dao = &core.ts_dao;
    let activity_tx = &core.activity_tx;

    // MQTT Transport
    if config.transport.mqtt.enabled {
        let mqtt_pool     = pool.clone();
        let mqtt_ts_dao   = ts_dao.clone();
        let mqtt_config   = config.transport.mqtt.clone();
        let re_sender     = Some(core.rule_engine_sender.clone());
        let mqtt_producer = state.queue_producer.clone();
        let mqtt_cache    = state.cache.clone();
        let ws_tx         = state.ws_tx.clone();
        let act_tx        = activity_tx.clone();
        let dev_registry  = state.device_rpc_registry.clone();
        let rpc_pending   = state.rpc_pending_registry.clone();
        let mqtt_chunk_kb = config.ota.chunk_size_kb;
        tasks.spawn("mqtt-transport", async move {
            vl_transport::run_mqtt(mqtt_pool, mqtt_ts_dao, mqtt_config, re_sender, mqtt_producer, mqtt_cache, ws_tx, act_tx, dev_registry, rpc_pending, mqtt_chunk_kb).await;
        });
        info!("MQTT transport enabled on port {}", config.transport.mqtt.port);
    }

    // HTTP Device API Transport
    if config.transport.http.enabled {
        let http_pool     = pool.clone();
        let http_ts_dao   = ts_dao.clone();
        let http_config   = config.transport.http.clone();
        let http_producer = state.queue_producer.clone();
        let http_cache    = state.cache.clone();
        let ws_tx         = state.ws_tx.clone();
        let act_tx        = activity_tx.clone();
        let rpc_pending   = state.rpc_pending_registry.clone();
        tasks.spawn("http-transport", async move {
            vl_transport::run_http(http_pool, http_ts_dao, http_config, None, http_producer, http_cache, ws_tx, act_tx, rpc_pending).await;
        });
        info!("HTTP transport enabled on port {}", config.transport.http.port);
    }

    // CoAP Transport
    if config.transport.coap.enabled {
        let coap_pool     = pool.clone();
        let coap_ts_dao   = ts_dao.clone();
        let coap_config   = config.transport.coap.clone();
        let coap_producer = state.queue_producer.clone();
        let coap_cache    = state.cache.clone();
        let ws_tx         = state.ws_tx.clone();
        let act_tx        = activity_tx.clone();
        tasks.spawn("coap-transport", async move {
            vl_transport::run_coap(coap_pool, coap_ts_dao, coap_config, None, coap_producer, coap_cache, ws_tx, act_tx).await;
        });
        info!("CoAP transport enabled on port {}", config.transport.coap.port);
    }

    // LwM2M Transport
    if config.transport.lwm2m.enabled {
        let lwm2m_pool     = pool.clone();
        let lwm2m_ts_dao   = ts_dao.clone();
        let lwm2m_config   = config.transport.lwm2m.clone();
        let lwm2m_producer = state.queue_producer.clone();
        let lwm2m_cache    = state.cache.clone();
        let ws_tx          = state.ws_tx.clone();
        let act_tx         = activity_tx.clone();
        tasks.spawn("lwm2m-transport", async move {
            vl_transport::run_lwm2m(lwm2m_pool, lwm2m_ts_dao, lwm2m_config, None, lwm2m_producer, lwm2m_cache, ws_tx, act_tx).await;
        });
        info!("LwM2M transport enabled on port {}", config.transport.lwm2m.port);
    }

    // Edge gRPC Server (Phase 56)
    if config.edge_grpc.enabled {
        let edge_registry = state.edge_session_registry.clone();
        let edge_handler = Arc::new(crate::services::edge_handler::EdgeUplinkHandlerImpl {
            edge_dao:           state.edge_dao.clone(),
            device_dao:         state.device_dao.clone(),
            device_profile_dao: state.device_profile_dao.clone(),
            rule_chain_dao:     state.rule_chain_dao.clone(),
            ts_dao:             ts_dao.clone(),
            activity_tx:        activity_tx.clone(),
            rule_engine_tx:     Arc::new(Some(core.rule_engine_sender.clone())),
        });

        // Start session cleanup for stale edge connections (timeout: 5 min)
        let cleanup_handle = edge_registry.start_session_cleanup(std::time::Duration::from_secs(300));
        tasks.spawn("edge-session-cleanup", async move { cleanup_handle.await.ok(); });
        let edge_bind = config.edge_grpc.bind.clone();
        let edge_port = config.edge_grpc.port;
        tasks.spawn("edge-grpc", async move {
            if let Err(e) = vl_cluster::run_edge_grpc(edge_bind, edge_port, edge_registry, edge_handler).await {
                tracing::error!("Edge gRPC server error: {}", e);
            }
        });
        info!("Edge gRPC server enabled on port {}", config.edge_grpc.port);
    }

    // SNMP Transport
    if config.transport.snmp.enabled {
        let snmp_pool     = pool.clone();
        let snmp_ts_dao   = ts_dao.clone();
        let snmp_config   = config.transport.snmp.clone();
        let snmp_producer = state.queue_producer.clone();
        let snmp_cache    = state.cache.clone();
        let ws_tx         = state.ws_tx.clone();
        let act_tx        = activity_tx.clone();
        tasks.spawn("snmp-transport", async move {
            vl_transport::run_snmp(snmp_pool, snmp_ts_dao, snmp_config, None, snmp_producer, snmp_cache, ws_tx, act_tx).await;
        });
        info!("SNMP transport enabled on port {}", config.transport.snmp.bind_port);
    }

    // LoRaWAN Bridge (P16)
    if config.transport.lorawan.enabled {
        let lora_pool     = pool.clone();
        let lora_ts_dao   = ts_dao.clone();
        let lora_config   = config.transport.lorawan.clone();
        let lora_producer = state.queue_producer.clone();
        let lora_cache    = state.cache.clone();
        let ws_tx         = state.ws_tx.clone();
        let act_tx        = activity_tx.clone();
        tasks.spawn("lorawan-bridge", async move {
            vl_transport::run_lorawan(lora_pool, lora_ts_dao, lora_config.clone(), None, lora_producer, lora_cache, ws_tx, act_tx).await;
        });
        info!("LoRaWAN bridge enabled → {}", config.transport.lorawan.chirpstack_url);
    }
}
