use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ============================================================================
// Database Models
// ============================================================================

/// User merit record - stores current merit points for a user
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserMerit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub merit_points: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Merit history record - audit log of merit changes
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MeritHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub admin_id: Option<Uuid>,
    pub change_amount: i32,
    pub previous_total: i32,
    pub new_total: i32,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

/// Merit history with admin username for display
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MeritHistoryWithAdmin {
    pub id: Uuid,
    pub user_id: Uuid,
    pub admin_id: Option<Uuid>,
    pub admin_username: Option<String>,
    pub change_amount: i32,
    pub previous_total: i32,
    pub new_total: i32,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// JWT Claims (for validating tokens from auth service)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // user_id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub token_type: String,
}

// ============================================================================
// Request Types
// ============================================================================

/// Request to update merit points (admin only)
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMeritRequest {
    /// The user ID to update merit for
    pub user_id: Uuid,
    /// Amount to change (positive to add, negative to remove)
    pub change_amount: i32,
    /// Required reason for the change
    #[validate(length(min = 3, max = 500, message = "Reason must be between 3 and 500 characters"))]
    pub reason: String,
}

/// Query parameters for listing merit history
#[derive(Debug, Deserialize)]
pub struct MeritHistoryQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

// ============================================================================
// Response Types
// ============================================================================

/// Public profile response - shown to everyone
#[derive(Debug, Serialize)]
pub struct PublicProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub year_joined: i32,
    pub created_at: DateTime<Utc>,
}

/// Private profile response - includes merit (shown to self)
#[derive(Debug, Serialize)]
pub struct PrivateProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub reg_number: String,
    pub year_joined: i32,
    pub phone_number: String,
    pub email_verified: bool,
    pub merit_points: i32,
    pub created_at: DateTime<Utc>,
}

/// Admin profile response - full details including merit
#[derive(Debug, Serialize)]
pub struct AdminProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub reg_number: String,
    pub year_joined: i32,
    pub phone_number: String,
    pub email_verified: bool,
    pub merit_points: i32,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

/// Merit response for current user
#[derive(Debug, Serialize)]
pub struct MeritResponse {
    pub user_id: Uuid,
    pub merit_points: i32,
    pub updated_at: DateTime<Utc>,
}

/// Merit history list response
#[derive(Debug, Serialize)]
pub struct MeritHistoryResponse {
    pub history: Vec<MeritHistoryWithAdmin>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

/// User merit info for admin listing
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserMeritInfo {
    pub user_id: Uuid,
    pub username: String,
    pub merit_points: i32,
}

/// Admin merit list response
#[derive(Debug, Serialize)]
pub struct AdminMeritListResponse {
    pub users: Vec<UserMeritInfo>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

// ============================================================================
// Award Models
// ============================================================================

/// Award tier enum - matches PostgreSQL enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "award_tier", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AwardTier {
    Bronze,
    Silver,
    Gold,
}

impl std::fmt::Display for AwardTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AwardTier::Bronze => write!(f, "bronze"),
            AwardTier::Silver => write!(f, "silver"),
            AwardTier::Gold => write!(f, "gold"),
        }
    }
}

impl AwardTier {
    /// Check if this tier can be upgraded to target tier
    pub fn can_upgrade_to(&self, target: &AwardTier) -> bool {
        match (self, target) {
            (AwardTier::Bronze, AwardTier::Silver) => true,
            (AwardTier::Bronze, AwardTier::Gold) => true,
            (AwardTier::Silver, AwardTier::Gold) => true,
            _ => false,
        }
    }
}

/// Award database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Award {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tier: AwardTier,
    pub awarded_by: Option<Uuid>,
    pub awarded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Award with admin username for display
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AwardWithAdmin {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,  // recipient's username
    pub title: String,
    pub description: Option<String>,
    pub tier: AwardTier,
    pub awarded_by: Option<Uuid>,
    pub awarded_by_username: Option<String>,
    pub awarded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Award history record - tracks creation and upgrades
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AwardHistory {
    pub id: Uuid,
    pub award_id: Uuid,
    pub user_id: Uuid,
    pub admin_id: Option<Uuid>,
    pub previous_tier: Option<AwardTier>,
    pub new_tier: AwardTier,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

/// Award history with admin username
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AwardHistoryWithAdmin {
    pub id: Uuid,
    pub award_id: Uuid,
    pub user_id: Uuid,
    pub admin_id: Option<Uuid>,
    pub admin_username: Option<String>,
    pub previous_tier: Option<AwardTier>,
    pub new_tier: AwardTier,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Award Request Types
// ============================================================================

/// Request to create a new award (admin only)
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAwardRequest {
    pub user_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Title must be between 1 and 255 characters"))]
    pub title: String,
    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,
    pub tier: AwardTier,
    #[validate(length(min = 3, max = 500, message = "Reason must be between 3 and 500 characters"))]
    pub reason: String,
}

/// Request to upgrade an award tier (admin only)
#[derive(Debug, Deserialize, Validate)]
pub struct UpgradeAwardRequest {
    pub new_tier: AwardTier,
    #[validate(length(min = 1, max = 255, message = "Title must be between 1 and 255 characters"))]
    pub new_title: Option<String>,
    #[validate(length(min = 3, max = 500, message = "Reason must be between 3 and 500 characters"))]
    pub reason: String,
}

/// Request to edit an award (admin only)
#[derive(Debug, Deserialize, Validate)]
pub struct EditAwardRequest {
    #[validate(length(min = 1, max = 255, message = "Title must be between 1 and 255 characters"))]
    pub title: String,
    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,
    pub tier: AwardTier,
}

// ============================================================================
// Award Response Types
// ============================================================================

/// Public award response (shown on profile)
#[derive(Debug, Clone, Serialize)]
pub struct AwardResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tier: AwardTier,
    pub awarded_at: DateTime<Utc>,
}

/// Detailed award response for admin
#[derive(Debug, Serialize)]
pub struct AwardDetailResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub title: String,
    pub description: Option<String>,
    pub tier: AwardTier,
    pub awarded_by: Option<Uuid>,
    pub awarded_by_username: Option<String>,
    pub awarded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Award list response
#[derive(Debug, Serialize)]
pub struct AwardListResponse {
    pub awards: Vec<AwardResponse>,
    pub total: i64,
}

/// Award history response
#[derive(Debug, Serialize)]
pub struct AwardHistoryResponse {
    pub history: Vec<AwardHistoryWithAdmin>,
    pub total: i64,
}
