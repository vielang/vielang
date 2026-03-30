//! Phase 31 — User session and role-based access control.

use bevy::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

// ── Role ─────────────────────────────────────────────────────────────────────

/// Application role — controls what UI actions are permitted.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AppRole {
    Admin,    // Full access: edit layout, import models, manage alarms
    Operator, // Can view + ack/clear alarms; cannot edit layout
    #[default]
    Viewer,   // Read-only: no alarm actions, no layout changes
}

impl AppRole {
    /// Map a ThingsBoard JWT authority scope to an application role.
    pub fn from_authority(authority: &str) -> Self {
        match authority {
            "TENANT_ADMIN" | "SYS_ADMIN" => AppRole::Admin,
            "CUSTOMER_USER"               => AppRole::Operator,
            _                             => AppRole::Viewer,
        }
    }

    pub fn can_edit_layout(&self)        -> bool { matches!(self, AppRole::Admin) }
    pub fn can_ack_alarms(&self)         -> bool { !matches!(self, AppRole::Viewer) }
    pub fn can_import_models(&self)      -> bool { matches!(self, AppRole::Admin) }
    pub fn can_modify_thresholds(&self)  -> bool { !matches!(self, AppRole::Viewer) }
    pub fn can_edit_dashboard(&self)     -> bool { matches!(self, AppRole::Admin) }
}

impl std::fmt::Display for AppRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppRole::Admin    => write!(f, "Admin"),
            AppRole::Operator => write!(f, "Operator"),
            AppRole::Viewer   => write!(f, "Viewer"),
        }
    }
}

// ── Session resource ──────────────────────────────────────────────────────────

/// Current user session. `Default` = unauthenticated (Viewer).
#[derive(Resource, Clone)]
pub struct UserSession {
    pub token:       Option<String>,
    pub username:    String,
    pub tenant_id:   Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub role:        AppRole,
    pub expires_at:  Option<std::time::Instant>,
}

impl Default for UserSession {
    fn default() -> Self {
        Self {
            token:       None,
            username:    String::new(),
            tenant_id:   None,
            customer_id: None,
            role:        AppRole::Viewer,
            expires_at:  None,
        }
    }
}

impl UserSession {
    /// Returns true when a valid non-expired token is present.
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
            && self.expires_at
                .map(|e| e > std::time::Instant::now())
                .unwrap_or(false)
    }

    /// Convenience — returns the current role (defaults to Viewer).
    pub fn role(&self) -> &AppRole {
        &self.role
    }
}

// ── JWT parsing ───────────────────────────────────────────────────────────────

