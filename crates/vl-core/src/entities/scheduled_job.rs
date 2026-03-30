use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledJob {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub job_type: String,
    pub schedule_type: String,
    pub interval_ms: Option<i64>,
    pub cron_expression: Option<String>,
    pub configuration: serde_json::Value,
    pub enabled: bool,
    pub last_run_at: Option<i64>,
    pub next_run_at: i64,
    pub created_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobExecution {
    pub id: Uuid,
    pub job_id: Uuid,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub status: String,
    pub error_message: Option<String>,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobRequest {
    pub name: String,
    pub job_type: String,
    pub schedule_type: String,
    pub interval_ms: Option<i64>,
    pub cron_expression: Option<String>,
    pub configuration: serde_json::Value,
    pub enabled: bool,
}
