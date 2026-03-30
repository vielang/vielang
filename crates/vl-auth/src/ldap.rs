/// LDAP / Active Directory authentication provider — P4
///
/// Flow:
///  1. Bind with service account to search for user DN
///  2. Re-bind with user's credentials to verify password
///  3. Extract attributes (email, first_name, last_name) from search entry
///
/// Compatible with: Active Directory (sAMAccountName), OpenLDAP (uid), Azure AD DS.

use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use tracing::{debug, warn};

use crate::AuthError;

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LdapConfig {
    /// ldap://host:389  or  ldaps://host:636
    pub server_url:       String,
    /// Enable STARTTLS (upgrade plain connection to TLS)
    pub use_tls:          bool,
    /// Search base — e.g. "DC=company,DC=com"
    pub base_dn:          String,
    /// User search filter — `{username}` is replaced at runtime.
    /// Example: "(sAMAccountName={username})"
    pub search_filter:    String,
    /// Service account DN for initial search bind
    pub bind_dn:          String,
    pub bind_password:    String,
    /// Attribute used as the username identifier (e.g. "sAMAccountName")
    pub username_attr:    String,
    pub first_name_attr:  String,
    pub last_name_attr:   String,
    pub email_attr:       String,
    /// Default ThingsBoard authority for auto-provisioned users
    pub default_authority: String,
    /// Group search base (can be same as base_dn). Used for periodic sync.
    pub group_search_base: Option<String>,
    /// LDAP filter to find all members of a group. Example: "(memberOf=CN=vielang-users,DC=company,DC=com)"
    pub group_filter: Option<String>,
}

// ── Result ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LdapUserInfo {
    pub email:      String,
    pub first_name: Option<String>,
    pub last_name:  Option<String>,
    pub username:   String,
    pub dn:         String,
}

// ── Provider ──────────────────────────────────────────────────────────────────

pub struct LdapAuthProvider {
    config: LdapConfig,
}

impl LdapAuthProvider {
    pub fn new(config: LdapConfig) -> Self {
        Self { config }
    }

    /// Authenticate `username` / `password` against the LDAP directory.
    ///
    /// Returns `LdapUserInfo` on success, `AuthError` on failure.
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LdapUserInfo, AuthError> {
        if password.is_empty() {
            // Anonymous bind (empty password) must be rejected explicitly —
            // some LDAP servers accept empty passwords as anonymous bind.
            return Err(AuthError::InvalidCredentials);
        }

        let settings = if self.config.use_tls {
            LdapConnSettings::new().set_starttls(true)
        } else {
            LdapConnSettings::new()
        };

        // ── Step 1: bind with service account ──────────────────────────────
        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &self.config.server_url)
            .await
            .map_err(|e| AuthError::LdapError(format!("Connect failed: {}", e)))?;

        // Drive the connection in the background
        ldap3::drive!(conn);

        ldap.simple_bind(&self.config.bind_dn, &self.config.bind_password)
            .await
            .map_err(|e| AuthError::LdapError(format!("Service bind failed: {}", e)))?
            .success()
            .map_err(|e| AuthError::LdapError(format!("Service bind error: {}", e)))?;

        // ── Step 2: search for user DN ──────────────────────────────────────
        let filter = self.config.search_filter.replace("{username}", username);
        let attrs = vec![
            "dn",
            &self.config.email_attr,
            &self.config.first_name_attr,
            &self.config.last_name_attr,
            &self.config.username_attr,
        ];

        let (entries, _res) = ldap
            .search(
                &self.config.base_dn,
                Scope::Subtree,
                &filter,
                attrs,
            )
            .await
            .map_err(|e| AuthError::LdapError(format!("Search failed: {}", e)))?
            .success()
            .map_err(|e| AuthError::LdapError(format!("Search error: {}", e)))?;

        let entry = entries.into_iter().next()
            .ok_or(AuthError::UserNotFound)?;
        let entry = SearchEntry::construct(entry);
        let user_dn = entry.dn.clone();

        debug!(user_dn = %user_dn, "LDAP user found");

        // ── Step 3: re-bind with user credentials ──────────────────────────
        ldap.simple_bind(&user_dn, password)
            .await
            .map_err(|e| AuthError::LdapError(format!("User bind failed: {}", e)))?
            .success()
            .map_err(|_| AuthError::InvalidCredentials)?;

        // ── Step 4: extract attributes ─────────────────────────────────────
        let get_attr = |attr: &str| -> Option<String> {
            entry.attrs.get(attr)?.first().cloned()
        };

