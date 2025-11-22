use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CsrfToken {
    pub id: Uuid,
    pub token: String,
    pub user_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,  // JWT ID - unique identifier for each token
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_response_from_user() {
        let user = User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            salt: "salt".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let user_id = user.id;
        let response: UserResponse = user.into();
        
        assert_eq!(response.id, user_id);
        assert_eq!(response.username, "testuser");
        assert_eq!(response.email, "test@example.com");
    }

    #[test]
    fn test_token_type_serialization() {
        let access = TokenType::Access;
        let refresh = TokenType::Refresh;

        let access_json = serde_json::to_string(&access).unwrap();
        let refresh_json = serde_json::to_string(&refresh).unwrap();

        assert_eq!(access_json, "\"access\"");
        assert_eq!(refresh_json, "\"refresh\"");
    }

    #[test]
    fn test_register_request_validation() {
        let valid_request = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "securepassword123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_username = RegisterRequest {
            username: "ab".to_string(),  // Too short
            email: "test@example.com".to_string(),
            password: "securepassword123".to_string(),
        };
        assert!(invalid_username.validate().is_err());

        let invalid_email = RegisterRequest {
            username: "testuser".to_string(),
            email: "invalid-email".to_string(),
            password: "securepassword123".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        let invalid_password = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(),  // Too short
        };
        assert!(invalid_password.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_username = LoginRequest {
            username: "".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_username.validate().is_err());
    }
}
