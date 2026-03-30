use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `alarm`.
/// Java: org.thingsboard.server.common.data.alarm.Alarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarm {
    pub id: Uuid,
    pub created_time: i64,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,

    pub alarm_type: String,

    pub originator_id: Uuid,
    /// Khớp Java: EntityType enum → integer trong DB
    pub originator_type: EntityType,

    pub severity: AlarmSeverity,
    pub acknowledged: bool,
    pub cleared: bool,

    pub assignee_id: Option<Uuid>,

    pub start_ts: i64,
    pub end_ts: i64,
    pub ack_ts: Option<i64>,
    pub clear_ts: Option<i64>,
    pub assign_ts: i64,

    /// JSONB — alarm details
    pub details: Option<serde_json::Value>,

    pub propagate: bool,
    pub propagate_to_owner: bool,
    pub propagate_to_tenant: bool,
    /// CSV của relation types
    pub propagate_relation_types: Option<String>,
}

impl Alarm {
    pub fn status(&self) -> AlarmStatus {
        match (self.cleared, self.acknowledged) {
            (false, false) => AlarmStatus::ActiveUnack,
            (false, true)  => AlarmStatus::ActiveAck,
            (true, false)  => AlarmStatus::ClearedUnack,
            (true, true)   => AlarmStatus::ClearedAck,
        }
    }
}

/// Khớp Java: AlarmSeverity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlarmSeverity {
    Critical,
    Major,
    Minor,
    Warning,
    Indeterminate,
}

/// Computed — không có cột riêng trong DB
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlarmStatus {
    ActiveUnack,
    ActiveAck,
    ClearedUnack,
    ClearedAck,
}

/// Khớp Java: EntityType — integer trong cột `originator_type`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Tenant = 0,
    Customer = 1,
    User = 2,
    Dashboard = 3,
    Asset = 4,
    Device = 5,
    AlarmEntity = 6,
    RuleChain = 7,
    RuleNode = 8,
    EntityView = 9,
    TenantProfile = 10,
    DeviceProfile = 11,
    AssetProfile = 12,
    Edge = 13,
    OtaPackage = 14,
    RuleEngineQueue = 15,
    NotificationTemplate = 16,
    NotificationTarget = 17,
    NotificationRule = 18,
    DashboardWidget = 19,
}

/// Khớp Java: AlarmComment — comment gắn vào alarm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlarmComment {
    pub id: Uuid,
    pub created_time: i64,
    pub alarm_id: Uuid,
    pub user_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub comment_type: String,
    pub comment: serde_json::Value,
}

impl TryFrom<i32> for EntityType {
    type Error = ();
    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            0  => Ok(Self::Tenant),
            1  => Ok(Self::Customer),
            2  => Ok(Self::User),
            3  => Ok(Self::Dashboard),
            4  => Ok(Self::Asset),
            5  => Ok(Self::Device),
            6  => Ok(Self::AlarmEntity),
            7  => Ok(Self::RuleChain),
            8  => Ok(Self::RuleNode),
            9  => Ok(Self::EntityView),
            10 => Ok(Self::TenantProfile),
            11 => Ok(Self::DeviceProfile),
            12 => Ok(Self::AssetProfile),
            13 => Ok(Self::Edge),
            _  => Err(()),
        }
    }
}

impl From<EntityType> for i32 {
    fn from(e: EntityType) -> i32 {
        e as i32
    }
}
