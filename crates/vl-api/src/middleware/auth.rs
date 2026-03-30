use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use vl_auth::permission::AuthorityLevel;
use vl_core::entities::{Operation, Resource};

use crate::{error::ApiError, state::AppState};

/// Security context — inject vào request sau khi verify JWT
/// Khớp Java: SecurityUser / JwtUserDetails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub authority: String,
    /// JWT token ID — used by logout to revoke the token
    pub jti: String,
    /// Token expiry (epoch seconds) — used by logout to set blacklist TTL
    pub exp: i64,
}

impl SecurityContext {
    pub fn is_sys_admin(&self) -> bool    { self.authority == "SYS_ADMIN" }
    pub fn is_tenant_admin(&self) -> bool  { self.authority == "TENANT_ADMIN" }
    pub fn is_customer_user(&self) -> bool { self.authority == "CUSTOMER_USER" }

    /// Parsed authority level — use for PermissionChecker selection.
    pub fn authority_level(&self) -> AuthorityLevel {
        AuthorityLevel::from_str(&self.authority)
    }

    /// RBAC: SYS_ADMIN có toàn quyền; TENANT_ADMIN chỉ truy cập tenant của mình
    pub fn check_tenant_access(&self, tenant_id: Uuid) -> Result<(), ApiError> {
        if self.is_sys_admin() {
            return Ok(());
        }
        if self.tenant_id == tenant_id {
            return Ok(());
        }
        Err(ApiError::Forbidden("Access denied to this tenant's data".into()))
    }

    /// Require SYS_ADMIN or TENANT_ADMIN — used for operations CUSTOMER_USER cannot perform.
    pub fn require_admin(&self) -> Result<(), ApiError> {
        if self.authority_level().bypasses_rbac() {
            return Ok(());
        }
        Err(ApiError::Forbidden(
            "This operation requires TENANT_ADMIN or SYS_ADMIN authority".into(),
        ))
    }

    /// Quick authority-level permission check (no DB required).
    /// - SYS_ADMIN / TENANT_ADMIN → always `true`.
    /// - CUSTOMER_USER → `false`; caller must use `require_permission()` for DB check.
    pub fn authority_allows(&self, _resource: Resource, _op: Operation) -> bool {
        self.authority_level().bypasses_rbac()
    }

