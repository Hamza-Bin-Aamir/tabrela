use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use rand::{distributions::Alphanumeric, Rng};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{database::Database, AppState};

const CSRF_TOKEN_HEADER: &str = "X-CSRF-Token";
const CSRF_TOKEN_LENGTH: usize = 32;

/// Generate a random CSRF token
pub fn generate_csrf_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(CSRF_TOKEN_LENGTH)
        .map(char::from)
        .collect()
}

/// Middleware to validate CSRF tokens on state-changing requests
pub async fn csrf_protection_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let method = request.method().clone();

    // Only check CSRF for state-changing methods
    if !matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") {
        return Ok(next.run(request).await);
    }

    // Skip CSRF check for login and register endpoints
    let path = request.uri().path();
    if path.ends_with("/register") 
        || path.ends_with("/login")
        || path.ends_with("/verify-email")
        || path.ends_with("/verify-otp")
        || path.ends_with("/resend-verification")
        || path.ends_with("/request-password-reset")
        || path.ends_with("/reset-password")
        || path.ends_with("/csrf-token")
    {
        return Ok(next.run(request).await);
    }

    // Extract CSRF token from headers
    let csrf_token = headers
        .get(CSRF_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "CSRF token missing"})),
            )
        })?;

    // Validate CSRF token
    let is_valid = state
        .db
        .validate_csrf_token(csrf_token)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to validate CSRF token"})),
            )
        })?
        .is_some();

    if !is_valid {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Invalid or expired CSRF token"})),
        ));
    }

    Ok(next.run(request).await)
}

/// Create a CSRF token for a user
pub async fn create_csrf_token(
    db: &Database,
    user_id: Option<Uuid>,
    expiry_seconds: i64,
) -> Result<String, sqlx::Error> {
    let token = generate_csrf_token();
    db.create_csrf_token(&token, user_id, expiry_seconds)
        .await?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_csrf_token() {
        let token1 = generate_csrf_token();
        let token2 = generate_csrf_token();

        assert_eq!(token1.len(), CSRF_TOKEN_LENGTH);
        assert_eq!(token2.len(), CSRF_TOKEN_LENGTH);
        assert_ne!(token1, token2); // Should generate different tokens

        // Should only contain alphanumeric characters
        assert!(token1.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_csrf_token_uniqueness() {
        let tokens: Vec<String> = (0..100).map(|_| generate_csrf_token()).collect();
        let unique_tokens: std::collections::HashSet<_> = tokens.iter().collect();

        // All tokens should be unique
        assert_eq!(tokens.len(), unique_tokens.len());
    }
}
