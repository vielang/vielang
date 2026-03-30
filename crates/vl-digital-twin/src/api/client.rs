//! REST API client — fetches device list and other data from the VieLang backend.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::TwinConfig;

/// REST API configuration.
#[derive(Resource, Debug, Clone)]
pub struct ApiConfig {
    /// e.g. "http://localhost:8080"
    pub base_url:  String,
    pub jwt_token: String,
}

impl ApiConfig {
    pub fn from_twin_config(cfg: &TwinConfig) -> Self {
        Self {
            base_url:  cfg.server.base_url.clone(),
            jwt_token: cfg.auth.token.clone(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        // Fallback khi không có TwinConfig — đọc env vars
        Self {
            base_url:  std::env::var("TB_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
            jwt_token: std::env::var("TB_TOKEN").unwrap_or_default(),
        }
    }
}

/// Simplified device DTO matching `/api/tenant/devices` response.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDto {
    pub id:   DeviceId,
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: DeviceId,
    pub created_time: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceId {
    pub id: Uuid,
    #[serde(rename = "entityType")]
    pub entity_type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageData<T> {
    pub data:           Vec<T>,
    pub total_pages:    i64,
    pub total_elements: i64,
    pub has_next:       bool,
}

/// Login response from `/api/auth/login`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token:         String,
    pub refresh_token: String,
    pub scope:         String,
}

/// Login request body.
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Bevy Resource that caches the fetched device list.
#[derive(Resource, Default, Debug)]
pub struct DeviceListCache {
    pub devices:  Vec<DeviceDto>,
    pub loaded:   bool,
    pub error:    Option<String>,
}

// ── REST Client (native only) ─────────────────────────────────────────────────

/// High-level API client wrapping reqwest.
#[cfg(not(target_arch = "wasm32"))]
pub struct ApiClient {
    pub config: ApiConfig,
    client:     reqwest::Client,
}

#[cfg(not(target_arch = "wasm32"))]
impl ApiClient {
    pub fn new(config: ApiConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    /// POST /api/auth/login and return JWT token.
    pub async fn login(&self, username: &str, password: &str) -> Result<String, String> {
        let url = format!("{}/api/auth/login", self.config.base_url);
        let body = LoginRequest {
            username: username.into(),
            password: password.into(),
        };
        let resp = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            let login: LoginResponse = resp.json().await.map_err(|e| e.to_string())?;
            Ok(login.token)
        } else {
            Err(format!("Login failed: HTTP {}", resp.status()))
        }
    }

    /// GET /api/tenant/devices?pageSize=N&page=0
    pub async fn get_devices(&self, page_size: i32) -> Result<Vec<DeviceDto>, String> {
        let url = format!(
            "{}/api/tenant/devices?pageSize={}&page=0",
            self.config.base_url, page_size
        );
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.jwt_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            let page: PageData<DeviceDto> = resp.json().await.map_err(|e| e.to_string())?;
            Ok(page.data)
        } else {
            Err(format!("Get devices failed: HTTP {}", resp.status()))
        }
    }
}

// ── Timeseries / historical query types ──────────────────────────────────────

/// Aggregation function for historical timeseries queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AggregationType {
    Avg,
    Min,
    Max,
    Sum,
    Count,
    None,
}

impl AggregationType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Avg   => "AVG",
            Self::Min   => "MIN",
            Self::Max   => "MAX",
            Self::Sum   => "SUM",
            Self::Count => "COUNT",
            Self::None  => "NONE",
        }
    }
}

/// Parameters for a timeseries history query.
#[derive(Debug, Clone)]
pub struct TimeseriesQuery {
    pub entity_type: String,
    pub entity_id:   Uuid,
    /// Telemetry keys to fetch (e.g. ["temperature", "humidity"]).
    pub keys:        Vec<String>,
    pub start_ts:    i64,
    pub end_ts:      i64,
    /// Aggregation interval in ms — None = raw data.
    pub interval_ms: Option<i64>,
    /// Max data points per key.
    pub limit:       Option<u32>,
    pub agg:         AggregationType,
    /// true = ASC order, false = DESC.
    pub order_asc:   bool,
}

/// A single telemetry data point: (timestamp_ms, value).
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub ts:    i64,
    pub value: f64,
}

impl<'de> Deserialize<'de> for DataPoint {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // ThingsBoard format: [ts_ms, "value_as_string"]
        let arr: [serde_json::Value; 2] = Deserialize::deserialize(d)?;
        let ts    = arr[0].as_i64().unwrap_or(0);
        let value = arr[1]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| arr[1].as_f64())
            .unwrap_or(0.0);
        Ok(Self { ts, value })
    }
}

