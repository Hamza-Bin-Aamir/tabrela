-- Events table - stores event information (tournaments, matches, meetings, etc.)
CREATE TABLE IF NOT EXISTS events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    event_type VARCHAR(50) NOT NULL,  -- 'tournament', 'weekly_match', 'meeting', 'other'
    event_date TIMESTAMPTZ NOT NULL,
    location VARCHAR(255),
    created_by UUID NOT NULL,  -- References auth service users table
    is_locked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Attendance records - tracks user availability and check-in status
-- By default, users are unavailable (no record = unavailable)
CREATE TABLE IF NOT EXISTS attendance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,  -- References auth service users table
    is_available BOOLEAN NOT NULL DEFAULT FALSE,
    is_checked_in BOOLEAN NOT NULL DEFAULT FALSE,
    checked_in_by UUID,  -- Admin who confirmed the check-in
    checked_in_at TIMESTAMPTZ,
    availability_set_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Each user can only have one attendance record per event
    CONSTRAINT unique_user_event UNIQUE (event_id, user_id)
);

-- Indexes for faster queries
CREATE INDEX IF NOT EXISTS idx_events_event_date ON events(event_date);
CREATE INDEX IF NOT EXISTS idx_events_created_by ON events(created_by);
CREATE INDEX IF NOT EXISTS idx_events_event_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_attendance_event_id ON attendance_records(event_id);
CREATE INDEX IF NOT EXISTS idx_attendance_user_id ON attendance_records(user_id);
CREATE INDEX IF NOT EXISTS idx_attendance_is_available ON attendance_records(is_available);
CREATE INDEX IF NOT EXISTS idx_attendance_is_checked_in ON attendance_records(is_checked_in);

-- Comments
COMMENT ON TABLE events IS 'Stores events like tournaments, weekly matches, meetings etc.';
COMMENT ON TABLE attendance_records IS 'Tracks user availability and check-in status for events. No record = unavailable.';
COMMENT ON COLUMN events.is_locked IS 'When true, prevents any attendance changes unless unlocked by admin.';
COMMENT ON COLUMN attendance_records.checked_in_by IS 'Admin user who confirmed the physical attendance.';
