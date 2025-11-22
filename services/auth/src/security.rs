use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::fmt;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub enum SecurityError {
    HashingError(String),
    VerificationError,
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::HashingError(msg) => write!(f, "Hashing error: {}", msg),
            SecurityError::VerificationError => write!(f, "Verification failed"),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Hash a password with a salt and pepper using Argon2
pub fn hash_password(password: &str, pepper: &str) -> Result<(String, String), SecurityError> {
    let salt = SaltString::generate(&mut OsRng);

    // Combine password with pepper
    let peppered_password = format!("{}{}", password, pepper);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(peppered_password.as_bytes(), &salt)
        .map_err(|e| SecurityError::HashingError(e.to_string()))?
        .to_string();

    Ok((password_hash, salt.to_string()))
}

/// Verify a password against a hash using the stored salt and pepper
pub fn verify_password(password: &str, hash: &str, pepper: &str) -> Result<bool, SecurityError> {
    let peppered_password = format!("{}{}", password, pepper);

    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| SecurityError::HashingError(e.to_string()))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(peppered_password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Hash a username with a salt and pepper for storage
pub fn hash_username(username: &str, pepper: &str) -> Result<(String, String), SecurityError> {
    let salt = SaltString::generate(&mut OsRng);

    let peppered_username = format!("{}{}", username, pepper);

    let argon2 = Argon2::default();

    let username_hash = argon2
        .hash_password(peppered_username.as_bytes(), &salt)
        .map_err(|e| SecurityError::HashingError(e.to_string()))?
        .to_string();

    Ok((username_hash, salt.to_string()))
}

/// Hash a token deterministically using HMAC-SHA256
/// This is used for refresh tokens and CSRF tokens where we need to look up by hash
pub fn hash_token(token: &str, secret: &str) -> Result<String, SecurityError> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| SecurityError::HashingError(e.to_string()))?;

    mac.update(token.as_bytes());
    let result = mac.finalize();
    let bytes = result.into_bytes();

    // Convert to hex string
    Ok(hex::encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let pepper = "test_pepper";

        let result = hash_password(password, pepper);
        assert!(result.is_ok());

        let (hash, salt) = result.unwrap();
        assert!(!hash.is_empty());
        assert!(!salt.is_empty());
        assert_ne!(hash, password);
    }

    #[test]
    fn test_hash_password_different_salts() {
        let password = "test_password_123";
        let pepper = "test_pepper";

        let (hash1, salt1) = hash_password(password, pepper).unwrap();
        let (hash2, salt2) = hash_password(password, pepper).unwrap();

        // Different salts should produce different hashes
        assert_ne!(salt1, salt2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_password_success() {
        let password = "test_password_123";
        let pepper = "test_pepper";

        let (hash, _salt) = hash_password(password, pepper).unwrap();

        let result = verify_password(password, &hash, pepper);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_wrong_password() {
        let password = "test_password_123";
        let wrong_password = "wrong_password";
        let pepper = "test_pepper";

        let (hash, _salt) = hash_password(password, pepper).unwrap();

        let result = verify_password(wrong_password, &hash, pepper);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_wrong_pepper() {
        let password = "test_password_123";
        let pepper = "test_pepper";
        let wrong_pepper = "wrong_pepper";

        let (hash, _salt) = hash_password(password, pepper).unwrap();

        let result = verify_password(password, &hash, wrong_pepper);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let password = "test_password_123";
        let pepper = "test_pepper";
        let invalid_hash = "not_a_valid_hash";

        let result = verify_password(password, invalid_hash, pepper);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_username() {
        let username = "testuser";
        let pepper = "test_pepper";

        let result = hash_username(username, pepper);
        assert!(result.is_ok());

        let (hash, salt) = result.unwrap();
        assert!(!hash.is_empty());
        assert!(!salt.is_empty());
        assert_ne!(hash, username);
    }

    #[test]
    fn test_hash_username_different_salts() {
        let username = "testuser";
        let pepper = "test_pepper";

        let (hash1, salt1) = hash_username(username, pepper).unwrap();
        let (hash2, salt2) = hash_username(username, pepper).unwrap();

        // Different salts should produce different hashes
        assert_ne!(salt1, salt2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_pepper_affects_hash() {
        let password = "test_password_123";
        let pepper1 = "pepper1";
        let pepper2 = "pepper2";

        let (hash1, _) = hash_password(password, pepper1).unwrap();
        let (hash2, _) = hash_password(password, pepper2).unwrap();

        // Different peppers should produce different hashes
        assert_ne!(hash1, hash2);
    }
}
