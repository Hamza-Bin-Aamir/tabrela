use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    database::UpdateAllocationParams,
    models::{
        AdjudicatorResponse, Allocation, AllocationHistory, AllocationHistoryResponse,
        AllocationPoolResponse, AllocationRole, Ballot, BallotResponse, CheckedInUserResponse,
        CreateAllocationRequest, CreateMatchRequest, CreateSeriesRequest, CurrentAllocationInfo,
        Match, MatchListQuery, MatchListResponse, MatchResponse, MatchSeries, MatchStatus,
        MatchTeamResponse, PerformanceQuery, PerformanceResponse, RankingCount,
        ReleaseToggleRequest, ResourceResponse, SeriesListQuery, SeriesListResponse,
        SeriesResponse, SpeakerResponse, SpeakerScore, SpeakerScoreResponse, SubmitBallotRequest,
        SubmitFeedbackRequest, SwapAllocationRequest, TeamFormat, TeamRanking, TeamRankingResponse,
        UpdateAllocationRequest, UpdateMatchRequest, UpdateSeriesRequest, UpdateTeamRequest,
    },
    AppState,
};

// ============================================================================
// Match Series Handlers
// ============================================================================

/// Create a new match series (admin only) - FR-01
pub async fn create_series(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Json(payload): Json<CreateSeriesRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    // Verify event exists
    let event = state
        .db
        .get_event_by_id(payload.event_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking event: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            )
        })?;

    if event.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Event not found"})),
        ));
    }

    let now = Utc::now();
    let series = MatchSeries {
        id: Uuid::new_v4(),
        event_id: payload.event_id,
        name: payload.name,
        description: payload.description,
        round_number: payload.round_number,
        team_format: payload.team_format,
        allow_reply_speeches: payload.allow_reply_speeches,
        is_break_round: payload.is_break_round,
        created_by: admin_id,
        created_at: now,
        updated_at: now,
    };

    let created = state.db.create_series(&series).await.map_err(|e| {
        tracing::error!("Database error creating series: {:?}", e);
        if e.to_string().contains("unique_event_round") {
            (
                StatusCode::CONFLICT,
                Json(json!({"error": "Round number already exists for this event"})),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to create series: {}", e)})),
            )
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Series created successfully",
            "series": created
        })),
    ))
}

/// List series for an event
pub async fn list_series(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SeriesListQuery>,
) -> Result<Json<SeriesListResponse>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let (series_list, total) = state
        .db
        .list_series_by_event(query.event_id, page, per_page)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    let mut series_responses = Vec::new();
    for s in series_list {
        let match_count = state.db.get_series_match_count(s.id).await.unwrap_or(0);
        series_responses.push(SeriesResponse {
            id: s.id,
            event_id: s.event_id,
            name: s.name,
            description: s.description,
            round_number: s.round_number,
            team_format: s.team_format,
            allow_reply_speeches: s.allow_reply_speeches,
            is_break_round: s.is_break_round,
            match_count,
            created_at: s.created_at,
            updated_at: s.updated_at,
        });
    }

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    Ok(Json(SeriesListResponse {
        series: series_responses,
        total,
        page,
        per_page,
        total_pages,
    }))
}

/// Get a single series by ID
pub async fn get_series(
    State(state): State<Arc<AppState>>,
    Path(series_id): Path<Uuid>,
) -> Result<Json<SeriesResponse>, (StatusCode, Json<Value>)> {
    let series = state
        .db
        .get_series_by_id(series_id)
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
                Json(json!({"error": "Series not found"})),
            )
        })?;

    let match_count = state
        .db
        .get_series_match_count(series.id)
        .await
        .unwrap_or(0);

    Ok(Json(SeriesResponse {
        id: series.id,
        event_id: series.event_id,
        name: series.name,
        description: series.description,
        round_number: series.round_number,
        team_format: series.team_format,
        allow_reply_speeches: series.allow_reply_speeches,
        is_break_round: series.is_break_round,
        match_count,
        created_at: series.created_at,
        updated_at: series.updated_at,
    }))
}

/// Update a series (admin only)
pub async fn update_series(
    State(state): State<Arc<AppState>>,
    Path(series_id): Path<Uuid>,
    Json(payload): Json<UpdateSeriesRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify series exists
    state
        .db
        .get_series_by_id(series_id)
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
                Json(json!({"error": "Series not found"})),
            )
        })?;

    let updated = state
        .db
        .update_series(
            series_id,
            payload.name.as_deref(),
            payload.description.as_deref(),
            payload.allow_reply_speeches,
            payload.is_break_round,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update series"})),
            )
        })?;

    Ok(Json(json!({
        "message": "Series updated successfully",
        "series": updated
    })))
}

/// Delete a series (admin only)
pub async fn delete_series(
    State(state): State<Arc<AppState>>,
    Path(series_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify series exists
    state
        .db
        .get_series_by_id(series_id)
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
                Json(json!({"error": "Series not found"})),
            )
        })?;

    state.db.delete_series(series_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to delete series"})),
        )
    })?;

    Ok(Json(json!({"message": "Series deleted successfully"})))
}

