use crate::models::{CsrfToken, EmailVerificationToken, PasswordResetToken, RefreshToken, User};
use chrono::{DateTime, Duration, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

/// Parameters for creating a new user
pub struct CreateUserParams<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub salt: &'a str,
    pub reg_number: &'a str,
    pub year_joined: i32,
    pub phone_number: &'a str,
}

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Create a new user - uses parameterized queries to prevent SQL injection
    pub async fn create_user(&self, params: CreateUserParams<'_>) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, salt, reg_number, year_joined, phone_number, email_verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false, $9, $10)
            RETURNING id, username, email, password_hash, salt, reg_number, year_joined, phone_number, email_verified, email_verified_at, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(params.username)
        .bind(params.email)
        .bind(params.password_hash)
        .bind(params.salt)
        .bind(params.reg_number)
        .bind(params.year_joined)
        .bind(params.phone_number)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find a user by username - uses parameterized queries
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, salt, reg_number, year_joined, phone_number, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find a user by email - uses parameterized queries
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, salt, reg_number, year_joined, phone_number, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find a user by phone number - uses parameterized queries
    pub async fn find_user_by_phone(
        &self,
        phone_number: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, salt, reg_number, year_joined, phone_number, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE phone_number = $1
            "#,
        )
        .bind(phone_number)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find a user by registration number - uses parameterized queries
    pub async fn find_user_by_reg_number(
        &self,
        reg_number: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, salt, reg_number, year_joined, phone_number, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE reg_number = $1
            "#,
        )
        .bind(reg_number)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find a user by ID - uses parameterized queries
    pub async fn find_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, salt, reg_number, year_joined, phone_number, email_verified, email_verified_at, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Delete a user by ID (for cleaning up unverified registrations)
    pub async fn delete_user_by_id(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Store a refresh token - uses parameterized queries
    pub async fn store_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken, sqlx::Error> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, token_hash, expires_at, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(refresh_token)
    }

    /// Find a refresh token by hash - uses parameterized queries
    pub async fn find_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, sqlx::Error> {
        let token = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT id, user_id, token_hash, expires_at, created_at
            FROM refresh_tokens
            WHERE token_hash = $1 AND expires_at > $2
            "#,
        )
        .bind(token_hash)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;

        Ok(token)
    }

    /// Delete a refresh token - uses parameterized queries
    pub async fn delete_refresh_token(&self, token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete all refresh tokens for a user - uses parameterized queries
    pub async fn delete_user_refresh_tokens(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired refresh tokens - uses parameterized queries
    pub async fn cleanup_expired_refresh_tokens(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < $1
            "#,
        )
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a CSRF token - uses parameterized queries
    pub async fn create_csrf_token(
        &self,
        token: &str,
        user_id: Option<Uuid>,
        expiry_seconds: i64,
    ) -> Result<CsrfToken, sqlx::Error> {
        let expires_at = Utc::now() + Duration::seconds(expiry_seconds);

        let csrf_token = sqlx::query_as::<_, CsrfToken>(
            r#"
            INSERT INTO csrf_tokens (id, token, user_id, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, token, user_id, expires_at, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(token)
        .bind(user_id)
        .bind(expires_at)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(csrf_token)
    }

    /// Validate a CSRF token - uses parameterized queries
    pub async fn validate_csrf_token(&self, token: &str) -> Result<Option<CsrfToken>, sqlx::Error> {
        let csrf_token = sqlx::query_as::<_, CsrfToken>(
            r#"
            SELECT id, token, user_id, expires_at, created_at
            FROM csrf_tokens
            WHERE token = $1 AND expires_at > $2
            "#,
        )
        .bind(token)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;

        Ok(csrf_token)
    }

    /// Delete a CSRF token - uses parameterized queries
    pub async fn delete_csrf_token(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM csrf_tokens
            WHERE token = $1
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired CSRF tokens - uses parameterized queries
    pub async fn cleanup_expired_csrf_tokens(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM csrf_tokens
            WHERE expires_at < $1
            "#,
        )
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// Email verification and password reset methods
impl Database {
    /// Create or update an email verification OTP
    pub async fn create_email_verification_otp(
        &self,
        user_id: Uuid,
        otp: &str,
        expiry_seconds: i64,
    ) -> Result<EmailVerificationToken, sqlx::Error> {
        let expires_at = Utc::now() + Duration::seconds(expiry_seconds);

        // Delete any existing OTP for this user
        sqlx::query(
            r#"
            DELETE FROM email_verification_tokens
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        // Create new OTP
        let verification_token = sqlx::query_as::<_, EmailVerificationToken>(
            r#"
            INSERT INTO email_verification_tokens (id, user_id, otp, attempts, expires_at, created_at, last_sent_at)
            VALUES ($1, $2, $3, 0, $4, $5, $6)
            RETURNING id, user_id, otp, attempts, expires_at, created_at, last_sent_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(otp)
        .bind(expires_at)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(verification_token)
    }

    /// Find an email verification OTP by user_id
    pub async fn find_email_verification_otp_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<Option<EmailVerificationToken>, sqlx::Error> {
        let verification_token = sqlx::query_as::<_, EmailVerificationToken>(
            r#"
            SELECT id, user_id, otp, attempts, expires_at, created_at, last_sent_at
            FROM email_verification_tokens
            WHERE user_id = $1 AND expires_at > $2
            "#,
        )
        .bind(user_id)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;

        Ok(verification_token)
    }

    /// Verify OTP and increment attempts
    pub async fn verify_email_otp(&self, user_id: Uuid, otp: &str) -> Result<bool, sqlx::Error> {
        // Get the OTP record
        let token = self.find_email_verification_otp_by_user(user_id).await?;

        match token {
            Some(token) => {
                if token.attempts >= 5 {
                    // Too many attempts
                    return Ok(false);
                }

                if token.otp == otp {
                    // Correct OTP - mark email as verified and delete token
                    sqlx::query(
                        r#"
                        UPDATE users
                        SET email_verified = true, email_verified_at = $1
                        WHERE id = $2
                        "#,
                    )
                    .bind(Utc::now())
                    .bind(user_id)
                    .execute(&self.pool)
                    .await?;

                    sqlx::query(
                        r#"
                        DELETE FROM email_verification_tokens
                        WHERE user_id = $1
                        "#,
                    )
                    .bind(user_id)
                    .execute(&self.pool)
                    .await?;

                    Ok(true)
                } else {
                    // Incorrect OTP - increment attempts
                    sqlx::query(
                        r#"
                        UPDATE email_verification_tokens
                        SET attempts = attempts + 1
                        WHERE user_id = $1
                        "#,
                    )
                    .bind(user_id)
                    .execute(&self.pool)
                    .await?;

                    Ok(false)
                }
            }
            None => Ok(false),
        }
    }

    /// Delete email verification token by user_id
    pub async fn delete_email_verification_token_by_user(
        &self,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM email_verification_tokens
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Legacy method - kept for backwards compatibility with password reset
    pub async fn find_email_verification_token(
        &self,
        token: &str,
    ) -> Result<Option<EmailVerificationToken>, sqlx::Error> {
        // This is now only used for password reset, but keeping the signature
        let verification_token = sqlx::query_as::<_, EmailVerificationToken>(
            r#"
            SELECT id, user_id, otp as token, attempts, expires_at, created_at, last_sent_at
            FROM email_verification_tokens
            WHERE otp = $1 AND expires_at > $2
            "#,
        )
        .bind(token)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;

        Ok(verification_token)
    }

    /// Legacy method - delete by token
    pub async fn delete_email_verification_token(&self, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM email_verification_tokens
            WHERE otp = $1
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete all email verification tokens for a user
    pub async fn delete_user_verification_tokens(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM email_verification_tokens
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark user email as verified
    pub async fn verify_user_email(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = true, email_verified_at = $1, updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create or update a password reset OTP
    pub async fn create_password_reset_otp(
        &self,
        user_id: Uuid,
        email: &str,
        otp_hash: &str,
        expiry_seconds: i64,
    ) -> Result<PasswordResetToken, sqlx::Error> {
        let expires_at = Utc::now() + Duration::seconds(expiry_seconds);

        // Delete any existing OTP for this email
        sqlx::query("DELETE FROM password_reset_tokens WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await?;

        let reset_token = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, email, otp, attempts, expires_at, created_at, last_sent_at, used)
            VALUES ($1, $2, $3, $4, 0, $5, $6, $6, false)
            RETURNING id, user_id, email, otp, attempts, expires_at, created_at, last_sent_at, used
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(otp_hash)
        .bind(expires_at)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(reset_token)
    }

    /// Find a password reset token by email
    pub async fn find_password_reset_by_email(
        &self,
        email: &str,
    ) -> Result<Option<PasswordResetToken>, sqlx::Error> {
        let reset_token = sqlx::query_as::<_, PasswordResetToken>(
            r#"
            SELECT id, user_id, email, otp, attempts, expires_at, created_at, last_sent_at, used
            FROM password_reset_tokens
            WHERE email = $1 AND expires_at > $2 AND used = false
            "#,
        )
        .bind(email)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;

        Ok(reset_token)
    }

    /// Increment OTP verification attempts
    pub async fn increment_password_reset_attempts(&self, email: &str) -> Result<i32, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            UPDATE password_reset_tokens
            SET attempts = attempts + 1
            WHERE email = $1
            RETURNING attempts
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    /// Mark a password reset token as used
    pub async fn mark_password_reset_used(&self, email: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used = true
            WHERE email = $1
            "#,
        )
        .bind(email)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update user password
    pub async fn update_user_password(
        &self,
        user_id: Uuid,
        password_hash: &str,
        salt: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, salt = $2, updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(password_hash)
        .bind(salt)
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired email verification tokens
    pub async fn cleanup_expired_verification_tokens(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM email_verification_tokens
            WHERE expires_at < $1
            "#,
        )
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired password reset tokens
    pub async fn cleanup_expired_reset_tokens(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM password_reset_tokens
            WHERE expires_at < $1
            "#,
        )
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a test database to be set up
    // They are marked with #[ignore] to prevent running during regular test runs
    // Run with: cargo test -- --ignored

    async fn setup_test_db() -> Database {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let database_url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/auth_test".to_string());

        Database::new(&database_url).await.unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_and_find_user() {
        let db = setup_test_db().await;

        let username = format!("testuser_{}", Uuid::new_v4());
        let email = format!("test_{}@example.com", Uuid::new_v4());
        let password_hash = "hash";
        let salt = "salt";
        // Generate unique reg_number (20 + 5 digits from random number)
        let random_suffix: u32 = rand::random::<u32>() % 100000;
        let reg_number = format!("20{:05}", random_suffix);
        // Generate unique phone number
        let phone_suffix: u32 = rand::random::<u32>() % 1000000000;
        let phone_number = format!("+92{:010}", phone_suffix);

        let user = db
            .create_user(CreateUserParams {
                username: &username,
                email: &email,
                password_hash,
                salt,
                reg_number: &reg_number,
                year_joined: 2023,
                phone_number: &phone_number,
            })
            .await
            .unwrap();

        assert_eq!(user.username, username);
        assert_eq!(user.email, email);

        let found_user = db.find_user_by_username(&username).await.unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().username, username);
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_user_by_email() {
        let db = setup_test_db().await;

        let username = format!("testuser_{}", Uuid::new_v4());
        let email = format!("test_{}@example.com", Uuid::new_v4());
        let password_hash = "hash";
        let salt = "salt";
        // Generate unique reg_number (20 + 5 digits from random number)
        let random_suffix: u32 = rand::random::<u32>() % 100000;
        let reg_number = format!("20{:05}", random_suffix);
        // Generate unique phone number
        let phone_suffix: u32 = rand::random::<u32>() % 1000000000;
        let phone_number = format!("+92{:010}", phone_suffix);

        let user = db
            .create_user(CreateUserParams {
                username: &username,
                email: &email,
                password_hash,
                salt,
                reg_number: &reg_number,
                year_joined: 2023,
                phone_number: &phone_number,
            })
            .await
            .unwrap();

        let found_user = db.find_user_by_email(&email).await.unwrap();
        assert!(found_user.is_some());
        assert_eq!(found_user.unwrap().id, user.id);
    }

    #[tokio::test]
    #[ignore]
    async fn test_store_and_find_refresh_token() {
        let db = setup_test_db().await;

        // Create a user first to satisfy foreign key constraint
        let username = format!("testuser_{}", Uuid::new_v4());
        let email = format!("test_{}@example.com", Uuid::new_v4());
        // Generate unique reg_number (20 + 5 digits from random number)
        let random_suffix: u32 = rand::random::<u32>() % 100000;
        let reg_number = format!("20{:05}", random_suffix);
        // Generate unique phone number
        let phone_suffix: u32 = rand::random::<u32>() % 1000000000;
        let phone_number = format!("+92{:010}", phone_suffix);
        
        let user = db
            .create_user(CreateUserParams {
                username: &username,
                email: &email,
                password_hash: "hash",
                salt: "salt",
                reg_number: &reg_number,
                year_joined: 2023,
                phone_number: &phone_number,
            })
            .await
            .unwrap();

        let token_hash = format!("token_hash_{}", Uuid::new_v4());
        let expires_at = Utc::now() + Duration::hours(1);

        db.store_refresh_token(user.id, &token_hash, expires_at)
            .await
            .unwrap();

        let found_token = db.find_refresh_token(&token_hash).await.unwrap();
        assert!(found_token.is_some());
        assert_eq!(found_token.unwrap().user_id, user.id);
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_refresh_token() {
        let db = setup_test_db().await;

        // Create a user first to satisfy foreign key constraint
        let username = format!("testuser_{}", Uuid::new_v4());
        let email = format!("test_{}@example.com", Uuid::new_v4());
        // Generate unique reg_number (20 + 5 digits from random number)
        let random_suffix: u32 = rand::random::<u32>() % 100000;
        let reg_number = format!("20{:05}", random_suffix);
        // Generate unique phone number
        let phone_suffix: u32 = rand::random::<u32>() % 1000000000;
        let phone_number = format!("+92{:010}", phone_suffix);
        
        let user = db
            .create_user(CreateUserParams {
                username: &username,
                email: &email,
                password_hash: "hash",
                salt: "salt",
                reg_number: &reg_number,
                year_joined: 2023,
                phone_number: &phone_number,
            })
            .await
            .unwrap();

        let token_hash = format!("token_hash_{}", Uuid::new_v4());
        let expires_at = Utc::now() + Duration::hours(1);

        db.store_refresh_token(user.id, &token_hash, expires_at)
            .await
            .unwrap();
        db.delete_refresh_token(&token_hash).await.unwrap();

        let found_token = db.find_refresh_token(&token_hash).await.unwrap();
        assert!(found_token.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_and_validate_csrf_token() {
        let db = setup_test_db().await;

        // Create a user first to satisfy foreign key constraint
        let username = format!("testuser_{}", Uuid::new_v4());
        let email = format!("test_{}@example.com", Uuid::new_v4());
        // Generate unique reg_number (20 + 5 digits from random number)
        let random_suffix: u32 = rand::random::<u32>() % 100000;
        let reg_number = format!("20{:05}", random_suffix);
        // Generate unique phone number
        let phone_suffix: u32 = rand::random::<u32>() % 1000000000;
        let phone_number = format!("+92{:010}", phone_suffix);
        
        let user = db
            .create_user(CreateUserParams {
                username: &username,
                email: &email,
                password_hash: "hash",
                salt: "salt",
                reg_number: &reg_number,
                year_joined: 2023,
                phone_number: &phone_number,
            })
            .await
            .unwrap();

        let token = format!("csrf_token_{}", Uuid::new_v4());
        let user_id = Some(user.id);
        let expiry_seconds = 3600;

        db.create_csrf_token(&token, user_id, expiry_seconds)
            .await
            .unwrap();

        let found_token = db.validate_csrf_token(&token).await.unwrap();
        assert!(found_token.is_some());
        assert_eq!(found_token.unwrap().user_id, user_id);
    }
}
