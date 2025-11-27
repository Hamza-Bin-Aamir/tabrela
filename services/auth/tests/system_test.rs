/// System tests for the authentication microservice
/// These tests validate the entire system end-to-end
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

const BASE_URL: &str = "http://localhost:8081";

/// Helper function to get OTP from database for email verification
/// This is the ONLY database operation allowed in system tests
/// to maintain test integrity while enabling email verification flow
async fn get_otp_from_db(pool: &PgPool, email: &str) -> Result<String, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        SELECT evt.otp 
        FROM email_verification_tokens evt
        JOIN users u ON evt.user_id = u.id
        WHERE u.email = $1
        "#,
        email
    )
    .fetch_one(pool)
    .await?;

    Ok(record.otp)
}

/// Setup database connection for OTP retrieval
async fn setup_db_pool() -> PgPool {
    dotenv::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for system tests");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

/// Helper struct to manage test client
struct TestClient {
    client: Client,
    pool: PgPool,
}

impl TestClient {
    async fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            pool: setup_db_pool().await,
        }
    }

    async fn health_check(&self) -> bool {
        match self.client.get(format!("{}/health", BASE_URL)).send().await {
            Ok(response) => response.status() == StatusCode::OK,
            Err(_) => false,
        }
    }

    async fn register(&self, username: &str, email: &str, password: &str) -> Result<Value, String> {
        // Generate unique valid test data for each registration
        let reg_number = format!("20{:05}", rand::random::<u32>() % 100000);
        let phone_number = format!("+9230{:08}", rand::random::<u32>() % 100000000);

        self.register_with_details(username, email, password, &reg_number, 2023, &phone_number)
            .await
    }

    async fn register_with_details(
        &self,
        username: &str,
        email: &str,
        password: &str,
        reg_number: &str,
        year_joined: i32,
        phone_number: &str,
    ) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("{}/register", BASE_URL))
            .json(&json!({
                "username": username,
                "email": email,
                "password": password,
                "reg_number": reg_number,
                "year_joined": year_joined,
                "phone_number": phone_number
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let body = response.json::<Value>().await.map_err(|e| e.to_string())?;

        if status == StatusCode::CREATED {
            Ok(body)
        } else {
            Err(format!("Registration failed: {:?}", body))
        }
    }

    async fn login(&self, username: &str, password: &str) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("{}/login", BASE_URL))
            .json(&json!({
                "username_or_email": username,
                "password": password
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let body = response.json::<Value>().await.map_err(|e| e.to_string())?;

        if status == StatusCode::OK {
            Ok(body)
        } else {
            Err(format!("Login failed: {:?}", body))
        }
    }

    async fn get_me(&self, access_token: &str) -> Result<Value, String> {
        let response = self
            .client
            .get(format!("{}/me", BASE_URL))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let body = response.json::<Value>().await.map_err(|e| e.to_string())?;

        if status == StatusCode::OK {
            Ok(body)
        } else {
            Err(format!("Get me failed: {:?}", body))
        }
    }

    async fn refresh_token(&self, refresh_token: &str, csrf_token: &str) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("{}/refresh", BASE_URL))
            .header("X-CSRF-Token", csrf_token)
            .json(&json!({
                "refresh_token": refresh_token
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let body = response.json::<Value>().await.map_err(|e| e.to_string())?;

        if status == StatusCode::OK {
            Ok(body)
        } else {
            Err(format!("Token refresh failed: {:?}", body))
        }
    }

    async fn logout(&self, access_token: &str, csrf_token: &str) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("{}/logout", BASE_URL))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("X-CSRF-Token", csrf_token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let body = response.json::<Value>().await.map_err(|e| e.to_string())?;

        if status == StatusCode::OK {
            Ok(body)
        } else {
            Err(format!("Logout failed: {:?}", body))
        }
    }

    async fn get_csrf_token(&self) -> Result<String, String> {
        let response = self
            .client
            .get(format!("{}/csrf-token", BASE_URL))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body = response.json::<Value>().await.map_err(|e| e.to_string())?;

        body["csrf_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "CSRF token not found".to_string())
    }

    async fn verify_email(&self, email: &str, otp: &str) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("{}/verify-email", BASE_URL))
            .json(&json!({
                "email": email,
                "otp": otp
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status();
        let body = response.json::<Value>().await.map_err(|e| e.to_string())?;

        if status == StatusCode::OK {
            Ok(body)
        } else {
            Err(format!("Email verification failed: {:?}", body))
        }
    }

    /// Helper to register a user and verify their email
    /// This maintains test integrity by going through the proper flow:
    /// 1. Register via API (creates unverified user)
    /// 2. Retrieve OTP from database (simulates email receipt)
    /// 3. Verify email via API (completes verification flow)
    async fn register_and_verify(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<(), String> {
        // Step 1: Register the user
        self.register(username, email, password).await?;

        // Step 2: Get OTP from database (simulating email receipt)
        let otp = get_otp_from_db(&self.pool, email)
            .await
            .map_err(|e| format!("Failed to get OTP: {}", e))?;

        // Step 3: Verify email
        self.verify_email(email, &otp).await?;

        Ok(())
    }
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_system_health_check() {
    let client = TestClient::new().await;

    // Wait for server to be ready
    for _ in 0..10 {
        if client.health_check().await {
            return;
        }
        sleep(Duration::from_secs(1)).await;
    }

    panic!("Server health check failed");
}

#[tokio::test]
#[ignore]
async fn test_system_complete_auth_flow() {
    let client = TestClient::new().await;

    // Ensure server is ready
    assert!(client.health_check().await, "Server is not ready");

    // 1. Register and verify a new user
    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    client
        .register_and_verify(&username, &email, password)
        .await
        .unwrap();

    // 2. Login to get tokens
    let login_response = client.login(&username, password).await.unwrap();
    let initial_access_token = login_response["auth"]["access_token"].as_str().unwrap();
    let initial_refresh_token = login_response["auth"]["refresh_token"].as_str().unwrap();
    let csrf_token = login_response["csrf_token"].as_str().unwrap();

    // 3. Get user info with access token
    let me_response = client.get_me(initial_access_token).await.unwrap();
    assert_eq!(me_response["username"], username);
    assert_eq!(me_response["email"], email);

    // 4. Refresh the access token
    let refresh_response = client
        .refresh_token(initial_refresh_token, csrf_token)
        .await
        .unwrap();
    let new_access_token = refresh_response["access_token"].as_str().unwrap();
    let new_refresh_token = refresh_response["refresh_token"].as_str().unwrap();

    assert_ne!(new_access_token, initial_access_token);
    assert_ne!(new_refresh_token, initial_refresh_token);

    // 5. Use new access token to get user info
    let me_response2 = client.get_me(new_access_token).await.unwrap();
    assert_eq!(me_response2["username"], username);

    // 6. Logout
    let logout_response = client.logout(new_access_token, csrf_token).await.unwrap();
    assert!(logout_response["message"].as_str().is_some());

    // 7. Try to use refresh token after logout (should fail)
    let refresh_after_logout = client.refresh_token(new_refresh_token, csrf_token).await;
    assert!(refresh_after_logout.is_err());
}

#[tokio::test]
#[ignore]
async fn test_system_login_flow() {
    let client = TestClient::new().await;

    assert!(client.health_check().await, "Server is not ready");

    // 1. Register and verify a user
    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    client
        .register_and_verify(&username, &email, password)
        .await
        .unwrap();

    // 2. Login with the same credentials
    let login_response = client.login(&username, password).await.unwrap();

    assert_eq!(login_response["user"]["username"], username);
    assert!(login_response["auth"]["access_token"].is_string());
    assert!(login_response["auth"]["refresh_token"].is_string());

    // 3. Try to login with wrong password
    let wrong_login = client.login(&username, "wrongpassword").await;
    assert!(wrong_login.is_err());
}

#[tokio::test]
#[ignore]
async fn test_system_duplicate_registration() {
    let client = TestClient::new().await;

    assert!(client.health_check().await, "Server is not ready");

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";
    let reg_number = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone_number = format!("+9230{:08}", rand::random::<u32>() % 100000000);

    // First registration and verification should succeed
    client
        .register_with_details(
            &username,
            &email,
            password,
            &reg_number,
            2023,
            &phone_number,
        )
        .await
        .unwrap();

    // Get OTP and verify the user
    let otp = get_otp_from_db(&client.pool, &email).await.unwrap();
    client.verify_email(&email, &otp).await.unwrap();

    // Second registration with same username but different email (and different reg/phone) should fail
    // because username is now verified and unique
    let other_reg = format!("20{:05}", rand::random::<u32>() % 100000);
    let other_phone = format!("+9230{:08}", rand::random::<u32>() % 100000000);
    let duplicate = client
        .register_with_details(
            &username,
            &format!("other_{}", email),
            password,
            &other_reg,
            2023,
            &other_phone,
        )
        .await;
    assert!(
        duplicate.is_err(),
        "Should not allow duplicate username for verified user"
    );

    // Registration with same email but different username (and different reg/phone) should also fail
    let other_reg2 = format!("20{:05}", rand::random::<u32>() % 100000);
    let other_phone2 = format!("+9230{:08}", rand::random::<u32>() % 100000000);
    let duplicate_email = client
        .register_with_details(
            &format!("other_{}", username),
            &email,
            password,
            &other_reg2,
            2023,
            &other_phone2,
        )
        .await;
    assert!(
        duplicate_email.is_err(),
        "Should not allow duplicate email for verified user"
    );
}

#[tokio::test]
#[ignore]
async fn test_system_csrf_protection() {
    let client = TestClient::new().await;

    assert!(client.health_check().await, "Server is not ready");

    // Register and verify a user
    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    client
        .register_and_verify(&username, &email, password)
        .await
        .unwrap();

    // Login to get tokens
    let login_response = client.login(&username, password).await.unwrap();
    let access_token = login_response["auth"]["access_token"].as_str().unwrap();

    // Try to logout without CSRF token
    let response = client
        .client
        .post(format!("{}/logout", BASE_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[ignore]
async fn test_system_invalid_tokens() {
    let client = TestClient::new().await;

    assert!(client.health_check().await, "Server is not ready");

    // Try to access protected endpoint with invalid token
    let result = client.get_me("invalid_token").await;
    assert!(result.is_err());

    // Try to refresh with invalid token (get a valid CSRF token first)
    let csrf_token = client.get_csrf_token().await.unwrap();
    let result = client.refresh_token("invalid_token", &csrf_token).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore]
async fn test_system_csrf_token_generation() {
    let client = TestClient::new().await;

    assert!(client.health_check().await, "Server is not ready");

    // Get CSRF token
    let csrf_token = client.get_csrf_token().await.unwrap();
    assert!(!csrf_token.is_empty());

    // Get another CSRF token (should be different)
    let csrf_token2 = client.get_csrf_token().await.unwrap();
    assert_ne!(csrf_token, csrf_token2);
}

#[tokio::test]
#[ignore]
async fn test_system_concurrent_requests() {
    let client = TestClient::new().await;

    assert!(client.health_check().await, "Server is not ready");

    // Register multiple users concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                let client = TestClient::new().await;
                let username = format!("testuser_{}_{}", i, Uuid::new_v4());
                let email = format!("test_{}_{}@example.com", i, Uuid::new_v4());
                client.register(&username, &email, "password123").await
            })
        })
        .collect();

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
#[ignore]
async fn test_system_token_expiry_behavior() {
    let client = TestClient::new().await;

    assert!(client.health_check().await, "Server is not ready");

    // Register and verify a user
    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    client
        .register_and_verify(&username, &email, password)
        .await
        .unwrap();

    // Login to get tokens
    let login_response = client.login(&username, password).await.unwrap();
    let access_token = login_response["auth"]["access_token"].as_str().unwrap();

    // Access token should work immediately
    let me_response = client.get_me(access_token).await.unwrap();
    assert_eq!(me_response["username"], username);

    // Note: To test actual expiry, you would need to wait for token expiration
    // or use a test environment with very short token lifetimes
}

#[tokio::test]
#[ignore]
async fn test_system_password_security() {
    let client = TestClient::new().await;

    assert!(client.health_check().await, "Server is not ready");

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    // Test with weak password (should fail validation)
    let weak_result = client.register(&username, &email, "weak").await;
    assert!(weak_result.is_err());

    // Test with strong password (should succeed)
    let strong_password = "StrongP@ssw0rd123!";
    let strong_result = client.register(&username, &email, strong_password).await;
    assert!(strong_result.is_ok());
}
