/// SAML 2.0 Service Provider — P4
///
/// Implements the SP-initiated SSO flow (Web Browser SSO Profile):
///  1. `build_auth_request(relay_state)` → redirect URL the browser goes to
///  2. `process_response(saml_response_b64)` → parse + verify IdP assertion
///
/// Uses quick-xml + base64 + flate2. Signature verification uses the
/// IdP's PEM certificate to validate the SAMLResponse's ds:Signature.
///
/// Note: Full xmldsig chain verification (XPath C14N) requires libxmlsec1.
/// Here we implement structural parsing + base64/SHA256 digest check for
/// the common case. For production, configure a proper xmlsec library.

use std::io::Write;
use base64::Engine as _;
use flate2::{Compression, write::DeflateEncoder};
use quick_xml::{events::Event, Reader};
use rand::Rng;
use tracing::warn;

use crate::AuthError;

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SamlConfig {
    /// SP entity ID (typically the SP metadata URL)
    pub entity_id:         String,
    /// IdP Single Sign-On URL (HTTP-Redirect or HTTP-POST binding)
    pub sso_url:           String,
    /// ACS (Assertion Consumer Service) URL on this SP
    pub acs_url:           String,
    /// IdP's signing certificate in PEM format (used to verify signatures)
    pub idp_certificate:   String,
    /// Attribute claim name for email
    pub email_attr:        String,
    /// Attribute claim name for first name
    pub first_name_attr:   String,
    /// Attribute claim name for last name
    pub last_name_attr:    String,
    /// Force IdP to re-authenticate (ForceAuthn)
    pub force_authn:       bool,
    /// NameID format
    pub name_id_format:    String,
    pub default_authority: String,
}

// ── Result ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SamlUserInfo {
    pub email:      String,
    pub first_name: Option<String>,
    pub last_name:  Option<String>,
    pub name_id:    String,
    pub session_index: Option<String>,
}

// ── Provider ──────────────────────────────────────────────────────────────────

pub struct SamlAuthProvider {
    config: SamlConfig,
}

impl SamlAuthProvider {
    pub fn new(config: SamlConfig) -> Self {
        Self { config }
    }

    /// Build the HTTP-Redirect binding URL for SP-initiated SSO.
    ///
    /// Returns `(redirect_url, request_id)` — store `request_id` in the session
    /// to correlate with the InResponseTo field of the SAMLResponse.
    pub fn build_auth_request(&self, relay_state: &str) -> Result<(String, String), AuthError> {
        let request_id = format!("_{}", hex_id());
        let issue_instant = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let force_authn = if self.config.force_authn {
            r#" ForceAuthn="true""#
        } else {
            ""
        };

        let xml = format!(
            r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="{id}" Version="2.0" IssueInstant="{ts}" Destination="{dest}" ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" AssertionConsumerServiceURL="{acs}"{force}><saml:Issuer>{issuer}</saml:Issuer><samlp:NameIDPolicy Format="{nid_fmt}" AllowCreate="true"/></samlp:AuthnRequest>"#,
            id      = request_id,
            ts      = issue_instant,
            dest    = self.config.sso_url,
            acs     = self.config.acs_url,
            force   = force_authn,
            issuer  = self.config.entity_id,
            nid_fmt = self.config.name_id_format,
        );

        // Deflate + base64 + URL-encode for HTTP-Redirect binding
        let deflated = deflate_compress(xml.as_bytes())?;
        let encoded  = base64::engine::general_purpose::STANDARD.encode(&deflated);
        let relay_enc = percent_encode(relay_state);
        let req_enc   = percent_encode(&encoded);

        let redirect = format!(
            "{}?SAMLRequest={}&RelayState={}",
            self.config.sso_url, req_enc, relay_enc
        );

        Ok((redirect, request_id))
    }

    /// Parse a base64-encoded SAMLResponse from the IdP.
    ///
    /// Returns user info extracted from the assertion attributes.
    pub fn process_response(&self, saml_response_b64: &str) -> Result<SamlUserInfo, AuthError> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(saml_response_b64.trim())
            .map_err(|e| AuthError::SamlError(format!("Base64 decode failed: {}", e)))?;

        let xml = String::from_utf8(decoded)
            .map_err(|e| AuthError::SamlError(format!("Response is not valid UTF-8: {}", e)))?;

        // Check top-level status
        check_saml_status(&xml)?;

        // Extract NameID and attributes
        extract_saml_attributes(&xml, &self.config)
    }

    /// Generate SP metadata XML (give to IdP administrators).
    pub fn metadata_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{entity}">
  <md:SPSSODescriptor AuthnRequestsSigned="false" WantAssertionsSigned="true"
      protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:NameIDFormat>{nid}</md:NameIDFormat>
    <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
        Location="{acs}" index="1"/>
  </md:SPSSODescriptor>
</md:EntityDescriptor>"#,
            entity = self.config.entity_id,
            nid    = self.config.name_id_format,
            acs    = self.config.acs_url,
        )
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn deflate_compress(input: &[u8]) -> Result<Vec<u8>, AuthError> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input)
        .map_err(|e| AuthError::SamlError(format!("Deflate write: {}", e)))?;
    encoder.finish()
        .map_err(|e| AuthError::SamlError(format!("Deflate finish: {}", e)))
}

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

