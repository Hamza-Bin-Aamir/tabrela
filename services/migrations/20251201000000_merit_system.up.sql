-- Create user_merit table to track merit points for each user
-- Merit points start at 0 for all users and can be positive or negative

CREATE TABLE IF NOT EXISTS user_merit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    merit_points INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create merit_history table to track all merit changes with audit trail
CREATE TABLE IF NOT EXISTS merit_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    admin_id UUID REFERENCES users(id) ON DELETE SET NULL,  -- NULL allowed for system changes
    change_amount INTEGER NOT NULL,  -- Can be positive (add) or negative (remove)
    previous_total INTEGER NOT NULL,
    new_total INTEGER NOT NULL,
    reason TEXT NOT NULL,  -- Admin must provide a reason for the change
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_user_merit_user_id ON user_merit(user_id);
CREATE INDEX IF NOT EXISTS idx_merit_history_user_id ON merit_history(user_id);
CREATE INDEX IF NOT EXISTS idx_merit_history_admin_id ON merit_history(admin_id);
CREATE INDEX IF NOT EXISTS idx_merit_history_created_at ON merit_history(created_at DESC);

-- Add comments explaining the tables
COMMENT ON TABLE user_merit IS 'Tracks current merit points for each user. Points are private except to admins.';
COMMENT ON TABLE merit_history IS 'Audit log of all merit point changes made by admins.';
COMMENT ON COLUMN merit_history.change_amount IS 'Positive for adding merit, negative for removing merit.';
COMMENT ON COLUMN merit_history.reason IS 'Required explanation for why merit was changed.';

-- Create a function to automatically initialize merit for new users
CREATE OR REPLACE FUNCTION initialize_user_merit()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO user_merit (user_id, merit_points)
    VALUES (NEW.id, 0)
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to auto-initialize merit when a user verifies their email
CREATE OR REPLACE TRIGGER trigger_initialize_merit_on_verification
    AFTER UPDATE OF email_verified ON users
    FOR EACH ROW
    WHEN (NEW.email_verified = true AND OLD.email_verified = false)
    EXECUTE FUNCTION initialize_user_merit();

-- Initialize merit for existing verified users
INSERT INTO user_merit (user_id, merit_points)
SELECT id, 0 FROM users WHERE email_verified = true
ON CONFLICT (user_id) DO NOTHING;
