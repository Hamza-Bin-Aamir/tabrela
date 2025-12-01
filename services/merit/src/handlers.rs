use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{
        AdminMeritListResponse, AdminProfileResponse, AwardListResponse, AwardResponse,
        AwardHistoryResponse, CreateAwardRequest, EditAwardRequest, MeritHistoryQuery,
        MeritHistoryResponse, MeritResponse, PrivateProfileResponse, PublicProfileResponse,
        UpdateMeritRequest, UpgradeAwardRequest,
    },
    AppState,
};

// ============================================================================
// Profile Handlers
// ============================================================================

/// Get public profile by username - accessible by anyone (including unauthenticated users)
/// Returns limited info; merit is only visible to self or admins
pub async fn get_profile_by_username(
    State(state): State<Arc<AppState>>,
    current_user_id: Option<Extension<Uuid>>,
    Path(username): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let profile = state
        .db
        .get_user_by_username(&username)
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

    // Check if we have an authenticated user
    let current_user_id = current_user_id.map(|Extension(id)| id);
    
    // Check if the current user is viewing their own profile
    let is_own_profile = current_user_id.map_or(false, |id| profile.id == id);

    // Check if current user is admin
    let is_admin = match current_user_id {
        Some(id) => state.db.is_user_admin(id).await.unwrap_or(false),
        None => false,
    };

    if is_admin {
        // Admin can see everything including merit and admin status
        Ok(Json(json!(AdminProfileResponse {
            id: profile.id,
            username: profile.username,
            email: profile.email,
            reg_number: profile.reg_number,
            year_joined: profile.year_joined,
            phone_number: profile.phone_number,
            email_verified: profile.email_verified,
            merit_points: profile.merit_points,
            is_admin: profile.is_admin,
            created_at: profile.created_at,
        })))
    } else if is_own_profile {
        // User viewing their own profile - can see their own merit
        Ok(Json(json!(PrivateProfileResponse {
            id: profile.id,
            username: profile.username,
            email: profile.email,
            reg_number: profile.reg_number,
            year_joined: profile.year_joined,
            phone_number: profile.phone_number,
            email_verified: profile.email_verified,
            merit_points: profile.merit_points,
            created_at: profile.created_at,
        })))
    } else {
        // Someone else viewing the profile - public info only, no merit
        Ok(Json(json!(PublicProfileResponse {
            id: profile.id,
            username: profile.username,
            year_joined: profile.year_joined,
            created_at: profile.created_at,
        })))
    }
}

// ============================================================================
// Merit Handlers (for own merit)
// ============================================================================

/// Get own merit points
pub async fn get_my_merit(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<MeritResponse>, (StatusCode, Json<Value>)> {
    let merit = state
        .db
        .get_user_merit(user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    match merit {
        Some(m) => Ok(Json(MeritResponse {
            user_id: m.user_id,
            merit_points: m.merit_points,
            updated_at: m.updated_at,
        })),
        None => {
            // Initialize merit if not exists
            let m = state
                .db
                .initialize_user_merit(user_id)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to initialize merit"})),
                    )
                })?;
            Ok(Json(MeritResponse {
                user_id: m.user_id,
                merit_points: m.merit_points,
                updated_at: m.updated_at,
            }))
        }
    }
}

/// Get own merit history
pub async fn get_my_merit_history(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Query(query): Query<MeritHistoryQuery>,
) -> Result<Json<MeritHistoryResponse>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let (history, total) = state
        .db
        .get_merit_history(user_id, page, per_page)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    Ok(Json(MeritHistoryResponse {
        history,
        total,
        page,
        per_page,
        total_pages,
    }))
}

// ============================================================================
// Admin Merit Handlers
// ============================================================================