// ── RPC types ─────────────────────────────────────────────────────────────────

/// Request body for one-way or two-way RPC calls.
#[derive(Debug, Serialize)]
pub struct RpcRequest {
    pub method: String,
    pub params: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

/// One entry in the RPC history returned by the persistent RPC endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcHistoryEntry {
    pub id:           RpcId,
    pub created_time: i64,
    pub status:       String,
    pub request:      serde_json::Value,
    pub response:     Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcId {
    pub id: Uuid,
}

// ── Native RPC methods ────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
impl ApiClient {
    /// GET /api/plugins/telemetry/DEVICE/{id}/values/timeseries
    /// Returns map of key → sorted Vec<DataPoint>.
    pub async fn get_timeseries(
        &self,
        query: &TimeseriesQuery,
    ) -> Result<std::collections::HashMap<String, Vec<DataPoint>>, String> {
        let keys_str = query.keys.join(",");
        let mut url = format!(
            "{}/api/plugins/telemetry/{}/{}/values/timeseries\
             ?keys={}&startTs={}&endTs={}&agg={}&orderBy={}&useStrictDataTypes=true",
            self.config.base_url,
            query.entity_type,
            query.entity_id,
            keys_str,
            query.start_ts,
            query.end_ts,
            query.agg.as_str(),
            if query.order_asc { "ASC" } else { "DESC" },
        );
        if let Some(interval) = query.interval_ms {
            url.push_str(&format!("&interval={interval}"));
        }
        if let Some(limit) = query.limit {
            url.push_str(&format!("&limit={limit}"));
        }

        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.jwt_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            resp.json::<std::collections::HashMap<String, Vec<DataPoint>>>()
                .await
                .map_err(|e| e.to_string())
        } else {
            Err(format!("Timeseries query failed: HTTP {}", resp.status()))
        }
    }

    /// GET latest values for a set of keys (no time range — returns current values).
    pub async fn get_latest_timeseries(
        &self,
        entity_id: Uuid,
        keys:      &[&str],
    ) -> Result<std::collections::HashMap<String, f64>, String> {
        let keys_str = keys.join(",");
        let url = format!(
            "{}/api/plugins/telemetry/DEVICE/{entity_id}/values/timeseries?keys={keys_str}",
            self.config.base_url
        );
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.jwt_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Get latest timeseries failed: HTTP {}", resp.status()));
        }

        let raw: std::collections::HashMap<String, Vec<DataPoint>> =
            resp.json().await.map_err(|e| e.to_string())?;

        Ok(raw.into_iter()
            .filter_map(|(k, pts)| pts.last().map(|p| (k, p.value)))
            .collect())
    }

    /// POST /api/plugins/rpc/oneway/{deviceId} — fire-and-forget command.
    pub async fn send_rpc_oneway(
        &self,
        device_id: Uuid,
        method:    &str,
        params:    serde_json::Value,
    ) -> Result<(), String> {
        let url  = format!("{}/api/plugins/rpc/oneway/{device_id}", self.config.base_url);
        let body = RpcRequest { method: method.into(), params, timeout: None };
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.jwt_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            Err(format!("RPC oneway failed: HTTP {status} — {body}"))
        }
    }

    /// POST /api/plugins/rpc/twoway/{deviceId} — waits for device response (30 s timeout).
    pub async fn send_rpc_twoway(
        &self,
        device_id: Uuid,
        method:    &str,
        params:    serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let url  = format!("{}/api/plugins/rpc/twoway/{device_id}", self.config.base_url);
        let body = RpcRequest { method: method.into(), params, timeout: Some(30_000) };
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.jwt_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            resp.json::<serde_json::Value>().await.map_err(|e| e.to_string())
        } else {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            Err(format!("RPC twoway failed: HTTP {status} — {body}"))
        }
    }

    /// GET /api/plugins/rpc/persistent/device/{deviceId} — fetch recent RPC history.
    pub async fn get_rpc_history(
        &self,
        device_id: Uuid,
        page_size: i32,
    ) -> Result<Vec<RpcHistoryEntry>, String> {
        let url  = format!(
            "{}/api/plugins/rpc/persistent/device/{device_id}?pageSize={page_size}&page=0",
            self.config.base_url
        );
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.jwt_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            let page: PageData<RpcHistoryEntry> = resp.json().await.map_err(|e| e.to_string())?;
            Ok(page.data)
        } else {
            Err(format!("Get RPC history failed: HTTP {}", resp.status()))
        }
    }
}

// ── WASM stub ─────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub struct ApiClient {
    pub config: ApiConfig,
}

#[cfg(target_arch = "wasm32")]
impl ApiClient {
    pub fn new(config: ApiConfig) -> Self {
        Self { config }
    }
}
