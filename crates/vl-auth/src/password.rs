use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;  // must use rand_core 0.6 — argon2/password-hash 0.5 requires it

use crate::AuthError;

/// Hash plain-text password bằng Argon2id
pub fn hash_password(plain: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AuthError::PasswordError(e.to_string()))
}

/// Verify plain-text password against Argon2 hash
pub fn verify_password(plain: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AuthError::PasswordError(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_password_verifies() {
        let hash = hash_password("my-secret-password").unwrap();
        assert!(verify_password("my-secret-password", &hash).unwrap());
    }

    #[test]
    fn wrong_password_does_not_verify() {
        let hash = hash_password("correct-password").unwrap();
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn empty_password_handled() {
        let hash = hash_password("").unwrap();
        assert!(verify_password("", &hash).unwrap());
        assert!(!verify_password("x", &hash).unwrap());
    }

    #[test]
    fn same_password_produces_different_hashes_due_to_random_salt() {
        let h1 = hash_password("password123").unwrap();
        let h2 = hash_password("password123").unwrap();
        assert_ne!(h1, h2);
        // But both verify correctly
        assert!(verify_password("password123", &h1).unwrap());
        assert!(verify_password("password123", &h2).unwrap());
    }

    #[test]
    fn hash_uses_argon2_phc_format() {
        let hash = hash_password("any-password").unwrap();
        assert!(
            hash.starts_with("$argon2"),
            "Expected Argon2 PHC string, got: {hash}"
        );
    }

    #[test]
    fn invalid_hash_format_returns_error() {
        let result = verify_password("password", "not-a-valid-hash");
        assert!(result.is_err());
    }

    #[test]
    fn unicode_password_handled() {
        let hash = hash_password("пароль123🔑").unwrap();
        assert!(verify_password("пароль123🔑", &hash).unwrap());
        assert!(!verify_password("пароль123", &hash).unwrap());
    }
}
