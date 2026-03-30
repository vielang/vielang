/// RBAC PermissionChecker — authority-level and role-based permission evaluation.
///
/// This module provides the `PermissionChecker` trait and implementations for each
/// authority level. The actual DB-backed check for `CUSTOMER_USER` is performed in
/// `vl-api` (which has access to `RbacDao`).
///
/// Flow:
/// - SYS_ADMIN → `AllowAllChecker` — always grants.
/// - TENANT_ADMIN → `TenantAdminChecker` — grants all ops on all resources.
/// - CUSTOMER_USER → `RolePermissionsChecker` — checks preloaded `RolePermissions`.
///   The caller (vl-api auth middleware or handler) must load roles from DB and
///   construct a `RolePermissionsChecker` before calling handlers.

use vl_core::entities::{Operation, Resource, RolePermissions};

// ── PermissionChecker trait ───────────────────────────────────────────────────

/// Synchronous permission check — used for in-memory evaluation after DB permissions
/// have been loaded, or for authority-level bypass (SYS_ADMIN / TENANT_ADMIN).
pub trait PermissionChecker: Send + Sync {
    fn can(&self, resource: Resource, op: Operation) -> bool;
}

// ── Authority level ───────────────────────────────────────────────────────────

/// Parsed authority level from JWT claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityLevel {
    SysAdmin,
    TenantAdmin,
    CustomerUser,
    /// System-level refresh token — must never reach handlers.
    RefreshToken,
}

impl AuthorityLevel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "SYS_ADMIN"     => AuthorityLevel::SysAdmin,
            "TENANT_ADMIN"  => AuthorityLevel::TenantAdmin,
            "REFRESH_TOKEN" => AuthorityLevel::RefreshToken,
            _               => AuthorityLevel::CustomerUser,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            AuthorityLevel::SysAdmin     => "SYS_ADMIN",
            AuthorityLevel::TenantAdmin  => "TENANT_ADMIN",
            AuthorityLevel::CustomerUser => "CUSTOMER_USER",
            AuthorityLevel::RefreshToken => "REFRESH_TOKEN",
        }
    }

    /// Returns `true` for SYS_ADMIN and TENANT_ADMIN — these authorities bypass
    /// resource-level RBAC (they are checked at tenant boundary only).
    pub fn bypasses_rbac(self) -> bool {
        matches!(self, AuthorityLevel::SysAdmin | AuthorityLevel::TenantAdmin)
    }

    /// Returns `true` only for SYS_ADMIN — used for cross-tenant operations.
    pub fn is_sys_admin(self) -> bool {
        self == AuthorityLevel::SysAdmin
    }
}

// ── Concrete checkers ─────────────────────────────────────────────────────────

/// Always grants every operation — for SYS_ADMIN.
pub struct AllowAllChecker;

impl PermissionChecker for AllowAllChecker {
    fn can(&self, _: Resource, _: Operation) -> bool { true }
}

/// Grants all operations — for TENANT_ADMIN (operates within own tenant).
pub struct TenantAdminChecker;

impl PermissionChecker for TenantAdminChecker {
    fn can(&self, _: Resource, _: Operation) -> bool { true }
}

/// Checks against a preloaded `RolePermissions` — for CUSTOMER_USER.
///
/// Caller must load roles from DB and pass them here.
pub struct RolePermissionsChecker {
    permissions: RolePermissions,
}

impl RolePermissionsChecker {
    pub fn new(permissions: RolePermissions) -> Self {
        Self { permissions }
    }
}

impl PermissionChecker for RolePermissionsChecker {
    fn can(&self, resource: Resource, op: Operation) -> bool {
        self.permissions.can_typed(resource, op)
    }
}

/// Deny-all checker — for when no roles are loaded or user has no roles.
pub struct DenyAllChecker;

impl PermissionChecker for DenyAllChecker {
    fn can(&self, _: Resource, _: Operation) -> bool { false }
}

/// Build the appropriate checker for the given authority level.
/// CUSTOMER_USER requires role permissions to be loaded separately.
pub fn build_authority_checker(authority: AuthorityLevel) -> Box<dyn PermissionChecker> {
    match authority {
        AuthorityLevel::SysAdmin     => Box::new(AllowAllChecker),
        AuthorityLevel::TenantAdmin  => Box::new(TenantAdminChecker),
        AuthorityLevel::CustomerUser => Box::new(DenyAllChecker),
        AuthorityLevel::RefreshToken => Box::new(DenyAllChecker),
    }
}

