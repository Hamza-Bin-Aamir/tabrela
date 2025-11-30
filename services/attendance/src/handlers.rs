use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{
        AttendanceResponse, AttendanceStats, CheckInRequest, CreateEventRequest,
        EventAttendanceResponse, EventListParams, EventListResponse, EventResponse,
        LockEventRequest, RevokeAvailabilityRequest, SetAvailabilityRequest, UpdateEventRequest,
    },
    AppState,
};

// ============================================================================
// Event Handlers
// ============================================================================

/// Create a new event (Admin only)
pub async fn create_event(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Validation error: {}", e)})),
        )
    })?;

    let event = state
        .db
        .create_event(
            &payload.title,
            payload.description.as_deref(),
            &payload.event_type.to_string(),
            payload.event_date,
            payload.location.as_deref(),
            user_id,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create event: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create event"})),
            )
        })?;

    let response: EventResponse = event.into();
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Event created successfully",
            "event": response
        })),
    ))
}

/// Get a single event by ID
pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let event = state
        .db
        .get_event_by_id(event_id)
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
                Json(json!({"error": "Event not found"})),
            )
        })?;

    let response: EventResponse = event.into();
    Ok((StatusCode::OK, Json(json!(response))))
}

/// List all events with optional filters
pub async fn list_events(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<EventListParams>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let upcoming_only = params.upcoming_only.unwrap_or(false);

    let (events, total) = state
        .db
        .list_events(page, per_page, params.event_type.as_deref(), upcoming_only)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch events"})),
            )
        })?;

    let event_responses: Vec<EventResponse> = events.into_iter().map(|e| e.into()).collect();
    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    let response = EventListResponse {
        events: event_responses,
        total,
        page,
        per_page,
        total_pages,
    };

    Ok((StatusCode::OK, Json(json!(response))))
}

/// Update an event (Admin only)
pub async fn update_event(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<UpdateEventRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Validation error: {}", e)})),
        )
    })?;

    let event = state
        .db
        .update_event(
            event_id,
            payload.title.as_deref(),
            payload.description.as_deref(),
            payload.event_type.map(|t| t.to_string()).as_deref(),
            payload.event_date,
            payload.location.as_deref(),
        )
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update event"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Event not found"})),
            )
        })?;

    let response: EventResponse = event.into();
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Event updated successfully",
            "event": response
        })),
    ))
}

/// Delete an event (Admin only)
pub async fn delete_event(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let deleted = state.db.delete_event(event_id).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to delete event"})),
        )
    })?;

    if !deleted {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Event not found"})),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({"message": "Event deleted successfully"})),
    ))
}

/// Lock or unlock an event (Admin only)
pub async fn lock_event(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<LockEventRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let event = state
        .db
        .lock_event(event_id, payload.is_locked)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update event lock status"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Event not found"})),
            )
        })?;

    let response: EventResponse = event.into();
    let message = if payload.is_locked {
        "Event locked successfully"
    } else {
        "Event unlocked successfully"
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": message,
            "event": response
        })),
    ))
}

// ============================================================================
// Attendance Handlers
// ============================================================================

/// Get event with all attendance records
pub async fn get_event_attendance(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let event = state
        .db
        .get_event_by_id(event_id)
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
                Json(json!({"error": "Event not found"})),
            )
        })?;

    let records = state
        .db
        .get_event_attendance(event_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch attendance"})),
            )
        })?;

    let (total_available, total_checked_in) = state
        .db
        .get_attendance_stats(event_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch attendance stats"})),
            )
        })?;

    let attendance_responses: Vec<AttendanceResponse> =
        records.into_iter().map(|r| r.into()).collect();

    let response = EventAttendanceResponse {
        event: event.into(),
        attendance: attendance_responses,
        stats: AttendanceStats {
            total_available,
            total_checked_in,
            total_unavailable: 0, // Can't know total users from here
        },
    };

    Ok((StatusCode::OK, Json(json!(response))))
}

/// Set user's own availability for an event
pub async fn set_availability(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<SetAvailabilityRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Check if event exists and is not locked
    let event = state
        .db
        .get_event_by_id(event_id)
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
                Json(json!({"error": "Event not found"})),
            )
        })?;

    if event.is_locked {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Event attendance is locked and cannot be modified"})),
        ));
    }

    let record = state
        .db
        .set_availability(event_id, user_id, payload.is_available)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set availability: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to set availability"})),
            )
        })?;

    let response: AttendanceResponse = record.into();
    let message = if payload.is_available {
        "Marked as available"
    } else {
        "Marked as unavailable"
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": message,
            "attendance": response
        })),
    ))
}

