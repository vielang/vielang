use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use cron::Schedule;
use vl_core::entities::scheduled_job::ScheduledJob;
use vl_dao::ScheduledJobDao;
use tracing::{error, info, warn};

// ── JobHandler trait ──────────────────────────────────────────────────────────

/// A pluggable handler that executes the business logic for a specific job_type.
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// The job_type string this handler is registered for (e.g. "CLEANUP", "NOTIFICATION").
    fn job_type(&self) -> &'static str;

    /// Execute the job. Returns a JSON result on success, or an error message on failure.
    async fn execute(&self, job: &ScheduledJob) -> Result<serde_json::Value, String>;
}

// ── CleanupJobHandler ─────────────────────────────────────────────────────────

/// Runs a partial telemetry/event/alarm/rpc cleanup pass (same logic as HousekeeperService
/// but driven by the job's `configuration` for per-tenant TTL overrides).
///
/// Configuration fields (all optional; fall back to housekeeper defaults):
/// ```json
/// {
///   "ts_ttl_days": 365,
///   "events_ttl_days": 7,
///   "alarms_ttl_days": 30,
///   "rpc_ttl_days": 1,
///   "batch_size": 10000
/// }
/// ```
pub struct CleanupJobHandler {
    dao: Arc<vl_dao::HousekeeperDao>,
    default_config: vl_config::HousekeeperConfig,
}

impl CleanupJobHandler {
    pub fn new(dao: Arc<vl_dao::HousekeeperDao>, default_config: vl_config::HousekeeperConfig) -> Self {
        Self { dao, default_config }
    }
}

#[async_trait]
impl JobHandler for CleanupJobHandler {
    fn job_type(&self) -> &'static str { "CLEANUP" }

    async fn execute(&self, job: &ScheduledJob) -> Result<serde_json::Value, String> {
        let cfg = &job.configuration;
        let ms_per_day = 86_400_000i64;
        let now_ms = chrono::Utc::now().timestamp_millis();

        let ts_ttl_days    = cfg.get("ts_ttl_days")   .and_then(|v| v.as_i64()).unwrap_or(self.default_config.ts_ttl_days);
        let events_ttl_days = cfg.get("events_ttl_days").and_then(|v| v.as_i64()).unwrap_or(self.default_config.events_ttl_days);
        let alarms_ttl_days = cfg.get("alarms_ttl_days").and_then(|v| v.as_i64()).unwrap_or(self.default_config.alarms_ttl_days);
        let rpc_ttl_days    = cfg.get("rpc_ttl_days")  .and_then(|v| v.as_i64()).unwrap_or(self.default_config.rpc_ttl_days);
        let batch_size      = cfg.get("batch_size")    .and_then(|v| v.as_i64()).unwrap_or(self.default_config.batch_size);

        let cleaned_ts = self.dao
            .delete_old_telemetry(now_ms - ts_ttl_days * ms_per_day, batch_size)
            .await
            .map_err(|e| format!("telemetry cleanup: {e}"))?;

        let cleaned_events = self.dao
            .delete_old_events(now_ms - events_ttl_days * ms_per_day, batch_size)
            .await
            .map_err(|e| format!("events cleanup: {e}"))?;

        let cleaned_alarms = self.dao
            .delete_old_alarms(now_ms - alarms_ttl_days * ms_per_day, batch_size)
            .await
            .map_err(|e| format!("alarms cleanup: {e}"))?;

        let cleaned_rpc = self.dao
            .delete_old_rpc(now_ms - rpc_ttl_days * ms_per_day, batch_size)
            .await
            .map_err(|e| format!("rpc cleanup: {e}"))?;

        info!(
            job_id = %job.id,
            "CleanupJobHandler done: ts={} events={} alarms={} rpc={}",
            cleaned_ts, cleaned_events, cleaned_alarms, cleaned_rpc
        );

        Ok(serde_json::json!({
            "cleanedTelemetry": cleaned_ts,
            "cleanedEvents":    cleaned_events,
            "cleanedAlarms":    cleaned_alarms,
            "cleanedRpc":       cleaned_rpc,
        }))
    }
}

// ── ScheduledNotificationJobHandler ──────────────────────────────────────────

