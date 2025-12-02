use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{models::Claims, AppState};

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
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid or expired token"})),
        )
    })?;

    let claims = token_data.claims;

    // Verify it's an access token
    if claims.token_type != "access" {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid token type"})),
        ));
    }

    // Parse user_id from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid user ID in token"})),
        )
    })?;

    // Add user_id and username to request extensions
    request.extensions_mut().insert(user_id);
    request.extensions_mut().insert(claims.username);

    Ok(next.run(request).await)
}

/// Optional auth middleware - extracts user info if valid token present, but allows anonymous access
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers();

    // Try to extract token from Authorization header
    if let Some(auth_header) = headers.get("Authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Try to validate the token
            let mut validation = Validation::default();
            validation.validate_exp = true;

            if let Ok(token_data) = decode::<Claims>(
                token,
                &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
                &validation,
            ) {
                let claims = token_data.claims;

                // Only use if it's an access token
                if claims.token_type == "access" {
                    if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                        request.extensions_mut().insert(user_id);
                        request.extensions_mut().insert(claims.username);
                    }
                }
            }
        }
    }

    next.run(request).await
}

/// Admin middleware - requires authenticated user to be an admin
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

    let token = &auth_header[7..];

    // Validate the access token
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid or expired token"})),
        )
    })?;

    let claims = token_data.claims;

    if claims.token_type != "access" {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid token type"})),
        ));
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid user ID in token"})),
        )
    })?;

    // Check if user is admin
    let is_admin = state.db.is_user_admin(user_id).await.map_err(|e| {
        tracing::error!("Database error checking admin status: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Database error: {}", e)})),
        )
    })?;

    if !is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin access required"})),
        ));
    }

    request.extensions_mut().insert(user_id);
    request.extensions_mut().insert(claims.username);

    Ok(next.run(request).await)
}

/// Adjudicator middleware - requires user to be allocated as an adjudicator for the match
pub async fn adjudicator_middleware(
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

    if !auth_header.starts_with("Bearer ") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid authorization header format"})),
        ));
    }

    let token = &auth_header[7..];

    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid or expired token"})),
        )
    })?;

    let claims = token_data.claims;

    if claims.token_type != "access" {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid token type"})),
        ));
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid user ID in token"})),
        )
    })?;

    request.extensions_mut().insert(user_id);
    request.extensions_mut().insert(claims.username);

    Ok(next.run(request).await)
}
