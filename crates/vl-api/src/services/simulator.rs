use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn, instrument};
use uuid::Uuid;

use vl_config::SimulatorCfg;
use vl_core::entities::message::{msg_type, TbMsg};
use vl_core::entities::kv::TsRecord;
use vl_core::entities::simulator::{
    GeneratorType, SimDataType, SimulationStatus, SimulationStatusResponse, SimulatorConfig,
    TransportMode,
};
use vl_dao::postgres::simulator::SimulatorDao;
use vl_dao::TimeseriesDao;
use vl_queue::TbProducer;

/// Runtime info for a running simulation.
struct RunningSimulation {
    _handle: JoinHandle<()>,
    cancel: CancellationToken,
    config: SimulatorConfig,
    tick_count: Arc<std::sync::atomic::AtomicU64>,
    last_tick_ts: Arc<std::sync::atomic::AtomicI64>,
}

/// Cached token with expiry
struct CachedToken {
    token: String,
    expires_at: i64, // ms since epoch
}

const TOKEN_CACHE_TTL_MS: i64 = 3600_000; // 1 hour
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const HTTP_MAX_RETRIES: u32 = 3;
const MQTT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct SimulatorService {
    config: SimulatorCfg,
    dao: Arc<SimulatorDao>,
    ts_dao: Arc<dyn TimeseriesDao>,
    ws_tx: broadcast::Sender<TbMsg>,
    queue_producer: Arc<dyn TbProducer>,
    running: RwLock<HashMap<Uuid, RunningSimulation>>,
    http_client: reqwest::Client,
    /// HTTP device transport port (default 8081, NOT 8080 which is the API port)
    http_transport_port: u16,
    /// MQTT broker port (default 1883)
    mqtt_port: u16,
    /// Cached device tokens with TTL
    token_cache: RwLock<HashMap<Uuid, CachedToken>>,
}

