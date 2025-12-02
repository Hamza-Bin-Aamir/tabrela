use crate::models::{
    Allocation, AllocationHistory, AllocationRole, AllocationWithUser, AttendanceInfo, Ballot,
    EventInfo, FourTeamPosition, FourTeamSpeakerRole, Match, MatchSeries, MatchStatus, MatchTeam,
    SpeakerScore, TeamFormat, TeamRanking, TwoTeamPosition, TwoTeamSpeakerRole, UserInfo,
};
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
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
    // User Methods (for validation and lookups)
    // ========================================================================

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<UserInfo>, sqlx::Error> {
        sqlx::query_as::<_, UserInfo>(
            "SELECT id, username FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn is_user_admin(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM admin_users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(count,)| count > 0).unwrap_or(false))
    }

    // ========================================================================
    // Event Methods
    // ========================================================================

    pub async fn get_event_by_id(&self, event_id: Uuid) -> Result<Option<EventInfo>, sqlx::Error> {
        sqlx::query_as::<_, EventInfo>(
            "SELECT id, title, is_locked FROM events WHERE id = $1"
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await
    }

    // ========================================================================
    // Match Series Methods
    // ========================================================================

    pub async fn create_series(&self, series: &MatchSeries) -> Result<MatchSeries, sqlx::Error> {
        sqlx::query_as::<_, MatchSeries>(
            r#"
            INSERT INTO match_series (id, event_id, name, description, round_number, team_format, 
                allow_reply_speeches, is_break_round, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(series.id)
        .bind(series.event_id)
        .bind(&series.name)
        .bind(&series.description)
        .bind(series.round_number)
        .bind(series.team_format)
        .bind(series.allow_reply_speeches)
        .bind(series.is_break_round)
        .bind(series.created_by)
        .bind(series.created_at)
        .bind(series.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_series_by_id(&self, series_id: Uuid) -> Result<Option<MatchSeries>, sqlx::Error> {
        sqlx::query_as::<_, MatchSeries>(
            "SELECT * FROM match_series WHERE id = $1"
        )
        .bind(series_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_series_by_event(
        &self,
        event_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<MatchSeries>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM match_series WHERE event_id = $1"
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await?;

        let series = sqlx::query_as::<_, MatchSeries>(
            r#"
            SELECT * FROM match_series 
            WHERE event_id = $1 
            ORDER BY round_number ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(event_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((series, total.0))
    }

    pub async fn update_series(
        &self,
        series_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        allow_reply_speeches: Option<bool>,
        is_break_round: Option<bool>,
    ) -> Result<MatchSeries, sqlx::Error> {
        sqlx::query_as::<_, MatchSeries>(
            r#"
            UPDATE match_series SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                allow_reply_speeches = COALESCE($4, allow_reply_speeches),
                is_break_round = COALESCE($5, is_break_round),
                updated_at = $6
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(series_id)
        .bind(name)
        .bind(description)
        .bind(allow_reply_speeches)
        .bind(is_break_round)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_series(&self, series_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM match_series WHERE id = $1")
            .bind(series_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_series_match_count(&self, series_id: Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM matches WHERE series_id = $1"
        )
        .bind(series_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(result.0)
    }

    // ========================================================================
    // Match Methods
    // ========================================================================

    pub async fn create_match(&self, match_record: &Match) -> Result<Match, sqlx::Error> {
        sqlx::query_as::<_, Match>(
            r#"
            INSERT INTO matches (id, series_id, room_name, motion, info_slide, status, 
                scheduled_time, scores_released, rankings_released, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(match_record.id)
        .bind(match_record.series_id)
        .bind(&match_record.room_name)
        .bind(&match_record.motion)
        .bind(&match_record.info_slide)
        .bind(match_record.status)
        .bind(match_record.scheduled_time)
        .bind(match_record.scores_released)
        .bind(match_record.rankings_released)
        .bind(match_record.created_at)
        .bind(match_record.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_match_by_id(&self, match_id: Uuid) -> Result<Option<Match>, sqlx::Error> {
        sqlx::query_as::<_, Match>(
            "SELECT * FROM matches WHERE id = $1"
        )
        .bind(match_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_matches_by_series(
        &self,
        series_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<Match>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM matches WHERE series_id = $1"
        )
        .bind(series_id)
        .fetch_one(&self.pool)
        .await?;

        let matches = sqlx::query_as::<_, Match>(
            r#"
            SELECT * FROM matches 
            WHERE series_id = $1 
            ORDER BY room_name ASC, created_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(series_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((matches, total.0))
    }

    pub async fn list_matches_by_event(
        &self,
        event_id: Uuid,
        status: Option<MatchStatus>,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<Match>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;

        let (total, matches) = if let Some(status) = status {
            let total: (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM matches m
                JOIN match_series ms ON m.series_id = ms.id
                WHERE ms.event_id = $1 AND m.status = $2
                "#,
            )
            .bind(event_id)
            .bind(status)
            .fetch_one(&self.pool)
            .await?;

            let matches = sqlx::query_as::<_, Match>(
                r#"
                SELECT m.* FROM matches m
                JOIN match_series ms ON m.series_id = ms.id
                WHERE ms.event_id = $1 AND m.status = $2
                ORDER BY ms.round_number ASC, m.room_name ASC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(event_id)
            .bind(status)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

            (total, matches)
        } else {
            let total: (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM matches m
                JOIN match_series ms ON m.series_id = ms.id
                WHERE ms.event_id = $1
                "#,
            )
            .bind(event_id)
            .fetch_one(&self.pool)
            .await?;

            let matches = sqlx::query_as::<_, Match>(
                r#"
                SELECT m.* FROM matches m
                JOIN match_series ms ON m.series_id = ms.id
                WHERE ms.event_id = $1
                ORDER BY ms.round_number ASC, m.room_name ASC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(event_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

            (total, matches)
        };

        Ok((matches, total.0))
    }

    pub async fn update_match(
        &self,
        match_id: Uuid,
        room_name: Option<&str>,
        motion: Option<&str>,
        info_slide: Option<&str>,
        status: Option<MatchStatus>,
        scheduled_time: Option<chrono::DateTime<Utc>>,
    ) -> Result<Match, sqlx::Error> {
        sqlx::query_as::<_, Match>(
            r#"
            UPDATE matches SET
                room_name = COALESCE($2, room_name),
                motion = COALESCE($3, motion),
                info_slide = COALESCE($4, info_slide),
                status = COALESCE($5, status),
                scheduled_time = COALESCE($6, scheduled_time),
                updated_at = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(match_id)
        .bind(room_name)
        .bind(motion)
        .bind(info_slide)
        .bind(status)
        .bind(scheduled_time)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_match_release(
        &self,
        match_id: Uuid,
        scores_released: Option<bool>,
        rankings_released: Option<bool>,
    ) -> Result<Match, sqlx::Error> {
        sqlx::query_as::<_, Match>(
            r#"
            UPDATE matches SET
                scores_released = COALESCE($2, scores_released),
                rankings_released = COALESCE($3, rankings_released),
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(match_id)
        .bind(scores_released)
        .bind(rankings_released)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_match(&self, match_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM matches WHERE id = $1")
            .bind(match_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Team Methods
    // ========================================================================

    pub async fn create_team(&self, team: &MatchTeam) -> Result<MatchTeam, sqlx::Error> {
        sqlx::query_as::<_, MatchTeam>(
            r#"
            INSERT INTO match_teams (id, match_id, two_team_position, four_team_position, 
                team_name, institution, final_rank, total_speaker_points, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(team.id)
        .bind(team.match_id)
        .bind(team.two_team_position)
        .bind(team.four_team_position)
        .bind(&team.team_name)
        .bind(&team.institution)
        .bind(team.final_rank)
        .bind(team.total_speaker_points)
        .bind(team.created_at)
        .bind(team.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_team_by_id(&self, team_id: Uuid) -> Result<Option<MatchTeam>, sqlx::Error> {
        sqlx::query_as::<_, MatchTeam>(
            "SELECT * FROM match_teams WHERE id = $1"
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_teams_by_match(&self, match_id: Uuid) -> Result<Vec<MatchTeam>, sqlx::Error> {
        sqlx::query_as::<_, MatchTeam>(
            r#"
            SELECT * FROM match_teams 
            WHERE match_id = $1 
            ORDER BY two_team_position, four_team_position
            "#,
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_team(
        &self,
        team_id: Uuid,
        team_name: Option<&str>,
        institution: Option<&str>,
    ) -> Result<MatchTeam, sqlx::Error> {
        sqlx::query_as::<_, MatchTeam>(
            r#"
            UPDATE match_teams SET
                team_name = COALESCE($2, team_name),
                institution = COALESCE($3, institution),
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(team_id)
        .bind(team_name)
        .bind(institution)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_team_results(
        &self,
        team_id: Uuid,
        final_rank: i32,
        total_speaker_points: Decimal,
    ) -> Result<MatchTeam, sqlx::Error> {
        sqlx::query_as::<_, MatchTeam>(
            r#"
            UPDATE match_teams SET
                final_rank = $2,
                total_speaker_points = $3,
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(team_id)
        .bind(final_rank)
        .bind(total_speaker_points)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_team(&self, team_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM match_teams WHERE id = $1")
            .bind(team_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Create teams for a match based on format
    pub async fn create_teams_for_match(
        &self,
        match_id: Uuid,
        team_format: TeamFormat,
    ) -> Result<Vec<MatchTeam>, sqlx::Error> {
        let now = Utc::now();
        let mut teams = Vec::new();

        match team_format {
            TeamFormat::TwoTeam => {
                // Create Government and Opposition teams
                for position in [TwoTeamPosition::Government, TwoTeamPosition::Opposition] {
                    let team = MatchTeam {
                        id: Uuid::new_v4(),
                        match_id,
                        two_team_position: Some(position),
                        four_team_position: None,
                        team_name: None,
                        institution: None,
                        final_rank: None,
                        total_speaker_points: None,
                        created_at: now,
                        updated_at: now,
                    };
                    let created = self.create_team(&team).await?;
                    teams.push(created);
                }
            }
            TeamFormat::FourTeam => {
                // Create OG, OO, CG, CO teams
                for position in [
                    FourTeamPosition::OpeningGovernment,
                    FourTeamPosition::OpeningOpposition,
                    FourTeamPosition::ClosingGovernment,
                    FourTeamPosition::ClosingOpposition,
                ] {
                    let team = MatchTeam {
                        id: Uuid::new_v4(),
                        match_id,
                        two_team_position: None,
                        four_team_position: Some(position),
                        team_name: None,
                        institution: None,
                        final_rank: None,
                        total_speaker_points: None,
                        created_at: now,
                        updated_at: now,
                    };
                    let created = self.create_team(&team).await?;
                    teams.push(created);
                }
            }
        }

        Ok(teams)
    }

    // ========================================================================
    // Allocation Methods
    // ========================================================================

    pub async fn create_allocation(&self, allocation: &Allocation) -> Result<Allocation, sqlx::Error> {
        sqlx::query_as::<_, Allocation>(
            r#"
            INSERT INTO allocations (id, match_id, user_id, guest_name, role, team_id, 
                two_team_speaker_role, four_team_speaker_role, is_chair, 
                allocated_at, allocated_by, was_checked_in, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(allocation.id)
        .bind(allocation.match_id)
        .bind(allocation.user_id)
        .bind(&allocation.guest_name)
        .bind(allocation.role)
        .bind(allocation.team_id)
        .bind(allocation.two_team_speaker_role)
        .bind(allocation.four_team_speaker_role)
        .bind(allocation.is_chair)
        .bind(allocation.allocated_at)
        .bind(allocation.allocated_by)
        .bind(allocation.was_checked_in)
        .bind(allocation.created_at)
        .bind(allocation.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_allocation_by_id(&self, allocation_id: Uuid) -> Result<Option<Allocation>, sqlx::Error> {
        sqlx::query_as::<_, Allocation>(
            "SELECT * FROM allocations WHERE id = $1"
        )
        .bind(allocation_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_allocation_by_user_match(
        &self,
        match_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Allocation>, sqlx::Error> {
        sqlx::query_as::<_, Allocation>(
            "SELECT * FROM allocations WHERE match_id = $1 AND user_id = $2 LIMIT 1"
        )
        .bind(match_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// Get the adjudicator allocation for a user in a specific match
    pub async fn get_adjudicator_allocation_by_user_match(
        &self,
        match_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Allocation>, sqlx::Error> {
        sqlx::query_as::<_, Allocation>(
            "SELECT * FROM allocations WHERE match_id = $1 AND user_id = $2 AND role IN ('voting_adjudicator', 'non_voting_adjudicator') LIMIT 1"
        )
        .bind(match_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_allocations_by_match(&self, match_id: Uuid) -> Result<Vec<AllocationWithUser>, sqlx::Error> {
        sqlx::query_as::<_, AllocationWithUser>(
            r#"
            SELECT a.id, a.match_id, a.user_id, a.guest_name,
                COALESCE(u.username, a.guest_name, 'Unknown') as username, 
                a.role, a.team_id,
                a.two_team_speaker_role, a.four_team_speaker_role, a.is_chair,
                a.allocated_at, a.allocated_by, a.was_checked_in
            FROM allocations a
            LEFT JOIN users u ON a.user_id = u.id
            WHERE a.match_id = $1
            ORDER BY a.role, a.team_id, a.created_at
            "#,
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_allocations_by_team(&self, team_id: Uuid) -> Result<Vec<AllocationWithUser>, sqlx::Error> {
        sqlx::query_as::<_, AllocationWithUser>(
            r#"
            SELECT a.id, a.match_id, a.user_id, a.guest_name,
                COALESCE(u.username, a.guest_name, 'Unknown') as username, 
                a.role, a.team_id,
                a.two_team_speaker_role, a.four_team_speaker_role, a.is_chair,
                a.allocated_at, a.allocated_by, a.was_checked_in
            FROM allocations a
            LEFT JOIN users u ON a.user_id = u.id
            WHERE a.team_id = $1
            ORDER BY a.two_team_speaker_role, a.four_team_speaker_role
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_allocation(
        &self,
        allocation_id: Uuid,
        role: Option<AllocationRole>,
        team_id: Option<Uuid>,
        two_team_speaker_role: Option<TwoTeamSpeakerRole>,
        four_team_speaker_role: Option<FourTeamSpeakerRole>,
        is_chair: Option<bool>,
        allocated_by: Uuid,
    ) -> Result<Allocation, sqlx::Error> {
        sqlx::query_as::<_, Allocation>(
            r#"
            UPDATE allocations SET
                role = COALESCE($2, role),
                team_id = COALESCE($3, team_id),
                two_team_speaker_role = COALESCE($4, two_team_speaker_role),
                four_team_speaker_role = COALESCE($5, four_team_speaker_role),
                is_chair = COALESCE($6, is_chair),
                allocated_at = $7,
                allocated_by = $8,
                updated_at = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(allocation_id)
        .bind(role)
        .bind(team_id)
        .bind(two_team_speaker_role)
        .bind(four_team_speaker_role)
        .bind(is_chair)
        .bind(Utc::now())
        .bind(allocated_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_allocation(&self, allocation_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM allocations WHERE id = $1")
            .bind(allocation_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Check if user is allocated to any match in a series
    pub async fn get_user_allocation_in_series(
        &self,
        series_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Allocation>, sqlx::Error> {
        sqlx::query_as::<_, Allocation>(
            r#"
            SELECT a.* FROM allocations a
            JOIN matches m ON a.match_id = m.id
            WHERE m.series_id = $1 AND a.user_id = $2
            "#,
        )
        .bind(series_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }

    // ========================================================================
    // Allocation History Methods
    // ========================================================================

    pub async fn create_allocation_history(&self, history: &AllocationHistory) -> Result<AllocationHistory, sqlx::Error> {
        sqlx::query_as::<_, AllocationHistory>(
            r#"
            INSERT INTO allocation_history (id, allocation_id, match_id, user_id, guest_name, action,
                previous_role, new_role, previous_team_id, new_team_id, changed_by, changed_at, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(history.id)
        .bind(history.allocation_id)
        .bind(history.match_id)
        .bind(history.user_id)
        .bind(&history.guest_name)
        .bind(&history.action)
        .bind(history.previous_role)
        .bind(history.new_role)
        .bind(history.previous_team_id)
        .bind(history.new_team_id)
        .bind(history.changed_by)
        .bind(history.changed_at)
        .bind(&history.notes)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_allocation_history(
        &self,
        match_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<AllocationHistory>, i64), sqlx::Error> {
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM allocation_history WHERE match_id = $1"
        )
        .bind(match_id)
        .fetch_one(&self.pool)
        .await?;

        let history = sqlx::query_as::<_, AllocationHistory>(
            r#"
            SELECT * FROM allocation_history 
            WHERE match_id = $1 
            ORDER BY changed_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(match_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((history, total.0))
    }

    // ========================================================================
    // Ballot Methods
    // ========================================================================

    pub async fn create_ballot(&self, ballot: &Ballot) -> Result<Ballot, sqlx::Error> {
        sqlx::query_as::<_, Ballot>(
            r#"
            INSERT INTO ballots (id, match_id, adjudicator_id, is_voting, is_submitted, 
                submitted_at, notes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(ballot.id)
        .bind(ballot.match_id)
        .bind(ballot.adjudicator_id)
        .bind(ballot.is_voting)
        .bind(ballot.is_submitted)
        .bind(ballot.submitted_at)
        .bind(&ballot.notes)
        .bind(ballot.created_at)
        .bind(ballot.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get_ballot_by_id(&self, ballot_id: Uuid) -> Result<Option<Ballot>, sqlx::Error> {
        sqlx::query_as::<_, Ballot>(
            "SELECT * FROM ballots WHERE id = $1"
        )
        .bind(ballot_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_ballot_by_adjudicator_match(
        &self,
        match_id: Uuid,
        adjudicator_id: Uuid,
    ) -> Result<Option<Ballot>, sqlx::Error> {
        sqlx::query_as::<_, Ballot>(
            "SELECT * FROM ballots WHERE match_id = $1 AND adjudicator_id = $2"
        )
        .bind(match_id)
        .bind(adjudicator_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_ballots_by_match(&self, match_id: Uuid) -> Result<Vec<Ballot>, sqlx::Error> {
        sqlx::query_as::<_, Ballot>(
            r#"
            SELECT * FROM ballots 
            WHERE match_id = $1 
            ORDER BY is_voting DESC, created_at ASC
            "#,
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn submit_ballot(
        &self,
        ballot_id: Uuid,
        notes: Option<&str>,
    ) -> Result<Ballot, sqlx::Error> {
        sqlx::query_as::<_, Ballot>(
            r#"
            UPDATE ballots SET
                is_submitted = true,
                submitted_at = $2,
                notes = COALESCE($3, notes),
                updated_at = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(ballot_id)
        .bind(Utc::now())
        .bind(notes)
        .fetch_one(&self.pool)
        .await
    }

    // ========================================================================
    // Speaker Score Methods
    // ========================================================================

    pub async fn create_speaker_score(&self, score: &SpeakerScore) -> Result<SpeakerScore, sqlx::Error> {
        sqlx::query_as::<_, SpeakerScore>(
            r#"
            INSERT INTO speaker_scores (id, ballot_id, allocation_id, score, feedback, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(score.id)
        .bind(score.ballot_id)
        .bind(score.allocation_id)
        .bind(score.score)
        .bind(&score.feedback)
        .bind(score.created_at)
        .bind(score.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_speaker_scores_by_ballot(&self, ballot_id: Uuid) -> Result<Vec<SpeakerScore>, sqlx::Error> {
        sqlx::query_as::<_, SpeakerScore>(
            "SELECT * FROM speaker_scores WHERE ballot_id = $1"
        )
        .bind(ballot_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete_speaker_scores_by_ballot(&self, ballot_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM speaker_scores WHERE ballot_id = $1")
            .bind(ballot_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Get average score for a speaker allocation from all submitted voting ballots
    pub async fn get_allocation_average_score(&self, allocation_id: Uuid) -> Result<Option<Decimal>, sqlx::Error> {
        let result: Option<(Option<Decimal>,)> = sqlx::query_as(
            r#"
            SELECT AVG(ss.score) as avg_score
            FROM speaker_scores ss
            JOIN ballots b ON ss.ballot_id = b.id
            WHERE ss.allocation_id = $1 
              AND b.is_submitted = true 
              AND b.is_voting = true
            "#,
        )
        .bind(allocation_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.and_then(|(avg,)| avg))
    }

    pub async fn get_average_speaker_score(&self, user_id: Uuid, event_id: Option<Uuid>) -> Result<Option<Decimal>, sqlx::Error> {
        let result: Option<(Option<Decimal>,)> = if let Some(event_id) = event_id {
            sqlx::query_as(
                r#"
                SELECT AVG(ss.score) as avg_score
                FROM speaker_scores ss
                JOIN ballots b ON ss.ballot_id = b.id
                JOIN allocations a ON ss.allocation_id = a.id
                JOIN matches m ON a.match_id = m.id
                JOIN match_series ms ON m.series_id = ms.id
                WHERE a.user_id = $1 AND ms.event_id = $2 AND b.is_submitted = true
                "#,
            )
            .bind(user_id)
            .bind(event_id)
            .fetch_optional(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT AVG(ss.score) as avg_score
                FROM speaker_scores ss
                JOIN ballots b ON ss.ballot_id = b.id
                JOIN allocations a ON ss.allocation_id = a.id
                WHERE a.user_id = $1 AND b.is_submitted = true
                "#,
            )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
        };

        Ok(result.and_then(|(avg,)| avg))
    }

    // ========================================================================
    // Team Ranking Methods
    // ========================================================================

    pub async fn create_team_ranking(&self, ranking: &TeamRanking) -> Result<TeamRanking, sqlx::Error> {
        sqlx::query_as::<_, TeamRanking>(
            r#"
            INSERT INTO team_rankings (id, ballot_id, team_id, rank, is_winner, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(ranking.id)
        .bind(ranking.ballot_id)
        .bind(ranking.team_id)
        .bind(ranking.rank)
        .bind(ranking.is_winner)
        .bind(ranking.created_at)
        .bind(ranking.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_team_rankings_by_ballot(&self, ballot_id: Uuid) -> Result<Vec<TeamRanking>, sqlx::Error> {
        sqlx::query_as::<_, TeamRanking>(
            "SELECT * FROM team_rankings WHERE ballot_id = $1 ORDER BY rank ASC"
        )
        .bind(ballot_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn delete_team_rankings_by_ballot(&self, ballot_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM team_rankings WHERE ballot_id = $1")
            .bind(ballot_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Get aggregated team rankings from all submitted voting ballots for a match
    /// Returns (team_id, average_rank) tuples sorted by average rank ascending
    pub async fn get_match_team_rankings(&self, match_id: Uuid) -> Result<Vec<(Uuid, f64)>, sqlx::Error> {
        let results: Vec<(Uuid, Option<f64>)> = sqlx::query_as(
            r#"
            SELECT tr.team_id, AVG(tr.rank::float8) as avg_rank
            FROM team_rankings tr
            JOIN ballots b ON tr.ballot_id = b.id
            WHERE b.match_id = $1 AND b.is_submitted = true AND b.is_voting = true
            GROUP BY tr.team_id
            ORDER BY avg_rank ASC
            "#,
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results.into_iter()
            .filter_map(|(team_id, avg)| avg.map(|a| (team_id, a)))
            .collect())
    }

    // ========================================================================
    // Attendance Integration (for allocation pool)
    // ========================================================================

    pub async fn get_checked_in_users_for_event(&self, event_id: Uuid) -> Result<Vec<AttendanceInfo>, sqlx::Error> {
        sqlx::query_as::<_, AttendanceInfo>(
            r#"
            SELECT id, event_id, user_id, is_checked_in, checked_in_at
            FROM attendance_records
            WHERE event_id = $1 AND is_checked_in = true
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Check if a specific user is checked in for an event
    pub async fn is_user_checked_in(&self, event_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result: Option<(bool,)> = sqlx::query_as(
            "SELECT is_checked_in FROM attendance_records WHERE event_id = $1 AND user_id = $2"
        )
        .bind(event_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(is_checked_in,)| is_checked_in).unwrap_or(false))
    }

    // ========================================================================
    // Performance/Statistics Methods
    // ========================================================================

    pub async fn get_user_round_counts(
        &self,
        user_id: Uuid,
        event_id: Option<Uuid>,
    ) -> Result<(i64, i64, i64), sqlx::Error> {
        // Returns (total_rounds, speaker_rounds, adjudicator_rounds)
        let result: (i64, i64, i64) = if let Some(event_id) = event_id {
            sqlx::query_as(
                r#"
                SELECT 
                    COUNT(DISTINCT a.match_id) as total,
                    COUNT(DISTINCT CASE WHEN a.role = 'speaker' THEN a.match_id END) as speaker,
                    COUNT(DISTINCT CASE WHEN a.role IN ('voting_adjudicator', 'non_voting_adjudicator') THEN a.match_id END) as adjudicator
                FROM allocations a
                JOIN matches m ON a.match_id = m.id
                JOIN match_series ms ON m.series_id = ms.id
                WHERE a.user_id = $1 AND ms.event_id = $2
                "#,
            )
            .bind(user_id)
            .bind(event_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT 
                    COUNT(DISTINCT match_id) as total,
                    COUNT(DISTINCT CASE WHEN role = 'speaker' THEN match_id END) as speaker,
                    COUNT(DISTINCT CASE WHEN role IN ('voting_adjudicator', 'non_voting_adjudicator') THEN match_id END) as adjudicator
                FROM allocations
                WHERE user_id = $1
                "#,
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(result)
    }

    pub async fn get_user_win_loss(
        &self,
        user_id: Uuid,
        event_id: Option<Uuid>,
    ) -> Result<(i64, i64), sqlx::Error> {
        // Returns (wins, losses)
        let result: (i64, i64) = if let Some(event_id) = event_id {
            sqlx::query_as(
                r#"
                SELECT 
                    COUNT(CASE WHEN tr.is_winner = true THEN 1 END) as wins,
                    COUNT(CASE WHEN tr.is_winner = false THEN 1 END) as losses
                FROM allocations a
                JOIN match_teams mt ON a.team_id = mt.id
                JOIN team_rankings tr ON mt.id = tr.team_id
                JOIN ballots b ON tr.ballot_id = b.id
                JOIN matches m ON a.match_id = m.id
                JOIN match_series ms ON m.series_id = ms.id
                WHERE a.user_id = $1 AND ms.event_id = $2 AND a.role = 'speaker' AND b.is_voting = true AND b.is_submitted = true
                "#,
            )
            .bind(user_id)
            .bind(event_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT 
                    COUNT(CASE WHEN tr.is_winner = true THEN 1 END) as wins,
                    COUNT(CASE WHEN tr.is_winner = false THEN 1 END) as losses
                FROM allocations a
                JOIN match_teams mt ON a.team_id = mt.id
                JOIN team_rankings tr ON mt.id = tr.team_id
                JOIN ballots b ON tr.ballot_id = b.id
                WHERE a.user_id = $1 AND a.role = 'speaker' AND b.is_voting = true AND b.is_submitted = true
                "#,
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(result)
    }

    pub async fn get_user_ranking_distribution(
        &self,
        user_id: Uuid,
        event_id: Option<Uuid>,
    ) -> Result<Vec<(i32, i64)>, sqlx::Error> {
        // Returns vec of (rank, count) for BP format
        let results: Vec<(i32, i64)> = if let Some(event_id) = event_id {
            sqlx::query_as(
                r#"
                SELECT tr.rank, COUNT(*) as count
                FROM allocations a
                JOIN match_teams mt ON a.team_id = mt.id
                JOIN team_rankings tr ON mt.id = tr.team_id
                JOIN ballots b ON tr.ballot_id = b.id
                JOIN matches m ON a.match_id = m.id
                JOIN match_series ms ON m.series_id = ms.id
                WHERE a.user_id = $1 AND ms.event_id = $2 AND a.role = 'speaker' AND b.is_voting = true AND b.is_submitted = true
                GROUP BY tr.rank
                ORDER BY tr.rank
                "#,
            )
            .bind(user_id)
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT tr.rank, COUNT(*) as count
                FROM allocations a
                JOIN match_teams mt ON a.team_id = mt.id
                JOIN team_rankings tr ON mt.id = tr.team_id
                JOIN ballots b ON tr.ballot_id = b.id
                WHERE a.user_id = $1 AND a.role = 'speaker' AND b.is_voting = true AND b.is_submitted = true
                GROUP BY tr.rank
                ORDER BY tr.rank
                "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(results)
    }
}
