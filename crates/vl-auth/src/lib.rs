pub mod ldap;
pub mod oauth2;
pub mod password;
pub mod permission;
pub mod saml;
pub mod totp;

pub use totp::TotpService;

#[cfg(test)]
mod tests {
    use super::*;

    /// Secret đủ dài cho HS512
    const SECRET: &str = "vielang-test-secret-key-must-be-long-enough-for-hs512";

    fn svc() -> JwtService {
        JwtService::new(SECRET, 9000, 604_800)
    }

    // ── issue_token + validate_token ─────────────────────────────────────────

    #[test]
    fn issue_and_validate_access_token() {
        let svc = svc();
        let user_id  = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let pair = svc
            .issue_token(user_id, Some(tenant_id), None, "TENANT_ADMIN", vec!["TENANT_ADMIN".into()])
            .unwrap();

        let claims = svc.validate_token(&pair.token).unwrap();
        assert_eq!(claims.user_id(), user_id);
        assert_eq!(claims.tenant_uuid(), Some(tenant_id));
        assert_eq!(claims.authority, "TENANT_ADMIN");
        assert!(!claims.is_refresh_token());
    }

    #[test]
    fn refresh_token_has_refresh_authority() {
        let svc = svc();
        let pair = svc
            .issue_token(Uuid::new_v4(), Some(Uuid::new_v4()), None, "TENANT_ADMIN", vec![])
            .unwrap();

        let claims = svc.validate_token(&pair.refresh_token).unwrap();
        assert_eq!(claims.authority, "REFRESH_TOKEN");
        assert!(claims.is_refresh_token());
    }

    #[test]
    fn access_and_refresh_tokens_share_user_id() {
        let svc = svc();
        let user_id = Uuid::new_v4();
        let pair = svc
            .issue_token(user_id, None, None, "SYS_ADMIN", vec![])
            .unwrap();

        let ac = svc.validate_token(&pair.token).unwrap();
        let rc = svc.validate_token(&pair.refresh_token).unwrap();
        assert_eq!(ac.user_id(), rc.user_id());
        assert_eq!(ac.user_id(), user_id);
    }

    #[test]
    fn sys_admin_token_has_no_tenant() {
        let svc = svc();
        let pair = svc
            .issue_token(Uuid::new_v4(), None, None, "SYS_ADMIN", vec![])
            .unwrap();

        let claims = svc.validate_token(&pair.token).unwrap();
        assert!(claims.tenant_uuid().is_none());
        assert!(claims.customer_uuid().is_none());
    }

    #[test]
    fn customer_user_token_has_customer_and_tenant() {
        let svc = svc();
        let tenant_id   = Uuid::new_v4();
        let customer_id = Uuid::new_v4();

        let pair = svc
            .issue_token(
                Uuid::new_v4(),
                Some(tenant_id),
                Some(customer_id),
                "CUSTOMER_USER",
                vec![],
            )
            .unwrap();

        let claims = svc.validate_token(&pair.token).unwrap();
        assert_eq!(claims.tenant_uuid(), Some(tenant_id));
        assert_eq!(claims.customer_uuid(), Some(customer_id));
        assert_eq!(claims.authority, "CUSTOMER_USER");
    }

    #[test]
    fn scopes_preserved_in_claims() {
        let svc = svc();
        let scopes = vec!["TENANT_ADMIN".into(), "READ_DEVICES".into()];
        let pair = svc
            .issue_token(Uuid::new_v4(), None, None, "TENANT_ADMIN", scopes.clone())
            .unwrap();

        let claims = svc.validate_token(&pair.token).unwrap();
        assert_eq!(claims.scopes, scopes);
    }

    // ── Validation failures ───────────────────────────────────────────────────

    #[test]
    fn malformed_token_rejected() {
        let svc = svc();
        assert!(matches!(
            svc.validate_token("not.a.valid.token"),
            Err(AuthError::InvalidToken(_))
        ));
    }

    #[test]
    fn empty_string_rejected() {
        let svc = svc();
        assert!(svc.validate_token("").is_err());
    }

    #[test]
    fn tampered_signature_rejected() {
        let svc = svc();
        let pair = svc
            .issue_token(Uuid::new_v4(), None, None, "SYS_ADMIN", vec![])
            .unwrap();

        // Corrupt signature (3rd part of JWT)
        let parts: Vec<&str> = pair.token.splitn(3, '.').collect();
        let tampered = format!("{}.{}.INVALIDSIGNATURE", parts[0], parts[1]);
        assert!(matches!(
            svc.validate_token(&tampered),
            Err(AuthError::InvalidToken(_))
        ));
    }