// ============================================================================
// Match Handlers
// ============================================================================

/// Create a new match (admin only) - FR-01, FR-02
pub async fn create_match(
    State(state): State<Arc<AppState>>,
    Extension(_admin_id): Extension<Uuid>,
    Json(payload): Json<CreateMatchRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    // Get series to determine team format
    let series = state
        .db
        .get_series_by_id(payload.series_id)
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
                Json(json!({"error": "Series not found"})),
            )
        })?;

    let now = Utc::now();
    let match_record = Match {
        id: Uuid::new_v4(),
        series_id: payload.series_id,
        room_name: payload.room_name,
        motion: payload.motion,
        info_slide: payload.info_slide,
        status: MatchStatus::Draft,
        scheduled_time: payload.scheduled_time,
        scores_released: false,
        rankings_released: false,
        created_at: now,
        updated_at: now,
    };

    let created = state.db.create_match(&match_record).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to create match"})),
        )
    })?;

    // Create teams based on format (FR-03, FR-04)
    let teams = state
        .db
        .create_teams_for_match(created.id, series.team_format)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create teams for match"})),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Match created successfully",
            "match": created,
            "teams": teams
        })),
    ))
}

/// List matches
pub async fn list_matches(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MatchListQuery>,
) -> Result<Json<MatchListResponse>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let (matches, total) = if let Some(series_id) = query.series_id {
        state
            .db
            .list_matches_by_series(series_id, page, per_page)
            .await
    } else if let Some(event_id) = query.event_id {
        state
            .db
            .list_matches_by_event(event_id, query.status, page, per_page)
            .await
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Must provide series_id or event_id"})),
        ));
    }
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Database error"})),
        )
    })?;

    let mut match_responses = Vec::new();
    for m in matches {
        let response = build_match_response(&state, &m, false).await?;
        match_responses.push(response);
    }

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    Ok(Json(MatchListResponse {
        matches: match_responses,
        total,
        page,
        per_page,
        total_pages,
    }))
}

/// Get a single match by ID
pub async fn get_match(
    State(state): State<Arc<AppState>>,
    current_user_id: Option<Extension<Uuid>>,
    Path(match_id): Path<Uuid>,
) -> Result<Json<MatchResponse>, (StatusCode, Json<Value>)> {
    let match_record = state
        .db
        .get_match_by_id(match_id)
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
                Json(json!({"error": "Match not found"})),
            )
        })?;

    // Check if user is admin to show scores
    let is_admin = if let Some(Extension(user_id)) = current_user_id {
        state.db.is_user_admin(user_id).await.unwrap_or(false)
    } else {
        false
    };

    let response = build_match_response(&state, &match_record, is_admin).await?;

    Ok(Json(response))
}

/// Update a match (admin only)
pub async fn update_match(
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<Uuid>,
    Json(payload): Json<UpdateMatchRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify match exists
    state
        .db
        .get_match_by_id(match_id)
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
                Json(json!({"error": "Match not found"})),
            )
        })?;

    let updated = state
        .db
        .update_match(
            match_id,
            payload.room_name.as_deref(),
            payload.motion.as_deref(),
            payload.info_slide.as_deref(),
            payload.status,
            payload.scheduled_time,
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update match"})),
            )
        })?;

    Ok(Json(json!({
        "message": "Match updated successfully",
        "match": updated
    })))
}

/// Toggle score/ranking release (admin only) - FR-16, FR-17
pub async fn toggle_release(
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<Uuid>,
    Json(payload): Json<ReleaseToggleRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify match exists
    let _match_record = state
        .db
        .get_match_by_id(match_id)
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
                Json(json!({"error": "Match not found"})),
            )
        })?;

    // FR-16: If releasing scores, also release rankings
    let scores_released = payload.scores_released;
    let mut rankings_released = payload.rankings_released;

    if let Some(true) = scores_released {
        rankings_released = Some(true);
    }

    let updated = state
        .db
        .update_match_release(match_id, scores_released, rankings_released)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update release status"})),
            )
        })?;

    Ok(Json(json!({
        "message": "Release status updated successfully",
        "match": updated
    })))
}

/// Delete a match (admin only)
pub async fn delete_match(
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify match exists
    state
        .db
        .get_match_by_id(match_id)
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
                Json(json!({"error": "Match not found"})),
            )
        })?;

    state.db.delete_match(match_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to delete match"})),
        )
    })?;

    Ok(Json(json!({"message": "Match deleted successfully"})))
}

// ============================================================================
// Team Handlers
// ============================================================================

/// Update a team
pub async fn update_team(
    State(state): State<Arc<AppState>>,
    Path(team_id): Path<Uuid>,
    Json(payload): Json<UpdateTeamRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let updated = state
        .db
        .update_team(
            team_id,
            payload.team_name.as_deref(),
            payload.institution.as_deref(),
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update team"})),
            )
        })?;

    Ok(Json(json!({
        "message": "Team updated successfully",
        "team": updated
    })))
}

