use axum::http::{StatusCode, HeaderName, HeaderValue};
use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

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
            "password": "securepassword123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: Value = response.json();
    assert!(body["user"]["id"].is_string());
    assert_eq!(body["user"]["username"], username);
    assert_eq!(body["user"]["email"], email);
    assert!(body["auth"]["access_token"].is_string());
    assert!(body["auth"]["refresh_token"].is_string());
    assert!(body["csrf_token"].is_string());
}

#[tokio::test]
#[ignore]
async fn test_register_duplicate_username() {
    let server = create_test_server().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email1 = format!("test1_{}@example.com", Uuid::new_v4());
    let email2 = format!("test2_{}@example.com", Uuid::new_v4());

    // Register first user
    server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email1,
            "password": "securepassword123"
        }))
        .await;

    // Try to register with same username
    let response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email2,
            "password": "securepassword123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Username already exists"));
}

#[tokio::test]
#[ignore]
async fn test_register_duplicate_email() {
    let server = create_test_server().await;

    let username1 = format!("testuser1_{}", Uuid::new_v4());
    let username2 = format!("testuser2_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    // Register first user
    server
        .post("/register")
        .json(&json!({
            "username": username1,
            "email": email,
            "password": "securepassword123"
        }))
        .await;

    // Try to register with same email
    let response = server
        .post("/register")
        .json(&json!({
            "username": username2,
            "email": email,
            "password": "securepassword123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
    let body: Value = response.json();
    assert!(body["error"].as_str().unwrap().contains("Email already exists"));
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
            "password": "securepassword123"
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
            "password": "short"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_login_success() {
    let server = create_test_server().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    // Register user
    server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .await;

    // Login
    let response = server
        .post("/login")
        .json(&json!({
            "username": username,
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

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    // Register user
    server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "correctpassword123"
        }))
        .await;

    // Try to login with wrong password
    let response = server
        .post("/login")
        .json(&json!({
            "username": username,
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
            "username": "nonexistentuser",
            "password": "password123"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore]
async fn test_refresh_token() {
    let server = create_test_server().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    // Get CSRF token
    let csrf_response = server.get("/csrf-token").await;
    let csrf_body: Value = csrf_response.json();
    let csrf_token = csrf_body["csrf_token"].as_str().unwrap();

    // Register user
    let register_response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "securepassword123"
        }))
        .await;

    let register_body: Value = register_response.json();
    let refresh_token = register_body["auth"]["refresh_token"].as_str().unwrap();

    // Refresh token
    let response = server
        .post("/refresh")
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(csrf_token).unwrap())
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
        .add_header(HeaderName::from_static("x-csrf-token"), HeaderValue::from_str(csrf_token).unwrap())
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

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    // Register user
    let register_response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "securepassword123"
        }))
        .await;

    let register_body: Value = register_response.json();
    let access_token = register_body["auth"]["access_token"].as_str().unwrap();

    // Get user info
    let response = server
        .get("/me")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap()
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
            HeaderValue::from_static("Bearer invalid_token")
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore]
async fn test_logout() {
    let server = create_test_server().await;

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    // Register user
    let register_response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "securepassword123"
        }))
        .await;

    let register_body: Value = register_response.json();
    let access_token = register_body["auth"]["access_token"].as_str().unwrap();
    let csrf_token = register_body["csrf_token"].as_str().unwrap();

    // Logout
    let response = server
        .post("/logout")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap()
        )
        .add_header(
            HeaderName::from_static("x-csrf-token"),
            HeaderValue::from_str(csrf_token).unwrap()
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

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    // Register user
    let register_response = server
        .post("/register")
        .json(&json!({
            "username": username,
            "email": email,
            "password": "securepassword123"
        }))
        .await;

    let register_body: Value = register_response.json();
    let access_token = register_body["auth"]["access_token"].as_str().unwrap();

    // Try to logout without CSRF token
    let response = server
        .post("/logout")
        .add_header(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(&format!("Bearer {}", access_token)).unwrap()
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}
