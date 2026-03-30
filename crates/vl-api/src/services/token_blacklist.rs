use std::sync::Arc;
use std::time::Duration;

use vl_cache::TbCache;

/// JWT token revocation blacklist backed by the configured TbCache (in-memory or Redis).
///
/// When a user logs out or changes their password, the token's `jti` is stored with
/// TTL = remaining token lifetime.  Auth middleware checks this before allowing access.
///
/// Fail-open: if the cache is unavailable, tokens are NOT blocked (availability > strict revocation).
pub struct TokenBlacklist {
    cache: Arc<dyn TbCache>,
}

impl TokenBlacklist {
    const KEY_PREFIX: &'static str = "tb:revoked_tokens:";

    pub fn new(cache: Arc<dyn TbCache>) -> Self {
        Self { cache }
    }

    /// Revoke a token by jti.  TTL is set to the remaining token lifetime so the
    /// blacklist entry expires automatically when the token would have expired anyway.
    pub async fn revoke(&self, jti: &str, expires_at: i64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let remaining = (expires_at - now).max(0) as u64;
        if remaining == 0 {
            return; // Token already expired — nothing to blacklist
        }
        let key = format!("{}{}", Self::KEY_PREFIX, jti);
        if let Err(e) = self.cache.put_bytes(&key, b"1".to_vec(), Some(Duration::from_secs(remaining))).await {
            tracing::warn!("Token blacklist write failed (non-fatal): {}", e);
        }
    }

    const USER_REVOKE_PREFIX: &'static str = "tb:user_revoke_before:";

    /// Revoke ALL active tokens for a user by storing a "revoke before" timestamp.
    /// Any token with `iat` ≤ this timestamp will be rejected by auth middleware.
    ///
    /// TTL matches the JWT refresh token lifetime (default 7 days) so the entry
    /// expires when all issued tokens would have expired anyway.
    pub async fn revoke_all_for_user(&self, user_id: uuid::Uuid, refresh_token_ttl_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
        let key = format!("{}{}", Self::USER_REVOKE_PREFIX, user_id);
        let ttl = Duration::from_secs(refresh_token_ttl_secs);
        if let Err(e) = self.cache.put_bytes(&key, now.into_bytes(), Some(ttl)).await {
            tracing::warn!("User revoke-all write failed (non-fatal): {}", e);
        }
    }

    /// Returns `true` if the user's "revoke all before" timestamp is set and
    /// the token's `iat` falls at or before it — meaning this token was issued
    /// before the last "revoke all sessions" call.
    pub async fn is_user_session_revoked(&self, user_id: uuid::Uuid, token_iat: i64) -> bool {
        let key = format!("{}{}", Self::USER_REVOKE_PREFIX, user_id);
        match self.cache.get_bytes(&key).await {
            Ok(Some(bytes)) => {
                if let Ok(s) = std::str::from_utf8(&bytes) {
                    if let Ok(revoke_before) = s.parse::<i64>() {
                        return token_iat <= revoke_before;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Returns `true` if the token is blacklisted (revoked).
    /// On cache error: returns `false` (fail-open — availability over strict revocation).
    pub async fn is_revoked(&self, jti: &str) -> bool {
        let key = format!("{}{}", Self::KEY_PREFIX, jti);
        match self.cache.get_bytes(&key).await {
            Ok(Some(_)) => true,
            Ok(None)    => false,
            Err(e) => {
                tracing::warn!("Token blacklist check failed — allowing token: {}", e);
                false
            }
        }
    }
}
