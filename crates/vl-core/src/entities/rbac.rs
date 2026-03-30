use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Resource enum ─────────────────────────────────────────────────────────────

/// Tài nguyên trong hệ thống — khớp Java `Resource.java` (ThingsBoard PE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Resource {
    AdminSettings,
    Alarm,
    ApiUsageState,
    Asset,
    AssetProfile,
    AuditLog,
    ComponentDescriptor,
    Customer,
    Dashboard,
    Device,
    DeviceCredentials,
    DeviceProfile,
    Edge,
    EntityView,
    Image,
    MobileAppBundle,
    Notification,
    OtaPackage,
    Profile,
    Queue,
    RuleChain,
    RuleNode,
    SchedulerEvent,
    TbResource,
    Tenant,
    TenantProfile,
    User,
    VersionControl,
    WhiteLabeling,
    WidgetType,
    WidgetsBundle,
    Telemetry,
    Attribute,
}

impl Resource {
    pub fn as_str(self) -> &'static str {
        match self {
            Resource::AdminSettings      => "ADMIN_SETTINGS",
            Resource::Alarm              => "ALARM",
            Resource::ApiUsageState      => "API_USAGE_STATE",
            Resource::Asset              => "ASSET",
            Resource::AssetProfile       => "ASSET_PROFILE",
            Resource::AuditLog           => "AUDIT_LOG",
            Resource::ComponentDescriptor => "COMPONENT_DESCRIPTOR",
            Resource::Customer           => "CUSTOMER",
            Resource::Dashboard          => "DASHBOARD",
            Resource::Device             => "DEVICE",
            Resource::DeviceCredentials  => "DEVICE_CREDENTIALS",
            Resource::DeviceProfile      => "DEVICE_PROFILE",
            Resource::Edge               => "EDGE",
            Resource::EntityView         => "ENTITY_VIEW",
            Resource::Image              => "IMAGE",
            Resource::MobileAppBundle    => "MOBILE_APP_BUNDLE",
            Resource::Notification       => "NOTIFICATION",
            Resource::OtaPackage         => "OTA_PACKAGE",
            Resource::Profile            => "PROFILE",
            Resource::Queue              => "QUEUE",
            Resource::RuleChain          => "RULE_CHAIN",
            Resource::RuleNode           => "RULE_NODE",
            Resource::SchedulerEvent     => "SCHEDULER_EVENT",
            Resource::TbResource         => "TB_RESOURCE",
            Resource::Tenant             => "TENANT",
            Resource::TenantProfile      => "TENANT_PROFILE",
            Resource::User               => "USER",
            Resource::VersionControl     => "VERSION_CONTROL",
            Resource::WhiteLabeling      => "WHITE_LABELING",
            Resource::WidgetType         => "WIDGET_TYPE",
            Resource::WidgetsBundle      => "WIDGETS_BUNDLE",
            Resource::Telemetry          => "TELEMETRY",
            Resource::Attribute          => "ATTRIBUTE",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "ADMIN_SETTINGS"      => Resource::AdminSettings,
            "ALARM"               => Resource::Alarm,
            "API_USAGE_STATE"     => Resource::ApiUsageState,
            "ASSET"               => Resource::Asset,
            "ASSET_PROFILE"       => Resource::AssetProfile,
            "AUDIT_LOG"           => Resource::AuditLog,
            "COMPONENT_DESCRIPTOR" => Resource::ComponentDescriptor,
            "CUSTOMER"            => Resource::Customer,
            "DASHBOARD"           => Resource::Dashboard,
            "DEVICE"              => Resource::Device,
            "DEVICE_CREDENTIALS"  => Resource::DeviceCredentials,
            "DEVICE_PROFILE"      => Resource::DeviceProfile,
            "EDGE"                => Resource::Edge,
            "ENTITY_VIEW"         => Resource::EntityView,
            "IMAGE"               => Resource::Image,
            "MOBILE_APP_BUNDLE"   => Resource::MobileAppBundle,
            "NOTIFICATION"        => Resource::Notification,
            "OTA_PACKAGE"         => Resource::OtaPackage,
            "PROFILE"             => Resource::Profile,
            "QUEUE"               => Resource::Queue,
            "RULE_CHAIN"          => Resource::RuleChain,
            "RULE_NODE"           => Resource::RuleNode,
            "SCHEDULER_EVENT"     => Resource::SchedulerEvent,
            "TB_RESOURCE"         => Resource::TbResource,
            "TENANT"              => Resource::Tenant,
            "TENANT_PROFILE"      => Resource::TenantProfile,
            "USER"                => Resource::User,
            "VERSION_CONTROL"     => Resource::VersionControl,
            "WHITE_LABELING"      => Resource::WhiteLabeling,
            "WIDGET_TYPE"         => Resource::WidgetType,
            "WIDGETS_BUNDLE"      => Resource::WidgetsBundle,
            "TELEMETRY"           => Resource::Telemetry,
            "ATTRIBUTE"           => Resource::Attribute,
            _ => return None,
        })
    }
}