/// Sends a scheduled push notification to a list of recipients.
///
/// Configuration fields:
/// ```json
/// {
///   "subject":           "Optional title",
///   "body":              "Notification body text",
///   "notification_type": "SCHEDULED_ALERT",
///   "severity":          "INFO",
///   "recipients":        ["uuid1", "uuid2"]
/// }
/// ```
pub struct ScheduledNotificationJobHandler {
    delivery_svc: Arc<crate::services::notification_delivery::NotificationDeliveryService>,
}

impl ScheduledNotificationJobHandler {
    pub fn new(delivery_svc: Arc<crate::services::notification_delivery::NotificationDeliveryService>) -> Self {
        Self { delivery_svc }
    }
}

#[async_trait]
impl JobHandler for ScheduledNotificationJobHandler {
    fn job_type(&self) -> &'static str { "NOTIFICATION" }

    async fn execute(&self, job: &ScheduledJob) -> Result<serde_json::Value, String> {
        let cfg = &job.configuration;

        let subject = cfg.get("subject").and_then(|v| v.as_str()).map(str::to_owned);
        let body = cfg.get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "configuration.body is required for NOTIFICATION jobs".to_string())?;
        let notification_type = cfg.get("notification_type")
            .and_then(|v| v.as_str())
            .unwrap_or("SCHEDULED_NOTIFICATION");
        let severity = cfg.get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("INFO");

        let recipients: Vec<uuid::Uuid> = cfg
            .get("recipients")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| uuid::Uuid::parse_str(s).ok())
                    .collect()
            })
            .unwrap_or_default();

        if recipients.is_empty() {
            warn!(job_id = %job.id, "NOTIFICATION job has no recipients; skipping delivery");
            return Ok(serde_json::json!({ "sent": 0 }));
        }

        self.delivery_svc.deliver(
            job.tenant_id,
            subject.as_deref(),
            body,
            Some(notification_type),
            severity,
            &recipients,
        ).await;

        info!(job_id = %job.id, "ScheduledNotificationJobHandler sent to {} recipients", recipients.len());
        Ok(serde_json::json!({ "sent": recipients.len() }))
    }
}

// ── RuleChainTriggerJobHandler ────────────────────────────────────────────────

/// Pushes a synthetic TbMsg into the rule engine to trigger a rule chain.
///
/// Configuration fields:
/// ```json
/// {
///   "originator_type": "DEVICE",
///   "originator_id":   "uuid",
///   "msg_type":        "POST_TELEMETRY_REQUEST",
///   "data":            {}
/// }
/// ```
pub struct RuleChainTriggerJobHandler {
    rule_engine: Arc<vl_rule_engine::RuleEngine>,
}

impl RuleChainTriggerJobHandler {
    pub fn new(rule_engine: Arc<vl_rule_engine::RuleEngine>) -> Self {
        Self { rule_engine }
    }
}

#[async_trait]
impl JobHandler for RuleChainTriggerJobHandler {
    fn job_type(&self) -> &'static str { "RULE_CHAIN_TRIGGER" }

    async fn execute(&self, job: &ScheduledJob) -> Result<serde_json::Value, String> {
        use vl_core::entities::TbMsg;

        let cfg = &job.configuration;

        let originator_type = cfg.get("originator_type")
            .and_then(|v| v.as_str())
            .unwrap_or("TENANT")
            .to_owned();
        let originator_id = cfg.get("originator_id")
            .and_then(|v| v.as_str())
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .unwrap_or(job.tenant_id);
        let msg_type = cfg.get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("SCHEDULED_JOB")
            .to_owned();
        let data = cfg.get("data")
            .and_then(|v| serde_json::to_string(v).ok())
            .unwrap_or_else(|| "{}".to_owned());

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("jobId".to_owned(), job.id.to_string());
        metadata.insert("jobName".to_owned(), job.name.clone());

        let msg = TbMsg {
            id:              uuid::Uuid::new_v4(),
            ts:              chrono::Utc::now().timestamp_millis(),
            msg_type,
            originator_id,
            originator_type,
            customer_id:     None,
            metadata,
            data,
            rule_chain_id:   None,
            rule_node_id:    None,
            tenant_id:       Some(job.tenant_id),
        };

        self.rule_engine.send_async(msg).await;

        info!(job_id = %job.id, "RuleChainTriggerJobHandler dispatched message to rule engine");
        Ok(serde_json::json!({ "dispatched": true }))
    }
}

