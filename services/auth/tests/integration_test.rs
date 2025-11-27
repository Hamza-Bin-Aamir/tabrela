use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum_test::TestServer;
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

// TODO: Login-related tests need to be updated for the new email verification flow
// Users must now verify their email before they can log in
// These tests should either:
// 1. Include the full registration -> verify OTP -> login flow
// 2. Use a helper function to create verified users directly in the database for testing

/// Helper function to create a verified user directly in the database for testing
/// This bypasses the normal registration + email verification flow
async fn create_verified_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password: &str,
    reg_number: &str,
    year_joined: i32,
    phone_number: &str,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    // Use the same password hashing as the app
    let password_pepper =
        std::env::var("PASSWORD_PEPPER").unwrap_or_else(|_| "test_pepper".to_string());

    // Use auth's hash_password function
    let (password_hash, salt) = auth::security::hash_password(password, &password_pepper)
        .map_err(|e| format!("Failed to hash password: {}", e))?;

    let user_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (
            id, username, email, password_hash, salt, 
            reg_number, year_joined, phone_number, 
            email_verified, email_verified_at, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, $10, $11)
        "#,
    )
    .bind(user_id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(salt)
    .bind(reg_number)
    .bind(year_joined)
    .bind(phone_number)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(user_id)
}

/// Helper to get database pool from the app
async fn get_database_pool() -> PgPool {
    dotenv::dotenv().ok();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be set");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

// Helper to create test server
async fn create_test_server() -> TestServer {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Use TEST_DATABASE_URL if set for tests, otherwise use existing DATABASE_URL
    if let Ok(test_db_url) = std::env::var("TEST_DATABASE_URL") {
        std::env::set_var("DATABASE_URL", test_db_url);
    }

    // Only set test values for keys that aren't already set
    if std::env::var("JWT_SECRET").is_err() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing");
    }
    if std::env::var("PASSWORD_PEPPER").is_err() {
        std::env::set_var("PASSWORD_PEPPER", "test_pepper_for_testing");
    }
    if std::env::var("ALLOWED_ORIGINS").is_err() {
        std::env::set_var("ALLOWED_ORIGINS", "*");
    }
    if std::env::var("CORS_STRICT_MODE").is_err() {
        std::env::set_var("CORS_STRICT_MODE", "false");
    }

    // Create the app
    let app = auth::create_app().await.expect("Failed to create app");

    TestServer::new(app).expect("Failed to create test server")
}

// Helper to clean database
#[allow(dead_code)]
async fn clean_database(pool: &PgPool) {
    sqlx::query("DELETE FROM csrf_tokens")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM refresh_tokens")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users")
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore] // Requires database setup
async fn test_health_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    assert_eq!(response.text(), "OK");
}

#[tokio::test]
#[ignore]
async fn test_register_new_user() {
    let server = create_test_server().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    let response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "securepassword123",
            "reg_number": "2012345",
            "year_joined": 2023,
            "phone_number": "+923001234567"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: Value = response.json();
    assert_eq!(
        body["message"],
        "Registration successful. Please check your email for the verification code."
    );
    assert_eq!(body["email"], email);
}

#[tokio::test]
#[ignore]
async fn test_register_duplicate_username() {
    let server = create_test_server().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email1 = format!("test1_{}@example.com", Uuid::new_v4());
    let email2 = format!("test2_{}@example.com", Uuid::new_v4());
    let reg_num1 = format!("20{:05}", rand::random::<u32>() % 100000);
    let reg_num2 = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone1 = format!("+9230012{:05}", rand::random::<u32>() % 100000);
    let phone2 = format!("+9230012{:05}", rand::random::<u32>() % 100000);

    // Register first user (unverified)
    server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email1,
            "password": "securepassword123",
            "reg_number": reg_num1,
            "year_joined": 2023,
            "phone_number": phone1
        }))
        .await;

    // Try to register with same username but different email/phone/reg (should succeed and replace unverified user)
    let response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email2,
            "password": "securepassword123",
            "reg_number": reg_num2,
            "year_joined": 2023,
            "phone_number": phone2
        }))
        .await;

    // Should succeed because first user was unverified
    assert_eq!(response.status_code(), StatusCode::CREATED);
}