impl SimulatorService {
    pub fn new(
        config: SimulatorCfg,
        dao: Arc<SimulatorDao>,
        ts_dao: Arc<dyn TimeseriesDao>,
        ws_tx: broadcast::Sender<TbMsg>,
        queue_producer: Arc<dyn TbProducer>,
    ) -> Self {
        // Build HTTP client with timeout + connection pooling
        let http_client = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .connect_timeout(Duration::from_secs(3))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_default();

        Self {
            config,
            dao,
            ts_dao,
            ws_tx,
            queue_producer,
            running: RwLock::new(HashMap::new()),
            http_client,
            http_transport_port: 8081, // HTTP Device API (NOT the main API on 8080)
            mqtt_port: 1883,
            token_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Load all enabled configs and start simulations. Called at server startup.
    pub async fn start_all(self: &Arc<Self>) {
        if !self.config.enabled {
            info!("Simulator service disabled by config");
            return;
        }
        match self.dao.find_enabled().await {
            Ok(configs) => {
                let count = configs.len();
                for cfg in configs {
                    self.start_one(cfg).await;
                }
                if count > 0 {
                    info!("Simulator: started {} enabled simulations", count);
                }
            }
            Err(e) => error!("Failed to load enabled simulator configs: {}", e),
        }
    }

    /// Start a single simulation by its config.
    pub async fn start_one(self: &Arc<Self>, sim_config: SimulatorConfig) {
        let config_id = sim_config.id;
        let mut guard = self.running.write().await;
        if guard.contains_key(&config_id) { return; }

        let cancel = CancellationToken::new();
        let tick_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let last_tick_ts = Arc::new(std::sync::atomic::AtomicI64::new(0));

        let svc = Arc::clone(self);
        let sim = sim_config.clone();
        let cancel_clone = cancel.clone();
        let tc = tick_count.clone();
        let lt = last_tick_ts.clone();

        let handle = tokio::spawn(async move {
            let interval_ms = sim.interval_ms.max(svc.config.min_interval_ms) as u64;
            let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
            interval.tick().await;

            loop {
                tokio::select! {
                    _ = cancel_clone.cancelled() => break,
                    _ = interval.tick() => {
                        let tick = tc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let now_ms = chrono::Utc::now().timestamp_millis();
                        lt.store(now_ms, std::sync::atomic::Ordering::Relaxed);
                        svc.generate_tick(&sim, tick, now_ms).await;
                    }
                }
            }
        });

        guard.insert(config_id, RunningSimulation {
            _handle: handle, cancel, config: sim_config, tick_count, last_tick_ts,
        });
    }

    pub async fn stop_one(&self, config_id: Uuid) -> bool {
        let mut guard = self.running.write().await;
        if let Some(sim) = guard.remove(&config_id) {
            sim.cancel.cancel();
            true
        } else { false }
    }

    pub async fn get_status_for_tenant(&self, tenant_id: Uuid) -> Vec<SimulationStatusResponse> {
        let guard = self.running.read().await;
        guard.values()
            .filter(|s| s.config.tenant_id == tenant_id)
            .map(|s| self.build_status_response(s))
            .collect()
    }

    pub async fn get_status(&self, config_id: Uuid) -> SimulationStatusResponse {
        let guard = self.running.read().await;
        guard.get(&config_id)
            .map(|s| self.build_status_response(s))
            .unwrap_or(SimulationStatusResponse {
                config_id, config_name: String::new(), device_id: Uuid::nil(),
                status: SimulationStatus::Stopped, tick_count: 0,
                last_tick_ts: None, error_message: None,
            })
    }

    pub async fn is_running(&self, config_id: Uuid) -> bool {
        self.running.read().await.contains_key(&config_id)
    }

    fn build_status_response(&self, s: &RunningSimulation) -> SimulationStatusResponse {
        SimulationStatusResponse {
            config_id: s.config.id,
            config_name: s.config.name.clone(),
            device_id: s.config.device_id,
            status: SimulationStatus::Running,
            tick_count: s.tick_count.load(std::sync::atomic::Ordering::Relaxed),
            last_tick_ts: {
                let ts = s.last_tick_ts.load(std::sync::atomic::Ordering::Relaxed);
                if ts > 0 { Some(ts) } else { None }
            },
            error_message: None,
        }
    }

    // ══════════════════════════════════════════════════════════════════════════
    // Telemetry generation + transport routing
    // ══════════════════════════════════════════════════════════════════════════

    #[instrument(skip(self, sim_config), fields(config_id = %sim_config.id, device_id = %sim_config.device_id))]
    async fn generate_tick(&self, sim_config: &SimulatorConfig, tick: u64, now_ms: i64) {
        let device_id = sim_config.device_id;
        let mut records = Vec::with_capacity(sim_config.telemetry_schema.len());
        let mut json_map = serde_json::Map::new();

        for field in &sim_config.telemetry_schema {
            let value = evaluate_generator(&field.generator, &field.data_type, tick, now_ms);
            let mut record = TsRecord {
                entity_id: device_id, key: field.key.clone(), ts: now_ms,
                bool_v: None, str_v: None, long_v: None, dbl_v: None, json_v: None,
            };
            match &value {
                serde_json::Value::Number(n) => {
                    if let Some(f) = n.as_f64() { record.dbl_v = Some(f); }
                    else if let Some(i) = n.as_i64() { record.long_v = Some(i); }
                }
                serde_json::Value::Bool(b) => record.bool_v = Some(*b),
                serde_json::Value::String(s) => record.str_v = Some(s.clone()),
                other => record.json_v = Some(other.clone()),
            }
            json_map.insert(field.key.clone(), value);
            records.push(record);
        }

        let payload_json = serde_json::Value::Object(json_map);

        match &sim_config.transport_mode {
            TransportMode::Http => {
                self.send_via_http(device_id, &payload_json).await;
            }
            TransportMode::Mqtt => {
                self.send_via_mqtt(device_id, &payload_json).await;
            }
            TransportMode::Direct => {
                if let Err(e) = self.ts_dao.save_latest_batch("DEVICE", &records).await {
                    error!("Simulator save_latest failed: {}", e);
                }
                let _ = self.ts_dao.save_batch("DEVICE", &records).await;
                let msg = TbMsg::new(
                    msg_type::POST_TELEMETRY_REQUEST, device_id, "DEVICE",
                    payload_json.to_string(),
                ).with_tenant(sim_config.tenant_id);
                let _ = self.ws_tx.send(msg.clone());
                let _ = self.queue_producer.send_tb_msg(vl_queue::topics::VL_TRANSPORT_API_REQUESTS, &msg).await;
            }
        }
    }

    // ── HTTP Transport (with retries + timeout) ──────────────────────────────

    async fn send_via_http(&self, device_id: Uuid, payload: &serde_json::Value) {
        let token = match self.get_device_token(device_id).await {
            Some(t) => t,
            None => { error!("No access token for device {}", device_id); return; }
        };

        // HTTP Device API runs on transport port (8081), NOT main API port (8080)
        let url = format!("http://127.0.0.1:{}/api/v1/{}/telemetry", self.http_transport_port, token);

        // Retry with exponential backoff
        let backoffs = [100, 500, 2000]; // ms
        for attempt in 0..HTTP_MAX_RETRIES {
            match self.http_client.post(&url).json(payload).send().await {
                Ok(resp) if resp.status().is_success() => return, // success
                Ok(resp) if resp.status().as_u16() == 401 => {
                    // Token invalid — invalidate cache and stop retrying
                    warn!("HTTP 401 for device {} — invalidating token cache", device_id);
                    self.token_cache.write().await.remove(&device_id);
                    return;
                }
                Ok(resp) => {
                    warn!("HTTP transport {} (attempt {}/{}): device={}",
                        resp.status(), attempt + 1, HTTP_MAX_RETRIES, device_id);
                }
                Err(e) => {
                    warn!("HTTP transport error (attempt {}/{}): {}",
                        attempt + 1, HTTP_MAX_RETRIES, e);
                }
            }
            if attempt < HTTP_MAX_RETRIES - 1 {
                tokio::time::sleep(Duration::from_millis(backoffs[attempt as usize])).await;
            }
        }
        error!("HTTP transport failed after {} retries: device={}", HTTP_MAX_RETRIES, device_id);
    }

    // ── MQTT Transport (persistent connection per publish) ───────────────────

    async fn send_via_mqtt(&self, device_id: Uuid, payload: &serde_json::Value) {
        let token = match self.get_device_token(device_id).await {
            Some(t) => t,
            None => { error!("No access token for device {}", device_id); return; }
        };

        let client_id = format!("sim-{}-{}", device_id.as_simple(), chrono::Utc::now().timestamp_millis() % 10000);
        let mut opts = rumqttc::MqttOptions::new(&client_id, "127.0.0.1", self.mqtt_port);
        opts.set_credentials(&token, "");
        opts.set_keep_alive(Duration::from_secs(10));

        let (client, mut eventloop) = rumqttc::AsyncClient::new(opts, 10);
        let payload_bytes = serde_json::to_vec(payload).unwrap_or_default();

        // Drive eventloop until connected, then publish, then disconnect
        let result = tokio::time::timeout(MQTT_CONNECT_TIMEOUT, async {
            // Wait for ConnAck
            loop {
                match eventloop.poll().await {
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_))) => break,
                    Ok(_) => continue,
                    Err(e) => return Err(format!("MQTT connect error: {}", e)),
                }
            }
            // Publish
            client.publish(
                "v1/devices/me/telemetry",
                rumqttc::QoS::AtLeastOnce,
                false,
                payload_bytes,
            ).await.map_err(|e| format!("MQTT publish error: {}", e))?;

            // Wait for PubAck (QoS 1 confirmation)
            for _ in 0..5 {
                match eventloop.poll().await {
                    Ok(rumqttc::Event::Incoming(rumqttc::Packet::PubAck(_))) => {
                        let _ = client.disconnect().await;
                        return Ok(());
                    }
                    Ok(_) => continue,
                    Err(e) => return Err(format!("MQTT ack error: {}", e)),
                }
            }
            let _ = client.disconnect().await;
            Ok(()) // best-effort if no PubAck received
        }).await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => error!("MQTT transport: {}", e),
            Err(_) => error!("MQTT transport timeout for device {}", device_id),
        }
    }

    // ── Device Token Cache (with TTL + invalidation on 401) ──────────────────

    async fn get_device_token(&self, device_id: Uuid) -> Option<String> {
        let now = chrono::Utc::now().timestamp_millis();
        {
            let cache = self.token_cache.read().await;
            if let Some(entry) = cache.get(&device_id) {
                if entry.expires_at > now {
                    return Some(entry.token.clone());
                }
            }
        }
        // Cache miss or expired — lookup from DB
        match self.dao.find_device_token(device_id).await {
            Ok(Some(t)) => {
                self.token_cache.write().await.insert(device_id, CachedToken {
                    token: t.clone(),
                    expires_at: now + TOKEN_CACHE_TTL_MS,
                });
                Some(t)
            }
            Ok(None) => { warn!("Device {} has no access token", device_id); None }
            Err(e) => { error!("Token lookup failed: {}", e); None }
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Generator evaluation
// ══════════════════════════════════════════════════════════════════════════════

fn evaluate_generator(
    generator: &GeneratorType,
    data_type: &SimDataType,
    tick: u64,
    now_ms: i64,
) -> serde_json::Value {
    match generator {
        GeneratorType::Random { min, max } => {
            let range = max - min;
            let seed = (tick as f64 * 7919.0 + now_ms as f64).sin().abs();
            let val = min + seed * range;
            match data_type {
                SimDataType::Long => serde_json::json!(val as i64),
                SimDataType::Boolean => serde_json::json!(val > (min + range / 2.0)),
                _ => serde_json::json!(round2(val)),
            }
        }
        GeneratorType::SineWave { amplitude, offset, period_ms } => {
            let phase = 2.0 * std::f64::consts::PI * (now_ms as f64 / *period_ms as f64);
            let val = offset + amplitude * phase.sin();
            match data_type {
                SimDataType::Long => serde_json::json!(val as i64),
                _ => serde_json::json!(round2(val)),
            }
        }
        GeneratorType::Linear { start, step, max } => {
            let mut val = start + step * tick as f64;
            if let Some(m) = max {
                if *step > 0.0 && val > *m { val = *start; }
                else if *step < 0.0 && val < *m { val = *start; }
            }
            match data_type {
                SimDataType::Long => serde_json::json!(val as i64),
                _ => serde_json::json!(round2(val)),
            }
        }
        GeneratorType::Constant { value } => value.clone(),
        GeneratorType::Script { expression: _ } => serde_json::json!(0),
    }
}

fn round2(v: f64) -> f64 { (v * 100.0).round() / 100.0 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_generator_within_bounds() {
        for tick in 0..100 {
            let val = evaluate_generator(
                &GeneratorType::Random { min: 10.0, max: 50.0 },
                &SimDataType::Double, tick, 1711612800000 + tick as i64 * 1000,
            );
            let f = val.as_f64().expect("should be f64");
            assert!(f >= 10.0 && f <= 50.0, "out of bounds: {}", f);
        }
    }

    #[test]
    fn sine_wave_generator() {
        let val = evaluate_generator(
            &GeneratorType::SineWave { amplitude: 10.0, offset: 20.0, period_ms: 60000 },
            &SimDataType::Double, 0, 15000,
        );
        let f = val.as_f64().expect("should be f64");
        assert!((f - 30.0).abs() < 0.01, "expected ~30, got {}", f);
    }

    #[test]
    fn linear_generator_wraps() {
        let linear = GeneratorType::Linear { start: 0.0, step: 10.0, max: Some(50.0) };
        let val = evaluate_generator(&linear, &SimDataType::Double, 6, 0);
        let f = val.as_f64().expect("should be f64");
        assert_eq!(f, 0.0);
    }

    #[test]
    fn constant_generator() {
        let val = evaluate_generator(
            &GeneratorType::Constant { value: serde_json::json!("hello") },
            &SimDataType::String, 0, 0,
        );
        assert_eq!(val, serde_json::json!("hello"));
    }
}
