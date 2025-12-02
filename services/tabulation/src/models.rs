use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Enums - Match PostgreSQL types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "team_format", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TeamFormat {
    TwoTeam,
    FourTeam,
}

impl std::fmt::Display for TeamFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamFormat::TwoTeam => write!(f, "two_team"),
            TeamFormat::FourTeam => write!(f, "four_team"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "two_team_position", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TwoTeamPosition {
    Government,
    Opposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "four_team_position", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FourTeamPosition {
    OpeningGovernment,
    OpeningOpposition,
    ClosingGovernment,
    ClosingOpposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "two_team_speaker_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TwoTeamSpeakerRole {
    PrimeMinister,
    DeputyPrimeMinister,
    GovernmentWhip,
    LeaderOfOpposition,
    DeputyLeaderOfOpposition,
    OppositionWhip,
    GovernmentReply,
    OppositionReply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "four_team_speaker_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FourTeamSpeakerRole {
    PrimeMinister,
    DeputyPrimeMinister,
    LeaderOfOpposition,
    DeputyLeaderOfOpposition,
    MemberOfGovernment,
    GovernmentWhip,
    MemberOfOpposition,
    OppositionWhip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "allocation_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AllocationRole {
    Speaker,
    Resource,
    VotingAdjudicator,
    NonVotingAdjudicator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "match_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    Draft,
    Published,
    InProgress,
    Completed,
    Cancelled,
}

// ============================================================================
// Database Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MatchSeries {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub round_number: Option<i32>, // Optional for friendly matches
    pub team_format: TeamFormat,
    pub allow_reply_speeches: bool,
    pub is_break_round: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Match {
    pub id: Uuid,
    pub series_id: Uuid,
    pub room_name: Option<String>,
    pub motion: Option<String>,
    pub info_slide: Option<String>,
    pub status: MatchStatus,
    pub scheduled_time: Option<DateTime<Utc>>,
    pub scores_released: bool,
    pub rankings_released: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MatchTeam {
    pub id: Uuid,
    pub match_id: Uuid,
    pub two_team_position: Option<TwoTeamPosition>,
    pub four_team_position: Option<FourTeamPosition>,
    pub team_name: Option<String>,
    pub institution: Option<String>,
    pub final_rank: Option<i32>,
    pub total_speaker_points: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Allocation {
    pub id: Uuid,
    pub match_id: Uuid,
    pub user_id: Option<Uuid>,      // Nullable for guest allocations
    pub guest_name: Option<String>, // Name for non-user participants
    pub role: AllocationRole,
    pub team_id: Option<Uuid>,
    pub two_team_speaker_role: Option<TwoTeamSpeakerRole>,
    pub four_team_speaker_role: Option<FourTeamSpeakerRole>,
    pub is_chair: Option<bool>,
    pub allocated_at: DateTime<Utc>,
    pub allocated_by: Uuid,
    pub was_checked_in: bool, // Track if user was checked in when allocated
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AllocationWithUser {
    pub id: Uuid,
    pub match_id: Uuid,
    pub user_id: Option<Uuid>,      // Nullable for guest allocations
    pub guest_name: Option<String>, // Name for guest participants
    pub username: String,           // Will be guest_name if user_id is None
    pub role: AllocationRole,
    pub team_id: Option<Uuid>,
    pub two_team_speaker_role: Option<TwoTeamSpeakerRole>,
    pub four_team_speaker_role: Option<FourTeamSpeakerRole>,
    pub is_chair: Option<bool>,
    pub allocated_at: DateTime<Utc>,
    pub allocated_by: Uuid,
    pub was_checked_in: bool, // Track if user was checked in when allocated
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Ballot {
    pub id: Uuid,
    pub match_id: Uuid,
    pub adjudicator_id: Uuid,
    pub is_voting: bool,
    pub is_submitted: bool,
    pub submitted_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SpeakerScore {
    pub id: Uuid,
    pub ballot_id: Uuid,
    pub allocation_id: Uuid,
    pub score: Decimal,
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamRanking {
    pub id: Uuid,
    pub ballot_id: Uuid,
    pub team_id: Uuid,
    pub rank: i32,
    pub is_winner: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AllocationHistory {
    pub id: Uuid,
    pub allocation_id: Option<Uuid>,
    pub match_id: Uuid,
    pub user_id: Option<Uuid>,
    pub guest_name: Option<String>,
    pub action: String,
    pub previous_role: Option<AllocationRole>,
    pub new_role: Option<AllocationRole>,
    pub previous_team_id: Option<Uuid>,
    pub new_team_id: Option<Uuid>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub notes: Option<String>,
}

// ============================================================================
// JWT Claims
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub token_type: String,
}

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSeriesRequest {
    pub event_id: Uuid,
    #[validate(length(
        min = 1,
        max = 255,
        message = "Name must be between 1 and 255 characters"
    ))]
    pub name: String,
    pub description: Option<String>,
    pub round_number: Option<i32>, // Optional for friendly matches
    pub team_format: TeamFormat,
    #[serde(default)]
    pub allow_reply_speeches: bool,
    #[serde(default)]
    pub is_break_round: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSeriesRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub allow_reply_speeches: Option<bool>,
    pub is_break_round: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMatchRequest {
    pub series_id: Uuid,
    #[validate(length(max = 255))]
    pub room_name: Option<String>,
    pub motion: Option<String>,
    pub info_slide: Option<String>,
    pub scheduled_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMatchRequest {
    #[validate(length(max = 255))]
    pub room_name: Option<String>,
    pub motion: Option<String>,
    pub info_slide: Option<String>,
    pub status: Option<MatchStatus>,
    pub scheduled_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseToggleRequest {
    pub scores_released: Option<bool>,
    pub rankings_released: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTeamRequest {
    pub match_id: Uuid,
    pub two_team_position: Option<TwoTeamPosition>,
    pub four_team_position: Option<FourTeamPosition>,
    #[validate(length(max = 255))]
    pub team_name: Option<String>,
    #[validate(length(max = 255))]
    pub institution: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTeamRequest {
    #[validate(length(max = 255))]
    pub team_name: Option<String>,
    #[validate(length(max = 255))]
    pub institution: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAllocationRequest {
    pub match_id: Uuid,
    pub user_id: Option<Uuid>,      // Optional for guest allocations
    pub guest_name: Option<String>, // Required if user_id is None
    pub role: AllocationRole,
    pub team_id: Option<Uuid>,
    pub two_team_speaker_role: Option<TwoTeamSpeakerRole>,
    pub four_team_speaker_role: Option<FourTeamSpeakerRole>,
    #[serde(default)]
    pub is_chair: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAllocationRequest {
    pub role: Option<AllocationRole>,
    pub team_id: Option<Uuid>,
    pub two_team_speaker_role: Option<TwoTeamSpeakerRole>,
    pub four_team_speaker_role: Option<FourTeamSpeakerRole>,
    pub is_chair: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SwapAllocationRequest {
    pub allocation_id_1: Uuid,
    pub allocation_id_2: Uuid,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SubmitBallotRequest {
    pub match_id: Uuid,
    #[validate(length(max = 5000))]
    pub notes: Option<String>,
    pub speaker_scores: Vec<SpeakerScoreInput>,
    pub team_rankings: Vec<TeamRankingInput>,
}

#[derive(Debug, Deserialize)]
pub struct SpeakerScoreInput {
    pub allocation_id: Uuid,
    pub score: f64, // Accept as f64, convert to Decimal when storing
    pub feedback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TeamRankingInput {
    pub team_id: Uuid,
    pub rank: i32,
    pub is_winner: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SubmitFeedbackRequest {
    pub match_id: Uuid,
    #[validate(length(max = 5000))]
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct SeriesListQuery {
    pub event_id: Uuid,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MatchListQuery {
    pub series_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub status: Option<MatchStatus>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PerformanceQuery {
    pub event_id: Option<Uuid>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SeriesResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub round_number: Option<i32>, // Optional for friendly matches
    pub team_format: TeamFormat,
    pub allow_reply_speeches: bool,
    pub is_break_round: bool,
    pub match_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SeriesListResponse {
    pub series: Vec<SeriesResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Serialize)]
pub struct MatchResponse {
    pub id: Uuid,
    pub series_id: Uuid,
    pub series_name: String,
    pub room_name: Option<String>,
    pub motion: Option<String>,
    pub info_slide: Option<String>,
    pub status: MatchStatus,
    pub scheduled_time: Option<DateTime<Utc>>,
    pub scores_released: bool,
    pub rankings_released: bool,
    pub teams: Vec<MatchTeamResponse>,
    pub adjudicators: Vec<AdjudicatorResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MatchListResponse {
    pub matches: Vec<MatchResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Serialize)]
pub struct MatchTeamResponse {
    pub id: Uuid,
    pub two_team_position: Option<TwoTeamPosition>,
    pub four_team_position: Option<FourTeamPosition>,
    pub team_name: Option<String>,
    pub institution: Option<String>,
    pub final_rank: Option<i32>,
    pub total_speaker_points: Option<Decimal>,
    pub speakers: Vec<SpeakerResponse>,
    pub resources: Vec<ResourceResponse>,
}

#[derive(Debug, Serialize)]
pub struct SpeakerResponse {
    pub allocation_id: Uuid,
    pub user_id: Option<Uuid>,
    pub guest_name: Option<String>,
    pub username: String,
    pub two_team_speaker_role: Option<TwoTeamSpeakerRole>,
    pub four_team_speaker_role: Option<FourTeamSpeakerRole>,
    pub score: Option<Decimal>, // Only shown if scores_released
}

#[derive(Debug, Serialize)]
pub struct ResourceResponse {
    pub allocation_id: Uuid,
    pub user_id: Option<Uuid>,
    pub guest_name: Option<String>,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct AdjudicatorResponse {
    pub allocation_id: Uuid,
    pub user_id: Option<Uuid>,
    pub guest_name: Option<String>,
    pub username: String,
    pub is_voting: bool,
    pub is_chair: bool,
    pub has_submitted: bool,
}

#[derive(Debug, Serialize)]
pub struct BallotResponse {
    pub id: Uuid,
    pub match_id: Uuid,
    pub adjudicator_id: Uuid,
    pub adjudicator_username: String,
    pub is_voting: bool,
    pub is_submitted: bool,
    pub submitted_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub speaker_scores: Vec<SpeakerScoreResponse>,
    pub team_rankings: Vec<TeamRankingResponse>,
}

#[derive(Debug, Serialize)]
pub struct SpeakerScoreResponse {
    pub id: Uuid,
    pub allocation_id: Uuid,
    pub speaker_username: String,
    pub score: Decimal,
    pub feedback: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TeamRankingResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: Option<String>,
    pub rank: i32,
    pub is_winner: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PerformanceResponse {
    pub user_id: Uuid,
    pub username: String,
    pub total_rounds: i64,
    pub rounds_as_speaker: i64,
    pub rounds_as_adjudicator: i64,
    pub average_speaker_score: Option<Decimal>,
    pub total_wins: i64,
    pub total_losses: i64,
    pub win_rate: Option<Decimal>,
    pub rankings: Vec<RankingCount>,
}

#[derive(Debug, Serialize)]
pub struct RankingCount {
    pub rank: i32,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct PerformanceListResponse {
    pub performances: Vec<PerformanceResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

#[derive(Debug, Serialize)]
pub struct CheckedInUserResponse {
    pub user_id: Uuid,
    pub username: String,
    pub checked_in_at: DateTime<Utc>,
    pub is_allocated: bool,
    pub current_allocation: Option<CurrentAllocationInfo>,
}

#[derive(Debug, Serialize)]
pub struct CurrentAllocationInfo {
    pub match_id: Uuid,
    pub room_name: Option<String>,
    pub role: AllocationRole,
}

#[derive(Debug, Serialize)]
pub struct AllocationPoolResponse {
    pub event_id: Uuid,
    pub series_id: Uuid,
    pub checked_in_users: Vec<CheckedInUserResponse>,
    pub total_checked_in: i64,
    pub total_allocated: i64,
    pub total_available: i64,
}

#[derive(Debug, Serialize)]
pub struct AllocationHistoryResponse {
    pub history: Vec<AllocationHistory>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

// User info for joining with other tables
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
}

// Event info for validation
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventInfo {
    pub id: Uuid,
    pub title: String,
    pub is_locked: bool,
}

// Attendance record for checking check-in status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttendanceInfo {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub is_checked_in: bool,
    pub checked_in_at: Option<DateTime<Utc>>,
}