// ============================================================================
// Allocation Handlers - FR-05 to FR-09
// ============================================================================

/// Get allocation pool for a series (checked-in users) - FR-05
pub async fn get_allocation_pool(
    State(state): State<Arc<AppState>>,
    Path(series_id): Path<Uuid>,
) -> Result<Json<AllocationPoolResponse>, (StatusCode, Json<Value>)> {
    // Get series to find event
    let series = state
        .db
        .get_series_by_id(series_id)
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
                Json(json!({"error": "Series not found"})),
            )
        })?;

    // Get all checked-in users for this event
    let checked_in = state
        .db
        .get_checked_in_users_for_event(series.event_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get checked-in users"})),
            )
        })?;

    let mut users = Vec::new();
    let mut total_allocated = 0i64;

    for attendance in &checked_in {
        // Get user info
        let user = state
            .db
            .get_user_by_id(attendance.user_id)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Database error"})),
                )
            })?;

        if let Some(user) = user {
            // Check if user is already allocated in this series
            let allocation = state
                .db
                .get_user_allocation_in_series(series_id, attendance.user_id)
                .await
                .ok()
                .flatten();

            let is_allocated = allocation.is_some();
            if is_allocated {
                total_allocated += 1;
            }

            let current_allocation = if let Some(alloc) = allocation {
                let match_record = state
                    .db
                    .get_match_by_id(alloc.match_id)
                    .await
                    .ok()
                    .flatten();
                Some(CurrentAllocationInfo {
                    match_id: alloc.match_id,
                    room_name: match_record.and_then(|m| m.room_name),
                    role: alloc.role,
                })
            } else {
                None
            };

            users.push(CheckedInUserResponse {
                user_id: user.id,
                username: user.username,
                checked_in_at: attendance.checked_in_at.unwrap_or_else(Utc::now),
                is_allocated,
                current_allocation,
            });
        }
    }

    let total_checked_in = checked_in.len() as i64;
    let total_available = total_checked_in - total_allocated;

    Ok(Json(AllocationPoolResponse {
        event_id: series.event_id,
        series_id,
        checked_in_users: users,
        total_checked_in,
        total_allocated,
        total_available,
    }))
}

/// Create an allocation - FR-07
pub async fn create_allocation(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Json(payload): Json<CreateAllocationRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Validate: either user_id or guest_name must be provided
    if payload.user_id.is_none() && payload.guest_name.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Either user_id or guest_name must be provided"})),
        ));
    }

    // Verify match exists
    let match_record = state
        .db
        .get_match_by_id(payload.match_id)
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
                Json(json!({"error": "Match not found"})),
            )
        })?;

    // If user_id provided, verify user exists
    let mut was_checked_in = false;
    if let Some(user_id) = payload.user_id {
        let _user = state
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

        // Get the series to find the event
        let series = state
            .db
            .get_series_by_id(match_record.series_id)
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
                    Json(json!({"error": "Series not found"})),
                )
            })?;

        // For friendly matches, we allow very flexible allocations:
        // - Same user can have multiple speaker roles (e.g., PM and Reply)
        // - Same user can be speaker AND adjudicator
        // - We only prevent exact duplicate allocations (same role + same speaker position)
        let existing_allocations = state
            .db
            .list_allocations_by_match(payload.match_id)
            .await
            .unwrap_or_default();

        let has_exact_duplicate = existing_allocations.iter().any(|a| {
            if a.user_id != Some(user_id) {
                return false;
            }
            if a.role != payload.role {
                return false;
            }
            // For speakers, also check the specific speaker role
            if payload.role == AllocationRole::Speaker {
                // Allow same user as speaker if they have different speaker roles
                let same_two_team_role = a.two_team_speaker_role == payload.two_team_speaker_role
                    && payload.two_team_speaker_role.is_some();
                let same_four_team_role = a.four_team_speaker_role
                    == payload.four_team_speaker_role
                    && payload.four_team_speaker_role.is_some();
                return same_two_team_role || same_four_team_role;
            }
            // For adjudicators, don't allow duplicate (can only be voting OR non-voting once)
            true
        });

        if has_exact_duplicate {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({"error": "User already has this exact role in this match"})),
            ));
        }

        // Check if user is checked in for this event
        was_checked_in = state
            .db
            .is_user_checked_in(series.event_id, user_id)
            .await
            .unwrap_or(false);
    }

    // Get series for validation (if not already fetched)
    let series = state
        .db
        .get_series_by_id(match_record.series_id)
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
                Json(json!({"error": "Series not found"})),
            )
        })?;

    // Validate team_id if speaker role
    if payload.role == AllocationRole::Speaker && payload.team_id.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Speakers must be assigned to a team"})),
        ));
    }

    // Validate speaker role based on team format
    if payload.role == AllocationRole::Speaker {
        match series.team_format {
            TeamFormat::TwoTeam => {
                if payload.two_team_speaker_role.is_none() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "Two-team speaker role required"})),
                    ));
                }
            }
            TeamFormat::FourTeam => {
                if payload.four_team_speaker_role.is_none() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "Four-team speaker role required"})),
                    ));
                }
            }
        }
    }

    let now = Utc::now();
    let allocation = Allocation {
        id: Uuid::new_v4(),
        match_id: payload.match_id,
        user_id: payload.user_id,
        guest_name: payload.guest_name.clone(),
        role: payload.role,
        team_id: payload.team_id,
        two_team_speaker_role: payload.two_team_speaker_role,
        four_team_speaker_role: payload.four_team_speaker_role,
        is_chair: Some(payload.is_chair),
        allocated_at: now,
        allocated_by: admin_id,
        was_checked_in,
        created_at: now,
        updated_at: now,
    };

    let created = state.db.create_allocation(&allocation).await.map_err(|e| {
        tracing::error!("Failed to create allocation: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to create allocation"})),
        )
    })?;

    // Create allocation history
    let history = AllocationHistory {
        id: Uuid::new_v4(),
        allocation_id: Some(created.id),
        match_id: payload.match_id,
        user_id: payload.user_id,
        guest_name: payload.guest_name.clone(),
        action: "created".to_string(),
        previous_role: None,
        new_role: Some(payload.role),
        previous_team_id: None,
        new_team_id: payload.team_id,
        changed_by: admin_id,
        changed_at: now,
        notes: None,
    };
    let _ = state.db.create_allocation_history(&history).await;

    // If this is an adjudicator with a user account, create a ballot for them
    // (Guest adjudicators don't submit ballots through the system)
    if let Some(user_id) = payload.user_id {
        if payload.role == AllocationRole::VotingAdjudicator
            || payload.role == AllocationRole::NonVotingAdjudicator
        {
            let ballot = Ballot {
                id: Uuid::new_v4(),
                match_id: payload.match_id,
                adjudicator_id: user_id,
                is_voting: payload.role == AllocationRole::VotingAdjudicator,
                is_submitted: false,
                submitted_at: None,
                notes: None,
                created_at: now,
                updated_at: now,
            };
            let _ = state.db.create_ballot(&ballot).await;
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Allocation created successfully",
            "allocation": created
        })),
    ))
}