/// Build a CUSTOMER_USER checker with preloaded permissions.
pub fn build_customer_checker(permissions: RolePermissions) -> Box<dyn PermissionChecker> {
    Box::new(RolePermissionsChecker::new(permissions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_perms(resource: &str, ops: &[&str]) -> RolePermissions {
        let mut map = HashMap::new();
        map.insert(resource.to_string(), ops.iter().map(|s| s.to_string()).collect());
        RolePermissions(map)
    }

    // ── AuthorityLevel ────────────────────────────────────────────────────────

    #[test]
    fn test_authority_level_from_str() {
        assert_eq!(AuthorityLevel::from_str("SYS_ADMIN"),    AuthorityLevel::SysAdmin);
        assert_eq!(AuthorityLevel::from_str("TENANT_ADMIN"), AuthorityLevel::TenantAdmin);
        assert_eq!(AuthorityLevel::from_str("CUSTOMER_USER"), AuthorityLevel::CustomerUser);
        assert_eq!(AuthorityLevel::from_str("UNKNOWN"),      AuthorityLevel::CustomerUser);
        assert_eq!(AuthorityLevel::from_str("REFRESH_TOKEN"), AuthorityLevel::RefreshToken);
    }

    #[test]
    fn test_authority_level_bypasses_rbac() {
        assert!(AuthorityLevel::SysAdmin.bypasses_rbac());
        assert!(AuthorityLevel::TenantAdmin.bypasses_rbac());
        assert!(!AuthorityLevel::CustomerUser.bypasses_rbac());
        assert!(!AuthorityLevel::RefreshToken.bypasses_rbac());
    }

    #[test]
    fn test_authority_level_round_trip() {
        let levels = [
            AuthorityLevel::SysAdmin,
            AuthorityLevel::TenantAdmin,
            AuthorityLevel::CustomerUser,
            AuthorityLevel::RefreshToken,
        ];
        for l in levels {
            assert_eq!(AuthorityLevel::from_str(l.as_str()), l);
        }
    }

    // ── AllowAllChecker ───────────────────────────────────────────────────────

    #[test]
    fn test_allow_all_checker_grants_everything() {
        let checker = AllowAllChecker;
        assert!(checker.can(Resource::Device, Operation::Read));
        assert!(checker.can(Resource::Device, Operation::Delete));
        assert!(checker.can(Resource::TenantProfile, Operation::Write));
        assert!(checker.can(Resource::User, Operation::ImpersonateToken));
    }

    // ── TenantAdminChecker ────────────────────────────────────────────────────

    #[test]
    fn test_tenant_admin_checker_grants_everything() {
        let checker = TenantAdminChecker;
        assert!(checker.can(Resource::Alarm, Operation::All));
        assert!(checker.can(Resource::Device, Operation::RpcCall));
    }

    // ── DenyAllChecker ────────────────────────────────────────────────────────

    #[test]
    fn test_deny_all_checker_blocks_everything() {
        let checker = DenyAllChecker;
        assert!(!checker.can(Resource::Device, Operation::Read));
        assert!(!checker.can(Resource::Alarm, Operation::All));
    }

    // ── RolePermissionsChecker ────────────────────────────────────────────────

    #[test]
    fn test_role_permissions_checker_exact() {
        let perms = make_perms("DEVICE", &["READ", "WRITE"]);
        let checker = RolePermissionsChecker::new(perms);
        assert!(checker.can(Resource::Device, Operation::Read));
        assert!(checker.can(Resource::Device, Operation::Write));
        assert!(!checker.can(Resource::Device, Operation::Delete));
        assert!(!checker.can(Resource::Asset, Operation::Read));
    }

    #[test]
    fn test_role_permissions_checker_all_wildcard() {
        let perms = make_perms("ALARM", &["ALL"]);
        let checker = RolePermissionsChecker::new(perms);
        assert!(checker.can(Resource::Alarm, Operation::Read));
        assert!(checker.can(Resource::Alarm, Operation::Delete));
        assert!(!checker.can(Resource::Device, Operation::Read));
    }

    // ── build_authority_checker ───────────────────────────────────────────────

    #[test]
    fn test_build_sys_admin_checker_allows_all() {
        let checker = build_authority_checker(AuthorityLevel::SysAdmin);
        assert!(checker.can(Resource::Tenant, Operation::Delete));
    }

    #[test]
    fn test_build_customer_checker_uses_permissions() {
        let perms = make_perms("DASHBOARD", &["READ"]);
        let checker = build_customer_checker(perms);
        assert!(checker.can(Resource::Dashboard, Operation::Read));
        assert!(!checker.can(Resource::Dashboard, Operation::Write));
    }

    #[test]
    fn test_build_refresh_token_denies_all() {
        let checker = build_authority_checker(AuthorityLevel::RefreshToken);
        assert!(!checker.can(Resource::Device, Operation::Read));
    }
}