    #[test]
    fn wrong_secret_rejected() {
        let svc1 = JwtService::new("secret-alpha-must-be-long-enough-chars!!", 9000, 604_800);
        let svc2 = JwtService::new("secret-beta--must-be-long-enough-chars!!", 9000, 604_800);

        let pair = svc1
            .issue_token(Uuid::new_v4(), None, None, "SYS_ADMIN", vec![])
            .unwrap();
        assert!(svc2.validate_token(&pair.token).is_err());
    }

    #[test]
    fn expired_token_returns_token_expired_error() {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

        let secret = SECRET;
        let svc = JwtService::new(secret, 9000, 604_800);

        let past = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - 7200; // 2 hours ago

        let uid = Uuid::new_v4().to_string();
        let expired_claims = Claims {
            sub: uid.clone(),
            jti: Uuid::new_v4().to_string(),
            user_id_str: uid,
            iss: "vielang".to_string(),
            iat: past - 3600,
            exp: past, // already expired
            tenant_id: None,
            customer_id: None,
            authority: "SYS_ADMIN".to_string(),
            scopes: vec![],
        };

        let token = encode(
            &Header::new(Algorithm::HS512),
            &expired_claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        assert!(matches!(svc.validate_token(&token), Err(AuthError::TokenExpired)));
    }

    // ── Claims helpers ────────────────────────────────────────────────────────

    #[test]
    fn is_refresh_token_true_only_for_refresh_authority() {
        let uid = Uuid::new_v4().to_string();
        let claims = Claims {
            sub: uid.clone(),
            jti: Uuid::new_v4().to_string(),
            user_id_str: uid,
            iss: "vielang".into(),
            iat: 0,
            exp: 9999999999,
            tenant_id: None,
            customer_id: None,
            authority: "REFRESH_TOKEN".into(),
            scopes: vec![],
        };
        assert!(claims.is_refresh_token());

        let mut other = claims.clone();
        other.authority = "TENANT_ADMIN".into();
        assert!(!other.is_refresh_token());
    }
}

use std::collections::HashMap;
use jsonwebtoken::{decode, decode_header, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Password error: {0}")]
    PasswordError(String),
    #[error("LDAP error: {0}")]
    LdapError(String),
    #[error("SAML error: {0}")]
    SamlError(String),
    #[error("OIDC error: {0}")]
    OidcError(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
}

/// JWT Claims — khớp ThingsBoard JwtUserDetails
/// Serde renames match Java TB claims so Angular UI can decode them correctly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject = user_id (UUID string) — standard JWT claim
    pub sub: String,
    /// Unique token ID — used for token revocation blacklist
    pub jti: String,
    /// Explicit userId claim — Angular reads authUser.userId (not sub)
    #[serde(rename = "userId")]
    pub user_id_str: String,
    /// Issuer
    pub iss: String,
    /// Issued at (epoch seconds)
    pub iat: i64,
    /// Expiration (epoch seconds)
    pub exp: i64,
    /// camelCase to match ThingsBoard Java JWT claims
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<String>,
    #[serde(rename = "customerId")]
    pub customer_id: Option<String>,
    /// "SYS_ADMIN", "TENANT_ADMIN", "CUSTOMER_USER", "REFRESH_TOKEN"
    pub authority: String,
    pub scopes: Vec<String>,
}

impl Claims {
    pub fn user_id(&self) -> Uuid {
        Uuid::parse_str(&self.sub).unwrap_or_default()
    }

    pub fn tenant_uuid(&self) -> Option<Uuid> {
        self.tenant_id.as_deref().and_then(|s| Uuid::parse_str(s).ok())
    }

    pub fn customer_uuid(&self) -> Option<Uuid> {
        self.customer_id.as_deref().and_then(|s| Uuid::parse_str(s).ok())
    }

    pub fn is_refresh_token(&self) -> bool {
        self.authority == "REFRESH_TOKEN"
    }

    pub fn is_pre_verification_token(&self) -> bool {
        self.authority == "PRE_VERIFICATION_TOKEN"
    }
}

pub struct JwtPair {
    pub token: String,
    pub refresh_token: String,
}

pub struct JwtService {
    /// Maps key_id → (encoding_key, decoding_key). Current key_id = 1.
    keys: HashMap<u8, (EncodingKey, DecodingKey)>,
    current_key_id: u8,
    expiration_secs: u64,
    refresh_expiration_secs: u64,
}

impl JwtService {
    /// Create with a single secret (no rotation window).
    pub fn new(secret: &str, expiration_secs: u64, refresh_expiration_secs: u64) -> Self {
        let mut keys = HashMap::new();
        keys.insert(1u8, (
            EncodingKey::from_secret(secret.as_bytes()),
            DecodingKey::from_secret(secret.as_bytes()),
        ));
        Self { keys, current_key_id: 1, expiration_secs, refresh_expiration_secs }
    }

