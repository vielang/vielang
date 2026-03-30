use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use vl_dao::postgres::{
    alarm::AlarmDao, asset::AssetDao, customer::CustomerDao,
    device::DeviceDao, device_profile::DeviceProfileDao,
    event::EventDao, geofence::GeofenceDao,
    kv::KvDao, relation::RelationDao, tenant::TenantDao,
};
use vl_core::entities::{TbMsg, EdgeSender};
use crate::error::RuleEngineError;

// ── RelationType ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    Success,
    Failure,
    True,
    False,
    Other(String),
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationType::Success    => write!(f, "Success"),
            RelationType::Failure    => write!(f, "Failure"),
            RelationType::True       => write!(f, "True"),
            RelationType::False      => write!(f, "False"),
            RelationType::Other(s)   => write!(f, "{}", s),
        }
    }
}

impl From<&str> for RelationType {
    fn from(s: &str) -> Self {
        match s {
            "Success" => RelationType::Success,
            "Failure" => RelationType::Failure,
            "True"    => RelationType::True,
            "False"   => RelationType::False,
            other     => RelationType::Other(other.to_string()),
        }
    }
}

// ── DaoServices ───────────────────────────────────────────────────────────────

/// Services that rule nodes can use to interact with DB
pub struct DaoServices {
    pub kv:             Arc<KvDao>,
    pub alarm:          Arc<AlarmDao>,
    pub device:         Arc<DeviceDao>,
    pub device_profile: Arc<DeviceProfileDao>,
    pub asset:          Arc<AssetDao>,
    pub relation:       Arc<RelationDao>,
    pub customer:       Arc<CustomerDao>,
    pub tenant:         Arc<TenantDao>,
    /// Event DAO — used by debug node and chain-level debug event persistence.
    pub event:          Arc<EventDao>,
    /// Geofence DAO — used by GpsGeofencingFilterNode in DB-backed mode.
    pub geofence:       Arc<GeofenceDao>,
}

// ── RuleNodeCtx ───────────────────────────────────────────────────────────────

pub struct RuleNodeCtx {
    pub node_id:      Uuid,
    pub tenant_id:    Uuid,
    pub dao:          Arc<DaoServices>,
    /// Optional edge sender — None khi rule engine chạy không có edge gRPC server.
    /// Set bởi AppState khi khởi động với EdgeSessionRegistry.
    pub edge_sender:  Option<Arc<dyn EdgeSender>>,
}

// ── RuleNode trait ────────────────────────────────────────────────────────────

#[async_trait]
pub trait RuleNode: Send + Sync {
    /// Process a message. Returns a list of (relation_type, output_message) pairs.
    /// Each pair routes the output message along the given relation to downstream nodes.
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError>;
}
