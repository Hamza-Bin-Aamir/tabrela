-- Drop indexes first
DROP INDEX IF EXISTS idx_attendance_is_checked_in;
DROP INDEX IF EXISTS idx_attendance_is_available;
DROP INDEX IF EXISTS idx_attendance_user_id;
DROP INDEX IF EXISTS idx_attendance_event_id;
DROP INDEX IF EXISTS idx_events_event_type;
DROP INDEX IF EXISTS idx_events_created_by;
DROP INDEX IF EXISTS idx_events_event_date;

-- Drop tables
DROP TABLE IF EXISTS attendance_records;
DROP TABLE IF EXISTS events;
