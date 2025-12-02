-- Migration: Add support for guest allocations (non-user participants)
-- These are participants without user accounts, only identified by name

-- Add guest_name column and make user_id nullable
ALTER TABLE allocations 
    ALTER COLUMN user_id DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS guest_name VARCHAR(255);

-- Update constraint: either user_id or guest_name must be provided
ALTER TABLE allocations 
    DROP CONSTRAINT IF EXISTS unique_user_match;

-- Add constraint: guest must have a name
ALTER TABLE allocations 
    ADD CONSTRAINT valid_participant CHECK (
        (user_id IS NOT NULL) OR (guest_name IS NOT NULL AND guest_name != '')
    );

-- NOTE: No unique constraint on (match_id, user_id) for friendly matches
-- The same user can be allocated to multiple roles in the same match
-- (e.g., speaker and resource, or multiple speaker positions)

-- Update speaker_scores to allow scores for guest allocations
-- No changes needed as it references allocation_id, not user_id directly

-- Add was_checked_in column if it doesn't exist (may have been added in previous migration)
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'allocations' AND column_name = 'was_checked_in'
    ) THEN
        ALTER TABLE allocations ADD COLUMN was_checked_in BOOLEAN DEFAULT false;
    END IF;
END $$;

-- Update allocation_history table to support guest allocations
ALTER TABLE allocation_history 
    ALTER COLUMN user_id DROP NOT NULL;

ALTER TABLE allocation_history 
    ADD COLUMN IF NOT EXISTS guest_name VARCHAR(255);
