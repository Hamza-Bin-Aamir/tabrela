use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Database Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub event_type: String,
    pub event_date: DateTime<Utc>,
    pub location: Option<String>,
    pub created_by: Uuid,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttendanceRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub is_available: bool,
    pub is_checked_in: bool,
    pub checked_in_by: Option<Uuid>,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub availability_set_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Attendance record with user info joined from users table
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttendanceRecordWithUser {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub is_available: bool,
    pub is_checked_in: bool,
    pub checked_in_by: Option<Uuid>,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub availability_set_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Tournament,
    WeeklyMatch,
    Meeting,
    Other,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Tournament => write!(f, "tournament"),
            EventType::WeeklyMatch => write!(f, "weekly_match"),
            EventType::Meeting => write!(f, "meeting"),
            EventType::Other => write!(f, "other"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tournament" => Ok(EventType::Tournament),
            "weekly_match" => Ok(EventType::WeeklyMatch),
            "meeting" => Ok(EventType::Meeting),
            "other" => Ok(EventType::Other),
            _ => Err(format!("Invalid event type: {}", s)),
        }
    }
}

// Event Requests
#[derive(Debug, Deserialize, Validate)]
pub struct CreateEventRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub description: Option<String>,
    pub event_type: EventType,
    pub event_date: DateTime<Utc>,
    #[validate(length(max = 255))]
    pub location: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEventRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub description: Option<String>,
    pub event_type: Option<EventType>,
    pub event_date: Option<DateTime<Utc>>,
    #[validate(length(max = 255))]
    pub location: Option<String>,
}

// Event Responses
#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub event_type: String,
    pub event_date: DateTime<Utc>,
    pub location: Option<String>,
    pub created_by: Uuid,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Event> for EventResponse {
    fn from(event: Event) -> Self {
        EventResponse {
            id: event.id,
            title: event.title,
            description: event.description,
            event_type: event.event_type,
            event_date: event.event_date,
            location: event.location,
            created_by: event.created_by,
            is_locked: event.is_locked,
            created_at: event.created_at,
            updated_at: event.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EventListResponse {
    pub events: Vec<EventResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i64,
}

// Attendance Requests
#[derive(Debug, Deserialize)]
pub struct SetAvailabilityRequest {
    pub is_available: bool,
}

#[derive(Debug, Deserialize)]
pub struct CheckInRequest {
    pub user_id: Uuid,
    pub is_checked_in: bool,
}

#[derive(Debug, Deserialize)]
pub struct RevokeAvailabilityRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AdminSetAvailabilityRequest {
    pub user_id: Uuid,
    pub is_available: bool,
}

// Attendance Responses
#[derive(Debug, Serialize)]
pub struct AttendanceResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub is_available: bool,
    pub is_checked_in: bool,
    pub checked_in_by: Option<Uuid>,
    pub checked_in_at: Option<DateTime<Utc>>,
    pub availability_set_at: DateTime<Utc>,
}

impl From<AttendanceRecord> for AttendanceResponse {
    fn from(record: AttendanceRecord) -> Self {
        AttendanceResponse {
            id: record.id,
            event_id: record.event_id,
            user_id: record.user_id,
            username: record.user_id.to_string(), // Fallback to UUID if no join
            is_available: record.is_available,
            is_checked_in: record.is_checked_in,
            checked_in_by: record.checked_in_by,
            checked_in_at: record.checked_in_at,
            availability_set_at: record.availability_set_at,
        }
    }
}

impl From<AttendanceRecordWithUser> for AttendanceResponse {
    fn from(record: AttendanceRecordWithUser) -> Self {
        AttendanceResponse {
            id: record.id,
            event_id: record.event_id,
            user_id: record.user_id,
            username: record.username,
            is_available: record.is_available,
            is_checked_in: record.is_checked_in,
            checked_in_by: record.checked_in_by,
            checked_in_at: record.checked_in_at,
            availability_set_at: record.availability_set_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EventAttendanceResponse {
    pub event: EventResponse,
    pub attendance: Vec<AttendanceResponse>,
    pub stats: AttendanceStats,
}

#[derive(Debug, Serialize)]
pub struct AttendanceStats {
    pub total_available: i64,
    pub total_checked_in: i64,
    pub total_unavailable: i64,
}

// Query parameters
#[derive(Debug, Deserialize)]
pub struct EventListParams {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub event_type: Option<String>,
    pub upcoming_only: Option<bool>,
}

// JWT Claims (for validating tokens from auth service)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub token_type: String,
}

// Lock/Unlock request
#[derive(Debug, Deserialize)]
pub struct LockEventRequest {
    pub is_locked: bool,
}

// ============================================================================
// Attendance Matrix/Dashboard Types
// ============================================================================

/// Summary info for an event in the matrix
#[derive(Debug, Clone, Serialize)]
pub struct EventSummary {
    pub id: Uuid,
    pub title: String,
    pub event_type: String,
    pub event_date: DateTime<Utc>,
    pub is_locked: bool,
    pub total_available: i64,
    pub total_checked_in: i64,
}

/// User info with their attendance stats
#[derive(Debug, Clone, Serialize)]
pub struct UserAttendanceSummary {
    pub user_id: Uuid,
    pub username: String,
    pub events_available: i64,
    pub events_checked_in: i64,
    pub total_events: i64,
    pub availability_rate: f64,
    pub attendance_rate: f64,
}

/// Single cell in the attendance matrix
#[derive(Debug, Serialize)]
pub struct AttendanceCell {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub status: AttendanceCellStatus,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AttendanceCellStatus {
    NoResponse,
    Available,
    CheckedIn,
    Unavailable,
}

/// Row in the attendance matrix (one per user)
#[derive(Debug, Serialize)]
pub struct AttendanceMatrixRow {
    pub user: UserAttendanceSummary,
    pub cells: Vec<AttendanceCellStatus>,
}

/// Aggregate statistics for the dashboard
#[derive(Debug, Serialize)]
pub struct AggregateStats {
    pub total_events: i64,
    pub total_users: i64,
    pub overall_availability_rate: f64,
    pub overall_attendance_rate: f64,
    pub avg_available_per_event: f64,
    pub avg_checked_in_per_event: f64,
    pub most_attended_event: Option<EventSummary>,
    pub least_attended_event: Option<EventSummary>,
    pub most_reliable_users: Vec<UserAttendanceSummary>,
    pub events_by_type: Vec<EventTypeStats>,
}

#[derive(Debug, Serialize)]
pub struct EventTypeStats {
    pub event_type: String,
    pub count: i64,
    pub avg_attendance: f64,
}

/// Full attendance matrix response
#[derive(Debug, Serialize)]
pub struct AttendanceMatrixResponse {
    pub events: Vec<EventSummary>,
    pub rows: Vec<AttendanceMatrixRow>,
    pub aggregate_stats: AggregateStats,
}
