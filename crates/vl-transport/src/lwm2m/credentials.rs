/// LwM2M device credentials — PSK, RPK, X.509, and NoSec modes.
///
/// Mirrors ThingsBoard `TbLwM2MSecurityInfo.java` and `LwM2mCredentialsSecurityInfoValidator.java`.
/// These structs represent the credential types supported by DTLS-secured LwM2M connections.
///
/// Note: Full DTLS handshake requires a DTLS library (e.g., dtls-rs or openssl bindings).
/// This module provides the data types for storing and validating credential configurations.

use serde::{Deserialize, Serialize};

/// LwM2M DTLS security mode.
/// Stored in Object 0 Resource 2 (Security Mode) on the device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityMode {
    /// Pre-Shared Key — symmetric key shared between device and server.
    /// Simplest to deploy; requires PSK identity + key.
    PreSharedKey,
    /// Raw Public Key — asymmetric key without a full certificate chain.
    /// Device and server exchange raw EC public keys.
    RawPublicKey,
    /// Certificate — X.509 certificate-based authentication.
    /// Most secure; requires PKI infrastructure.
    Certificate,
    /// No security — plaintext CoAP. Development/testing only.
    NoSec,
}

/// Pre-Shared Key credentials for SNMPv3 USM / LwM2M DTLS PSK mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PskCredentials {
    /// PSK identity (UTF-8 string sent during DTLS handshake as ClientIdentity)
    pub identity: String,
    /// Raw PSK bytes (hex-encoded in the DB, decoded for DTLS use)
    #[serde(with = "hex_bytes")]
    pub key: Vec<u8>,
}

/// Raw Public Key (RPK) credentials — EC public key without certificate chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpkCredentials {
    /// Device's EC public key (DER-encoded SubjectPublicKeyInfo)
    #[serde(with = "hex_bytes")]
    pub public_key: Vec<u8>,
}

/// X.509 certificate credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateCredentials {
    /// Device certificate in PEM format
    pub certificate: String,
    /// CN (Common Name) from the certificate — used as the device identifier
    pub common_name: Option<String>,
}

/// Unified LwM2M device credentials — holds whichever mode is active.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LwM2mCredentials {
    PreSharedKey(PskCredentials),
    RawPublicKey(RpkCredentials),
    Certificate(CertificateCredentials),
    NoSec,
}

impl LwM2mCredentials {
    pub fn security_mode(&self) -> SecurityMode {
        match self {
            Self::PreSharedKey(_) => SecurityMode::PreSharedKey,
            Self::RawPublicKey(_) => SecurityMode::RawPublicKey,
            Self::Certificate(_)  => SecurityMode::Certificate,
            Self::NoSec           => SecurityMode::NoSec,
        }
    }

    /// Return the PSK identity if this is a PSK credential.
    pub fn psk_identity(&self) -> Option<&str> {
        if let Self::PreSharedKey(psk) = self {
            Some(&psk.identity)
        } else {
            None
        }
    }

    /// Return the certificate CN if this is a certificate credential.
    pub fn certificate_cn(&self) -> Option<&str> {
        if let Self::Certificate(cert) = self {
            cert.common_name.as_deref()
        } else {
            None
        }
    }
}

/// Validate that the LwM2M endpoint name matches the credential's identity.
/// ThingsBoard uses the endpoint name as the device lookup key.
pub fn validate_endpoint_identity(endpoint: &str, credentials: &LwM2mCredentials) -> bool {
    match credentials {
        LwM2mCredentials::PreSharedKey(psk) => psk.identity == endpoint,
        LwM2mCredentials::Certificate(cert) => {
            cert.common_name.as_deref() == Some(endpoint)
        }
        // For RPK and NoSec, any endpoint name is accepted
        LwM2mCredentials::RawPublicKey(_) | LwM2mCredentials::NoSec => true,
    }
}

// ── Hex serialization helper ───────────────────────────────────────────────────

mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        s.serialize_str(&hex)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let hex = String::deserialize(d)?;
        (0..hex.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex[i..i + 2], 16)
                    .map_err(|e| serde::de::Error::custom(e.to_string()))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psk_security_mode() {
        let cred = LwM2mCredentials::PreSharedKey(PskCredentials {
            identity: "device-001".to_string(),
            key: vec![0x01, 0x02, 0x03],
        });
        assert_eq!(cred.security_mode(), SecurityMode::PreSharedKey);
        assert_eq!(cred.psk_identity(), Some("device-001"));
    }

    #[test]
    fn test_no_sec() {
        let cred = LwM2mCredentials::NoSec;
        assert_eq!(cred.security_mode(), SecurityMode::NoSec);
        assert!(cred.psk_identity().is_none());
    }

    #[test]
    fn test_validate_psk_endpoint() {
        let cred = LwM2mCredentials::PreSharedKey(PskCredentials {
            identity: "sensor-a1".to_string(),
            key: vec![0xDE, 0xAD],
        });
        assert!(validate_endpoint_identity("sensor-a1", &cred));
        assert!(!validate_endpoint_identity("sensor-b2", &cred));
    }

    #[test]
    fn test_validate_no_sec_any_endpoint() {
        assert!(validate_endpoint_identity("anything", &LwM2mCredentials::NoSec));
    }

    #[test]
    fn test_serde_roundtrip() {
        let cred = LwM2mCredentials::PreSharedKey(PskCredentials {
            identity: "dev-x".to_string(),
            key: vec![0xAB, 0xCD, 0xEF],
        });
        let json = serde_json::to_string(&cred).unwrap();
        let back: LwM2mCredentials = serde_json::from_str(&json).unwrap();
        assert_eq!(back.security_mode(), SecurityMode::PreSharedKey);
        assert_eq!(back.psk_identity(), Some("dev-x"));
    }
}
