-- Remove admin_users table
DROP INDEX IF EXISTS idx_admin_users_user_id;
DROP TABLE IF EXISTS admin_users;
