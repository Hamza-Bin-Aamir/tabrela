use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

use crate::{
    csrf::create_csrf_token,
    models::{
        AuthResponse, LoginRequest, RefreshTokenRequest, RegisterRequest, RequestPasswordResetRequest,
        ResendVerificationRequest, ResetPasswordRequest, UserResponse, VerifyEmailRequest,
    },
    security::{self, hash_password, verify_password},
    AppState,
};

/// Format validation errors into human-readable messages with expected formats
fn format_validation_error(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = match field {
                "username" => match error.code.as_ref() {
                    "length" => "Username must be between 3 and 50 characters".to_string(),
                    _ => "Invalid username format".to_string(),
                },
                "email" => "Invalid email format. Expected: user@example.com".to_string(),
                "password" => "Password must be at least 8 characters long".to_string(),
                "reg_number" => "Invalid registration number. Expected format: 20XXXXX (e.g., 2012345)".to_string(),
                "year_joined" => "Year joined must be between 2000 and 2099. Expected format: 20XX (e.g., 2023)".to_string(),
                "phone_number" => "Invalid phone number format. Expected: +[country code][number] (e.g., +923001234567)".to_string(),
                "username_or_email" => "Username or email is required".to_string(),
                "otp" => "OTP must be exactly 6 digits".to_string(),
                "new_password" => "New password must be at least 8 characters long".to_string(),
                _ => format!("Invalid value for field '{}'", field),
            };
            messages.push(message);
        }
    }
    
    if messages.is_empty() {
        "Validation error".to_string()
    } else {
        messages.join(". ")
    }
}

/// Format database errors into human-readable messages
fn format_database_error(error: &sqlx::Error) -> String {
    match error {
        sqlx::Error::Database(db_err) => {
            let constraint = db_err.constraint().unwrap_or("");
            let message = db_err.message();
            
            // Check for constraint violations
            if constraint.contains("users_username_key") || message.contains("users_username_key") {
                return "Username already exists. Please choose a different username.".to_string();
            }
            if constraint.contains("users_email_key") || message.contains("users_email_key") {
                return "Email already exists. Please use a different email address.".to_string();
            }
            if constraint.contains("users_phone_number_unique") || message.contains("users_phone_number_unique") {
                return "Phone number already exists. Please use a different phone number.".to_string();
            }
            if constraint.contains("users_reg_number_unique") || message.contains("users_reg_number_unique") {
                return "Registration number already exists. Please check your registration number.".to_string();
            }
            if message.contains("year_joined") && message.contains("check") {
                return "Year joined must be between 2000 and 2099. Expected format: 20XX (e.g., 2023)".to_string();
            }
            if message.contains("reg_number") && message.contains("check") {
                return "Invalid registration number format. Expected: 20XXXXX (e.g., 2012345)".to_string();
            }
            if message.contains("phone_number") && message.contains("check") {
                return "Invalid phone number format. Expected: +[country code][number] (e.g., +923001234567)".to_string();
            }
            
            "Database error occurred. Please try again.".to_string()
        }
        _ => "Database error occurred. Please try again.".to_string(),
    }
}

