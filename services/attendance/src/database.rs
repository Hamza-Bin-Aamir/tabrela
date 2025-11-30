use crate::models::{AttendanceRecord, AttendanceRecordWithUser, Event};
use chrono::{DateTime, Utc};
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
    // Event Methods
    // ========================================================================

    pub async fn create_event(
        &self,
        title: &str,
        description: Option<&str>,
        event_type: &str,
        event_date: DateTime<Utc>,
        location: Option<&str>,
        created_by: Uuid,
    ) -> Result<Event, sqlx::Error> {
        let event = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO events (id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8, $8)
            RETURNING id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(title)
        .bind(description)
        .bind(event_type)
        .bind(event_date)
        .bind(location)
        .bind(created_by)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(event)
    }

    pub async fn get_event_by_id(&self, event_id: Uuid) -> Result<Option<Event>, sqlx::Error> {
        let event = sqlx::query_as::<_, Event>(
            r#"
            SELECT id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
            FROM events
            WHERE id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(event)
    }

    pub async fn list_events(
        &self,
        page: i32,
        per_page: i32,
        event_type: Option<&str>,
        upcoming_only: bool,
    ) -> Result<(Vec<Event>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;

        // Build dynamic query based on filters
        let (events, total): (Vec<Event>, i64) = if upcoming_only {
            if let Some(et) = event_type {
                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM events WHERE event_type = $1 AND event_date >= $2",
                )
                .bind(et)
                .bind(Utc::now())
                .fetch_one(&self.pool)
                .await?;

                let events = sqlx::query_as::<_, Event>(
                    r#"
                    SELECT id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
                    FROM events
                    WHERE event_type = $1 AND event_date >= $2
                    ORDER BY event_date ASC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(et)
                .bind(Utc::now())
                .bind(per_page as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?;

                (events, total.0)
            } else {
                let total: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM events WHERE event_date >= $1")
                        .bind(Utc::now())
                        .fetch_one(&self.pool)
                        .await?;

                let events = sqlx::query_as::<_, Event>(
                    r#"
                    SELECT id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
                    FROM events
                    WHERE event_date >= $1
                    ORDER BY event_date ASC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(Utc::now())
                .bind(per_page as i64)
                .bind(offset as i64)
                .fetch_all(&self.pool)
                .await?;

                (events, total.0)
            }
        } else if let Some(et) = event_type {
            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events WHERE event_type = $1")
                .bind(et)
                .fetch_one(&self.pool)
                .await?;

            let events = sqlx::query_as::<_, Event>(
                r#"
                SELECT id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
                FROM events
                WHERE event_type = $1
                ORDER BY event_date DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(et)
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            (events, total.0)
        } else {
            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
                .fetch_one(&self.pool)
                .await?;

            let events = sqlx::query_as::<_, Event>(
                r#"
                SELECT id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
                FROM events
                ORDER BY event_date DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(per_page as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?;

            (events, total.0)
        };

        Ok((events, total))
    }

    pub async fn update_event(
        &self,
        event_id: Uuid,
        title: Option<&str>,
        description: Option<&str>,
        event_type: Option<&str>,
        event_date: Option<DateTime<Utc>>,
        location: Option<&str>,
    ) -> Result<Option<Event>, sqlx::Error> {
        // Get current event first
        let current = match self.get_event_by_id(event_id).await? {
            Some(e) => e,
            None => return Ok(None),
        };

        let event = sqlx::query_as::<_, Event>(
            r#"
            UPDATE events
            SET title = $1, description = $2, event_type = $3, event_date = $4, location = $5, updated_at = $6
            WHERE id = $7
            RETURNING id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
            "#,
        )
        .bind(title.unwrap_or(&current.title))
        .bind(description.or(current.description.as_deref()))
        .bind(event_type.unwrap_or(&current.event_type))
        .bind(event_date.unwrap_or(current.event_date))
        .bind(location.or(current.location.as_deref()))
        .bind(Utc::now())
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(event)
    }

    pub async fn delete_event(&self, event_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM events WHERE id = $1")
            .bind(event_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn lock_event(
        &self,
        event_id: Uuid,
        is_locked: bool,
    ) -> Result<Option<Event>, sqlx::Error> {
        let event = sqlx::query_as::<_, Event>(
            r#"
            UPDATE events
            SET is_locked = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
            "#,
        )
        .bind(is_locked)
        .bind(Utc::now())
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(event)
    }

    // ========================================================================
    // Attendance Methods
    // ========================================================================

    pub async fn get_attendance_record(
        &self,
        event_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<AttendanceRecord>, sqlx::Error> {
        let record = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            SELECT id, event_id, user_id, is_available, is_checked_in, checked_in_by, checked_in_at, availability_set_at, created_at, updated_at
            FROM attendance_records
            WHERE event_id = $1 AND user_id = $2
            "#,
        )
        .bind(event_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn set_availability(
        &self,
        event_id: Uuid,
        user_id: Uuid,
        is_available: bool,
    ) -> Result<AttendanceRecord, sqlx::Error> {
        // Upsert - insert or update
        let record = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            INSERT INTO attendance_records (id, event_id, user_id, is_available, is_checked_in, availability_set_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, false, $5, $5, $5)
            ON CONFLICT (event_id, user_id)
            DO UPDATE SET is_available = $4, availability_set_at = $5, updated_at = $5
            RETURNING id, event_id, user_id, is_available, is_checked_in, checked_in_by, checked_in_at, availability_set_at, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(user_id)
        .bind(is_available)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn check_in_user(
        &self,
        event_id: Uuid,
        user_id: Uuid,
        is_checked_in: bool,
        checked_in_by: Uuid,
    ) -> Result<AttendanceRecord, sqlx::Error> {
        let now = Utc::now();
        let checked_in_at = if is_checked_in { Some(now) } else { None };

        // Upsert with check-in information
        let record = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            INSERT INTO attendance_records (id, event_id, user_id, is_available, is_checked_in, checked_in_by, checked_in_at, availability_set_at, created_at, updated_at)
            VALUES ($1, $2, $3, true, $4, $5, $6, $7, $7, $7)
            ON CONFLICT (event_id, user_id)
            DO UPDATE SET is_checked_in = $4, checked_in_by = $5, checked_in_at = $6, updated_at = $7
            RETURNING id, event_id, user_id, is_available, is_checked_in, checked_in_by, checked_in_at, availability_set_at, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(event_id)
        .bind(user_id)
        .bind(is_checked_in)
        .bind(if is_checked_in { Some(checked_in_by) } else { None })
        .bind(checked_in_at)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn revoke_availability(
        &self,
        event_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<AttendanceRecord>, sqlx::Error> {
        let record = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            UPDATE attendance_records
            SET is_available = false, is_checked_in = false, checked_in_by = NULL, checked_in_at = NULL, updated_at = $1
            WHERE event_id = $2 AND user_id = $3
            RETURNING id, event_id, user_id, is_available, is_checked_in, checked_in_by, checked_in_at, availability_set_at, created_at, updated_at
            "#,
        )
        .bind(Utc::now())
        .bind(event_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_event_attendance(
        &self,
        event_id: Uuid,
    ) -> Result<Vec<AttendanceRecordWithUser>, sqlx::Error> {
        let records = sqlx::query_as::<_, AttendanceRecordWithUser>(
            r#"
            SELECT 
                ar.id, ar.event_id, ar.user_id, 
                u.username,
                ar.is_available, ar.is_checked_in, ar.checked_in_by, ar.checked_in_at, 
                ar.availability_set_at, ar.created_at, ar.updated_at
            FROM attendance_records ar
            JOIN users u ON ar.user_id = u.id
            WHERE ar.event_id = $1
            ORDER BY ar.is_checked_in DESC, ar.is_available DESC, ar.availability_set_at ASC
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_attendance_stats(&self, event_id: Uuid) -> Result<(i64, i64), sqlx::Error> {
        let available: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM attendance_records WHERE event_id = $1 AND is_available = true",
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await?;

        let checked_in: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM attendance_records WHERE event_id = $1 AND is_checked_in = true",
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((available.0, checked_in.0))
    }

    // ========================================================================
    // Attendance Matrix/Dashboard Methods
    // ========================================================================

    /// Get all events ordered by date for the matrix
    pub async fn get_all_events_for_matrix(&self) -> Result<Vec<Event>, sqlx::Error> {
        let events = sqlx::query_as::<_, Event>(
            r#"
            SELECT id, title, description, event_type, event_date, location, created_by, is_locked, created_at, updated_at
            FROM events
            ORDER BY event_date ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    pub async fn get_all_users(&self) -> Result<Vec<(Uuid, String)>, sqlx::Error> {
        let users: Vec<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT id, username
            FROM users
            ORDER BY username ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    /// Get all attendance records for the matrix (returns event_id, user_id, is_available, is_checked_in)
    pub async fn get_all_attendance_records(
        &self,
    ) -> Result<Vec<(Uuid, Uuid, bool, bool)>, sqlx::Error> {
        let records: Vec<(Uuid, Uuid, bool, bool)> = sqlx::query_as(
            r#"
            SELECT event_id, user_id, is_available, is_checked_in
            FROM attendance_records
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Get event stats (available count, checked-in count) for each event
    pub async fn get_all_event_stats(&self) -> Result<Vec<(Uuid, i64, i64)>, sqlx::Error> {
        let stats: Vec<(Uuid, i64, i64)> = sqlx::query_as(
            r#"
            SELECT 
                event_id,
                COUNT(*) FILTER (WHERE is_available = true) as available_count,
                COUNT(*) FILTER (WHERE is_checked_in = true) as checked_in_count
            FROM attendance_records
            GROUP BY event_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Get user attendance stats
    pub async fn get_user_attendance_stats(
        &self,
    ) -> Result<Vec<(Uuid, String, i64, i64)>, sqlx::Error> {
        let stats: Vec<(Uuid, String, i64, i64)> = sqlx::query_as(
            r#"
            SELECT 
                u.id,
                u.username,
                COUNT(*) FILTER (WHERE ar.is_available = true) as available_count,
                COUNT(*) FILTER (WHERE ar.is_checked_in = true) as checked_in_count
            FROM users u
            LEFT JOIN attendance_records ar ON u.id = ar.user_id
            GROUP BY u.id, u.username
            ORDER BY u.username ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Get event type statistics
    pub async fn get_event_type_stats(&self) -> Result<Vec<(String, i64, f64)>, sqlx::Error> {
        let stats: Vec<(String, i64, f64)> = sqlx::query_as(
            r#"
            SELECT 
                e.event_type,
                COUNT(DISTINCT e.id)::BIGINT as event_count,
                COALESCE(AVG(
                    CASE WHEN ar.is_checked_in THEN 1.0 ELSE 0.0 END
                )::FLOAT8 * 100, 0)::FLOAT8 as avg_attendance
            FROM events e
            LEFT JOIN attendance_records ar ON e.id = ar.event_id
            GROUP BY e.event_type
            ORDER BY event_count DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(stats)
    }
}