/// Update merit for a user (admin only)
pub async fn admin_update_merit(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Json(payload): Json<UpdateMeritRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate request
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    // Prevent changing own merit
    if payload.user_id == admin_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Cannot modify your own merit points"})),
        ));
    }

    // Verify target user exists
    let target_user = state
        .db
        .get_user_by_id(payload.user_id)
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

    // Update merit
    let (updated_merit, history) = state
        .db
        .update_merit(payload.user_id, admin_id, payload.change_amount, &payload.reason)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update merit"})),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Merit updated successfully",
            "user_id": payload.user_id,
            "username": target_user.username,
            "previous_merit": history.previous_total,
            "new_merit": updated_merit.merit_points,
            "change_amount": payload.change_amount,
            "reason": payload.reason
        })),
    ))
}

/// Get merit for any user (admin only)
pub async fn admin_get_user_merit(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user = state
        .db
        .get_user_by_id(user_id)
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

    Ok(Json(json!({
        "user_id": user.id,
        "username": user.username,
        "merit_points": user.merit_points,
        "is_admin": user.is_admin
    })))
}

/// Get merit history for any user (admin only)
pub async fn admin_get_user_merit_history(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<MeritHistoryQuery>,
) -> Result<Json<MeritHistoryResponse>, (StatusCode, Json<Value>)> {
    // Verify user exists
    state
        .db
        .get_user_by_id(user_id)
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

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let (history, total) = state
        .db
        .get_merit_history(user_id, page, per_page)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    Ok(Json(MeritHistoryResponse {
        history,
        total,
        page,
        per_page,
        total_pages,
    }))
}

/// List all users with their merit (admin only)
pub async fn admin_list_all_merits(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MeritHistoryQuery>,
) -> Result<Json<AdminMeritListResponse>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 100);

    let (users, total) = state
        .db
        .list_all_user_merits(page, per_page)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    Ok(Json(AdminMeritListResponse {
        users,
        total,
        page,
        per_page,
        total_pages,
    }))
}

// ============================================================================
// Award Handlers
// ============================================================================

/// Get all awards for a user (public - shown on profile)
pub async fn get_user_awards(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<Json<AwardListResponse>, (StatusCode, Json<Value>)> {
    // Find user by username
    let user = state
        .db
        .get_user_by_username(&username)
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

    let awards = state.db.get_user_awards(user.id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Database error"})),
        )
    })?;

    let award_responses: Vec<AwardResponse> = awards
        .into_iter()
        .map(|a| AwardResponse {
            id: a.id,
            title: a.title,
            description: a.description,
            tier: a.tier,
            awarded_at: a.awarded_at,
        })
        .collect();

    let total = award_responses.len() as i64;
    Ok(Json(AwardListResponse {
        awards: award_responses,
        total,
    }))
}

/// Create a new award for a user (admin only)
pub async fn admin_create_award(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Json(payload): Json<CreateAwardRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate request
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    // Verify target user exists
    let target_user = state
        .db
        .get_user_by_id(payload.user_id)
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

    // Create the award
    let (award, _history) = state
        .db
        .create_award(
            payload.user_id,
            admin_id,
            &payload.title,
            payload.description.as_deref(),
            payload.tier,
            &payload.reason,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create award"})),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Award created successfully",
            "award": {
                "id": award.id,
                "user_id": award.user_id,
                "username": target_user.username,
                "title": award.title,
                "description": award.description,
                "tier": award.tier,
                "awarded_at": award.awarded_at
            }
        })),
    ))
}

/// Get a specific award with details (admin only)
pub async fn admin_get_award(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let award = state
        .db
        .get_award_with_admin(award_id)
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
                Json(json!({"error": "Award not found"})),
            )
        })?;

    // Get user info
    let user = state.db.get_user_by_id(award.user_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Database error"})),
        )
    })?;

    let username = user.map(|u| u.username).unwrap_or_else(|| "Unknown".to_string());

    Ok(Json(json!({
        "id": award.id,
        "user_id": award.user_id,
        "username": username,
        "title": award.title,
        "description": award.description,
        "tier": award.tier,
        "awarded_by": award.awarded_by,
        "awarded_by_username": award.awarded_by_username,
        "awarded_at": award.awarded_at,
        "created_at": award.created_at,
        "updated_at": award.updated_at
    })))
}

