-- Create admin_users table to track which users have admin privileges
-- The first admin should be added manually by a DBA using:
-- INSERT INTO admin_users (id, user_id, granted_by, created_at) 
-- VALUES (gen_random_uuid(), '<user_id>', NULL, NOW());

CREATE TABLE IF NOT EXISTS admin_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    granted_by UUID REFERENCES users(id) ON DELETE SET NULL,  -- NULL for first admin (manually added by DBA)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on user_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_admin_users_user_id ON admin_users(user_id);

-- Add comment explaining the table purpose
COMMENT ON TABLE admin_users IS 'Tracks users with admin privileges. First admin must be added manually by DBA.';
COMMENT ON COLUMN admin_users.granted_by IS 'The admin user who granted this privilege. NULL if added by DBA directly.';
