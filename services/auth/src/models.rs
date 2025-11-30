use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub salt: String,
    pub reg_number: String,
    pub year_joined: i32,
    pub phone_number: String,
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
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

// Regex validators - defined here but used as string literals in validation
lazy_static::lazy_static! {
    pub static ref RE_REG_NUMBER: regex::Regex = regex::Regex::new(r"^20\d{5}$").unwrap();
    pub static ref RE_PHONE: regex::Regex = regex::Regex::new(r"^\+\d{1,3}\d{9,15}$").unwrap();
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 7, max = 7), regex(path = *RE_REG_NUMBER))]
    pub reg_number: String,
    #[validate(range(min = 2000, max = 2099))]
    pub year_joined: i32,
    #[validate(regex(path = *RE_PHONE))]
    pub phone_number: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub username_or_email: String, // Changed to accept either username or email
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
    pub reg_number: String,
    pub year_joined: i32,
    pub phone_number: String,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            reg_number: user.reg_number,
            year_joined: user.year_joined,
            phone_number: user.phone_number,
            email_verified: user.email_verified,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String, // JWT ID - unique identifier for each token
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
            reg_number: "REG123".to_string(),
            year_joined: 2023,
            phone_number: "1234567890".to_string(),
            email_verified: false,
            email_verified_at: None,
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
            reg_number: "2012345".to_string(), // Valid format: 20XXXXX
            year_joined: 2023,                 // Valid year between 2000-2099
            phone_number: "+923001234567".to_string(), // Valid format with country code
        };
        assert!(valid_request.validate().is_ok());

        let invalid_username = RegisterRequest {
            username: "ab".to_string(), // Too short
            email: "test@example.com".to_string(),
            password: "securepassword123".to_string(),
            reg_number: "2012345".to_string(),
            year_joined: 2023,
            phone_number: "+923001234567".to_string(),
        };
        assert!(invalid_username.validate().is_err());

        let invalid_email = RegisterRequest {
            username: "testuser".to_string(),
            email: "invalid-email".to_string(),
            password: "securepassword123".to_string(),
            reg_number: "2012345".to_string(),
            year_joined: 2023,
            phone_number: "+923001234567".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        let invalid_password = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(), // Too short
            reg_number: "2012345".to_string(),
            year_joined: 2023,
            phone_number: "+923001234567".to_string(),
        };
        assert!(invalid_password.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            username_or_email: "testuser".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_username = LoginRequest {
            username_or_email: "".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_username.validate().is_err());
    }
}

// Email verification and password reset models
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub otp: String,
    pub attempts: i32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_sent_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub otp: String, // Hashed OTP
    pub attempts: i32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_sent_at: DateTime<Utc>,
    pub used: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyEmailRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(equal = 6))]
    pub otp: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResendOtpRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RequestPasswordResetRequest {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(equal = 6))]
    pub otp: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResendVerificationRequest {
    #[validate(email)]
    pub email: String,
}

// Admin-related models
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<AdminUser> for AdminUserResponse {
    fn from(admin: AdminUser) -> Self {
        AdminUserResponse {
            id: admin.id,
            user_id: admin.user_id,
            granted_by: admin.granted_by,
            created_at: admin.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub reg_number: String,
    pub year_joined: i32,
    pub phone_number: String,
    pub email_verified: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PromoteToAdminRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct AdminListUsersResponse {
    pub users: Vec<UserListResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}