/// ThingsBoard `POST /api/auth/login` response.
#[derive(Deserialize)]
pub struct JwtResponse {
    pub token:         String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

/// Claims inside a ThingsBoard JWT payload.
#[derive(Deserialize)]
pub struct JwtClaims {
    pub sub:    String,
    pub scopes: Vec<String>,
    #[serde(rename = "tenantId")]
    pub tenant_id:   Option<String>,
    #[serde(rename = "customerId")]
    pub customer_id: Option<String>,
    pub exp:    u64,
}

/// Parse a JWT response into a `UserSession`.
pub fn parse_jwt_session(jwt: JwtResponse) -> Result<UserSession, String> {
    let parts: Vec<&str> = jwt.token.split('.').collect();
    if parts.len() != 3 { return Err("Invalid JWT format".into()); }

    let payload = base64_decode_unpadded(parts[1])?;
    let claims: JwtClaims = serde_json::from_str(&payload)
        .map_err(|_| "Failed to decode JWT claims".to_string())?;

    let authority = claims.scopes.first().map(|s| s.as_str()).unwrap_or("");
    let role      = AppRole::from_authority(authority);

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let ttl_secs = claims.exp.saturating_sub(now_secs);
    let expires  = std::time::Instant::now()
        + std::time::Duration::from_secs(ttl_secs);

    Ok(UserSession {
        token:       Some(jwt.token),
        username:    claims.sub,
        tenant_id:   claims.tenant_id.and_then(|s| Uuid::parse_str(&s).ok()),
        customer_id: claims.customer_id.and_then(|s| Uuid::parse_str(&s).ok()),
        role,
        expires_at:  Some(expires),
    })
}

fn base64_decode_unpadded(s: &str) -> Result<String, String> {
    // URL-safe base64 without padding (JWT standard)
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    let bytes = URL_SAFE_NO_PAD.decode(s).map_err(|e| format!("base64 error: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "UTF-8 decode error".into())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_role_from_authority() {
        assert_eq!(AppRole::from_authority("TENANT_ADMIN"), AppRole::Admin);
        assert_eq!(AppRole::from_authority("SYS_ADMIN"),    AppRole::Admin);
        assert_eq!(AppRole::from_authority("CUSTOMER_USER"), AppRole::Operator);
        assert_eq!(AppRole::from_authority("unknown"),       AppRole::Viewer);
    }

    #[test]
    fn role_permissions() {
        assert!(AppRole::Admin.can_edit_layout());
        assert!(AppRole::Admin.can_ack_alarms());
        assert!(!AppRole::Viewer.can_edit_layout());
        assert!(!AppRole::Viewer.can_ack_alarms());
        assert!(AppRole::Operator.can_ack_alarms());
        assert!(!AppRole::Operator.can_edit_layout());
    }

    #[test]
    fn user_session_default_unauthenticated() {
        let s = UserSession::default();
        assert!(!s.is_authenticated());
        assert_eq!(*s.role(), AppRole::Viewer);
    }

    // ── Role permissions exhaustive ────────────────────────────────────────────

    #[test]
    fn admin_has_all_permissions() {
        let role = AppRole::Admin;
        assert!(role.can_edit_layout());
        assert!(role.can_ack_alarms());
        assert!(role.can_import_models());
        assert!(role.can_modify_thresholds());
        assert!(role.can_edit_dashboard());
    }

    #[test]
    fn operator_can_ack_but_not_edit_layout() {
        let role = AppRole::Operator;
        assert!(role.can_ack_alarms(),         "operator can ack alarms");
        assert!(role.can_modify_thresholds(),  "operator can modify thresholds");
        assert!(!role.can_edit_layout(),       "operator cannot edit layout");
        assert!(!role.can_import_models(),     "operator cannot import models");
        assert!(!role.can_edit_dashboard(),    "operator cannot edit dashboard");
    }

    #[test]
    fn viewer_has_no_write_permissions() {
        let role = AppRole::Viewer;
        assert!(!role.can_edit_layout());
        assert!(!role.can_ack_alarms());
        assert!(!role.can_import_models());
        assert!(!role.can_modify_thresholds());
        assert!(!role.can_edit_dashboard());
    }

    #[test]
    fn app_role_display() {
        assert_eq!(AppRole::Admin.to_string(),    "Admin");
        assert_eq!(AppRole::Operator.to_string(), "Operator");
        assert_eq!(AppRole::Viewer.to_string(),   "Viewer");
    }

    #[test]
    fn app_role_default_is_viewer() {
        assert_eq!(AppRole::default(), AppRole::Viewer);
    }

    // ── JWT parsing ───────────────────────────────────────────────────────────

    #[test]
    fn parse_jwt_session_invalid_format_returns_error() {
        let jwt = JwtResponse {
            token:         "not.a.valid.jwt.format.extra".into(),
            refresh_token: "".into(),
        };
        // 5-part string is also invalid (JWT must be exactly 3 parts)
        // but we test a 4-part one to cover the len check:
        let jwt4 = JwtResponse {
            token:         "a.b.c.d".into(),
            refresh_token: "".into(),
        };
        assert!(parse_jwt_session(jwt4).is_err());
    }

    #[test]
    fn parse_jwt_session_bad_payload_returns_error() {
        // Header.invalid_base64.Signature
        let jwt = JwtResponse {
            token:         "eyJhbGciOiJSUzUxMiIsInR5cCI6IkpXVCJ9.NOT_VALID_BASE64!!.sig".into(),
            refresh_token: "".into(),
        };
        assert!(parse_jwt_session(jwt).is_err());
    }
}