        let email = get_attr(&self.config.email_attr)
            .ok_or_else(|| AuthError::LdapError("Missing email attribute in LDAP entry".into()))?;

        let first_name = get_attr(&self.config.first_name_attr);
        let last_name  = get_attr(&self.config.last_name_attr);
        let uname      = get_attr(&self.config.username_attr).unwrap_or_else(|| username.to_string());

        ldap.unbind().await
            .map_err(|e| warn!("LDAP unbind error (non-fatal): {}", e))
            .ok();

        Ok(LdapUserInfo { email, first_name, last_name, username: uname, dn: user_dn })
    }

    /// Search for all users matching the group filter. Used for periodic sync.
    pub async fn search_group_members(&self) -> Result<Vec<LdapUserInfo>, AuthError> {
        let base = self.config.group_search_base.as_deref()
            .unwrap_or(&self.config.base_dn);
        let filter = self.config.group_filter.as_deref()
            .unwrap_or("(objectClass=person)");

        let settings = if self.config.use_tls {
            LdapConnSettings::new().set_starttls(true)
        } else {
            LdapConnSettings::new()
        };
        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &self.config.server_url)
            .await
            .map_err(|e| AuthError::LdapError(e.to_string()))?;
        ldap3::drive!(conn);

        ldap.simple_bind(&self.config.bind_dn, &self.config.bind_password)
            .await
            .map_err(|e| AuthError::LdapError(e.to_string()))?
            .success()
            .map_err(|e| AuthError::LdapError(e.to_string()))?;

        let attrs = vec![
            self.config.email_attr.as_str(),
            self.config.first_name_attr.as_str(),
            self.config.last_name_attr.as_str(),
            self.config.username_attr.as_str(),
        ];
        let (entries, _res) = ldap
            .search(base, Scope::Subtree, filter, attrs)
            .await
            .map_err(|e| AuthError::LdapError(e.to_string()))?
            .success()
            .map_err(|e| AuthError::LdapError(e.to_string()))?;

        let mut users = Vec::new();
        for entry in entries {
            let se = SearchEntry::construct(entry);
            let email = se.attrs.get(&self.config.email_attr)
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_default();
            if email.is_empty() {
                continue;
            }
            users.push(LdapUserInfo {
                email,
                first_name: se.attrs.get(&self.config.first_name_attr)
                    .and_then(|v| v.first())
                    .map(String::from),
                last_name: se.attrs.get(&self.config.last_name_attr)
                    .and_then(|v| v.first())
                    .map(String::from),
                username: se.attrs.get(&self.config.username_attr)
                    .and_then(|v| v.first())
                    .map(String::from)
                    .unwrap_or_default(),
                dn: se.dn,
            });
        }
        ldap.unbind().await.ok();
        Ok(users)
    }

    /// Test the service-account bind only (no user search).
    /// Used by the admin API to validate LDAP configuration.
    pub async fn test_connection(&self) -> Result<(), AuthError> {
        let settings = if self.config.use_tls {
            LdapConnSettings::new().set_starttls(true)
        } else {
            LdapConnSettings::new()
        };

        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &self.config.server_url)
            .await
            .map_err(|e| AuthError::LdapError(format!("Connect failed: {}", e)))?;

        ldap3::drive!(conn);

        ldap.simple_bind(&self.config.bind_dn, &self.config.bind_password)
            .await
            .map_err(|e| AuthError::LdapError(format!("Bind failed: {}", e)))?
            .success()
            .map_err(|e| AuthError::LdapError(format!("Bind error: {}", e)))?;

        ldap.unbind().await.ok();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> LdapConfig {
        LdapConfig {
            server_url:        "ldap://localhost:389".into(),
            use_tls:           false,
            base_dn:           "DC=test,DC=local".into(),
            search_filter:     "(sAMAccountName={username})".into(),
            bind_dn:           "CN=svc,DC=test,DC=local".into(),
            bind_password:     "secret".into(),
            username_attr:     "sAMAccountName".into(),
            first_name_attr:   "givenName".into(),
            last_name_attr:    "sn".into(),
            email_attr:        "mail".into(),
            default_authority: "TENANT_ADMIN".into(),
            group_search_base: None,
            group_filter:      None,
        }
    }

    #[tokio::test]
    async fn empty_password_rejected() {
        let provider = LdapAuthProvider::new(sample_config());
        let result = provider.authenticate("user", "").await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }
}
