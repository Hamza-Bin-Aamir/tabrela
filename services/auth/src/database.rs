use crate::models::{CsrfToken, RefreshToken, User};
use chrono::{DateTime, Duration, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

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
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
        salt: &str,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, username, email, password_hash, salt, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, username, email, password_hash, salt, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(salt)
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
            SELECT id, username, email, password_hash, salt, created_at, updated_at
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
            SELECT id, username, email, password_hash, salt, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find a user by ID - uses parameterized queries
    pub async fn find_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, salt, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
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

        let user = db
            .create_user(&username, &email, password_hash, salt)
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

        let user = db
            .create_user(&username, &email, password_hash, salt)
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
        let user = db
            .create_user(&username, &email, "hash", "salt")
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
        let user = db
            .create_user(&username, &email, "hash", "salt")
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
        let user = db
            .create_user(&username, &email, "hash", "salt")
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
