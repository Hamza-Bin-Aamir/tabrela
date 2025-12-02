-- ============================================================================
-- Friendly Match Updates Migration
-- ============================================================================
-- This migration updates the match system for friendly/internal matches:
-- 1. Removes sequential round constraints (no unique round_number per event)
-- 2. Allows users to be allocated to multiple roles in a match
-- 3. Adds check-in status tracking for allocations
-- ============================================================================

-- Remove the unique constraint on round_number per event
-- (friendly matches don't have sequential rounds)
ALTER TABLE match_series DROP CONSTRAINT IF EXISTS unique_event_round;

-- Make round_number optional (can be null for informal matches)
ALTER TABLE match_series ALTER COLUMN round_number DROP NOT NULL;
ALTER TABLE match_series ALTER COLUMN round_number DROP DEFAULT;

-- Remove the unique constraint that prevents users from having multiple roles
-- (in friendly matches, someone might speak AND judge different matches)
ALTER TABLE allocations DROP CONSTRAINT IF EXISTS unique_user_match;

-- Add a unique constraint per user-match-role instead
-- (same user can't have the same role twice in the same match)
ALTER TABLE allocations ADD CONSTRAINT unique_user_match_role 
    UNIQUE (match_id, user_id, role);

-- Add check-in status to allocations for display purposes
-- (we still allow allocation of non-checked-in users, but track the status)
ALTER TABLE allocations ADD COLUMN IF NOT EXISTS was_checked_in BOOLEAN DEFAULT false;

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_allocations_checked_in ON allocations(match_id, was_checked_in);
