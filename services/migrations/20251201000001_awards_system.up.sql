-- Create awards table for user achievements
-- Awards are public and have three tiers: bronze, silver, gold

CREATE TYPE award_tier AS ENUM ('bronze', 'silver', 'gold');

CREATE TABLE IF NOT EXISTS awards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    tier award_tier NOT NULL DEFAULT 'bronze',
    awarded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    awarded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create award history table to track tier upgrades
CREATE TABLE IF NOT EXISTS award_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    award_id UUID NOT NULL REFERENCES awards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    admin_id UUID REFERENCES users(id) ON DELETE SET NULL,
    previous_tier award_tier,
    new_tier award_tier NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_awards_user_id ON awards(user_id);
CREATE INDEX IF NOT EXISTS idx_awards_tier ON awards(tier);
CREATE INDEX IF NOT EXISTS idx_award_history_award_id ON award_history(award_id);
CREATE INDEX IF NOT EXISTS idx_award_history_user_id ON award_history(user_id);

-- Add comments
COMMENT ON TABLE awards IS 'Public achievements/awards for users with bronze/silver/gold tiers';
COMMENT ON TABLE award_history IS 'Audit log of award creation and tier upgrades';
COMMENT ON COLUMN awards.tier IS 'Award tier: bronze (lowest), silver, gold (highest)';
COMMENT ON COLUMN award_history.previous_tier IS 'NULL if this is the initial award creation';
