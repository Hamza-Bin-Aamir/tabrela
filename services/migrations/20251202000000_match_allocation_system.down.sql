-- ============================================================================
-- Rollback Match & Allocation System Migration
-- ============================================================================

-- Drop indexes
DROP INDEX IF EXISTS idx_allocation_history_changed_at;
DROP INDEX IF EXISTS idx_allocation_history_user_id;
DROP INDEX IF EXISTS idx_allocation_history_match_id;
DROP INDEX IF EXISTS idx_team_rankings_team_id;
DROP INDEX IF EXISTS idx_team_rankings_ballot_id;
DROP INDEX IF EXISTS idx_speaker_scores_allocation_id;
DROP INDEX IF EXISTS idx_speaker_scores_ballot_id;
DROP INDEX IF EXISTS idx_ballots_submitted;
DROP INDEX IF EXISTS idx_ballots_adjudicator_id;
DROP INDEX IF EXISTS idx_ballots_match_id;
DROP INDEX IF EXISTS idx_allocations_role;
DROP INDEX IF EXISTS idx_allocations_team_id;
DROP INDEX IF EXISTS idx_allocations_user_id;
DROP INDEX IF EXISTS idx_allocations_match_id;
DROP INDEX IF EXISTS idx_match_teams_match_id;
DROP INDEX IF EXISTS idx_matches_scheduled_time;
DROP INDEX IF EXISTS idx_matches_status;
DROP INDEX IF EXISTS idx_matches_series_id;
DROP INDEX IF EXISTS idx_match_series_round;
DROP INDEX IF EXISTS idx_match_series_event_id;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS allocation_history;
DROP TABLE IF EXISTS team_rankings;
DROP TABLE IF EXISTS speaker_scores;
DROP TABLE IF EXISTS ballots;
DROP TABLE IF EXISTS allocations;
DROP TABLE IF EXISTS match_teams;
DROP TABLE IF EXISTS matches;
DROP TABLE IF EXISTS match_series;

-- Drop types
DROP TYPE IF EXISTS match_status;
DROP TYPE IF EXISTS allocation_role;
DROP TYPE IF EXISTS four_team_speaker_role;
DROP TYPE IF EXISTS two_team_speaker_role;
DROP TYPE IF EXISTS four_team_position;
DROP TYPE IF EXISTS two_team_position;
DROP TYPE IF EXISTS team_format;