/// Update an allocation - FR-09
pub async fn update_allocation(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Path(allocation_id): Path<Uuid>,
    Json(payload): Json<UpdateAllocationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get existing allocation
    let existing = state
        .db
        .get_allocation_by_id(allocation_id)
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
                Json(json!({"error": "Allocation not found"})),
            )
        })?;

    let updated = state
        .db
        .update_allocation(UpdateAllocationParams {
            allocation_id,
            role: payload.role,
            team_id: payload.team_id,
            two_team_speaker_role: payload.two_team_speaker_role,
            four_team_speaker_role: payload.four_team_speaker_role,
            is_chair: payload.is_chair,
            allocated_by: admin_id,
        })
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update allocation"})),
            )
        })?;

    // Create allocation history
    let history = AllocationHistory {
        id: Uuid::new_v4(),
        allocation_id: Some(allocation_id),
        match_id: existing.match_id,
        user_id: existing.user_id,
        guest_name: existing.guest_name.clone(),
        action: "updated".to_string(),
        previous_role: Some(existing.role),
        new_role: payload.role,
        previous_team_id: existing.team_id,
        new_team_id: payload.team_id,
        changed_by: admin_id,
        changed_at: Utc::now(),
        notes: None,
    };
    let _ = state.db.create_allocation_history(&history).await;

    Ok(Json(json!({
        "message": "Allocation updated successfully",
        "allocation": updated
    })))
}

/// Swap two allocations - US-1.6
pub async fn swap_allocations(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Json(payload): Json<SwapAllocationRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get both allocations
    let alloc1 = state
        .db
        .get_allocation_by_id(payload.allocation_id_1)
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
                Json(json!({"error": "First allocation not found"})),
            )
        })?;

    let alloc2 = state
        .db
        .get_allocation_by_id(payload.allocation_id_2)
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
                Json(json!({"error": "Second allocation not found"})),
            )
        })?;

    // Swap the team and role information
    let _ = state
        .db
        .update_allocation(UpdateAllocationParams {
            allocation_id: alloc1.id,
            role: Some(alloc2.role),
            team_id: alloc2.team_id,
            two_team_speaker_role: alloc2.two_team_speaker_role,
            four_team_speaker_role: alloc2.four_team_speaker_role,
            is_chair: alloc2.is_chair,
            allocated_by: admin_id,
        })
        .await;

    let _ = state
        .db
        .update_allocation(UpdateAllocationParams {
            allocation_id: alloc2.id,
            role: Some(alloc1.role),
            team_id: alloc1.team_id,
            two_team_speaker_role: alloc1.two_team_speaker_role,
            four_team_speaker_role: alloc1.four_team_speaker_role,
            is_chair: alloc1.is_chair,
            allocated_by: admin_id,
        })
        .await;

    // Create history for both
    let now = Utc::now();
    let history1 = AllocationHistory {
        id: Uuid::new_v4(),
        allocation_id: Some(alloc1.id),
        match_id: alloc1.match_id,
        user_id: alloc1.user_id,
        guest_name: alloc1.guest_name.clone(),
        action: "swapped".to_string(),
        previous_role: Some(alloc1.role),
        new_role: Some(alloc2.role),
        previous_team_id: alloc1.team_id,
        new_team_id: alloc2.team_id,
        changed_by: admin_id,
        changed_at: now,
        notes: Some(format!("Swapped with allocation {}", alloc2.id)),
    };
    let _ = state.db.create_allocation_history(&history1).await;

    let history2 = AllocationHistory {
        id: Uuid::new_v4(),
        allocation_id: Some(alloc2.id),
        match_id: alloc2.match_id,
        user_id: alloc2.user_id,
        guest_name: alloc2.guest_name.clone(),
        action: "swapped".to_string(),
        previous_role: Some(alloc2.role),
        new_role: Some(alloc1.role),
        previous_team_id: alloc2.team_id,
        new_team_id: alloc1.team_id,
        changed_by: admin_id,
        changed_at: now,
        notes: Some(format!("Swapped with allocation {}", alloc1.id)),
    };
    let _ = state.db.create_allocation_history(&history2).await;

    Ok(Json(json!({"message": "Allocations swapped successfully"})))
}

