-- Drop indexes
DROP INDEX IF EXISTS idx_award_history_user_id;
DROP INDEX IF EXISTS idx_award_history_award_id;
DROP INDEX IF EXISTS idx_awards_tier;
DROP INDEX IF EXISTS idx_awards_user_id;

-- Drop tables
DROP TABLE IF EXISTS award_history;
DROP TABLE IF EXISTS awards;

-- Drop enum type
DROP TYPE IF EXISTS award_tier;
