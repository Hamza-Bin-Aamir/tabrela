-- Rollback: This is intentionally a no-op
-- We cannot re-add the unique constraint because the data model now allows
-- multiple allocations per user per match (for friendly matches)
-- If you need to restore the constraint, first delete duplicate allocations manually
SELECT 1;
