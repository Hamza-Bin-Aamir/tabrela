use crate::models::{Claims, TokenType};
use chrono::Utc;
use pasetors::{
    claims::{Claims as PasetoClaims, ClaimsValidationRules},
    keys::SymmetricKey,
    local,
    token::UntrustedToken,
    version4::V4,
    Local,
};
use std::fmt;

#[derive(Debug)]
pub enum PasetoError {
    TokenCreationError(String),
    TokenValidationError(String),
    KeyError(String),
}

impl fmt::Display for PasetoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasetoError::TokenCreationError(msg) => write!(f, "Token creation error: {}", msg),
            PasetoError::TokenValidationError(msg) => write!(f, "Token validation error: {}", msg),
            PasetoError::KeyError(msg) => write!(f, "Key error: {}", msg),
        }
    }
}

impl std::error::Error for PasetoError {}

pub struct PasetoService {
    symmetric_key: SymmetricKey<V4>,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl PasetoService {
    pub fn new(
        secret: String,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Result<Self, PasetoError> {
        // Ensure the secret is exactly 32 bytes for V4 local tokens
        if secret.len() != 32 {
            return Err(PasetoError::KeyError(format!(
                "Secret key must be exactly 32 bytes, got {}",
                secret.len()
            )));
        }

        let symmetric_key = SymmetricKey::<V4>::from(secret.as_bytes())
            .map_err(|e| PasetoError::KeyError(e.to_string()))?;

        Ok(Self {
            symmetric_key,
            access_token_expiry,
            refresh_token_expiry,
        })
    }

    /// Create an access token
    pub fn create_access_token(
        &self,
        user_id: &str,
        username: &str,
    ) -> Result<String, PasetoError> {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(self.access_token_expiry);

        let mut claims =
            PasetoClaims::new().map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .expiration(&expires_at.to_rfc3339())
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("sub", serde_json::json!(user_id))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("username", serde_json::json!(username))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("iat", serde_json::json!(now.timestamp()))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("jti", serde_json::json!(uuid::Uuid::new_v4().to_string()))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("token_type", serde_json::json!("access"))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        local::encrypt(&self.symmetric_key, &claims, None, None)
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))
    }

    /// Create a refresh token
    pub fn create_refresh_token(
        &self,
        user_id: &str,
        username: &str,
    ) -> Result<String, PasetoError> {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(self.refresh_token_expiry);

        let mut claims =
            PasetoClaims::new().map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .expiration(&expires_at.to_rfc3339())
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("sub", serde_json::json!(user_id))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("username", serde_json::json!(username))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("iat", serde_json::json!(now.timestamp()))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("jti", serde_json::json!(uuid::Uuid::new_v4().to_string()))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        claims
            .add_additional("token_type", serde_json::json!("refresh"))
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))?;

        local::encrypt(&self.symmetric_key, &claims, None, None)
            .map_err(|e| PasetoError::TokenCreationError(e.to_string()))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<Claims, PasetoError> {
        let validation_rules = ClaimsValidationRules::new();
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|e| PasetoError::TokenValidationError(e.to_string()))?;

        let trusted_token = local::decrypt(
            &self.symmetric_key,
            &untrusted_token,
            &validation_rules,
            None,
            None,
        )
        .map_err(|e| PasetoError::TokenValidationError(e.to_string()))?;

        let claims_string = trusted_token
            .payload_claims()
            .ok_or_else(|| PasetoError::TokenValidationError("No claims in token".to_string()))?
            .to_string()
            .map_err(|e| PasetoError::TokenValidationError(e.to_string()))?;

        let payload: serde_json::Value = serde_json::from_str(&claims_string)
            .map_err(|e| PasetoError::TokenValidationError(e.to_string()))?;

        let sub = payload
            .get("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PasetoError::TokenValidationError("Missing 'sub' claim".to_string()))?
            .to_string();

        let username = payload
            .get("username")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PasetoError::TokenValidationError("Missing 'username' claim".to_string())
            })?
            .to_string();

        let exp_str = payload
            .get("exp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PasetoError::TokenValidationError("Missing 'exp' claim".to_string()))?;

        let exp = chrono::DateTime::parse_from_rfc3339(exp_str)
            .map_err(|e| PasetoError::TokenValidationError(e.to_string()))?
            .timestamp();

        let iat = payload
            .get("iat")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| PasetoError::TokenValidationError("Missing 'iat' claim".to_string()))?;

        let jti = payload
            .get("jti")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PasetoError::TokenValidationError("Missing 'jti' claim".to_string()))?
            .to_string();

        let token_type_str = payload
            .get("token_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PasetoError::TokenValidationError("Missing 'token_type' claim".to_string())
            })?;

        let token_type = match token_type_str {
            "access" => TokenType::Access,
            "refresh" => TokenType::Refresh,
            _ => {
                return Err(PasetoError::TokenValidationError(
                    "Invalid token type".to_string(),
                ))
            }
        };

        // Check if token has expired
        let now = Utc::now().timestamp();
        if exp < now {
            return Err(PasetoError::TokenValidationError(
                "Token has expired".to_string(),
            ));
        }

        Ok(Claims {
            sub,
            username,
            exp,
            iat,
            jti,
            token_type,
        })
    }

    /// Validate that the token is an access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, PasetoError> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Access {
            return Err(PasetoError::TokenValidationError(
                "Token is not an access token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Validate that the token is a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, PasetoError> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(PasetoError::TokenValidationError(
                "Token is not a refresh token".to_string(),
            ));
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_paseto_service() -> PasetoService {
        PasetoService::new("test_secret_key_for_testing_32b".to_string(), 900, 604800).unwrap()
    }

    #[test]
    fn test_create_access_token() {
        let paseto_service = create_test_paseto_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let result = paseto_service.create_access_token(user_id, username);
        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());
        assert!(token.starts_with("v4.local."));
    }

    #[test]
    fn test_create_refresh_token() {
        let paseto_service = create_test_paseto_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let result = paseto_service.create_refresh_token(user_id, username);
        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());
        assert!(token.starts_with("v4.local."));
    }

    #[test]
    fn test_validate_access_token() {
        let paseto_service = create_test_paseto_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = paseto_service
            .create_access_token(user_id, username)
            .unwrap();
        let result = paseto_service.validate_access_token(&token);

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_validate_refresh_token() {
        let paseto_service = create_test_paseto_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = paseto_service
            .create_refresh_token(user_id, username)
            .unwrap();
        let result = paseto_service.validate_refresh_token(&token);

        assert!(result.is_ok());
        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_wrong_token_type() {
        let paseto_service = create_test_paseto_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let access_token = paseto_service
            .create_access_token(user_id, username)
            .unwrap();

        // Try to validate access token as refresh token
        let result = paseto_service.validate_refresh_token(&access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_token() {
        let paseto_service = create_test_paseto_service();
        let result = paseto_service.validate_access_token("invalid_token");
        assert!(result.is_err());
    }
}
