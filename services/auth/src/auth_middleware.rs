use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

/// Middleware to authenticate requests using JWT access tokens
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let headers = request.headers();

    // Extract token from Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing authorization header"})),
            )
        })?;

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid authorization header format"})),
        ));
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Validate the access token
    let claims = state
        .jwt_service
        .validate_access_token(token)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid or expired token"})),
            )
        })?;

    // Parse user_id from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid user ID in token"})),
        )
    })?;

    // Verify user exists
    let user = state
        .db
        .find_user_by_id(user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            )
        })?;

    // Add user_id to request extensions for use in handlers
    request.extensions_mut().insert(user_id);
    request.extensions_mut().insert(user.username.clone());

    Ok(next.run(request).await)
}

/// Middleware to authenticate requests and verify admin privileges
pub async fn admin_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let headers = request.headers();

    // Extract token from Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing authorization header"})),
            )
        })?;

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid authorization header format"})),
        ));
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Validate the access token
    let claims = state
        .jwt_service
        .validate_access_token(token)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid or expired token"})),
            )
        })?;

    // Parse user_id from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid user ID in token"})),
        )
    })?;

    // Verify user exists
    let user = state
        .db
        .find_user_by_id(user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            )
        })?;

    // Check if user is an admin
    let is_admin = state.db.is_user_admin(user_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Database error"})),
        )
    })?;

    if !is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin access required"})),
        ));
    }

    // Add user_id to request extensions for use in handlers
    request.extensions_mut().insert(user_id);
    request.extensions_mut().insert(user.username.clone());

    Ok(next.run(request).await)
}

/// Extract authenticated user ID from request extensions
pub fn extract_user_id(headers: &HeaderMap) -> Option<Uuid> {
    headers
        .get("X-User-Id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_id() {
        let mut headers = HeaderMap::new();
        let user_id = Uuid::new_v4();

        headers.insert("X-User-Id", user_id.to_string().parse().unwrap());

        let extracted = extract_user_id(&headers);
        assert_eq!(extracted, Some(user_id));
    }

    #[test]
    fn test_extract_user_id_missing() {
        let headers = HeaderMap::new();
        let extracted = extract_user_id(&headers);
        assert_eq!(extracted, None);
    }

    #[test]
    fn test_extract_user_id_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert("X-User-Id", "invalid-uuid".parse().unwrap());

        let extracted = extract_user_id(&headers);
        assert_eq!(extracted, None);
    }
}