// ── JobSchedulerService ───────────────────────────────────────────────────────

pub struct JobSchedulerService {
    dao:              Arc<ScheduledJobDao>,
    check_interval_s: u64,
    max_concurrent:   usize,
    handlers:         HashMap<String, Arc<dyn JobHandler>>,
}

impl JobSchedulerService {
    pub fn new(dao: Arc<ScheduledJobDao>) -> Self {
        Self {
            dao,
            check_interval_s: 60,
            max_concurrent:   10,
            handlers:         HashMap::new(),
        }
    }

    pub fn with_config(dao: Arc<ScheduledJobDao>, check_interval_s: u64, max_concurrent: usize) -> Self {
        Self {
            dao,
            check_interval_s,
            max_concurrent,
            handlers: HashMap::new(),
        }
    }

    /// Register a handler. Called during AppState construction before `start()`.
    pub fn register(mut self, handler: Arc<dyn JobHandler>) -> Self {
        self.handlers.insert(handler.job_type().to_owned(), handler);
        self
    }

    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let interval_s = self.check_interval_s;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_s));
            interval.tick().await; // skip first immediate tick
            loop {
                interval.tick().await;
                if let Err(e) = self.run_tick().await {
                    error!("JobScheduler tick failed: {}", e);
                }
            }
        })
    }

    pub async fn run_tick(&self) -> anyhow::Result<()> {
        let now_ms = chrono::Utc::now().timestamp_millis();
        let due_jobs = self.dao.find_due_jobs(now_ms).await?;

        for job in due_jobs {
            let dao = self.dao.clone();
            let handler = self.handlers.get(&job.job_type).cloned();
            let job_id = job.id;
            let job_name = job.name.clone();
            let schedule_type = job.schedule_type.clone();
            let interval_ms = job.interval_ms;
            let cron_expression = job.cron_expression.clone();

            tokio::spawn(async move {
                info!("JobScheduler executing job '{}' ({}) type={}", job_name, job_id, job.job_type);

                let (status, error_msg, result) = match handler {
                    Some(h) => match h.execute(&job).await {
                        Ok(val) => ("SUCCESS", None, Some(val)),
                        Err(msg) => {
                            error!("Job '{}' ({}) failed: {}", job_name, job_id, msg);
                            ("FAILURE", Some(msg), None)
                        }
                    },
                    None => {
                        warn!("No handler registered for job_type '{}' (job '{}')", job.job_type, job_name);
                        ("SUCCESS", None, Some(serde_json::json!({ "note": "no handler" })))
                    }
                };

                let finished_at = chrono::Utc::now().timestamp_millis();

                if let Err(e) = dao
                    .record_execution(job_id, status, error_msg.as_deref(), result)
                    .await
                {
                    error!("Failed to record execution for job {}: {}", job_id, e);
                }

                let next_run_at = compute_next_run(
                    finished_at,
                    &schedule_type,
                    interval_ms,
                    cron_expression.as_deref(),
                );

                if let Err(e) = dao.update_next_run(job_id, finished_at, next_run_at).await {
                    error!("Failed to update next_run for job {}: {}", job_id, e);
                } else {
                    info!("Job '{}' done ({}); next_run_at={}", job_name, status, next_run_at);
                }
            });
        }

        Ok(())
    }

    /// Manually trigger a single job by id (runs in background).
    pub async fn trigger_job(&self, job_id: uuid::Uuid) -> anyhow::Result<()> {
        let job = self
            .dao
            .find_by_id(job_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Job not found: {}", job_id))?;

        let dao = self.dao.clone();
        let handler = self.handlers.get(&job.job_type).cloned();
        let job_name = job.name.clone();
        let schedule_type = job.schedule_type.clone();
        let interval_ms = job.interval_ms;
        let cron_expression = job.cron_expression.clone();

        tokio::spawn(async move {
            info!("JobScheduler manual trigger for job '{}' ({}) type={}", job_name, job_id, job.job_type);

            let (status, error_msg, result) = match handler {
                Some(h) => match h.execute(&job).await {
                    Ok(val) => ("SUCCESS", None, Some(val)),
                    Err(msg) => {
                        error!("Manual trigger job '{}' ({}) failed: {}", job_name, job_id, msg);
                        ("FAILURE", Some(msg), None)
                    }
                },
                None => {
                    warn!("No handler for job_type '{}' in manual trigger '{}'", job.job_type, job_name);
                    ("SUCCESS", None, Some(serde_json::json!({ "note": "no handler" })))
                }
            };

            let finished_at = chrono::Utc::now().timestamp_millis();

            if let Err(e) = dao
                .record_execution(job_id, status, error_msg.as_deref(), result)
                .await
            {
                error!("Failed to record execution for job {}: {}", job_id, e);
            }

            let next_run_at = compute_next_run(
                finished_at,
                &schedule_type,
                interval_ms,
                cron_expression.as_deref(),
            );

            if let Err(e) = dao.update_next_run(job_id, finished_at, next_run_at).await {
                error!("Failed to update next_run for job {}: {}", job_id, e);
            }
        });

        Ok(())
    }
}

