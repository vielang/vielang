use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HousekeeperExecution {
    pub id: Uuid,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub cleaned_telemetry: i64,
    pub cleaned_events: i64,
    pub cleaned_alarms: i64,
    pub cleaned_rpc: i64,
    pub status: String,
}
