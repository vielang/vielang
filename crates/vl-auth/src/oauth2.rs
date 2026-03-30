/// Generic OIDC client — P4: OAuth2 Complete
///
/// Supports:
///  - Discovery via .well-known/openid-configuration
///  - PKCE (S256 code_challenge)
///  - Authorization URL generation
///  - Code exchange (access + id_token)
///  - UserInfo endpoint fetch
///
/// Compatible with: Google, GitHub, Azure AD, AWS Cognito, any generic OIDC IdP.

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use rand::Rng;

use crate::AuthError;

fn percent_encode(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
                vec![c]
            } else {
                c.to_string()
                    .as_bytes()
                    .iter()
                    .flat_map(|b| format!("%{:02X}", b).chars().collect::<Vec<_>>())
                    .collect()
            }
        })
        .collect()
}

// ── Discovery document ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct OidcDiscovery {
    pub issuer:                 String,
    pub authorization_endpoint: String,
    pub token_endpoint:         String,
    pub userinfo_endpoint:      Option<String>,
    pub jwks_uri:               String,
    #[serde(default)]
    pub scopes_supported:       Vec<String>,
    #[serde(default)]
    pub response_types_supported: Vec<String>,
    #[serde(default)]
    pub code_challenge_methods_supported: Vec<String>,
}

// ── Token response ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token:  String,
    pub token_type:    String,
    pub expires_in:    Option<u64>,
    pub refresh_token: Option<String>,
    pub id_token:      Option<String>,
    pub scope:         Option<String>,
}

// ── PKCE ──────────────────────────────────────────────────────────────────────

/// A PKCE verifier+challenge pair (S256 method).
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    /// The code_verifier to store in session and send at token exchange.
    pub code_verifier:  String,
    /// The code_challenge to send in the authorization URL.
    pub code_challenge: String,
}

impl PkceChallenge {
    /// Generate a new random PKCE pair.
    pub fn new() -> Self {
        // 43-128 chars of URL-safe base64
        let verifier_bytes: Vec<u8> = (0..32).map(|_| rand::rng().random::<u8>()).collect();
        let code_verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(&verifier_bytes);

        let hash = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

        Self { code_verifier, code_challenge }
    }
}

impl Default for PkceChallenge {
    fn default() -> Self {
        Self::new()
    }
}

// ── UserInfo ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OidcUserInfo {
    pub email:      String,
    pub first_name: Option<String>,
    pub last_name:  Option<String>,
    pub sub:        String,
}

// ── OidcClient ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OidcClient {
    pub client_id:            String,
    pub client_secret:        String,
    pub redirect_uri:         String,
    pub scopes:               Vec<String>,
    pub email_attribute:      String,
    pub first_name_attribute: String,
    pub last_name_attribute:  String,
    discovery:                OidcDiscovery,
    http:                     reqwest::Client,
}

