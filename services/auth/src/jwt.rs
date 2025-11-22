use crate::models::{Claims, TokenType};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::fmt;

#[derive(Debug)]
pub enum JwtError {
    TokenCreationError(String),
    TokenValidationError(String),
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JwtError::TokenCreationError(msg) => write!(f, "Token creation error: {}", msg),
            JwtError::TokenValidationError(msg) => write!(f, "Token validation error: {}", msg),
        }
    }
}

impl std::error::Error for JwtError {}

pub struct JwtService {
    secret: String,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl JwtService {
    pub fn new(secret: String, access_token_expiry: i64, refresh_token_expiry: i64) -> Self {
        Self {
            secret,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    /// Create an access token
    pub fn create_access_token(&self, user_id: &str, username: &str) -> Result<String, JwtError> {
        let now = Utc::now().timestamp();
        let expires_at = now + self.access_token_expiry;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: expires_at,
            iat: now,
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::TokenCreationError(e.to_string()))
    }

    /// Create a refresh token
    pub fn create_refresh_token(&self, user_id: &str, username: &str) -> Result<String, JwtError> {
        let now = Utc::now().timestamp();
        let expires_at = now + self.refresh_token_expiry;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp: expires_at,
            iat: now,
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::TokenCreationError(e.to_string()))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let validation = Validation::default();

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| JwtError::TokenValidationError(e.to_string()))
    }

    /// Validate that the token is an access token
    pub fn validate_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Access {
            return Err(JwtError::TokenValidationError(
                "Token is not an access token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Validate that the token is a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(JwtError::TokenValidationError(
                "Token is not a refresh token".to_string(),
            ));
        }

        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jwt_service() -> JwtService {
        JwtService::new("test_secret".to_string(), 900, 604800)
    }

    #[test]
    fn test_create_access_token() {
        let jwt_service = create_test_jwt_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let result = jwt_service.create_access_token(user_id, username);
        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_create_refresh_token() {
        let jwt_service = create_test_jwt_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let result = jwt_service.create_refresh_token(user_id, username);
        assert!(result.is_ok());

        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_validate_token() {
        let jwt_service = create_test_jwt_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = jwt_service.create_access_token(user_id, username).unwrap();
        let result = jwt_service.validate_token(&token);

        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_validate_access_token() {
        let jwt_service = create_test_jwt_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = jwt_service.create_access_token(user_id, username).unwrap();
        let result = jwt_service.validate_access_token(&token);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_refresh_token() {
        let jwt_service = create_test_jwt_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = jwt_service.create_refresh_token(user_id, username).unwrap();
        let result = jwt_service.validate_refresh_token(&token);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_access_token_with_refresh_token_fails() {
        let jwt_service = create_test_jwt_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = jwt_service.create_refresh_token(user_id, username).unwrap();
        let result = jwt_service.validate_access_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_refresh_token_with_access_token_fails() {
        let jwt_service = create_test_jwt_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = jwt_service.create_access_token(user_id, username).unwrap();
        let result = jwt_service.validate_refresh_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_invalid_token() {
        let jwt_service = create_test_jwt_service();
        let invalid_token = "invalid.token.here";

        let result = jwt_service.validate_token(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_wrong_secret() {
        let jwt_service1 = JwtService::new("secret1".to_string(), 900, 604800);
        let jwt_service2 = JwtService::new("secret2".to_string(), 900, 604800);

        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = jwt_service1.create_access_token(user_id, username).unwrap();
        let result = jwt_service2.validate_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_token_contains_correct_claims() {
        let jwt_service = create_test_jwt_service();
        let user_id = "123e4567-e89b-12d3-a456-426614174000";
        let username = "testuser";

        let token = jwt_service.create_access_token(user_id, username).unwrap();
        let claims = jwt_service.validate_token(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
        assert_eq!(claims.token_type, TokenType::Access);
        assert!(claims.exp > claims.iat);
    }
}