// ── Operation enum ────────────────────────────────────────────────────────────

/// Các thao tác được phép trên tài nguyên — khớp Java `Operation.java`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operation {
    /// Cho phép tất cả thao tác trên tài nguyên này.
    All,
    Read,
    Write,
    Delete,
    /// Claim device ownership.
    Claim,
    /// Assign entity to customer/tenant.
    Assign,
    Unassign,
    ChangeOwner,
    ImpersonateToken,
    ReadCredentials,
    WriteCredentials,
    RpcCall,
    ReadAttributes,
    WriteAttributes,
    ReadTelemetry,
    WriteTelemetry,
}

impl Operation {
    pub fn as_str(self) -> &'static str {
        match self {
            Operation::All               => "ALL",
            Operation::Read              => "READ",
            Operation::Write             => "WRITE",
            Operation::Delete            => "DELETE",
            Operation::Claim             => "CLAIM",
            Operation::Assign            => "ASSIGN",
            Operation::Unassign          => "UNASSIGN",
            Operation::ChangeOwner       => "CHANGE_OWNER",
            Operation::ImpersonateToken  => "IMPERSONATE_TOKEN",
            Operation::ReadCredentials   => "READ_CREDENTIALS",
            Operation::WriteCredentials  => "WRITE_CREDENTIALS",
            Operation::RpcCall           => "RPC_CALL",
            Operation::ReadAttributes    => "READ_ATTRIBUTES",
            Operation::WriteAttributes   => "WRITE_ATTRIBUTES",
            Operation::ReadTelemetry     => "READ_TELEMETRY",
            Operation::WriteTelemetry    => "WRITE_TELEMETRY",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "ALL"                => Operation::All,
            "READ"               => Operation::Read,
            "WRITE"              => Operation::Write,
            "DELETE"             => Operation::Delete,
            "CLAIM"              => Operation::Claim,
            "ASSIGN"             => Operation::Assign,
            "UNASSIGN"           => Operation::Unassign,
            "CHANGE_OWNER"       => Operation::ChangeOwner,
            "IMPERSONATE_TOKEN"  => Operation::ImpersonateToken,
            "READ_CREDENTIALS"   => Operation::ReadCredentials,
            "WRITE_CREDENTIALS"  => Operation::WriteCredentials,
            "RPC_CALL"           => Operation::RpcCall,
            "READ_ATTRIBUTES"    => Operation::ReadAttributes,
            "WRITE_ATTRIBUTES"   => Operation::WriteAttributes,
            "READ_TELEMETRY"     => Operation::ReadTelemetry,
            "WRITE_TELEMETRY"    => Operation::WriteTelemetry,
            _ => return None,
        })
    }
}

/// Custom role — per-tenant, assignable to users
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TbRole {
    pub id:           Uuid,
    pub tenant_id:    Uuid,
    pub name:         String,
    pub role_type:    RoleType,
    pub permissions:  RolePermissions,
    pub created_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoleType {
    #[default]
    Generic,
    Group,
}

impl RoleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleType::Generic => "GENERIC",
            RoleType::Group   => "GROUP",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "GROUP" => RoleType::Group,
            _       => RoleType::Generic,
        }
    }
}

/// Permissions map: resource → allowed operations list
/// Ví dụ: { "DEVICE": ["READ", "WRITE"], "DASHBOARD": ["READ"] }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RolePermissions(pub HashMap<String, Vec<String>>);

impl RolePermissions {
    pub fn can(&self, resource: &str, operation: &str) -> bool {
        self.0
            .get(resource)
            .map(|ops| ops.iter().any(|op| op.eq_ignore_ascii_case(operation)))
            .unwrap_or(false)
    }

    /// Typed permission check — also matches Operation::All as wildcard.
    pub fn can_typed(&self, resource: Resource, op: Operation) -> bool {
        self.can(resource.as_str(), op.as_str())
            || self.can(resource.as_str(), Operation::All.as_str())
    }

    /// Build a minimal RolePermissions granting all operations on the given resources.
    pub fn with_all_ops(resources: &[Resource]) -> Self {
        let map = resources.iter()
            .map(|r| (r.as_str().to_string(), vec!["ALL".to_string()]))
            .collect();
        RolePermissions(map)
    }