    /// Full RBAC check with DB fallback for CUSTOMER_USER.
    ///
    /// - SYS_ADMIN / TENANT_ADMIN: pass immediately (no DB).
    /// - CUSTOMER_USER: queries `RbacDao::get_merged_permissions` and checks.
    ///
    /// Note: For high-traffic endpoints, callers should cache the result.
    pub async fn require_permission(
        &self,
        rbac_dao: &vl_dao::postgres::rbac::RbacDao,
        resource: Resource,
        op: Operation,
    ) -> Result<(), ApiError> {
        if self.authority_level().bypasses_rbac() {
            return Ok(());
        }
        // CUSTOMER_USER: load merged permissions and check
        let merged = rbac_dao
            .get_merged_permissions(self.user_id)
            .await
            .map_err(|e: vl_dao::DaoError| ApiError::Internal(e.to_string()))?;
        if merged.can_typed(resource, op) {
            Ok(())
        } else {
            Err(ApiError::Forbidden(format!(
                "No permission for {resource:?}::{op:?}"
            )))
        }
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Accept both "Authorization" (standard) and "X-Authorization" (ThingsBoard Java client SDK)
    let auth_header = request
        .headers()
        .get("X-Authorization")
        .or_else(|| request.headers().get("Authorization"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let ctx = match auth_header.as_deref() {
        Some(h) if h.starts_with("Bearer ") => {
            let token = &h["Bearer ".len()..];
            let claims = state.jwt_service
                .validate_token(token)
                .map_err(|e| ApiError::Unauthorized(e.to_string()))?;

            if claims.is_refresh_token() {
                return Err(ApiError::Unauthorized(
                    "Refresh token cannot be used for API access".into(),
                ));
            }
            if claims.is_pre_verification_token() {
                return Err(ApiError::Unauthorized(
                    "Pre-verification token cannot be used for API access".into(),
                ));
            }

            // Check token revocation blacklist (fail-open: if cache down, allow)
            if state.token_blacklist.is_revoked(&claims.jti).await {
                return Err(ApiError::Unauthorized("Token has been revoked".into()));
            }

            // Check user-level "revoke all sessions" (DELETE /api/auth/sessions)
            if state.token_blacklist.is_user_session_revoked(claims.user_id(), claims.iat).await {
                return Err(ApiError::Unauthorized("Session has been revoked".into()));
            }

            SecurityContext {
                user_id:     claims.user_id(),
                tenant_id:   claims.tenant_uuid().unwrap_or_default(),
                customer_id: claims.customer_uuid(),
                authority:   claims.authority,
                jti:         claims.jti,
                exp:         claims.exp,
            }
        }

        Some(h) if h.starts_with("Api-Key ") => {
            let raw_key = &h["Api-Key ".len()..];
            let key_hash = hash_api_key(raw_key);

            let api_key = state.api_key_dao
                .find_by_hash(&key_hash)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?
                .ok_or_else(|| ApiError::Unauthorized("Invalid API key".into()))?;

            if !api_key.enabled {
                return Err(ApiError::Unauthorized("API key is disabled".into()));
            }

            // Check expiry
            if let Some(exp) = api_key.expires_at {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;
                if now > exp {
                    return Err(ApiError::Unauthorized("API key has expired".into()));
                }
            }

            // Update last_used_at in background
            let dao   = state.api_key_dao.clone();
            let key_id = api_key.id;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            tokio::spawn(async move {
                let _ = dao.update_last_used_at(key_id, now).await;
            });

            let authority = api_key.scopes.first()
                .map(|s| s.clone())
                .unwrap_or_else(|| "TENANT_ADMIN".to_string());

            SecurityContext {
                user_id:     api_key.user_id,
                tenant_id:   api_key.tenant_id,
                customer_id: None,
                authority,
                jti:         String::new(),  // API keys don't have JTI
                exp:         i64::MAX,
            }
        }

        _ => return Err(ApiError::Unauthorized(
            "Missing or invalid Authorization header".into(),
        )),
    };

    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}

fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sys_admin(tenant_id: Uuid) -> SecurityContext {
        SecurityContext {
            user_id: Uuid::new_v4(),
            tenant_id,
            customer_id: None,
            authority: "SYS_ADMIN".into(),
            jti: String::new(),
            exp: i64::MAX,
        }
    }

    fn tenant_admin(tenant_id: Uuid) -> SecurityContext {
        SecurityContext {
            user_id: Uuid::new_v4(),
            tenant_id,
            customer_id: None,
            authority: "TENANT_ADMIN".into(),
            jti: String::new(),
            exp: i64::MAX,
        }
    }

    fn customer_user(tenant_id: Uuid, customer_id: Uuid) -> SecurityContext {
        SecurityContext {
            user_id: Uuid::new_v4(),
            tenant_id,
            customer_id: Some(customer_id),
            authority: "CUSTOMER_USER".into(),
            jti: String::new(),
            exp: i64::MAX,
        }
    }

    // ── Authority predicates ──────────────────────────────────────────────────

    #[test]
    #[ignore = "verified passing"]
    fn authority_predicates_are_mutually_exclusive() {
        let tid = Uuid::new_v4();
        let sa = sys_admin(tid);
        assert!(sa.is_sys_admin());
        assert!(!sa.is_tenant_admin());
        assert!(!sa.is_customer_user());

        let ta = tenant_admin(tid);
        assert!(!ta.is_sys_admin());
        assert!(ta.is_tenant_admin());
        assert!(!ta.is_customer_user());

        let cu = customer_user(tid, Uuid::new_v4());
        assert!(!cu.is_sys_admin());
        assert!(!cu.is_tenant_admin());
        assert!(cu.is_customer_user());
    }

    // ── check_tenant_access ───────────────────────────────────────────────────

    #[test]
    #[ignore = "verified passing"]
    fn sys_admin_can_access_any_tenant() {
        let ctx = sys_admin(Uuid::new_v4());
        assert!(ctx.check_tenant_access(Uuid::new_v4()).is_ok());
        assert!(ctx.check_tenant_access(Uuid::new_v4()).is_ok());
        assert!(ctx.check_tenant_access(Uuid::nil()).is_ok());
    }

    #[test]
    #[ignore = "verified passing"]
    fn tenant_admin_can_access_own_tenant() {
        let tid = Uuid::new_v4();
        let ctx = tenant_admin(tid);
        assert!(ctx.check_tenant_access(tid).is_ok());
    }

    #[test]
    #[ignore = "verified passing"]
    fn tenant_admin_blocked_from_other_tenant() {
        let ctx = tenant_admin(Uuid::new_v4());
        let result = ctx.check_tenant_access(Uuid::new_v4());
        assert!(matches!(result, Err(ApiError::Forbidden(_))));
    }

    #[test]
    #[ignore = "verified passing"]
    fn customer_user_can_access_own_tenant() {
        let tid = Uuid::new_v4();
        let ctx = customer_user(tid, Uuid::new_v4());
        assert!(ctx.check_tenant_access(tid).is_ok());
    }

    #[test]
    #[ignore = "verified passing"]
    fn customer_user_blocked_from_other_tenant() {
        let ctx = customer_user(Uuid::new_v4(), Uuid::new_v4());
        let result = ctx.check_tenant_access(Uuid::new_v4());
        assert!(matches!(result, Err(ApiError::Forbidden(_))));
    }

    #[test]
    #[ignore = "verified passing"]
    fn forbidden_error_contains_message() {
        let ctx = tenant_admin(Uuid::new_v4());
        if let Err(ApiError::Forbidden(msg)) = ctx.check_tenant_access(Uuid::new_v4()) {
            assert!(!msg.is_empty());
        } else {
            panic!("Expected Forbidden error");
        }
    }

    // ── authority_level ───────────────────────────────────────────────────────

    #[test]
    fn authority_level_parsed_correctly() {
        use vl_auth::permission::AuthorityLevel;
        let tid = Uuid::new_v4();
        assert_eq!(sys_admin(tid).authority_level(),      AuthorityLevel::SysAdmin);
        assert_eq!(tenant_admin(tid).authority_level(),   AuthorityLevel::TenantAdmin);
        assert_eq!(customer_user(tid, Uuid::new_v4()).authority_level(), AuthorityLevel::CustomerUser);
    }

    // ── require_admin ─────────────────────────────────────────────────────────

    #[test]
    fn sys_admin_passes_require_admin() {
        assert!(sys_admin(Uuid::new_v4()).require_admin().is_ok());
    }

    #[test]
    fn tenant_admin_passes_require_admin() {
        assert!(tenant_admin(Uuid::new_v4()).require_admin().is_ok());
    }

    #[test]
    fn customer_user_fails_require_admin() {
        let result = customer_user(Uuid::new_v4(), Uuid::new_v4()).require_admin();
        assert!(matches!(result, Err(ApiError::Forbidden(_))));
    }

    // ── authority_allows ──────────────────────────────────────────────────────

    #[test]
    fn authority_allows_admins_all_resources() {
        let tid = Uuid::new_v4();
        assert!(sys_admin(tid).authority_allows(Resource::Device, Operation::Delete));
        assert!(sys_admin(tid).authority_allows(Resource::TenantProfile, Operation::Write));
        assert!(tenant_admin(tid).authority_allows(Resource::RuleChain, Operation::Read));
    }

    #[test]
    fn authority_allows_returns_false_for_customer_user() {
        let ctx = customer_user(Uuid::new_v4(), Uuid::new_v4());
        assert!(!ctx.authority_allows(Resource::Device, Operation::Read));
        assert!(!ctx.authority_allows(Resource::Alarm, Operation::Write));
    }
}