/// Handler for user registration
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format_validation_error(&e)})),
        )
    })?;

    // Check if username already exists
    if let Some(existing_user) = state
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
        // If username exists and email is verified, it's a conflict
        if existing_user.email_verified {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({"error": "Username already exists"})),
            ));
        }
        // If username exists but email not verified, delete the old account
        // (user is re-registering with the same username)
        state
            .db
            .delete_user_by_id(existing_user.id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Database error"})),
                )
            })?;
    }

    // Check if email already exists
    if let Some(existing_user) = state
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
        // If email exists and is verified, it's a conflict
        if existing_user.email_verified {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({"error": "Email already exists"})),
            ));
        }
        // If email exists but not verified, delete the old account
        // (user is re-registering, maybe with different username)
        state
            .db
            .delete_user_by_id(existing_user.id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Database error"})),
                )
            })?;
    }

    // Check if phone number already exists
    if let Some(existing_user) = state
        .db
        .find_user_by_phone(&payload.phone_number)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
    {
        // If phone exists and is verified, it's a conflict
        if existing_user.email_verified {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({"error": "Phone number already exists"})),
            ));
        }
        // If phone exists but email not verified, delete the old account
        state
            .db
            .delete_user_by_id(existing_user.id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Database error"})),
                )
            })?;
    }

    // Check if registration number already exists
    if let Some(existing_user) = state
        .db
        .find_user_by_reg_number(&payload.reg_number)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
    {
        // If reg_number exists and is verified, it's a conflict
        if existing_user.email_verified {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({"error": "Registration number already exists"})),
            ));
        }
        // If reg_number exists but email not verified, delete the old account
        state
            .db
            .delete_user_by_id(existing_user.id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Database error"})),
                )
            })?;
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
        .create_user(
            &payload.username,
            &payload.email,
            &password_hash,
            &salt,
            &payload.reg_number,
            payload.year_joined,
            &payload.phone_number,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format_database_error(&e)})),
            )
        })?;

    // Generate OTP for email verification
    let otp = security::generate_otp();
    state
        .db
        .create_email_verification_otp(
            user.id,
            &otp,
            state.config.email_verification_expiry,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create verification OTP"})),
            )
        })?;

    // Send verification email (don't fail registration if email fails)
    if let Err(e) = state
        .email_client
        .send_verification_email(&user.email, &user.username, &otp)
        .await
    {
        tracing::error!("Failed to send verification email: {}", e);
    }

    // Return success without tokens - user must verify email first
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Registration successful. Please check your email for the verification code.",
            "email": user.email,
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
            Json(json!({"error": format_validation_error(&e)})),
        )
    })?;

    // Try to find user by username first, then by email
    let user = if payload.username_or_email.contains('@') {
        // Looks like an email
        state
            .db
            .find_user_by_email(&payload.username_or_email)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Database error"})),
                )
            })?
    } else {
        // Looks like a username
        state
            .db
            .find_user_by_username(&payload.username_or_email)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Database error"})),
                )
            })?
    }
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

    // Check if email is verified
    if !user.email_verified {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Please verify your email before logging in"})),
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
    let refresh_token_hash =
        security::hash_token(&payload.refresh_token, &state.config.password_pepper).map_err(
            |_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to hash refresh token"})),
                )
            },
        )?;

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
    let new_refresh_token_hash =
        security::hash_token(&new_refresh_token, &state.config.password_pepper).map_err(|_| {
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

    Ok((StatusCode::OK, Json(json!({"csrf_token": csrf_token}))))
}

/// Handler to verify email address
pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyEmailRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format_validation_error(&e)})),
        )
    })?;

    // Find user by email
    let user = state
        .db
        .find_user_by_email(&payload.email)
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

    // Verify the OTP
    let result = state
        .db
        .verify_email_otp(user.id, &payload.otp)
        .await
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("expired") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "OTP has expired. Please request a new one."})),
                )
            } else if error_msg.contains("attempts") {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "Too many failed attempts. Please request a new OTP."})),
                )
            } else if error_msg.contains("Invalid") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid OTP"})),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to verify OTP"})),
                )
            }
        })?;

    if !result {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid OTP"})),
        ));
    }

    // Get the updated user
    let user = state
        .db
        .find_user_by_id(user.id)
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

    // Send welcome email (don't fail if email fails)
    if let Err(e) = state
        .email_client
        .send_welcome_email(&user.email, &user.username)
        .await
    {
        tracing::error!("Failed to send welcome email: {}", e);
    }

    // Generate tokens for the verified user
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
            "message": "Email verified successfully",
            "user": UserResponse::from(user),
            "auth": response,
            "csrf_token": csrf_token,
        })),
    ))
}