    /// Merge another RolePermissions into this one (additive — never removes grants).
    pub fn merge(&mut self, other: &RolePermissions) {
        use std::collections::HashSet;
        for (resource, ops) in &other.0 {
            let entry = self.0.entry(resource.clone()).or_default();
            let existing: HashSet<String> = entry.iter().cloned().collect();
            for op in ops {
                if !existing.contains(op.as_str()) {
                    entry.push(op.clone());
                }
            }
        }
    }
}

/// Entity group — nhóm các entities (DEVICE, ASSET, DASHBOARD, ...) để áp dụng RBAC
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityGroup {
    pub id:              Uuid,
    pub tenant_id:       Uuid,
    pub customer_id:     Option<Uuid>,
    pub name:            String,
    pub entity_type:     String,
    pub additional_info: Option<serde_json::Value>,
    pub created_time:    i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Resource roundtrip ────────────────────────────────────────────────────

    #[test]
    fn test_resource_as_str_round_trip() {
        let resources = [
            Resource::Device, Resource::Asset, Resource::Dashboard,
            Resource::Alarm, Resource::User, Resource::Tenant,
            Resource::RuleChain, Resource::Telemetry, Resource::Attribute,
        ];
        for r in resources {
            let s = r.as_str();
            assert_eq!(Resource::from_str(s), Some(r), "round-trip failed for {s}");
        }
    }

    #[test]
    fn test_resource_from_str_unknown() {
        assert!(Resource::from_str("UNKNOWN_RESOURCE").is_none());
        assert!(Resource::from_str("").is_none());
    }

    #[test]
    fn test_resource_serde_round_trip() {
        let r = Resource::DeviceCredentials;
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, r#""DEVICE_CREDENTIALS""#);
        let back: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    // ── Operation roundtrip ───────────────────────────────────────────────────

    #[test]
    fn test_operation_as_str_round_trip() {
        let ops = [
            Operation::All, Operation::Read, Operation::Write,
            Operation::Delete, Operation::Claim, Operation::RpcCall,
            Operation::ReadCredentials, Operation::WriteCredentials,
            Operation::ReadTelemetry, Operation::WriteTelemetry,
        ];
        for op in ops {
            let s = op.as_str();
            assert_eq!(Operation::from_str(s), Some(op), "round-trip failed for {s}");
        }
    }

    #[test]
    fn test_operation_serde_round_trip() {
        let op = Operation::ChangeOwner;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, r#""CHANGE_OWNER""#);
        let back: Operation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, op);
    }

    // ── RolePermissions typed methods ─────────────────────────────────────────

    #[test]
    fn test_can_typed_direct_match() {
        let perms = RolePermissions(
            [("DEVICE".to_string(), vec!["READ".to_string(), "WRITE".to_string()])]
                .into_iter().collect()
        );
        assert!(perms.can_typed(Resource::Device, Operation::Read));
        assert!(perms.can_typed(Resource::Device, Operation::Write));
        assert!(!perms.can_typed(Resource::Device, Operation::Delete));
        assert!(!perms.can_typed(Resource::Asset, Operation::Read));
    }

    #[test]
    fn test_can_typed_all_wildcard() {
        let perms = RolePermissions(
            [("DEVICE".to_string(), vec!["ALL".to_string()])]
                .into_iter().collect()
        );
        assert!(perms.can_typed(Resource::Device, Operation::Read));
        assert!(perms.can_typed(Resource::Device, Operation::Delete));
        assert!(perms.can_typed(Resource::Device, Operation::RpcCall));
        // Other resources not granted
        assert!(!perms.can_typed(Resource::Asset, Operation::Read));
    }

    #[test]
    fn test_with_all_ops() {
        let perms = RolePermissions::with_all_ops(&[Resource::Device, Resource::Dashboard]);
        assert!(perms.can_typed(Resource::Device, Operation::Read));
        assert!(perms.can_typed(Resource::Dashboard, Operation::Write));
        assert!(!perms.can_typed(Resource::Asset, Operation::Read));
    }

    #[test]
    fn test_merge_permissions() {
        let mut base = RolePermissions(
            [("DEVICE".to_string(), vec!["READ".to_string()])].into_iter().collect()
        );
        let extra = RolePermissions(
            [
                ("DEVICE".to_string(), vec!["WRITE".to_string()]),
                ("ASSET".to_string(), vec!["READ".to_string()]),
            ].into_iter().collect()
        );
        base.merge(&extra);
        assert!(base.can_typed(Resource::Device, Operation::Read));
        assert!(base.can_typed(Resource::Device, Operation::Write));
        assert!(base.can_typed(Resource::Asset, Operation::Read));
    }
}