impl OidcClient {
    /// Fetch OIDC discovery document and construct client.
    pub async fn discover(
        discovery_url:        &str,
        client_id:            String,
        client_secret:        String,
        redirect_uri:         String,
        scopes:               Vec<String>,
        email_attribute:      String,
        first_name_attribute: String,
        last_name_attribute:  String,
    ) -> Result<Self, AuthError> {
        let http = reqwest::Client::new();
        let discovery = http
            .get(discovery_url)
            .send()
            .await
            .map_err(|e| AuthError::OidcError(format!("Discovery fetch failed: {}", e)))?
            .json::<OidcDiscovery>()
            .await
            .map_err(|e| AuthError::OidcError(format!("Discovery parse failed: {}", e)))?;

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            scopes,
            email_attribute,
            first_name_attribute,
            last_name_attribute,
            discovery,
            http,
        })
    }

    /// Build authorization URL and a PKCE pair.
    /// Store the returned `PkceChallenge.code_verifier` and `state` in the session.
    pub fn authorization_url(&self, state: &str) -> (String, PkceChallenge) {
        let pkce = PkceChallenge::new();
        let scope = self.scopes.join(" ");

        let url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            self.discovery.authorization_endpoint,
            percent_encode(&self.client_id),
            percent_encode(&self.redirect_uri),
            percent_encode(&scope),
            percent_encode(state),
            percent_encode(&pkce.code_challenge),
        );

        (url, pkce)
    }

    /// Exchange authorization code for tokens (PKCE-protected).
    pub async fn exchange_code(
        &self,
        code:          &str,
        code_verifier: &str,
    ) -> Result<TokenResponse, AuthError> {
        let resp = self.http
            .post(&self.discovery.token_endpoint)
            .form(&[
                ("grant_type",     "authorization_code"),
                ("code",           code),
                ("redirect_uri",   &self.redirect_uri),
                ("client_id",      &self.client_id),
                ("client_secret",  &self.client_secret),
                ("code_verifier",  code_verifier),
            ])
            .send()
            .await
            .map_err(|e| AuthError::OidcError(format!("Token exchange failed: {}", e)))?;

        let status = resp.status();
        let body = resp.json::<serde_json::Value>().await
            .map_err(|e| AuthError::OidcError(format!("Token parse failed: {}", e)))?;

        if !status.is_success() {
            let err = body["error_description"].as_str()
                .or_else(|| body["error"].as_str())
                .unwrap_or("unknown error");
            return Err(AuthError::OidcError(format!("Token endpoint error: {}", err)));
        }

        serde_json::from_value(body)
            .map_err(|e| AuthError::OidcError(format!("Token deserialize failed: {}", e)))
    }

    /// Fetch user info using the access token.
    pub async fn userinfo(&self, access_token: &str) -> Result<OidcUserInfo, AuthError> {
        let userinfo_url = self.discovery.userinfo_endpoint
            .as_deref()
            .ok_or_else(|| AuthError::OidcError("No userinfo_endpoint in discovery".into()))?;

        let resp = self.http
            .get(userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::OidcError(format!("UserInfo fetch failed: {}", e)))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AuthError::OidcError(format!("UserInfo parse failed: {}", e)))?;

        let email = resp[&self.email_attribute]
            .as_str()
            .ok_or_else(|| AuthError::OidcError(format!(
                "Could not find email attribute '{}' in userinfo", self.email_attribute
            )))?
            .to_string();

        let sub = resp["sub"].as_str().unwrap_or("").to_string();
        let first_name = resp[&self.first_name_attribute].as_str().map(String::from);
        let last_name  = resp[&self.last_name_attribute].as_str().map(String::from);

        Ok(OidcUserInfo { email, first_name, last_name, sub })
    }
}

// ── Well-known helper URLs ────────────────────────────────────────────────────

/// Google OIDC discovery URL
pub const GOOGLE_DISCOVERY_URL: &str =
    "https://accounts.google.com/.well-known/openid-configuration";

/// Azure AD discovery URL (requires tenant_id substitution)
pub fn azure_discovery_url(tenant_id: &str) -> String {
    format!(
        "https://login.microsoftonline.com/{}/v2.0/.well-known/openid-configuration",
        tenant_id
    )
}

/// AWS Cognito discovery URL (requires region + pool_id substitution)
pub fn cognito_discovery_url(region: &str, user_pool_id: &str) -> String {
    format!(
        "https://cognito-idp.{}.amazonaws.com/{}/.well-known/openid-configuration",
        region, user_pool_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_and_challenge_differ() {
        let pkce = PkceChallenge::new();
        assert_ne!(pkce.code_verifier, pkce.code_challenge);
        assert!(!pkce.code_verifier.is_empty());
        assert!(!pkce.code_challenge.is_empty());
    }

    #[test]
    fn pkce_challenge_is_sha256_of_verifier() {
        let pkce = PkceChallenge::new();
        let hash = Sha256::digest(pkce.code_verifier.as_bytes());
        let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);
        assert_eq!(pkce.code_challenge, expected);
    }
}
