use crate::models::{
    Award, AwardHistory, AwardHistoryWithAdmin, AwardTier, AwardWithAdmin,
    MeritHistory, MeritHistoryWithAdmin, UserMerit, UserMeritInfo,
};
use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("../migrations").run(&self.pool).await?;
        Ok(())
    }

    // ========================================================================
    // Merit Methods
    // ========================================================================

    /// Get merit for a user, returns None if user doesn't have merit initialized
    pub async fn get_user_merit(&self, user_id: Uuid) -> Result<Option<UserMerit>, sqlx::Error> {
        let merit = sqlx::query_as::<_, UserMerit>(
            r#"
            SELECT id, user_id, merit_points, created_at, updated_at
            FROM user_merit
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(merit)
    }

    /// Initialize merit for a user (called when user verifies email or manually by admin)
    pub async fn initialize_user_merit(&self, user_id: Uuid) -> Result<UserMerit, sqlx::Error> {
        let merit = sqlx::query_as::<_, UserMerit>(
            r#"
            INSERT INTO user_merit (user_id, merit_points, created_at, updated_at)
            VALUES ($1, 0, $2, $2)
            ON CONFLICT (user_id) DO UPDATE SET updated_at = $2
            RETURNING id, user_id, merit_points, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(merit)
    }

    /// Update merit points for a user (admin action)
    /// Returns the updated merit record and creates a history entry
    pub async fn update_merit(
        &self,
        user_id: Uuid,
        admin_id: Uuid,
        change_amount: i32,
        reason: &str,
    ) -> Result<(UserMerit, MeritHistory), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Get current merit or initialize if not exists
        let current_merit = sqlx::query_as::<_, UserMerit>(
            r#"
            INSERT INTO user_merit (user_id, merit_points, created_at, updated_at)
            VALUES ($1, 0, $2, $2)
            ON CONFLICT (user_id) DO NOTHING
            RETURNING id, user_id, merit_points, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(Utc::now())
        .fetch_optional(&mut *tx)
        .await?;

        let current_merit = match current_merit {
            Some(m) => m,
            None => {
                sqlx::query_as::<_, UserMerit>(
                    r#"
                    SELECT id, user_id, merit_points, created_at, updated_at
                    FROM user_merit
                    WHERE user_id = $1
                    "#,
                )
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?
            }
        };

        let previous_total = current_merit.merit_points;
        let new_total = previous_total + change_amount;

        // Update merit points
        let updated_merit = sqlx::query_as::<_, UserMerit>(
            r#"
            UPDATE user_merit
            SET merit_points = $2, updated_at = $3
            WHERE user_id = $1
            RETURNING id, user_id, merit_points, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(new_total)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await?;

        // Create history record
        let history = sqlx::query_as::<_, MeritHistory>(
            r#"
            INSERT INTO merit_history (user_id, admin_id, change_amount, previous_total, new_total, reason, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, admin_id, change_amount, previous_total, new_total, reason, created_at
            "#,
        )
        .bind(user_id)
        .bind(admin_id)
        .bind(change_amount)
        .bind(previous_total)
        .bind(new_total)
        .bind(reason)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((updated_merit, history))
    }

    /// Get merit history for a user with pagination
    pub async fn get_merit_history(
        &self,
        user_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<MeritHistoryWithAdmin>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM merit_history WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        let history = sqlx::query_as::<_, MeritHistoryWithAdmin>(
            r#"
            SELECT 
                mh.id,
                mh.user_id,
                mh.admin_id,
                u.username as admin_username,
                mh.change_amount,
                mh.previous_total,
                mh.new_total,
                mh.reason,
                mh.created_at
            FROM merit_history mh
            LEFT JOIN users u ON mh.admin_id = u.id
            WHERE mh.user_id = $1
            ORDER BY mh.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((history, total.0))
    }

    /// List all users with their merit (admin only) with pagination
    pub async fn list_all_user_merits(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<UserMeritInfo>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) 
            FROM users u 
            INNER JOIN user_merit um ON u.id = um.user_id
            WHERE u.email_verified = true
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let users = sqlx::query_as::<_, UserMeritInfo>(
            r#"
            SELECT 
                um.user_id,
                u.username,
                um.merit_points
            FROM user_merit um
            INNER JOIN users u ON um.user_id = u.id
            WHERE u.email_verified = true
            ORDER BY um.merit_points DESC, u.username ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((users, total.0))
    }

    // ========================================================================
    // User Profile Methods (read from users table)
    // ========================================================================

    /// Get public user profile by username
    pub async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserProfileRow>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT 
                u.id,
                u.username,
                u.email,
                u.reg_number,
                u.year_joined,
                u.phone_number,
                u.email_verified,
                u.created_at,
                COALESCE(um.merit_points, 0) as merit_points,
                CASE WHEN a.user_id IS NOT NULL THEN true ELSE false END as is_admin
            FROM users u
            LEFT JOIN user_merit um ON u.id = um.user_id
            LEFT JOIN admin_users a ON u.id = a.user_id
            WHERE u.username = $1 AND u.email_verified = true
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Get user profile by ID
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<UserProfileRow>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT 
                u.id,
                u.username,
                u.email,
                u.reg_number,
                u.year_joined,
                u.phone_number,
                u.email_verified,
                u.created_at,
                COALESCE(um.merit_points, 0) as merit_points,
                CASE WHEN a.user_id IS NOT NULL THEN true ELSE false END as is_admin
            FROM users u
            LEFT JOIN user_merit um ON u.id = um.user_id
            LEFT JOIN admin_users a ON u.id = a.user_id
            WHERE u.id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Check if a user is an admin
    pub async fn is_user_admin(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result: Option<(Uuid,)> =
            sqlx::query_as("SELECT user_id FROM admin_users WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(result.is_some())
    }

    // ========================================================================
    // Award Methods
    // ========================================================================

    /// Create a new award for a user
    pub async fn create_award(
        &self,
        user_id: Uuid,
        admin_id: Uuid,
        title: &str,
        description: Option<&str>,
        tier: AwardTier,
        reason: &str,
    ) -> Result<(Award, AwardHistory), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Create the award
        let award = sqlx::query_as::<_, Award>(
            r#"
            INSERT INTO awards (user_id, title, description, tier, awarded_by, awarded_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $6, $6)
            RETURNING id, user_id, title, description, tier, awarded_by, awarded_at, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(title)
        .bind(description)
        .bind(tier)
        .bind(admin_id)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await?;

        // Create history entry for award creation
        let history = sqlx::query_as::<_, AwardHistory>(
            r#"
            INSERT INTO award_history (award_id, user_id, admin_id, previous_tier, new_tier, reason, created_at)
            VALUES ($1, $2, $3, NULL, $4, $5, $6)
            RETURNING id, award_id, user_id, admin_id, previous_tier, new_tier, reason, created_at
            "#,
        )
        .bind(award.id)
        .bind(user_id)
        .bind(admin_id)
        .bind(tier)
        .bind(reason)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((award, history))
    }

    /// Upgrade an award's tier
    pub async fn upgrade_award(
        &self,
        award_id: Uuid,
        admin_id: Uuid,
        new_tier: AwardTier,
        new_title: Option<&str>,
        reason: &str,
    ) -> Result<(Award, AwardHistory), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Get current award
        let current_award = sqlx::query_as::<_, Award>(
            "SELECT id, user_id, title, description, tier, awarded_by, awarded_at, created_at, updated_at FROM awards WHERE id = $1",
        )
        .bind(award_id)
        .fetch_one(&mut *tx)
        .await?;

        let previous_tier = current_award.tier;

        // Update the award tier and optionally title
        let updated_award = if let Some(title) = new_title {
            sqlx::query_as::<_, Award>(
                r#"
                UPDATE awards
                SET tier = $2, title = $3, updated_at = $4
                WHERE id = $1
                RETURNING id, user_id, title, description, tier, awarded_by, awarded_at, created_at, updated_at
                "#,
            )
            .bind(award_id)
            .bind(new_tier)
            .bind(title)
            .bind(Utc::now())
            .fetch_one(&mut *tx)
            .await?
        } else {
            sqlx::query_as::<_, Award>(
                r#"
                UPDATE awards
                SET tier = $2, updated_at = $3
                WHERE id = $1
                RETURNING id, user_id, title, description, tier, awarded_by, awarded_at, created_at, updated_at
                "#,
            )
            .bind(award_id)
            .bind(new_tier)
            .bind(Utc::now())
            .fetch_one(&mut *tx)
            .await?
        };

        // Create history entry
        let history = sqlx::query_as::<_, AwardHistory>(
            r#"
            INSERT INTO award_history (award_id, user_id, admin_id, previous_tier, new_tier, reason, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, award_id, user_id, admin_id, previous_tier, new_tier, reason, created_at
            "#,
        )
        .bind(award_id)
        .bind(current_award.user_id)
        .bind(admin_id)
        .bind(previous_tier)
        .bind(new_tier)
        .bind(reason)
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((updated_award, history))
    }

    /// Get award by ID
    pub async fn get_award_by_id(&self, award_id: Uuid) -> Result<Option<Award>, sqlx::Error> {
        let award = sqlx::query_as::<_, Award>(
            "SELECT id, user_id, title, description, tier, awarded_by, awarded_at, created_at, updated_at FROM awards WHERE id = $1",
        )
        .bind(award_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(award)
    }

    /// Get award with admin info
    pub async fn get_award_with_admin(&self, award_id: Uuid) -> Result<Option<AwardWithAdmin>, sqlx::Error> {
        let award = sqlx::query_as::<_, AwardWithAdmin>(
            r#"
            SELECT 
                a.id, a.user_id, u2.username,
                a.title, a.description, a.tier,
                a.awarded_by, u.username as awarded_by_username,
                a.awarded_at, a.created_at, a.updated_at
            FROM awards a
            JOIN users u2 ON a.user_id = u2.id
            LEFT JOIN users u ON a.awarded_by = u.id
            WHERE a.id = $1
            "#,
        )
        .bind(award_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(award)
    }

    /// Get all awards for a user (public - for profile display)
    pub async fn get_user_awards(&self, user_id: Uuid) -> Result<Vec<Award>, sqlx::Error> {
        let awards = sqlx::query_as::<_, Award>(
            r#"
            SELECT id, user_id, title, description, tier, awarded_by, awarded_at, created_at, updated_at
            FROM awards
            WHERE user_id = $1
            ORDER BY 
                CASE tier 
                    WHEN 'gold' THEN 1 
                    WHEN 'silver' THEN 2 
                    WHEN 'bronze' THEN 3 
                END,
                awarded_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(awards)
    }

    /// Get award history for a specific award
    pub async fn get_award_history(&self, award_id: Uuid) -> Result<Vec<AwardHistoryWithAdmin>, sqlx::Error> {
        let history = sqlx::query_as::<_, AwardHistoryWithAdmin>(
            r#"
            SELECT 
                ah.id, ah.award_id, ah.user_id, ah.admin_id,
                u.username as admin_username,
                ah.previous_tier, ah.new_tier, ah.reason, ah.created_at
            FROM award_history ah
            LEFT JOIN users u ON ah.admin_id = u.id
            WHERE ah.award_id = $1
            ORDER BY ah.created_at DESC
            "#,
        )
        .bind(award_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// Delete an award
    pub async fn delete_award(&self, award_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM awards WHERE id = $1")
            .bind(award_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Edit an award (update title, description, tier)
    pub async fn edit_award(
        &self,
        award_id: Uuid,
        title: &str,
        description: Option<&str>,
        tier: AwardTier,
    ) -> Result<Award, sqlx::Error> {
        let award = sqlx::query_as::<_, Award>(
            r#"
            UPDATE awards
            SET title = $2, description = $3, tier = $4, updated_at = $5
            WHERE id = $1
            RETURNING id, user_id, title, description, tier, awarded_by, awarded_at, created_at, updated_at
            "#,
        )
        .bind(award_id)
        .bind(title)
        .bind(description)
        .bind(tier)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(award)
    }

    /// Get all award history for a user (across all their awards)
    pub async fn get_user_award_history(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<AwardHistoryWithAdmin>, sqlx::Error> {
        let history = sqlx::query_as::<_, AwardHistoryWithAdmin>(
            r#"
            SELECT 
                ah.id,
                ah.award_id,
                ah.user_id,
                ah.admin_id,
                u.username as admin_username,
                ah.previous_tier,
                ah.new_tier,
                ah.reason,
                ah.created_at
            FROM award_history ah
            LEFT JOIN users u ON u.id = ah.admin_id
            WHERE ah.user_id = $1
            ORDER BY ah.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// List all awards with pagination (admin)
    pub async fn list_all_awards(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<AwardWithAdmin>, i64), sqlx::Error> {
        let awards = sqlx::query_as::<_, AwardWithAdmin>(
            r#"
            SELECT 
                a.id,
                a.user_id,
                u.username,
                a.title,
                a.description,
                a.tier,
                a.awarded_by,
                u2.username as awarded_by_username,
                a.awarded_at,
                a.created_at,
                a.updated_at
            FROM awards a
            JOIN users u ON u.id = a.user_id
            LEFT JOIN users u2 ON u2.id = a.awarded_by
            ORDER BY a.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM awards")
            .fetch_one(&self.pool)
            .await?;

        Ok((awards, total.0))
    }
}

/// Internal row type for user profile queries
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserProfileRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub reg_number: String,
    pub year_joined: i32,
    pub phone_number: String,
    pub email_verified: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub merit_points: i32,
    pub is_admin: bool,
}
