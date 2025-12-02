-- Rollback: Remove guest allocation support

-- Remove the guest-specific constraints
ALTER TABLE allocations 
    DROP CONSTRAINT IF EXISTS valid_participant;

-- Remove the unique index
DROP INDEX IF EXISTS idx_unique_user_allocation;

-- Remove guest_name column
ALTER TABLE allocations 
    DROP COLUMN IF EXISTS guest_name;

-- Make user_id required again
ALTER TABLE allocations 
    ALTER COLUMN user_id SET NOT NULL;

-- Restore original unique constraint
ALTER TABLE allocations 
    ADD CONSTRAINT unique_user_match UNIQUE (match_id, user_id);