/// Get user's own attendance record for an event
pub async fn get_my_attendance(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(event_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Check if event exists
    state
        .db
        .get_event_by_id(event_id)
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
                Json(json!({"error": "Event not found"})),
            )
        })?;

    let record = state
        .db
        .get_attendance_record(event_id, user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Database error"})),
            )
        })?;

    match record {
        Some(r) => {
            let response: AttendanceResponse = r.into();
            Ok((StatusCode::OK, Json(json!(response))))
        }
        None => {
            // No record means unavailable by default
            Ok((
                StatusCode::OK,
                Json(json!({
                    "event_id": event_id,
                    "user_id": user_id,
                    "is_available": false,
                    "is_checked_in": false
                })),
            ))
        }
    }
}

/// Check in a user (Admin only)
pub async fn check_in_user(
    State(state): State<Arc<AppState>>,
    Extension(admin_user_id): Extension<Uuid>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<CheckInRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Check if event exists and is not locked
    let event = state
        .db
        .get_event_by_id(event_id)
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
                Json(json!({"error": "Event not found"})),
            )
        })?;

    if event.is_locked {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Event attendance is locked and cannot be modified"})),
        ));
    }

    let record = state
        .db
        .check_in_user(event_id, payload.user_id, payload.is_checked_in, admin_user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check in user: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update check-in status"})),
            )
        })?;

    let response: AttendanceResponse = record.into();
    let message = if payload.is_checked_in {
        "User checked in successfully"
    } else {
        "User check-in revoked"
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": message,
            "attendance": response
        })),
    ))
}

/// Revoke a user's availability (Admin only)
pub async fn revoke_availability(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<RevokeAvailabilityRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Check if event exists and is not locked
    let event = state
        .db
        .get_event_by_id(event_id)
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
                Json(json!({"error": "Event not found"})),
            )
        })?;

    if event.is_locked {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Event attendance is locked and cannot be modified"})),
        ));
    }

    let record = state
        .db
        .revoke_availability(event_id, payload.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to revoke availability: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to revoke availability"})),
            )
        })?;

    match record {
        Some(r) => {
            let response: AttendanceResponse = r.into();
            Ok((
                StatusCode::OK,
                Json(json!({
                    "message": "Availability revoked successfully",
                    "attendance": response
                })),
            ))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No attendance record found for this user"})),
        )),
    }
}

// ============================================================================
// Attendance Matrix/Dashboard Handlers (Admin only)
// ============================================================================

use crate::models::{
    AggregateStats, AttendanceCellStatus, AttendanceMatrixResponse, AttendanceMatrixRow,
    EventSummary, EventTypeStats, UserAttendanceSummary,
};
use std::collections::HashMap;