#[tokio::test]
#[ignore]
async fn test_register_duplicate_email() {
    let server = create_test_server().await;

    let username1 = format!("testuser1_{}", Uuid::new_v4());
    let username2 = format!("testuser2_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let reg_num1 = format!("20{:05}", rand::random::<u32>() % 100000);
    let reg_num2 = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone1 = format!("+9230012{:05}", rand::random::<u32>() % 100000);
    let phone2 = format!("+9230012{:05}", rand::random::<u32>() % 100000);

    // Register first user (unverified)
    server
        .post("/register")
        .json(&json!({
            "username": username1,
            "email": email,
            "password": "securepassword123",
            "reg_number": reg_num1,
            "year_joined": 2023,
            "phone_number": phone1
        }))
        .await;

    // Try to register with same email (should succeed and replace unverified user)
    let response = server
        .post("/register")
        .json(&json!({
            "username": username2,
            "email": email,
            "password": "securepassword123",
            "reg_number": reg_num2,
            "year_joined": 2023,
            "phone_number": phone2
        }))
        .await;

    // Should succeed because first user was unverified
    assert_eq!(response.status_code(), StatusCode::CREATED);
}

#[tokio::test]
#[ignore]
async fn test_register_invalid_email() {
    let server = create_test_server().await;

    let response = server
        .post("/register")
        .json(&json!({
            "username": "testuser",
            "email": "invalid-email",
            "password": "securepassword123",
            "reg_number": "2012345",
            "year_joined": 2023,
            "phone_number": "+923001234567"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_register_short_password() {
    let server = create_test_server().await;

    let response = server
        .post("/register")
        .json(&json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "short",
            "reg_number": "2012345",
            "year_joined": 2023,
            "phone_number": "+923001234567"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_login_success() {
    let server = create_test_server().await;
    let pool = get_database_pool().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";
    let reg_number = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone_number = format!("+9230{:08}", rand::random::<u32>() % 100000000);

    // Create a verified user directly in the database
    create_verified_user(
        &pool,
        &username,
        &email,
        password,
        &reg_number,
        2023,
        &phone_number,
    )
    .await
    .expect("Failed to create verified user");

    // Login with username
    let response = server
        .post("/login")
        .json(&json!({
            "username_or_email": username,
            "password": password
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["user"]["username"], username);
    assert!(body["auth"]["access_token"].is_string());
    assert!(body["auth"]["refresh_token"].is_string());
    assert!(body["csrf_token"].is_string());
}

#[tokio::test]
#[ignore]
async fn test_login_wrong_password() {
    let server = create_test_server().await;
    let pool = get_database_pool().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let reg_number = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone_number = format!("+9230{:08}", rand::random::<u32>() % 100000000);

    // Create a verified user with correct password
    create_verified_user(
        &pool,
        &username,
        &email,
        "correctpassword123",
        &reg_number,
        2023,
        &phone_number,
    )
    .await
    .expect("Failed to create verified user");

    // Try to login with wrong password
    let response = server
        .post("/login")
        .json(&json!({
            "username_or_email": username,
            "password": "wrongpassword123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore]
async fn test_login_nonexistent_user() {
    let server = create_test_server().await;

    let response = server
        .post("/login")
        .json(&json!({
            "username_or_email": "nonexistentuser",
            "password": "somepassword123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore]
async fn test_refresh_token() {
    let server = create_test_server().await;
    let pool = get_database_pool().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";
    let reg_number = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone_number = format!("+9230{:08}", rand::random::<u32>() % 100000000);

    // Create a verified user
    create_verified_user(
        &pool,
        &username,
        &email,
        password,
        &reg_number,
        2023,
        &phone_number,
    )
    .await
    .expect("Failed to create verified user");

    // Get CSRF token
    let csrf_response = server.get("/csrf-token").await;
    let csrf_body: Value = csrf_response.json();
    let csrf_token = csrf_body["csrf_token"].as_str().unwrap();

    // Login to get tokens
    let login_response = server
        .post("/login")
        .json(&json!({
            "username_or_email": username,
            "password": password
        }))
        .await;

    let login_body: Value = login_response.json();
    let refresh_token = login_body["auth"]["refresh_token"].as_str().unwrap();

    // Refresh token
    let response = server
        .post("/refresh")
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(csrf_token).unwrap(),
        )
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .await;

    // Debug output
    if response.status_code() != StatusCode::OK {
        let error_body: Value = response.json();
        eprintln!("Error response: {:?}", error_body);
        panic!("Expected 200 OK, got {}", response.status_code());
    }

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    // New refresh token should be different
    assert_ne!(body["refresh_token"].as_str().unwrap(), refresh_token);
}

#[tokio::test]
#[ignore]
async fn test_refresh_token_invalid() {
    let server = create_test_server().await;

    // Get CSRF token
    let csrf_response = server.get("/csrf-token").await;
    let csrf_body: Value = csrf_response.json();
    let csrf_token = csrf_body["csrf_token"].as_str().unwrap();

    let response = server
        .post("/refresh")
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(csrf_token).unwrap(),
        )
        .json(&json!({
            "refresh_token": "invalid_token"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore]
async fn test_me_endpoint() {
    let server = create_test_server().await;
    let pool = get_database_pool().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";
    let reg_number = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone_number = format!("+9230{:08}", rand::random::<u32>() % 100000000);

    // Create a verified user
    create_verified_user(
        &pool,
        &username,
        &email,
        password,
        &reg_number,
        2023,
        &phone_number,
    )
    .await
    .expect("Failed to create verified user");

    // Login to get access token
    let login_response = server
        .post("/login")
        .json(&json!({
            "username_or_email": username,
            "password": password
        }))
        .await;

    let login_body: Value = login_response.json();
    let access_token = login_body["auth"]["access_token"].as_str().unwrap();

    // Get user info
    let response = server
        .get("/me")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["username"], username);
    assert_eq!(body["email"], email);
}

#[tokio::test]
#[ignore]
async fn test_me_endpoint_unauthorized() {
    let server = create_test_server().await;

    let response = server.get("/me").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore]
async fn test_me_endpoint_invalid_token() {
    let server = create_test_server().await;

    let response = server
        .get("/me")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer invalid_token"),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore]
async fn test_logout() {
    let server = create_test_server().await;
    let pool = get_database_pool().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";
    let reg_number = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone_number = format!("+9230{:08}", rand::random::<u32>() % 100000000);

    // Create a verified user
    create_verified_user(
        &pool,
        &username,
        &email,
        password,
        &reg_number,
        2023,
        &phone_number,
    )
    .await
    .expect("Failed to create verified user");

    // Login to get tokens
    let login_response = server
        .post("/login")
        .json(&json!({
            "username_or_email": username,
            "password": password
        }))
        .await;

    let login_body: Value = login_response.json();
    let access_token = login_body["auth"]["access_token"].as_str().unwrap();
    let csrf_token = login_body["csrf_token"].as_str().unwrap();

    // Logout
    let response = server
        .post("/logout")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(csrf_token).unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_csrf_token_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/csrf-token").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert!(body["csrf_token"].is_string());
    assert!(!body["csrf_token"].as_str().unwrap().is_empty());
}

#[tokio::test]
#[ignore]
async fn test_csrf_protection() {
    let server = create_test_server().await;
    let pool = get_database_pool().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";
    let reg_number = format!("20{:05}", rand::random::<u32>() % 100000);
    let phone_number = format!("+9230{:08}", rand::random::<u32>() % 100000000);

    // Create a verified user
    create_verified_user(
        &pool,
        &username,
        &email,
        password,
        &reg_number,
        2023,
        &phone_number,
    )
    .await
    .expect("Failed to create verified user");

    // Login to get tokens
    let login_response = server
        .post("/login")
        .json(&json!({
            "username_or_email": username,
            "password": password
        }))
        .await;

    let login_body: Value = login_response.json();
    let access_token = login_body["auth"]["access_token"].as_str().unwrap();

    // Try to logout without CSRF token
    let response = server
        .post("/logout")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}