    /// Create with optional previous signing key for zero-downtime secret rotation.
    /// Tokens signed with the previous key remain valid during the rotation window.
    pub fn new_with_rotation(
        secret: &str,
        previous_secret: Option<&str>,
        expiration_secs: u64,
        refresh_expiration_secs: u64,
    ) -> Self {
        let mut keys = HashMap::new();
        keys.insert(1u8, (
            EncodingKey::from_secret(secret.as_bytes()),
            DecodingKey::from_secret(secret.as_bytes()),
        ));
        if let Some(prev) = previous_secret {
            keys.insert(0u8, (
                EncodingKey::from_secret(prev.as_bytes()),
                DecodingKey::from_secret(prev.as_bytes()),
            ));
        }
        Self { keys, current_key_id: 1, expiration_secs, refresh_expiration_secs }
    }

    pub fn issue_token(
        &self,
        user_id: Uuid,
        tenant_id: Option<Uuid>,
        customer_id: Option<Uuid>,
        authority: &str,
        scopes: Vec<String>,
    ) -> Result<JwtPair, AuthError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let tenant_str = tenant_id.map(|id| id.to_string());
        let customer_str = customer_id.map(|id| id.to_string());

        let uid = user_id.to_string();
        let claims = Claims {
            sub: uid.clone(),
            jti: Uuid::new_v4().to_string(),
            user_id_str: uid.clone(),
            iss: "vielang".to_string(),
            iat: now,
            exp: now + self.expiration_secs as i64,
            tenant_id: tenant_str.clone(),
            customer_id: customer_str.clone(),
            authority: authority.to_string(),
            scopes,
        };

        let mut header = Header::new(Algorithm::HS512);
        header.kid = Some(self.current_key_id.to_string());
        let (enc_key, _) = self.keys.get(&self.current_key_id)
            .ok_or_else(|| AuthError::InvalidToken("No signing key configured".into()))?;

        let token = encode(&header, &claims, enc_key)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        // Refresh token: REFRESH_TOKEN authority + longer TTL + own jti
        let refresh_claims = Claims {
            sub: uid.clone(),
            jti: Uuid::new_v4().to_string(),
            user_id_str: uid,
            iss: "vielang".to_string(),
            iat: now,
            exp: now + self.refresh_expiration_secs as i64,
            tenant_id: tenant_str,
            customer_id: customer_str,
            authority: "REFRESH_TOKEN".to_string(),
            scopes: vec!["REFRESH_TOKEN".to_string()],
        };

        let mut refresh_header = Header::new(Algorithm::HS512);
        refresh_header.kid = Some(self.current_key_id.to_string());
        let refresh_token = encode(&refresh_header, &refresh_claims, enc_key)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(JwtPair { token, refresh_token })
    }

    /// Issue a short-lived pre-verification token for 2FA flow (TTL: 5 min).
    pub fn issue_pre_verification_token(
        &self,
        user_id: Uuid,
        tenant_id: Option<Uuid>,
    ) -> Result<String, AuthError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let uid = user_id.to_string();
        let claims = Claims {
            sub:          uid.clone(),
            jti:          Uuid::new_v4().to_string(),
            user_id_str:  uid,
            iss:          "vielang".to_string(),
            iat:          now,
            exp:          now + 300, // 5 minutes
            tenant_id:    tenant_id.map(|id| id.to_string()),
            customer_id:  None,
            authority:    "PRE_VERIFICATION_TOKEN".to_string(),
            scopes:       vec!["PRE_VERIFICATION_TOKEN".to_string()],
        };

        let mut header = Header::new(Algorithm::HS512);
        header.kid = Some(self.current_key_id.to_string());
        let (enc_key, _) = self.keys.get(&self.current_key_id)
            .ok_or_else(|| AuthError::InvalidToken("No signing key configured".into()))?;
        encode(&header, &claims, enc_key)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        // Extract kid from header to select the right decoding key
        let kid: u8 = decode_header(token)
            .ok()
            .and_then(|h| h.kid)
            .and_then(|k| k.parse().ok())
            .unwrap_or(self.current_key_id);

        let (_, dec_key) = self.keys.get(&kid)
            .ok_or_else(|| AuthError::InvalidToken(format!("Unknown key id: {}", kid)))?;

        let mut validation = Validation::new(Algorithm::HS512);
        validation.set_issuer(&["vielang"]);

        decode::<Claims>(token, dec_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                use jsonwebtoken::errors::ErrorKind;
                match e.kind() {
                    ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::InvalidToken(e.to_string()),
                }
            })
    }
}