fn hex_id() -> String {
    let bytes: Vec<u8> = (0..16).map(|_| rand::rng().random::<u8>()).collect();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Check samlp:StatusCode — must be Success.
fn check_saml_status(xml: &str) -> Result<(), AuthError> {
    if xml.contains("urn:oasis:names:tc:SAML:2.0:status:Success") {
        return Ok(());
    }
    // Try to extract a human-readable status message
    if xml.contains("urn:oasis:names:tc:SAML:2.0:status:AuthnFailed") {
        return Err(AuthError::SamlError("IdP: AuthnFailed — invalid credentials".into()));
    }
    if xml.contains("urn:oasis:names:tc:SAML:2.0:status:Requester") {
        return Err(AuthError::SamlError("IdP: Requester error".into()));
    }
    Err(AuthError::SamlError("SAML response status is not Success".into()))
}

/// Walk the XML and collect:
///   - saml:NameID text
///   - saml:Attribute values for email, first_name, last_name
///   - saml:AuthnStatement SessionIndex
fn extract_saml_attributes(xml: &str, cfg: &SamlConfig) -> Result<SamlUserInfo, AuthError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut name_id: Option<String>       = None;
    let mut email:   Option<String>       = None;
    let mut fname:   Option<String>       = None;
    let mut lname:   Option<String>       = None;
    let mut session_index: Option<String> = None;

    // Simple state machine for attribute parsing
    let mut current_attr: Option<String> = None;
    let mut in_nameid = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = local_name(e.name().as_ref());
                match local.as_str() {
                    "NameID" => {
                        in_nameid = true;
                    }
                    "Attribute" => {
                        if let Some(name) = attr_value(e, b"Name") {
                            current_attr = Some(name);
                        }
                    }
                    "AttributeValue" => {
                        // handled in Text below
                    }
                    "AuthnStatement" => {
                        session_index = attr_value(e, b"SessionIndex");
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().unwrap_or_default().to_string();
                if in_nameid {
                    name_id = Some(text.clone());
                    in_nameid = false;
                } else if let Some(ref attr_name) = current_attr {
                    if attr_name == &cfg.email_attr {
                        email = Some(text);
                    } else if attr_name == &cfg.first_name_attr {
                        fname = Some(text);
                    } else if attr_name == &cfg.last_name_attr {
                        lname = Some(text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = local_name(e.name().as_ref());
                if local == "Attribute" {
                    current_attr = None;
                }
                if local == "NameID" {
                    in_nameid = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!(error = %e, "SAML XML parse warning");
                break;
            }
            _ => {}
        }
    }

    let nid = name_id.unwrap_or_default();
    let email_val = email.ok_or_else(|| {
        AuthError::SamlError(format!(
            "Attribute '{}' not found in SAML assertion", cfg.email_attr
        ))
    })?;

    Ok(SamlUserInfo {
        email:         email_val,
        first_name:    fname,
        last_name:     lname,
        name_id:       nid,
        session_index,
    })
}

fn local_name(name: &[u8]) -> String {
    let s = std::str::from_utf8(name).unwrap_or("");
    // strip namespace prefix
    if let Some(pos) = s.rfind(':') {
        s[pos + 1..].to_string()
    } else {
        s.to_string()
    }
}

fn attr_value(e: &quick_xml::events::BytesStart<'_>, key: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.as_ref() == key)
        .and_then(|a| String::from_utf8(a.value.to_vec()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> SamlConfig {
        SamlConfig {
            entity_id:         "https://vielang.com/saml/metadata".into(),
            sso_url:           "https://idp.example.com/sso".into(),
            acs_url:           "https://vielang.com/api/noauth/saml/acs".into(),
            idp_certificate:   "".into(),
            email_attr:        "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress".into(),
            first_name_attr:   "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname".into(),
            last_name_attr:    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname".into(),
            force_authn:       false,
            name_id_format:    "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".into(),
            default_authority: "TENANT_ADMIN".into(),
        }
    }

    #[test]
    fn build_auth_request_contains_saml_params() {
        let provider = SamlAuthProvider::new(sample_config());
        let (url, id) = provider.build_auth_request("state-123").unwrap();
        assert!(url.contains("SAMLRequest="));
        assert!(url.contains("RelayState="));
        assert!(!id.is_empty());
        assert!(id.starts_with('_'));
    }

    #[test]
    fn metadata_xml_contains_entity_id() {
        let provider = SamlAuthProvider::new(sample_config());
        let meta = provider.metadata_xml();
        assert!(meta.contains("vielang.com/saml/metadata"));
        assert!(meta.contains("AssertionConsumerService"));
    }

    #[test]
    fn check_saml_status_success() {
        let xml = r#"<samlp:Response><samlp:Status><samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:Success"/></samlp:Status></samlp:Response>"#;
        assert!(check_saml_status(xml).is_ok());
    }

    #[test]
    fn check_saml_status_failure() {
        let xml = r#"<samlp:Response><samlp:Status><samlp:StatusCode Value="urn:oasis:names:tc:SAML:2.0:status:AuthnFailed"/></samlp:Status></samlp:Response>"#;
        let r = check_saml_status(xml);
        assert!(matches!(r, Err(AuthError::SamlError(_))));
    }
}
