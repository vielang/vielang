use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Keys for individual API usage counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiUsageRecordKey {
    TransportMsg,
    TransportDp,
    ReExec,
    JsExec,
    Email,
    Sms,
    Alarm,
    ActiveDevices,
    // P12 additions
    StorageDp,
    Rpc,
    RuleEngineExec,
}

impl ApiUsageRecordKey {
    pub fn as_str(self) -> &'static str {
        match self {
            ApiUsageRecordKey::TransportMsg    => "TRANSPORT_MSG",
            ApiUsageRecordKey::TransportDp     => "TRANSPORT_DP",
            ApiUsageRecordKey::ReExec          => "RE_EXEC",
            ApiUsageRecordKey::JsExec          => "JS_EXEC",
            ApiUsageRecordKey::Email           => "EMAIL",
            ApiUsageRecordKey::Sms             => "SMS",
            ApiUsageRecordKey::Alarm           => "ALARM",
            ApiUsageRecordKey::ActiveDevices   => "ACTIVE_DEVICES",
            ApiUsageRecordKey::StorageDp       => "STORAGE_DP",
            ApiUsageRecordKey::Rpc             => "RPC",
            ApiUsageRecordKey::RuleEngineExec  => "RULE_ENGINE_EXEC",
        }
    }
}

/// Usage state for a single metric relative to its plan limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiUsageStateValue {
    Enabled,
    Warning,
    Disabled,
}

/// A single usage metric with its current value, plan limit, and computed state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetric {
    pub key:      String,
    pub value:    i64,
    pub limit:    i64,
    pub state:    ApiUsageStateValue,
    pub pct_used: f32,
}

impl UsageMetric {
    /// Build a `UsageMetric`, computing state automatically.
    ///
    /// * `limit <= 0` → unlimited → always `Enabled`, `pct_used = 0.0`
    /// * `pct_used >= 100.0` → `Disabled`
    /// * `pct_used >= 80.0`  → `Warning`
    /// * otherwise           → `Enabled`
    pub fn new(key: &str, value: i64, limit: i64) -> Self {
        let (state, pct_used) = if limit <= 0 {
            (ApiUsageStateValue::Enabled, 0.0_f32)
        } else {
            let pct = (value as f32 / limit as f32) * 100.0;
            let state = if pct >= 100.0 {
                ApiUsageStateValue::Disabled
            } else if pct >= 80.0 {
                ApiUsageStateValue::Warning
            } else {
                ApiUsageStateValue::Enabled
            };
            (state, pct)
        };

        UsageMetric {
            key: key.to_string(),
            value,
            limit,
            state,
            pct_used,
        }
    }
}

/// Full API usage summary for a tenant in a billing period, compared against their plan limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantApiUsage {
    pub tenant_id:         Uuid,
    pub billing_period:    String,
    pub transport_msg:     UsageMetric,
    pub transport_dp:      UsageMetric,
    pub re_exec:           UsageMetric,
    pub js_exec:           UsageMetric,
    pub email:             UsageMetric,
    pub sms:               UsageMetric,
    pub alarm:             UsageMetric,
    pub active_devices:    UsageMetric,
    // P12 additions
    pub storage_dp:        UsageMetric,
    pub rpc:               UsageMetric,
    pub rule_engine_exec:  UsageMetric,
}

/// Snapshot of all counters for a billing period — stored in api_usage_history.counters JSONB.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUsageHistory {
    pub id:           Uuid,
    pub tenant_id:    Uuid,
    pub period_start: i64,
    pub period_end:   i64,
    pub counters:     serde_json::Value,
    pub created_time: i64,
}
