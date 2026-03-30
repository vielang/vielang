//! MQTT Gateway session handler — manages child devices via a single gateway connection.
//!
//! Java: AbstractGatewaySessionHandler
//!
//! Gateway topics:
//! - `v1/gateway/connect`    — register child device
//! - `v1/gateway/disconnect` — unregister child device
//! - `v1/gateway/telemetry`  — telemetry for child devices
//! - `v1/gateway/attributes` — attribute updates for child devices
//! - `v1/gateway/claim`      — claim child device
//! - `v1/gateway/rpc`        — RPC response from child device

use std::collections::HashMap;
use std::sync::Arc;

use tracing::{debug, info, warn};
use uuid::Uuid;

use vl_core::entities::TsRecord;
use vl_dao::{postgres::device::DeviceDao, DbPool, TimeseriesDao};

/// Manages child devices registered through a gateway MQTT connection.
pub struct GatewaySession {
    gateway_device_id: Uuid,
    tenant_id: Uuid,
    pool: DbPool,
    ts_dao: Arc<dyn TimeseriesDao>,
    /// Child device name → device_id cache (avoid repeated DB lookups).
    children: HashMap<String, Uuid>,
}

impl GatewaySession {
    pub fn new(
        gateway_device_id: Uuid,
        tenant_id: Uuid,
        pool: DbPool,
        ts_dao: Arc<dyn TimeseriesDao>,
    ) -> Self {
        Self {
            gateway_device_id,
            tenant_id,
            pool,
            ts_dao,
            children: HashMap::new(),
        }
    }

    /// Handle gateway connect message — register/create child device.
    ///
    /// Payload: `{"device": "CHILD_NAME"}` or `{"device": "CHILD_NAME", "type": "TYPE"}`
    pub async fn on_connect(&mut self, payload: &[u8]) -> Option<Uuid> {
        let body: serde_json::Value = serde_json::from_slice(payload).ok()?;
        let device_name = body.get("device").and_then(|v| v.as_str())?;
        let device_type = body
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // Check cache first.
        if let Some(&id) = self.children.get(device_name) {
            debug!(gateway = %self.gateway_device_id, child = %device_name, "Gateway child already connected");
            return Some(id);
        }

        let dao = DeviceDao::new(self.pool.clone());

        // Try to find existing device by name.
        let device_id = match dao.find_by_name(self.tenant_id, device_name).await {
            Ok(Some(d)) => d.id,
            Ok(None) => {
                // Create new child device.
                let now = chrono::Utc::now().timestamp_millis();

                // Find default device profile for this tenant.
                let profile_dao =
                    vl_dao::postgres::device_profile::DeviceProfileDao::new(self.pool.clone());
                let profile_id = match profile_dao.find_default(self.tenant_id).await {
                    Ok(Some(p)) => p.id,
                    _ => {
                        warn!(
                            gateway = %self.gateway_device_id,
                            "No default device profile for tenant {}",
                            self.tenant_id
                        );
                        return None;
                    }
                };

                let device = vl_core::entities::Device {
                    id: Uuid::new_v4(),
                    created_time: now,
                    tenant_id: self.tenant_id,
                    customer_id: None,
                    device_profile_id: profile_id,
                    name: device_name.to_string(),
                    device_type: device_type.to_string(),
                    label: None,
                    device_data: Some(serde_json::json!({"gateway": self.gateway_device_id})),
                    firmware_id: None,
                    software_id: None,
                    external_id: None,
                    additional_info: None,
                    version: 1,
                };

                if let Err(e) = dao.save(&device).await {
                    warn!(
                        gateway = %self.gateway_device_id,
                        child = %device_name,
                        "Failed to create gateway child device: {e}"
                    );
                    return None;
                }

                // Generate access token.
                let token = Uuid::new_v4().to_string().replace('-', "");
                let creds = vl_core::entities::DeviceCredentials {
                    id: Uuid::new_v4(),
                    created_time: now,
                    device_id: device.id,
                    credentials_type: vl_core::entities::DeviceCredentialsType::AccessToken,
                    credentials_id: token,
                    credentials_value: None,
                };
                dao.save_credentials(&creds).await.ok();

                info!(
                    gateway = %self.gateway_device_id,
                    child = %device_name,
                    child_id = %device.id,
                    "Gateway child device created"
                );
                device.id
            }
            Err(e) => {
                warn!(gateway = %self.gateway_device_id, "Device lookup error: {e}");
                return None;
            }
        };

        self.children.insert(device_name.to_string(), device_id);
        Some(device_id)
    }

    /// Handle gateway disconnect — remove child from session.
    ///
    /// Payload: `{"device": "CHILD_NAME"}`
    pub fn on_disconnect(&mut self, payload: &[u8]) {
        if let Ok(body) = serde_json::from_slice::<serde_json::Value>(payload) {
            if let Some(name) = body.get("device").and_then(|v| v.as_str()) {
                self.children.remove(name);
                debug!(
                    gateway = %self.gateway_device_id,
                    child = %name,
                    "Gateway child disconnected"
                );
            }
        }
    }

