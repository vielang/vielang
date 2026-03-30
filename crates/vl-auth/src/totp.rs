use rand::Rng;
use sha2::{Digest, Sha256};
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};

#[derive(Debug, Error)]
pub enum TotpError {
    #[error("Invalid TOTP secret: {0}")]
    InvalidSecret(String),
    #[error("TOTP creation error: {0}")]
    Create(String),
}

pub struct TotpService;

impl TotpService {
    /// Generate a new random Base32-encoded TOTP secret.
    pub fn generate_secret() -> String {
        Secret::generate_secret().to_encoded().to_string()
    }

    /// Build an `otpauth://` provisioning URI for QR code rendering.
    pub fn get_provisioning_uri(
        secret: &str,
        account: &str,
        issuer: &str,
    ) -> Result<String, TotpError> {
        Ok(Self::build(secret, account, issuer)?.get_url())
    }

    /// Verify a 6-digit TOTP code (allows ±1 step drift).
    pub fn verify_code(secret: &str, code: &str) -> bool {
        match Self::build(secret, "", "vielang") {
            Ok(totp) => totp.check_current(code).unwrap_or(false),
            Err(_)   => false,
        }
    }

    /// Generate `count` one-time backup codes (format: XXXX-XXXX-XXXX).
    ///
    /// Returns `(plaintext_codes, hashed_codes)` — store hashed in DB,
    /// show plaintext to the user exactly once.
    pub fn generate_backup_codes(count: usize) -> (Vec<String>, Vec<String>) {
        let mut plain   = Vec::with_capacity(count);
        let mut hashed  = Vec::with_capacity(count);

        for _ in 0..count {
            // 12 random alphanumeric chars formatted as XXXX-XXXX-XXXX
            let raw: String = (0..12)
                .map(|_| {
                    let idx = rand::rng().random_range(0..36usize);
                    if idx < 10 {
                        (b'0' + idx as u8) as char
                    } else {
                        (b'A' + (idx - 10) as u8) as char
                    }
                })
                .collect();
            let code = format!("{}-{}-{}", &raw[..4], &raw[4..8], &raw[8..]);
            let hash_bytes = Sha256::digest(code.as_bytes());
            let hash_hex   = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
            plain.push(code);
            hashed.push(hash_hex);
        }

        (plain, hashed)
    }

    /// Verify a backup code against a list of SHA-256 hashes.
    ///
    /// Returns `Some(index)` of the matching code if found, `None` otherwise.
    /// The caller must remove the matching hash from storage (one-time use).
    pub fn verify_backup_code(code: &str, stored_hashes: &[String]) -> Option<usize> {
        let hash_bytes = Sha256::digest(code.trim().to_uppercase().as_bytes());
        let hash_hex: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        stored_hashes.iter().position(|h| *h == hash_hex)
    }

    fn build(secret: &str, account: &str, issuer: &str) -> Result<TOTP, TotpError> {
        let bytes = Secret::Encoded(secret.to_string())
            .to_bytes()
            .map_err(|e| TotpError::InvalidSecret(e.to_string()))?;

        TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            bytes,
            Some(issuer.to_string()),
            account.to_string(),
        )
        .map_err(|e| TotpError::Create(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_secret_is_non_empty_base32() {
        let secret = TotpService::generate_secret();
        assert!(!secret.is_empty());
        // Base32 uses A-Z and 2-7
        assert!(secret.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn provisioning_uri_contains_issuer_and_account() {
        let secret = TotpService::generate_secret();
        let uri = TotpService::get_provisioning_uri(&secret, "user@example.com", "VieLang")
            .unwrap();
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("VieLang"));
    }

    #[test]
    fn invalid_secret_returns_error() {
        let result = TotpService::get_provisioning_uri("NOT_VALID!!!", "user", "iss");
        assert!(result.is_err());
    }

    #[test]
    fn verify_code_with_invalid_secret_returns_false() {
        assert!(!TotpService::verify_code("BADSECRET!!!", "123456"));
    }
}