// ── scheduling helpers ────────────────────────────────────────────────────────

/// Compute next_run_at given current time and schedule config.
/// Supports INTERVAL (interval_ms) and CRON (@hourly, @daily, @weekly, basic 5-field).
fn compute_next_run(
    now_ms: i64,
    schedule_type: &str,
    interval_ms: Option<i64>,
    cron_expression: Option<&str>,
) -> i64 {
    match schedule_type {
        "INTERVAL" => {
            if let Some(interval) = interval_ms {
                if interval > 0 {
                    return now_ms + interval;
                }
            }
            warn!("INTERVAL job has no valid interval_ms; scheduling far future");
            now_ms + 3_600_000 // default 1 hour
        }
        "CRON" => {
            if let Some(expr) = cron_expression {
                return parse_cron_next(now_ms, expr);
            }
            warn!("CRON job has no cron_expression; scheduling far future");
            now_ms + 3_600_000
        }
        other => {
            warn!("Unknown schedule_type '{}'; scheduling +1h", other);
            now_ms + 3_600_000
        }
    }
}

/// Validate a cron expression. Returns Ok(()) if valid, Err(description) if not.
/// Accepts both 5-field (`min hour day mon dow`) and 6-field (`sec min hour day mon dow`) forms.
pub fn validate_cron(expr: &str) -> Result<(), String> {
    // The `cron` crate requires 6 fields (sec min hour day mon dow).
    // Expand 5-field expressions by prepending "0 ".
    let normalized = normalize_cron(expr);
    Schedule::from_str(&normalized)
        .map(|_| ())
        .map_err(|e| format!("Invalid cron expression '{}': {}", expr, e))
}

/// Compute the next run time (ms) for a cron expression after `now_ms`.
fn parse_cron_next(now_ms: i64, expr: &str) -> i64 {
    let normalized = normalize_cron(expr);
    match Schedule::from_str(&normalized) {
        Ok(schedule) => {
            let after = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(now_ms)
                .unwrap_or_else(chrono::Utc::now);
            if let Some(next) = schedule.after(&after).next() {
                return next.timestamp_millis();
            }
            warn!("Cron '{}' has no upcoming run; scheduling +1h", expr);
            now_ms + 3_600_000
        }
        Err(e) => {
            warn!("Unparseable cron '{}': {}; scheduling +1h", expr, e);
            now_ms + 3_600_000
        }
    }
}

/// Normalize a cron expression for the `cron` crate (which needs 6 fields: sec min hour day mon dow).
/// If the expression has 5 space-separated fields, prepend "0 " (seconds = 0).
fn normalize_cron(expr: &str) -> String {
    let expr = expr.trim();
    // Handle @aliases that the cron crate supports natively
    if expr.starts_with('@') {
        return expr.to_string();
    }
    let field_count = expr.split_whitespace().count();
    if field_count == 5 {
        format!("0 {}", expr)
    } else {
        expr.to_string()
    }
}