/// Delete an allocation
pub async fn delete_allocation(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Path(allocation_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get allocation for history
    let allocation = state
        .db
        .get_allocation_by_id(allocation_id)
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
                Json(json!({"error": "Allocation not found"})),
            )
        })?;

    // Create history before deletion
    let history = AllocationHistory {
        id: Uuid::new_v4(),
        allocation_id: Some(allocation_id),
        match_id: allocation.match_id,
        user_id: allocation.user_id,
        guest_name: allocation.guest_name.clone(),
        action: "deleted".to_string(),
        previous_role: Some(allocation.role),
        new_role: None,
        previous_team_id: allocation.team_id,
        new_team_id: None,
        changed_by: admin_id,
        changed_at: Utc::now(),
        notes: None,
    };
    let _ = state.db.create_allocation_history(&history).await;

    state
        .db
        .delete_allocation(allocation_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete allocation"})),
            )
        })?;

    Ok(Json(json!({"message": "Allocation deleted successfully"})))
}

/// Get allocation history for a match
pub async fn get_allocation_history(
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<Uuid>,
    Query(query): Query<PerformanceQuery>,
) -> Result<Json<AllocationHistoryResponse>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 100);

    let (history, total) = state
        .db
        .list_allocation_history(match_id, page, per_page)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;

    Ok(Json(AllocationHistoryResponse {
        history,
        total,
        page,
        per_page,
        total_pages,
    }))
}

// ============================================================================
// Ballot Handlers - FR-10 to FR-13
// ============================================================================

/// Get ballot for current adjudicator - US-2.1
pub async fn get_my_ballot(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(match_id): Path<Uuid>,
) -> Result<Json<BallotResponse>, (StatusCode, Json<Value>)> {
    // Verify user is allocated as adjudicator (use the specific adjudicator lookup)
    let allocation = state
        .db
        .get_adjudicator_allocation_by_user_match(match_id, user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "You are not an adjudicator for this match"})),
            )
        })?;

    // No need to check role again - the query already filters for adjudicators
    if allocation.role != AllocationRole::VotingAdjudicator
        && allocation.role != AllocationRole::NonVotingAdjudicator
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You are not an adjudicator for this match"})),
        ));
    }

    // Get or create ballot
    let ballot = match state
        .db
        .get_ballot_by_adjudicator_match(match_id, user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })? {
        Some(b) => b,
        None => {
            // Create ballot if doesn't exist
            let now = Utc::now();
            let new_ballot = Ballot {
                id: Uuid::new_v4(),
                match_id,
                adjudicator_id: user_id,
                is_voting: allocation.role == AllocationRole::VotingAdjudicator,
                is_submitted: false,
                submitted_at: None,
                notes: None,
                created_at: now,
                updated_at: now,
            };
            state.db.create_ballot(&new_ballot).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to create ballot"})),
                )
            })?
        }
    };

    let user = state.db.get_user_by_id(user_id).await.ok().flatten();
    let username = user.map(|u| u.username).unwrap_or_default();

    // Get speaker scores
    let scores = state
        .db
        .list_speaker_scores_by_ballot(ballot.id)
        .await
        .unwrap_or_default();

    let mut score_responses = Vec::new();
    for score in scores {
        let alloc = state
            .db
            .get_allocation_by_id(score.allocation_id)
            .await
            .ok()
            .flatten();
        let speaker_username = if let Some(a) = &alloc {
            if let Some(user_id) = a.user_id {
                state
                    .db
                    .get_user_by_id(user_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|u| u.username)
                    .unwrap_or_default()
            } else {
                a.guest_name.clone().unwrap_or_default()
            }
        } else {
            String::new()
        };

        score_responses.push(SpeakerScoreResponse {
            id: score.id,
            allocation_id: score.allocation_id,
            speaker_username,
            score: score.score,
            feedback: score.feedback,
        });
    }

    // Get team rankings
    let rankings = state
        .db
        .list_team_rankings_by_ballot(ballot.id)
        .await
        .unwrap_or_default();

    let mut ranking_responses = Vec::new();
    for ranking in rankings {
        let team = state
            .db
            .get_team_by_id(ranking.team_id)
            .await
            .ok()
            .flatten();
        ranking_responses.push(TeamRankingResponse {
            id: ranking.id,
            team_id: ranking.team_id,
            team_name: team.and_then(|t| t.team_name),
            rank: ranking.rank,
            is_winner: ranking.is_winner,
        });
    }

    Ok(Json(BallotResponse {
        id: ballot.id,
        match_id: ballot.match_id,
        adjudicator_id: ballot.adjudicator_id,
        adjudicator_username: username,
        is_voting: ballot.is_voting,
        is_submitted: ballot.is_submitted,
        submitted_at: ballot.submitted_at,
        notes: ballot.notes,
        speaker_scores: score_responses,
        team_rankings: ranking_responses,
    }))
}

