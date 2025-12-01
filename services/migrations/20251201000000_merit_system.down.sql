-- Drop trigger first
DROP TRIGGER IF EXISTS trigger_initialize_merit_on_verification ON users;

-- Drop function
DROP FUNCTION IF EXISTS initialize_user_merit();

-- Drop indexes
DROP INDEX IF EXISTS idx_merit_history_created_at;
DROP INDEX IF EXISTS idx_merit_history_admin_id;
DROP INDEX IF EXISTS idx_merit_history_user_id;
DROP INDEX IF EXISTS idx_user_merit_user_id;

-- Drop tables
DROP TABLE IF EXISTS merit_history;
DROP TABLE IF EXISTS user_merit;