    /// Handle gateway telemetry — save telemetry for child devices.
    ///
    /// Payload: `{"DEVICE_A": [{"ts": 123, "values": {"temp": 25}}], "DEVICE_B": [...]}`
    pub async fn on_telemetry(&mut self, payload: &[u8]) {
        let body: serde_json::Value = match serde_json::from_slice(payload) {
            Ok(v) => v,
            Err(_) => return,
        };

        let Some(obj) = body.as_object() else { return };

        for (device_name, data) in obj {
            // Ensure child is connected.
            let device_id = if let Some(&id) = self.children.get(device_name.as_str()) {
                id
            } else {
                // Auto-connect on first telemetry.
                let connect_payload =
                    serde_json::json!({"device": device_name}).to_string();
                match self.on_connect(connect_payload.as_bytes()).await {
                    Some(id) => id,
                    None => continue,
                }
            };

            // Parse telemetry entries.
            let now = chrono::Utc::now().timestamp_millis();
            let entries = match data {
                serde_json::Value::Array(arr) => {
                    let mut records = Vec::new();
                    for item in arr {
                        let ts = item.get("ts").and_then(|v| v.as_i64()).unwrap_or(now);
                        if let Some(values) = item.get("values").and_then(|v| v.as_object()) {
                            for (key, value) in values {
                                records.push(TsRecord {
                                    entity_id: device_id,
                                    key: key.clone(),
                                    ts,
                                    bool_v: value.as_bool(),
                                    long_v: value.as_i64().filter(|_| value.is_i64()),
                                    dbl_v: value.as_f64().filter(|_| value.is_f64()),
                                    str_v: value.as_str().map(String::from),
                                    json_v: if value.is_object() || value.is_array() {
                                        Some(value.clone())
                                    } else {
                                        None
                                    },
                                });
                            }
                        }
                    }
                    records
                }
                serde_json::Value::Object(kv) => {
                    // Flat format: {"temp": 25, "humidity": 60}
                    kv.iter()
                        .map(|(key, value)| TsRecord {
                            entity_id: device_id,
                            key: key.clone(),
                            ts: now,
                            bool_v: value.as_bool(),
                            long_v: value.as_i64().filter(|_| value.is_i64()),
                            dbl_v: value.as_f64().filter(|_| value.is_f64()),
                            str_v: value.as_str().map(String::from),
                            json_v: if value.is_object() || value.is_array() {
                                Some(value.clone())
                            } else {
                                None
                            },
                        })
                        .collect()
                }
                _ => continue,
            };

            if !entries.is_empty() {
                if let Err(e) = self.ts_dao.save_batch("DEVICE", &entries).await {
                    warn!(device = %device_name, "Gateway telemetry save failed: {e}");
                }
                if let Err(e) = self.ts_dao.save_latest_batch("DEVICE", &entries).await {
                    warn!(device = %device_name, "Gateway latest save failed: {e}");
                }
                debug!(
                    gateway = %self.gateway_device_id,
                    child = %device_name,
                    entries = entries.len(),
                    "Gateway telemetry saved"
                );
            }
        }
    }

    /// Handle gateway attribute updates for child devices.
    ///
    /// Payload: `{"DEVICE_A": {"attr1": "value1"}, "DEVICE_B": {"attr2": "value2"}}`
    pub async fn on_attributes(&mut self, payload: &[u8]) {
        let body: serde_json::Value = match serde_json::from_slice(payload) {
            Ok(v) => v,
            Err(_) => return,
        };

        let Some(obj) = body.as_object() else { return };
        let kv_dao = vl_dao::postgres::kv::KvDao::new(self.pool.clone());
        let now = chrono::Utc::now().timestamp_millis();

        for (device_name, attrs) in obj {
            let device_id = if let Some(&id) = self.children.get(device_name.as_str()) {
                id
            } else {
                continue;
            };

            if let Some(attr_obj) = attrs.as_object() {
                for (key, value) in attr_obj {
                    if let Ok(key_id) = kv_dao.get_or_create_key(key).await {
                        let attr = vl_core::entities::AttributeKvEntry {
                            entity_id: device_id,
                            attribute_type: vl_core::entities::AttributeScope::ClientScope,
                            attribute_key: key_id,
                            last_update_ts: now,
                            bool_v: value.as_bool(),
                            long_v: value.as_i64().filter(|_| value.is_i64()),
                            dbl_v: value.as_f64().filter(|_| value.is_f64()),
                            str_v: value.as_str().map(String::from),
                            json_v: if value.is_object() || value.is_array() {
                                Some(value.clone())
                            } else {
                                None
                            },
                            version: 0,
                        };
                        kv_dao.save_attribute(&attr).await.ok();
                    }
                }
            }
        }
    }

    /// Check if a topic is a gateway topic.
    pub fn is_gateway_topic(topic: &str) -> bool {
        topic.starts_with("v1/gateway/")
    }

    /// Get the child device count.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}