/// Submit ballot (voting adjudicator) - FR-10, FR-12, US-2.2
pub async fn submit_ballot(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<SubmitBallotRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    // Verify user is a voting adjudicator for this match (use specific adjudicator lookup)
    let allocation = state
        .db
        .get_adjudicator_allocation_by_user_match(payload.match_id, user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "You are not an adjudicator for this match"})),
            )
        })?;

    if allocation.role != AllocationRole::VotingAdjudicator {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Only voting adjudicators can submit ballots with scores"})),
        ));
    }

    // Get or create ballot
    let ballot = match state
        .db
        .get_ballot_by_adjudicator_match(payload.match_id, user_id)
        .await
        .ok()
        .flatten()
    {
        Some(b) => b,
        None => {
            let now = Utc::now();
            let new_ballot = Ballot {
                id: Uuid::new_v4(),
                match_id: payload.match_id,
                adjudicator_id: user_id,
                is_voting: true,
                is_submitted: false,
                submitted_at: None,
                notes: None,
                created_at: now,
                updated_at: now,
            };
            state.db.create_ballot(&new_ballot).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to create ballot"})),
                )
            })?
        }
    };

    // FR-13: Validate unique rankings
    let mut ranks: Vec<i32> = payload.team_rankings.iter().map(|r| r.rank).collect();
    ranks.sort();
    let unique_ranks: Vec<i32> = ranks
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    if ranks.len() != unique_ranks.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Rankings must be unique (no ties)"})),
        ));
    }

    // Delete existing scores and rankings (to support re-submission/updates)
    state
        .db
        .delete_speaker_scores_by_ballot(ballot.id)
        .await
        .ok();
    state
        .db
        .delete_team_rankings_by_ballot(ballot.id)
        .await
        .ok();

    // Create speaker scores
    let now = Utc::now();
    for score_input in &payload.speaker_scores {
        let score = SpeakerScore {
            id: Uuid::new_v4(),
            ballot_id: ballot.id,
            allocation_id: score_input.allocation_id,
            score: Decimal::from_f64_retain(score_input.score).unwrap_or_else(|| Decimal::from(75)),
            feedback: score_input.feedback.clone(),
            created_at: now,
            updated_at: now,
        };
        state.db.create_speaker_score(&score).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to save speaker score"})),
            )
        })?;
    }

    // Create team rankings
    for ranking_input in &payload.team_rankings {
        let ranking = TeamRanking {
            id: Uuid::new_v4(),
            ballot_id: ballot.id,
            team_id: ranking_input.team_id,
            rank: ranking_input.rank,
            is_winner: ranking_input.is_winner,
            created_at: now,
            updated_at: now,
        };
        state.db.create_team_ranking(&ranking).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to save team ranking"})),
            )
        })?;
    }

    // Mark ballot as submitted
    let submitted = state
        .db
        .submit_ballot(ballot.id, payload.notes.as_deref())
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to submit ballot"})),
            )
        })?;

    // Recalculate final rankings from all submitted voting ballots
    if let Ok(rankings) = state.db.get_match_team_rankings(payload.match_id).await {
        // Get total speaker points for each team
        let teams = state
            .db
            .list_teams_by_match(payload.match_id)
            .await
            .unwrap_or_default();

        for (rank_position, (team_id, _avg_rank)) in rankings.iter().enumerate() {
            // Calculate total speaker points from submitted ballots for this team
            let total_points = calculate_team_total_points(&state.db, *team_id).await;

            // Update team with final rank (1-indexed) and total points
            let _ = state
                .db
                .update_team_results(*team_id, (rank_position + 1) as i32, total_points)
                .await;
        }

        // Also update teams that don't have any rankings yet (set them to last place)
        for team in teams {
            if !rankings.iter().any(|(tid, _)| *tid == team.id) {
                let total_points = calculate_team_total_points(&state.db, team.id).await;
                let _ = state
                    .db
                    .update_team_results(team.id, (rankings.len() + 1) as i32, total_points)
                    .await;
            }
        }
    }

    Ok(Json(json!({
        "message": "Ballot submitted successfully",
        "ballot": submitted
    })))
}

