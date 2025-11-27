/// System tests for the authentication microservice
/// These tests validate the entire system end-to-end
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

const BASE_URL: &str = "http://localhost:8081";

/// Helper struct to manage test client
struct TestClient {
    client: Client,
}

impl TestClient {
    fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    async fn health_check(&self) -> bool {
        match self.client.get(format!("{}/health", BASE_URL)).send().await {
            Ok(response) => response.status() == StatusCode::OK,
            Err(_) => false,
        }
    }

    async fn register(&self, username: &str, email: &str, password: &str) -> Result<Value, String> {
        let response = self
            .client
            .post(format!("{}/register", BASE_URL))
            .json(&json!({
                "username": username,
                "email": email,
                "password": password,
                "reg_number": "REG123",
                "year_joined": 2023,
                "phone_number": "1234567890"
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
                "username": username,
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
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_system_health_check() {
    let client = TestClient::new();

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
    let client = TestClient::new();

    // Ensure server is ready
    assert!(client.health_check().await, "Server is not ready");

    // 1. Register a new user
    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    let register_response = client.register(&username, &email, password).await.unwrap();

    assert!(register_response["user"]["id"].is_string());
    assert_eq!(register_response["user"]["username"], username);
    assert_eq!(register_response["user"]["email"], email);

    let initial_access_token = register_response["auth"]["access_token"].as_str().unwrap();
    let initial_refresh_token = register_response["auth"]["refresh_token"].as_str().unwrap();
    let csrf_token = register_response["csrf_token"].as_str().unwrap();

    // 2. Get user info with access token
    let me_response = client.get_me(initial_access_token).await.unwrap();
    assert_eq!(me_response["username"], username);
    assert_eq!(me_response["email"], email);

    // 3. Refresh the access token
    let refresh_response = client
        .refresh_token(initial_refresh_token, csrf_token)
        .await
        .unwrap();
    let new_access_token = refresh_response["access_token"].as_str().unwrap();
    let new_refresh_token = refresh_response["refresh_token"].as_str().unwrap();

    assert_ne!(new_access_token, initial_access_token);
    assert_ne!(new_refresh_token, initial_refresh_token);

    // 4. Use new access token to get user info
    let me_response2 = client.get_me(new_access_token).await.unwrap();
    assert_eq!(me_response2["username"], username);

    // 5. Logout
    let logout_response = client.logout(new_access_token, csrf_token).await.unwrap();
    assert!(logout_response["message"].as_str().is_some());

    // 6. Try to use refresh token after logout (should fail)
    let refresh_after_logout = client.refresh_token(new_refresh_token, csrf_token).await;
    assert!(refresh_after_logout.is_err());
}

#[tokio::test]
#[ignore]
async fn test_system_login_flow() {
    let client = TestClient::new();

    assert!(client.health_check().await, "Server is not ready");

    // 1. Register a user
    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    client.register(&username, &email, password).await.unwrap();

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
    let client = TestClient::new();

    assert!(client.health_check().await, "Server is not ready");

    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    // First registration should succeed
    client.register(&username, &email, password).await.unwrap();

    // Second registration with same username should fail
    let duplicate = client
        .register(&username, &format!("other_{}", email), password)
        .await;
    assert!(duplicate.is_err());

    // Registration with same email should also fail
    let duplicate_email = client
        .register(&format!("other_{}", username), &email, password)
        .await;
    assert!(duplicate_email.is_err());
}

#[tokio::test]
#[ignore]
async fn test_system_csrf_protection() {
    let client = TestClient::new();

    assert!(client.health_check().await, "Server is not ready");

    // Register a user
    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    let register_response = client.register(&username, &email, password).await.unwrap();
    let access_token = register_response["auth"]["access_token"].as_str().unwrap();

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
    let client = TestClient::new();

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
    let client = TestClient::new();

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
    let client = TestClient::new();

    assert!(client.health_check().await, "Server is not ready");

    // Register multiple users concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let client = TestClient::new();
            tokio::spawn(async move {
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
    let client = TestClient::new();

    assert!(client.health_check().await, "Server is not ready");

    // Register a user
    let username = format!("testuser_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password = "securepassword123";

    let register_response = client.register(&username, &email, password).await.unwrap();
    let access_token = register_response["auth"]["access_token"].as_str().unwrap();

    // Access token should work immediately
    let me_response = client.get_me(access_token).await.unwrap();
    assert_eq!(me_response["username"], username);

    // Note: To test actual expiry, you would need to wait for token expiration
    // or use a test environment with very short token lifetimes
}

#[tokio::test]
#[ignore]
async fn test_system_password_security() {
    let client = TestClient::new();

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