/// Get the full attendance matrix with all statistics (Admin only)
pub async fn get_attendance_matrix(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Fetch all data in parallel-ish manner
    let events = state
        .db
        .get_all_events_for_matrix()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch events: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch events"})),
            )
        })?;

    let users = state
        .db
        .get_all_users()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch users: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch users"})),
            )
        })?;

    let attendance_records = state
        .db
        .get_all_attendance_records()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch attendance records: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch attendance records"})),
            )
        })?;

    let event_type_stats = state
        .db
        .get_event_type_stats()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch event type stats: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch event type stats"})),
            )
        })?;

    // Build lookup maps
    // Key: (event_id, user_id) -> (is_available, is_checked_in)
    let attendance_map: HashMap<(Uuid, Uuid), (bool, bool)> = attendance_records
        .iter()
        .map(|(event_id, user_id, is_available, is_checked_in)| {
            ((*event_id, *user_id), (*is_available, *is_checked_in))
        })
        .collect();

    // Event stats: event_id -> (available, checked_in)
    let mut event_stats: HashMap<Uuid, (i64, i64)> = HashMap::new();
    for (event_id, _user_id, is_available, is_checked_in) in &attendance_records {
        let entry = event_stats.entry(*event_id).or_insert((0, 0));
        if *is_available {
            entry.0 += 1;
        }
        if *is_checked_in {
            entry.1 += 1;
        }
    }

    // User stats: user_id -> (available_count, checked_in_count)
    let mut user_stats: HashMap<Uuid, (i64, i64)> = HashMap::new();
    for (_event_id, user_id, is_available, is_checked_in) in &attendance_records {
        let entry = user_stats.entry(*user_id).or_insert((0, 0));
        if *is_available {
            entry.0 += 1;
        }
        if *is_checked_in {
            entry.1 += 1;
        }
    }

    let total_events = events.len() as i64;
    let total_users = users.len() as i64;

    // Build event summaries
    let event_summaries: Vec<EventSummary> = events
        .iter()
        .map(|e| {
            let (available, checked_in) = event_stats.get(&e.id).copied().unwrap_or((0, 0));
            EventSummary {
                id: e.id,
                title: e.title.clone(),
                event_type: e.event_type.clone(),
                event_date: e.event_date,
                is_locked: e.is_locked,
                total_available: available,
                total_checked_in: checked_in,
            }
        })
        .collect();

    // Build user summaries and matrix rows
    let mut rows: Vec<AttendanceMatrixRow> = Vec::new();
    let mut all_user_summaries: Vec<UserAttendanceSummary> = Vec::new();

    for (user_id, username) in &users {
        let (events_available, events_checked_in) = user_stats.get(user_id).copied().unwrap_or((0, 0));
        
        let availability_rate = if total_events > 0 {
            (events_available as f64 / total_events as f64) * 100.0
        } else {
            0.0
        };
        
        let attendance_rate = if total_events > 0 {
            (events_checked_in as f64 / total_events as f64) * 100.0
        } else {
            0.0
        };

        let user_summary = UserAttendanceSummary {
            user_id: *user_id,
            username: username.clone(),
            events_available,
            events_checked_in,
            total_events,
            availability_rate,
            attendance_rate,
        };

        // Build cells for this user (one per event)
        let cells: Vec<AttendanceCellStatus> = events
            .iter()
            .map(|e| {
                match attendance_map.get(&(e.id, *user_id)) {
                    Some((is_available, is_checked_in)) => {
                        if *is_checked_in {
                            AttendanceCellStatus::CheckedIn
                        } else if *is_available {
                            AttendanceCellStatus::Available
                        } else {
                            AttendanceCellStatus::Unavailable
                        }
                    }
                    None => AttendanceCellStatus::NoResponse,
                }
            })
            .collect();

        all_user_summaries.push(user_summary.clone());
        rows.push(AttendanceMatrixRow {
            user: user_summary,
            cells,
        });
    }

    // Sort rows by attendance rate (descending)
    rows.sort_by(|a, b| b.user.attendance_rate.partial_cmp(&a.user.attendance_rate).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate aggregate statistics
    let total_availability_records: i64 = user_stats.values().map(|(a, _)| a).sum();
    let total_checkin_records: i64 = user_stats.values().map(|(_, c)| c).sum();
    let total_possible = total_events * total_users;

    let overall_availability_rate = if total_possible > 0 {
        (total_availability_records as f64 / total_possible as f64) * 100.0
    } else {
        0.0
    };

    let overall_attendance_rate = if total_possible > 0 {
        (total_checkin_records as f64 / total_possible as f64) * 100.0
    } else {
        0.0
    };

    let avg_available_per_event = if total_events > 0 {
        total_availability_records as f64 / total_events as f64
    } else {
        0.0
    };

    let avg_checked_in_per_event = if total_events > 0 {
        total_checkin_records as f64 / total_events as f64
    } else {
        0.0
    };

    // Find most and least attended events
    let most_attended_event = event_summaries
        .iter()
        .max_by_key(|e| e.total_checked_in)
        .cloned();

    let least_attended_event = if !event_summaries.is_empty() {
        event_summaries
            .iter()
            .min_by_key(|e| e.total_checked_in)
            .cloned()
    } else {
        None
    };

    // Get top 5 most reliable users (by attendance rate)
    let mut most_reliable_users = all_user_summaries.clone();
    most_reliable_users.sort_by(|a, b| b.attendance_rate.partial_cmp(&a.attendance_rate).unwrap_or(std::cmp::Ordering::Equal));
    most_reliable_users.truncate(5);

    // Build event type stats
    let events_by_type: Vec<EventTypeStats> = event_type_stats
        .iter()
        .map(|(event_type, count, avg_attendance)| EventTypeStats {
            event_type: event_type.clone(),
            count: *count,
            avg_attendance: *avg_attendance,
        })
        .collect();

    let aggregate_stats = AggregateStats {
        total_events,
        total_users,
        overall_availability_rate,
        overall_attendance_rate,
        avg_available_per_event,
        avg_checked_in_per_event,
        most_attended_event,
        least_attended_event,
        most_reliable_users,
        events_by_type,
    };

    let response = AttendanceMatrixResponse {
        events: event_summaries,
        rows,
        aggregate_stats,
    };

    Ok((StatusCode::OK, Json(json!(response))))
}