/// Upgrade an award's tier (admin only)
pub async fn admin_upgrade_award(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Path(award_id): Path<Uuid>,
    Json(payload): Json<UpgradeAwardRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate request
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    // Get current award
    let current_award = state
        .db
        .get_award_by_id(award_id)
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
                Json(json!({"error": "Award not found"})),
            )
        })?;

    // Check if upgrade is valid
    if !current_award.tier.can_upgrade_to(&payload.new_tier) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!(
                    "Cannot upgrade from {} to {}. Can only upgrade to a higher tier.",
                    current_award.tier, payload.new_tier
                )
            })),
        ));
    }

    // Perform the upgrade
    let (updated_award, _history) = state
        .db
        .upgrade_award(
            award_id,
            admin_id,
            payload.new_tier,
            payload.new_title.as_deref(),
            &payload.reason,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to upgrade award"})),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Award upgraded successfully",
            "award": {
                "id": updated_award.id,
                "title": updated_award.title,
                "previous_tier": current_award.tier,
                "new_tier": updated_award.tier
            }
        })),
    ))
}

/// Edit an award (admin only)
pub async fn admin_edit_award(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
    Json(payload): Json<EditAwardRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate request
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Validation error: {}", e)})),
        )
    })?;

    // Verify award exists
    state
        .db
        .get_award_by_id(award_id)
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
                Json(json!({"error": "Award not found"})),
            )
        })?;

    // Update the award
    let updated_award = state
        .db
        .edit_award(
            award_id,
            &payload.title,
            payload.description.as_deref(),
            payload.tier,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update award"})),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Award updated successfully",
            "award": {
                "id": updated_award.id,
                "title": updated_award.title,
                "description": updated_award.description,
                "tier": updated_award.tier
            }
        })),
    ))
}

/// Get award history (admin only)
pub async fn admin_get_award_history(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
) -> Result<Json<AwardHistoryResponse>, (StatusCode, Json<Value>)> {
    // Verify award exists
    state
        .db
        .get_award_by_id(award_id)
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
                Json(json!({"error": "Award not found"})),
            )
        })?;

    let history = state.db.get_award_history(award_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Database error"})),
        )
    })?;

    Ok(Json(AwardHistoryResponse {
        history: history.clone(),
        total: history.len() as i64,
    }))
}

/// Delete an award (admin only)
pub async fn admin_delete_award(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Verify award exists
    state
        .db
        .get_award_by_id(award_id)
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
                Json(json!({"error": "Award not found"})),
            )
        })?;

    state.db.delete_award(award_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to delete award"})),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!({"message": "Award deleted successfully"})),
    ))
}

// ============================================================================
// Additional Award Handlers for Routes
// ============================================================================

/// Get user awards - public endpoint (alias for get_user_awards)
pub async fn get_user_awards_public(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<Json<AwardListResponse>, (StatusCode, Json<Value>)> {
    get_user_awards(State(state), Path(username)).await
}

/// Get my own awards (authenticated user)
pub async fn get_my_awards(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<AwardListResponse>, (StatusCode, Json<Value>)> {
    let awards = state.db.get_user_awards(user_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Database error"})),
        )
    })?;

    let award_responses: Vec<AwardResponse> = awards
        .into_iter()
        .map(|a| AwardResponse {
            id: a.id,
            title: a.title,
            description: a.description,
            tier: a.tier,
            awarded_at: a.awarded_at,
        })
        .collect();

    let total = award_responses.len() as i64;
    Ok(Json(AwardListResponse {
        awards: award_responses,
        total,
    }))
}

/// Get my own award history (authenticated user)
pub async fn get_my_awards_history(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<AwardHistoryResponse>, (StatusCode, Json<Value>)> {
    let history = state.db.get_user_award_history(user_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Database error"})),
        )
    })?;

    Ok(Json(AwardHistoryResponse {
        history: history.clone(),
        total: history.len() as i64,
    }))
}

/// List all awards (admin only)
pub async fn admin_list_all_awards(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MeritHistoryQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * per_page;

    let (awards, total) = state
        .db
        .list_all_awards(per_page, offset)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    Ok(Json(json!({
        "awards": awards,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": total_pages
    })))
}
