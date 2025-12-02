-- Migration: Allow same user to have multiple allocations in the same match
-- This is needed for friendly matches where one person can fill multiple roles
-- (e.g., give multiple speeches as PM and Reply)

-- Drop the unique index that prevents multiple allocations per user per match
DROP INDEX IF EXISTS idx_unique_user_allocation;

-- Also drop any constraint that might have been created
ALTER TABLE allocations DROP CONSTRAINT IF EXISTS unique_user_match;

-- Drop the unique_user_match_role constraint that prevents same user from having
-- the same role (e.g., "speaker") multiple times - we want to allow this for
-- speakers who give multiple speeches (e.g., PM + Reply)
ALTER TABLE allocations DROP CONSTRAINT IF EXISTS unique_user_match_role;
