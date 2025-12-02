-- ============================================================================
-- Friendly Match Updates Migration - Rollback
-- ============================================================================

-- Remove check-in status column
ALTER TABLE allocations DROP COLUMN IF EXISTS was_checked_in;

-- Drop the new index
DROP INDEX IF EXISTS idx_allocations_checked_in;

-- Remove the new constraint
ALTER TABLE allocations DROP CONSTRAINT IF EXISTS unique_user_match_role;

-- Restore the original unique constraint
ALTER TABLE allocations ADD CONSTRAINT unique_user_match UNIQUE (match_id, user_id);

-- Restore round_number default and not null
ALTER TABLE match_series ALTER COLUMN round_number SET DEFAULT 1;
ALTER TABLE match_series ALTER COLUMN round_number SET NOT NULL;

-- Restore the unique constraint on round_number
ALTER TABLE match_series ADD CONSTRAINT unique_event_round UNIQUE (event_id, round_number);