/// Calculate total speaker points for a team from all submitted voting ballots
async fn calculate_team_total_points(db: &crate::database::Database, team_id: Uuid) -> Decimal {
    // Get all allocations for this team (speakers)
    let allocations = db
        .list_allocations_by_team(team_id)
        .await
        .unwrap_or_default();

    let mut total = Decimal::ZERO;
    for alloc in allocations {
        if let Ok(Some(avg_score)) = db.get_allocation_average_score(alloc.id).await {
            total += avg_score;
        }
    }
    total
}

/// Submit feedback only (non-voting adjudicator) - FR-11, US-2.3
pub async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<SubmitFeedbackRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    // Verify user is allocated to this match
    let allocation = state
        .db
        .get_allocation_by_user_match(payload.match_id, user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "You are not allocated to this match"})),
            )
        })?;

    if allocation.role != AllocationRole::NonVotingAdjudicator
        && allocation.role != AllocationRole::VotingAdjudicator
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Only adjudicators can submit feedback"})),
        ));
    }

    // Get or create ballot
    let ballot = match state
        .db
        .get_ballot_by_adjudicator_match(payload.match_id, user_id)
        .await
        .ok()
        .flatten()
    {
        Some(b) => b,
        None => {
            let now = Utc::now();
            let new_ballot = Ballot {
                id: Uuid::new_v4(),
                match_id: payload.match_id,
                adjudicator_id: user_id,
                is_voting: allocation.role == AllocationRole::VotingAdjudicator,
                is_submitted: false,
                submitted_at: None,
                notes: None,
                created_at: now,
                updated_at: now,
            };
            state.db.create_ballot(&new_ballot).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to create ballot"})),
                )
            })?
        }
    };

    // Submit with notes only
    let submitted = state
        .db
        .submit_ballot(ballot.id, Some(&payload.notes))
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to submit feedback"})),
            )
        })?;

    Ok(Json(json!({
        "message": "Feedback submitted successfully",
        "ballot": submitted
    })))
}

