use std::sync::Arc;
use std::time::Duration;
use dashmap::DashMap;
use uuid::Uuid;

use vl_dao::ApiUsageDao;

/// Thread-safe in-memory buffer for API usage counters.
///
/// Every transport message, rule-engine execution, email, etc. calls `record_*()`
/// which is non-blocking (just increments a DashMap counter). A background task
/// calls `flush()` every 60 seconds to atomically persist the deltas to
/// `api_usage_state` via a single upsert per tenant-period.
pub struct UsageTracker {
    /// (tenant_id, billing_period) → per-metric deltas
    buffer: DashMap<(Uuid, String), UsageDelta>,
    dao:    Arc<ApiUsageDao>,
}

#[derive(Default, Clone)]
struct UsageDelta {
    transport_msg:    i64,
    transport_dp:     i64,
    re_exec:          i64,
    js_exec:          i64,
    email:            i32,
    sms:              i32,
    alarm:            i32,
    // P12 counters
    storage_dp:       i64,
    rpc:              i64,
    rule_engine_exec: i64,
}

impl UsageTracker {
    pub fn new(dao: Arc<ApiUsageDao>) -> Self {
        Self {
            buffer: DashMap::new(),
            dao,
        }
    }

    fn current_period() -> String {
        chrono::Utc::now().format("%Y-%m").to_string()
    }

    // ── Record helpers ────────────────────────────────────────────────────────

    pub fn record_transport_msg(&self, tenant_id: Uuid, count: i64) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().transport_msg += count;
    }

    pub fn record_transport_dp(&self, tenant_id: Uuid, count: i64) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().transport_dp += count;
    }

    pub fn record_re_exec(&self, tenant_id: Uuid, count: i64) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().re_exec += count;
    }

    pub fn record_js_exec(&self, tenant_id: Uuid, count: i64) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().js_exec += count;
    }

    pub fn record_email(&self, tenant_id: Uuid) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().email += 1;
    }

    pub fn record_sms(&self, tenant_id: Uuid) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().sms += 1;
    }

    pub fn record_alarm(&self, tenant_id: Uuid) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().alarm += 1;
    }

    pub fn record_storage_dp(&self, tenant_id: Uuid, count: i64) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().storage_dp += count;
    }

    pub fn record_rpc(&self, tenant_id: Uuid) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().rpc += 1;
    }

    pub fn record_rule_engine_exec(&self, tenant_id: Uuid) {
        let key = (tenant_id, Self::current_period());
        self.buffer.entry(key).or_default().rule_engine_exec += 1;
    }

    // ── Flush ─────────────────────────────────────────────────────────────────

    /// Drain the buffer and persist all pending deltas to the database.
    /// Called by the background flush task every 60 seconds.
    pub async fn flush(&self) {
        // Snapshot and drain the buffer atomically per-key
        let entries: Vec<((Uuid, String), UsageDelta)> = self.buffer
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();

        for (key, _) in &entries {
            self.buffer.remove(key);
        }

        for ((tenant_id, period), delta) in entries {
            if let Err(e) = self.dao.increment(
                tenant_id,
                &period,
                delta.transport_msg,
                delta.transport_dp,
                delta.re_exec,
                delta.js_exec,
                delta.email,
                delta.sms,
                delta.alarm,
                delta.storage_dp,
                delta.rpc,
                delta.rule_engine_exec,
            ).await {
                tracing::warn!("UsageTracker flush error for tenant {}: {}", tenant_id, e);
            }
        }
    }

    /// Start the background flush loop. Returns a JoinHandle that runs forever.
    pub fn start_flush_loop(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                self.flush().await;
            }
        })
    }
}