/// Handler to resend verification email
pub async fn resend_verification(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format_validation_error(&e)})),
        )
    })?;

    // Find user by email
    let user = state
        .db
        .find_user_by_email(&payload.email)
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

    // Check if already verified
    if user.email_verified {
        return Ok((
            StatusCode::OK,
            Json(json!({"message": "Email already verified"})),
        ));
    }

    // Check if there's a recent OTP (rate limiting)
    if let Ok(Some(existing_otp)) = state.db.find_email_verification_otp_by_user(user.id).await {
        let time_since_last = Utc::now().signed_duration_since(existing_otp.last_sent_at);
        if time_since_last.num_seconds() < 60 {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": "Please wait before requesting a new OTP"})),
            ));
        }
    }

    // Generate new OTP
    let otp = security::generate_otp();
    state
        .db
        .create_email_verification_otp(
            user.id,
            &otp,
            state.config.email_verification_expiry,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create verification OTP"})),
            )
        })?;

    // Send verification email
    state
        .email_client
        .send_verification_email(&user.email, &user.username, &otp)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to send verification email"})),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({"message": "Verification OTP sent"})),
    ))
}

/// Handler to request password reset
pub async fn request_password_reset(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RequestPasswordResetRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format_validation_error(&e)})),
        )
    })?;

    // Find user by email
    let user = state
        .db
        .find_user_by_email(&payload.email)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    // Always return success even if user doesn't exist (security best practice)
    if user.is_none() {
        return Ok((
            StatusCode::OK,
            Json(json!({"message": "If the email exists, a password reset OTP has been sent"})),
        ));
    }

    let user = user.unwrap();

    // Generate password reset OTP (6 digits, stored directly)
    let otp = security::generate_otp();
    
    state
        .db
        .create_password_reset_otp(
            user.id, 
            &user.email,
            &otp, 
            state.config.password_reset_expiry
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create reset OTP"})),
            )
        })?;

    // Send password reset email (don't fail if email fails)
    if let Err(e) = state
        .email_client
        .send_password_reset_email(&user.email, &user.username, &otp)
        .await
    {
        tracing::error!("Failed to send password reset email: {}", e);
    }

    Ok((
        StatusCode::OK,
        Json(json!({"message": "If the email exists, a password reset OTP has been sent"})),
    ))
}

/// Handler to reset password with OTP
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate input
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format_validation_error(&e)})),
        )
    })?;

    // Find the reset token by email
    let token_record = state
        .db
        .find_password_reset_by_email(&payload.email)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid or expired OTP"})),
            )
        })?;

    // Check if too many attempts
    if token_record.attempts >= 5 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "Too many attempts. Please request a new OTP."})),
        ));
    }

    // Verify OTP (direct comparison, no hashing)
    if payload.otp != token_record.otp {
        // Increment attempts
        let new_attempts = state
            .db
            .increment_password_reset_attempts(&payload.email)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Database error"})),
                )
            })?;

        let remaining = 5 - new_attempts;
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid OTP",
                "attempts_remaining": remaining.max(0)
            })),
        ));
    }

    // Hash new password
    let (new_password_hash, new_salt) =
        hash_password(&payload.new_password, &state.config.password_pepper).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash password"})),
            )
        })?;

    // Update user password
    state
        .db
        .update_user_password(token_record.user_id, &new_password_hash, &new_salt)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update password"})),
            )
        })?;

    // Mark OTP as used
    state
        .db
        .mark_password_reset_used(&payload.email)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to mark OTP as used"})),
            )
        })?;

    // Delete all refresh tokens for the user (log them out everywhere)
    state
        .db
        .delete_user_refresh_tokens(token_record.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete refresh tokens"})),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({"message": "Password reset successfully"})),
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
            reg_number: "2012345".to_string(),  // Valid format: 20XXXXX
            year_joined: 2023,  // Valid year between 2000-2099
            phone_number: "+923001234567".to_string(),  // Valid format with country code
        };
        assert!(valid.validate().is_ok());

        let invalid_email = RegisterRequest {
            username: "testuser".to_string(),
            email: "invalid".to_string(),
            password: "securepassword123".to_string(),
            reg_number: "2012345".to_string(),
            year_joined: 2023,
            phone_number: "+923001234567".to_string(),
        };
        assert!(invalid_email.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        let valid = LoginRequest {
            username_or_email: "testuser".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = LoginRequest {
            username_or_email: "".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid.validate().is_err());
    }
}
