use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;
use chrono::{Duration, Utc};

use crate::{
    csrf::create_csrf_token,
    models::{AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest, UserResponse},
    security::{self, hash_password, verify_password},
    AppState,
};

/// Handler for user registration
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Validation error: {}", e)})),
        )
    })?;

    // Check if user already exists
    if let Some(_) = state
        .db
        .find_user_by_username(&payload.username)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "Username already exists"})),
        ));
    }

    // Check if email already exists
    if let Some(_) = state
        .db
        .find_user_by_email(&payload.email)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
    {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "Email already exists"})),
        ));
    }

    // Hash password with salt and pepper
    let (password_hash, salt) = hash_password(&payload.password, &state.config.password_pepper)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash password"})),
            )
        })?;

    // Create user
    let user = state
        .db
        .create_user(&payload.username, &payload.email, &password_hash, &salt)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create user"})),
            )
        })?;

    // Generate tokens
    let access_token = state
        .jwt_service
        .create_access_token(&user.id.to_string(), &user.username)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create access token"})),
            )
        })?;

    let refresh_token = state
        .jwt_service
        .create_refresh_token(&user.id.to_string(), &user.username)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create refresh token"})),
            )
        })?;

    // Hash and store refresh token
    let refresh_token_hash = security::hash_token(&refresh_token, &state.config.password_pepper)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash refresh token"})),
            )
        })?;

    let expires_at = Utc::now() + Duration::seconds(state.config.jwt_refresh_token_expiry);
    state
        .db
        .store_refresh_token(user.id, &refresh_token_hash, expires_at)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to store refresh token"})),
            )
        })?;

    // Generate CSRF token
    let csrf_token = create_csrf_token(&state.db, Some(user.id), state.config.csrf_token_expiry)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create CSRF token"})),
            )
        })?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_access_token_expiry,
    };

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "user": UserResponse::from(user),
            "auth": response,
            "csrf_token": csrf_token,
        })),
    ))
}

/// Handler for user login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Validation error: {}", e)})),
        )
    })?;

    // Find user by username
    let user = state
        .db
        .find_user_by_username(&payload.username)
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
                Json(json!({"error": "Invalid credentials"})),
            )
        })?;

    // Verify password
    let is_valid = verify_password(
        &payload.password,
        &user.password_hash,
        &state.config.password_pepper,
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Password verification failed"})),
        )
    })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        ));
    }

    // Generate tokens
    let access_token = state
        .jwt_service
        .create_access_token(&user.id.to_string(), &user.username)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create access token"})),
            )
        })?;

    let refresh_token = state
        .jwt_service
        .create_refresh_token(&user.id.to_string(), &user.username)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create refresh token"})),
            )
        })?;

    // Hash and store refresh token
    let refresh_token_hash = security::hash_token(&refresh_token, &state.config.password_pepper)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash refresh token"})),
            )
        })?;

    let expires_at = Utc::now() + Duration::seconds(state.config.jwt_refresh_token_expiry);
    state
        .db
        .store_refresh_token(user.id, &refresh_token_hash, expires_at)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to store refresh token"})),
            )
        })?;

    // Generate CSRF token
    let csrf_token = create_csrf_token(&state.db, Some(user.id), state.config.csrf_token_expiry)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create CSRF token"})),
            )
        })?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_access_token_expiry,
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "user": UserResponse::from(user),
            "auth": response,
            "csrf_token": csrf_token,
        })),
    ))
}

/// Handler for refreshing access token
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate refresh token
    let claims = state
        .jwt_service
        .validate_refresh_token(&payload.refresh_token)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid or expired refresh token"})),
            )
        })?;

    // Hash the refresh token to check against stored hash
    let refresh_token_hash = security::hash_token(&payload.refresh_token, &state.config.password_pepper)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash refresh token"})),
            )
        })?;

    // Verify refresh token exists in database
    let _stored_token = state
        .db
        .find_refresh_token(&refresh_token_hash)
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
                Json(json!({"error": "Refresh token not found or expired"})),
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

    // Delete old refresh token
    state
        .db
        .delete_refresh_token(&refresh_token_hash)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete old refresh token"})),
            )
        })?;

    // Generate new tokens
    let access_token = state
        .jwt_service
        .create_access_token(&user.id.to_string(), &user.username)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create access token"})),
            )
        })?;

    let new_refresh_token = state
        .jwt_service
        .create_refresh_token(&user.id.to_string(), &user.username)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create refresh token"})),
            )
        })?;

    // Hash and store new refresh token
    let new_refresh_token_hash = security::hash_token(&new_refresh_token, &state.config.password_pepper)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash refresh token"})),
            )
        })?;

    let expires_at = Utc::now() + Duration::seconds(state.config.jwt_refresh_token_expiry);
    state
        .db
        .store_refresh_token(user.id, &new_refresh_token_hash, expires_at)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to store refresh token"})),
            )
        })?;

    let response = AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_access_token_expiry,
    };

    Ok((StatusCode::OK, Json(json!(response))))
}

/// Handler for user logout
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Delete all refresh tokens for the user
    state
        .db
        .delete_user_refresh_tokens(user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete refresh tokens"})),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({"message": "Logged out successfully"})),
    ))
}

/// Handler to get current user info
pub async fn me(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
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
                StatusCode::NOT_FOUND,
                Json(json!({"error": "User not found"})),
            )
        })?;

    Ok((StatusCode::OK, Json(json!(UserResponse::from(user)))))
}

/// Handler to get a new CSRF token
pub async fn get_csrf_token(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let csrf_token = create_csrf_token(&state.db, None, state.config.csrf_token_expiry)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create CSRF token"})),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({"csrf_token": csrf_token})),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_validation() {
        let valid = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "securepassword123".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid_email = RegisterRequest {
            username: "testuser".to_string(),
            email: "invalid".to_string(),
            password: "securepassword123".to_string(),
        };
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        let valid = LoginRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = LoginRequest {
            username: "".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid.validate().is_err());
    }
}