/// Admin: Get all ballots for a match - US-3.2
pub async fn admin_get_match_ballots(
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<Uuid>,
) -> Result<Json<Vec<BallotResponse>>, (StatusCode, Json<Value>)> {
    let ballots = state
        .db
        .list_ballots_by_match(match_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    let mut responses = Vec::new();
    for ballot in ballots {
        let user = state
            .db
            .get_user_by_id(ballot.adjudicator_id)
            .await
            .ok()
            .flatten();
        let username = user.map(|u| u.username).unwrap_or_default();

        let scores = state
            .db
            .list_speaker_scores_by_ballot(ballot.id)
            .await
            .unwrap_or_default();

        let mut score_responses = Vec::new();
        for score in scores {
            let alloc = state
                .db
                .get_allocation_by_id(score.allocation_id)
                .await
                .ok()
                .flatten();
            let speaker_username = if let Some(a) = &alloc {
                if let Some(user_id) = a.user_id {
                    state
                        .db
                        .get_user_by_id(user_id)
                        .await
                        .ok()
                        .flatten()
                        .map(|u| u.username)
                        .unwrap_or_default()
                } else {
                    a.guest_name.clone().unwrap_or_default()
                }
            } else {
                String::new()
            };

            score_responses.push(SpeakerScoreResponse {
                id: score.id,
                allocation_id: score.allocation_id,
                speaker_username,
                score: score.score,
                feedback: score.feedback,
            });
        }

        let rankings = state
            .db
            .list_team_rankings_by_ballot(ballot.id)
            .await
            .unwrap_or_default();

        let mut ranking_responses = Vec::new();
        for ranking in rankings {
            let team = state
                .db
                .get_team_by_id(ranking.team_id)
                .await
                .ok()
                .flatten();
            ranking_responses.push(TeamRankingResponse {
                id: ranking.id,
                team_id: ranking.team_id,
                team_name: team.and_then(|t| t.team_name),
                rank: ranking.rank,
                is_winner: ranking.is_winner,
            });
        }

        responses.push(BallotResponse {
            id: ballot.id,
            match_id: ballot.match_id,
            adjudicator_id: ballot.adjudicator_id,
            adjudicator_username: username,
            is_voting: ballot.is_voting,
            is_submitted: ballot.is_submitted,
            submitted_at: ballot.submitted_at,
            notes: ballot.notes,
            speaker_scores: score_responses,
            team_rankings: ranking_responses,
        });
    }

    Ok(Json(responses))
}

// ============================================================================
// Performance/Tabulation Handlers - FR-14
// ============================================================================

/// Get performance tab for a user - US-3.1
pub async fn get_user_performance(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<PerformanceQuery>,
) -> Result<Json<PerformanceResponse>, (StatusCode, Json<Value>)> {
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

    let (total_rounds, speaker_rounds, adjudicator_rounds) = state
        .db
        .get_user_round_counts(user_id, query.event_id)
        .await
        .unwrap_or((0, 0, 0));

    let avg_score = state
        .db
        .get_average_speaker_score(user_id, query.event_id)
        .await
        .unwrap_or(None);

    let (wins, losses) = state
        .db
        .get_user_win_loss(user_id, query.event_id)
        .await
        .unwrap_or((0, 0));

    let win_rate = if wins + losses > 0 {
        Some(Decimal::from(wins * 100) / Decimal::from(wins + losses))
    } else {
        None
    };

    let ranking_dist = state
        .db
        .get_user_ranking_distribution(user_id, query.event_id)
        .await
        .unwrap_or_default();

    let rankings = ranking_dist
        .into_iter()
        .map(|(rank, count)| RankingCount { rank, count })
        .collect();

    Ok(Json(PerformanceResponse {
        user_id,
        username: user.username,
        total_rounds,
        rounds_as_speaker: speaker_rounds,
        rounds_as_adjudicator: adjudicator_rounds,
        average_speaker_score: avg_score,
        total_wins: wins,
        total_losses: losses,
        win_rate,
        rankings,
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn build_match_response(
    state: &Arc<AppState>,
    match_record: &Match,
    is_admin: bool,
) -> Result<MatchResponse, (StatusCode, Json<Value>)> {
    let series = state
        .db
        .get_series_by_id(match_record.series_id)
        .await
        .ok()
        .flatten();

    let series_name = series.as_ref().map(|s| s.name.clone()).unwrap_or_default();
    let _team_format = series
        .as_ref()
        .map(|s| s.team_format)
        .unwrap_or(TeamFormat::TwoTeam);

    let teams = state
        .db
        .list_teams_by_match(match_record.id)
        .await
        .unwrap_or_default();

    let mut team_responses = Vec::new();
    for team in teams {
        let allocations = state
            .db
            .list_allocations_by_team(team.id)
            .await
            .unwrap_or_default();

        let mut speakers = Vec::new();
        let mut resources = Vec::new();

        for alloc in allocations {
            match alloc.role {
                AllocationRole::Speaker => {
                    // Get average score from submitted voting ballots if scores are released
                    let score = if match_record.scores_released || is_admin {
                        state
                            .db
                            .get_allocation_average_score(alloc.id)
                            .await
                            .ok()
                            .flatten()
                    } else {
                        None
                    };

                    speakers.push(SpeakerResponse {
                        allocation_id: alloc.id,
                        user_id: alloc.user_id,
                        guest_name: alloc.guest_name.clone(),
                        username: alloc.username,
                        two_team_speaker_role: alloc.two_team_speaker_role,
                        four_team_speaker_role: alloc.four_team_speaker_role,
                        score,
                    });
                }
                AllocationRole::Resource => {
                    resources.push(ResourceResponse {
                        allocation_id: alloc.id,
                        user_id: alloc.user_id,
                        guest_name: alloc.guest_name.clone(),
                        username: alloc.username,
                    });
                }
                _ => {}
            }
        }

        team_responses.push(MatchTeamResponse {
            id: team.id,
            two_team_position: team.two_team_position,
            four_team_position: team.four_team_position,
            team_name: team.team_name,
            institution: team.institution,
            final_rank: if match_record.rankings_released || is_admin {
                team.final_rank
            } else {
                None
            },
            total_speaker_points: if match_record.scores_released || is_admin {
                team.total_speaker_points
            } else {
                None
            },
            speakers,
            resources,
        });
    }

    // Get adjudicators
    let all_allocations = state
        .db
        .list_allocations_by_match(match_record.id)
        .await
        .unwrap_or_default();

    let mut adjudicators = Vec::new();
    for alloc in all_allocations {
        if alloc.role == AllocationRole::VotingAdjudicator
            || alloc.role == AllocationRole::NonVotingAdjudicator
        {
            // Only look up ballot if this is a registered user (not a guest)
            let has_submitted = if let Some(user_id) = alloc.user_id {
                state
                    .db
                    .get_ballot_by_adjudicator_match(match_record.id, user_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|b| b.is_submitted)
                    .unwrap_or(false)
            } else {
                false // Guests cannot submit ballots
            };

            adjudicators.push(AdjudicatorResponse {
                allocation_id: alloc.id,
                user_id: alloc.user_id,
                guest_name: alloc.guest_name.clone(),
                username: alloc.username,
                is_voting: alloc.role == AllocationRole::VotingAdjudicator,
                is_chair: alloc.is_chair.unwrap_or(false),
                has_submitted,
            });
        }
    }

    Ok(MatchResponse {
        id: match_record.id,
        series_id: match_record.series_id,
        series_name,
        room_name: match_record.room_name.clone(),
        motion: match_record.motion.clone(),
        info_slide: match_record.info_slide.clone(),
        status: match_record.status,
        scheduled_time: match_record.scheduled_time,
        scores_released: match_record.scores_released,
        rankings_released: match_record.rankings_released,
        teams: team_responses,
        adjudicators,
        created_at: match_record.created_at,
        updated_at: match_record.updated_at,
    })
}
