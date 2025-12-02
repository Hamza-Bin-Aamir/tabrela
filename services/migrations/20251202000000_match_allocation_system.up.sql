-- ============================================================================
-- Match & Allocation System Migration
-- ============================================================================
-- This migration creates tables for:
-- 1. Match Series (groups of matches in a round)
-- 2. Matches (individual debate rooms)
-- 3. Teams (team assignments within matches)
-- 4. Allocations (participant roles within matches)
-- 5. Ballots (adjudicator submissions)
-- 6. Speaker Scores (individual speaker scoring)
-- ============================================================================

-- Enum for team formats
CREATE TYPE team_format AS ENUM ('two_team', 'four_team');

-- Enum for team positions in 2-team format
CREATE TYPE two_team_position AS ENUM ('government', 'opposition');

-- Enum for team positions in 4-team (BP) format
CREATE TYPE four_team_position AS ENUM ('opening_government', 'opening_opposition', 'closing_government', 'closing_opposition');

-- Enum for speaker roles in 2-team format
CREATE TYPE two_team_speaker_role AS ENUM (
    'prime_minister',
    'deputy_prime_minister', 
    'government_whip',
    'leader_of_opposition',
    'deputy_leader_of_opposition',
    'opposition_whip',
    'government_reply',
    'opposition_reply'
);

-- Enum for speaker roles in 4-team (BP) format
CREATE TYPE four_team_speaker_role AS ENUM (
    'prime_minister',           -- OG Speaker 1
    'deputy_prime_minister',    -- OG Speaker 2
    'leader_of_opposition',     -- OO Speaker 1
    'deputy_leader_of_opposition', -- OO Speaker 2
    'member_of_government',     -- CG Speaker 1
    'government_whip',          -- CG Speaker 2
    'member_of_opposition',     -- CO Speaker 1
    'opposition_whip'           -- CO Speaker 2
);

-- Enum for allocation roles
CREATE TYPE allocation_role AS ENUM (
    'speaker',
    'resource',
    'voting_adjudicator',
    'non_voting_adjudicator'
);

-- Enum for match status
CREATE TYPE match_status AS ENUM (
    'draft',
    'published',
    'in_progress',
    'completed',
    'cancelled'
);

-- ============================================================================
-- Match Series Table
-- ============================================================================
-- A series represents a round of debates (e.g., "Round 1", "Quarter Finals")
CREATE TABLE IF NOT EXISTS match_series (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    round_number INTEGER NOT NULL DEFAULT 1,
    team_format team_format NOT NULL DEFAULT 'two_team',
    allow_reply_speeches BOOLEAN NOT NULL DEFAULT false,
    is_break_round BOOLEAN NOT NULL DEFAULT false,  -- For elimination rounds
    created_by UUID NOT NULL,  -- References users table
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_event_round UNIQUE (event_id, round_number)
);

-- ============================================================================
-- Matches Table
-- ============================================================================
-- Individual debate matches/rooms within a series
CREATE TABLE IF NOT EXISTS matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    series_id UUID NOT NULL REFERENCES match_series(id) ON DELETE CASCADE,
    room_name VARCHAR(255),  -- e.g., "Room 1", "Venue A"
    motion TEXT,  -- The debate topic/motion
    info_slide TEXT,  -- Additional context for the motion
    status match_status NOT NULL DEFAULT 'draft',
    scheduled_time TIMESTAMPTZ,
    
    -- Release controls (FR-15 to FR-18)
    scores_released BOOLEAN NOT NULL DEFAULT false,
    rankings_released BOOLEAN NOT NULL DEFAULT false,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- Match Teams Table
-- ============================================================================
-- Teams within a match
CREATE TABLE IF NOT EXISTS match_teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    
    -- For 2-team format
    two_team_position two_team_position,
    
    -- For 4-team format
    four_team_position four_team_position,
    
    -- Team identification
    team_name VARCHAR(255),  -- Optional custom name
    institution VARCHAR(255),  -- School/university name
    
    -- Results (populated after balloting)
    final_rank INTEGER,  -- 1-4 for BP, or 1-2 for 2-team
    total_speaker_points DECIMAL(8,2),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT valid_position CHECK (
        (two_team_position IS NOT NULL AND four_team_position IS NULL) OR
        (two_team_position IS NULL AND four_team_position IS NOT NULL)
    )
);

-- ============================================================================
-- Allocations Table
-- ============================================================================
-- Assigns participants to matches with specific roles
CREATE TABLE IF NOT EXISTS allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,  -- References users table
    role allocation_role NOT NULL,
    
    -- For speakers: which team and position
    team_id UUID REFERENCES match_teams(id) ON DELETE SET NULL,
    
    -- Speaker role (only for speakers)
    two_team_speaker_role two_team_speaker_role,
    four_team_speaker_role four_team_speaker_role,
    
    -- For adjudicators: chair status
    is_chair BOOLEAN DEFAULT false,  -- Chair of the judging panel
    
    -- Timestamps for tracking changes (FR-09)
    allocated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    allocated_by UUID NOT NULL,  -- Admin who made the allocation
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Each user can only have one allocation per match
    CONSTRAINT unique_user_match UNIQUE (match_id, user_id),
    
    -- Speaker role validation
    CONSTRAINT valid_speaker_role CHECK (
        role != 'speaker' OR 
        (team_id IS NOT NULL AND (two_team_speaker_role IS NOT NULL OR four_team_speaker_role IS NOT NULL))
    ),
    
    -- Adjudicator validation
    CONSTRAINT valid_adjudicator CHECK (
        role NOT IN ('voting_adjudicator', 'non_voting_adjudicator') OR
        team_id IS NULL
    )
);

-- ============================================================================
-- Ballots Table
-- ============================================================================
-- Adjudicator ballot submissions (FR-10, FR-11)
CREATE TABLE IF NOT EXISTS ballots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    adjudicator_id UUID NOT NULL,  -- References users table
    
    -- Ballot type based on adjudicator role
    is_voting BOOLEAN NOT NULL,  -- true = voting adjudicator, false = trainee
    
    -- Status
    is_submitted BOOLEAN NOT NULL DEFAULT false,
    submitted_at TIMESTAMPTZ,
    
    -- General feedback/notes (FR-11, FR-12)
    notes TEXT,  -- Qualitative feedback
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Each adjudicator can only have one ballot per match
    CONSTRAINT unique_adjudicator_match UNIQUE (match_id, adjudicator_id)
);

-- ============================================================================
-- Speaker Scores Table
-- ============================================================================
-- Individual speaker scores within a ballot (FR-12)
CREATE TABLE IF NOT EXISTS speaker_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ballot_id UUID NOT NULL REFERENCES ballots(id) ON DELETE CASCADE,
    allocation_id UUID NOT NULL REFERENCES allocations(id) ON DELETE CASCADE,
    
    -- Score (typically 50-100 range in debate)
    score DECIMAL(5,2) NOT NULL,
    
    -- Individual speaker feedback
    feedback TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Each speaker gets one score per ballot
    CONSTRAINT unique_speaker_ballot UNIQUE (ballot_id, allocation_id),
    
    -- Score range validation
    CONSTRAINT valid_score CHECK (score >= 0 AND score <= 100)
);

-- ============================================================================
-- Team Rankings Table
-- ============================================================================
-- Team rankings within a ballot (FR-12, FR-13)
CREATE TABLE IF NOT EXISTS team_rankings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ballot_id UUID NOT NULL REFERENCES ballots(id) ON DELETE CASCADE,
    team_id UUID NOT NULL REFERENCES match_teams(id) ON DELETE CASCADE,
    
    -- Rank: 1-4 for BP format, 1-2 for two-team format
    rank INTEGER NOT NULL,
    
    -- Win/Loss for 2-team format
    is_winner BOOLEAN,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Each team gets one ranking per ballot
    CONSTRAINT unique_team_ballot UNIQUE (ballot_id, team_id),
    
    -- Rank validation (1-4)
    CONSTRAINT valid_rank CHECK (rank >= 1 AND rank <= 4)
);

-- ============================================================================
-- Allocation History Table
-- ============================================================================
-- Tracks changes to allocations for audit and real-time updates (FR-09)
CREATE TABLE IF NOT EXISTS allocation_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    allocation_id UUID REFERENCES allocations(id) ON DELETE SET NULL,
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    
    -- What changed
    action VARCHAR(50) NOT NULL,  -- 'created', 'updated', 'deleted'
    previous_role allocation_role,
    new_role allocation_role,
    previous_team_id UUID,
    new_team_id UUID,
    
    -- Who made the change
    changed_by UUID NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Additional context
    notes TEXT
);

-- ============================================================================
-- Indexes
-- ============================================================================
CREATE INDEX IF NOT EXISTS idx_match_series_event_id ON match_series(event_id);
CREATE INDEX IF NOT EXISTS idx_match_series_round ON match_series(event_id, round_number);

CREATE INDEX IF NOT EXISTS idx_matches_series_id ON matches(series_id);
CREATE INDEX IF NOT EXISTS idx_matches_status ON matches(status);
CREATE INDEX IF NOT EXISTS idx_matches_scheduled_time ON matches(scheduled_time);

CREATE INDEX IF NOT EXISTS idx_match_teams_match_id ON match_teams(match_id);

CREATE INDEX IF NOT EXISTS idx_allocations_match_id ON allocations(match_id);
CREATE INDEX IF NOT EXISTS idx_allocations_user_id ON allocations(user_id);
CREATE INDEX IF NOT EXISTS idx_allocations_team_id ON allocations(team_id);
CREATE INDEX IF NOT EXISTS idx_allocations_role ON allocations(role);

CREATE INDEX IF NOT EXISTS idx_ballots_match_id ON ballots(match_id);
CREATE INDEX IF NOT EXISTS idx_ballots_adjudicator_id ON ballots(adjudicator_id);
CREATE INDEX IF NOT EXISTS idx_ballots_submitted ON ballots(is_submitted);

CREATE INDEX IF NOT EXISTS idx_speaker_scores_ballot_id ON speaker_scores(ballot_id);
CREATE INDEX IF NOT EXISTS idx_speaker_scores_allocation_id ON speaker_scores(allocation_id);

CREATE INDEX IF NOT EXISTS idx_team_rankings_ballot_id ON team_rankings(ballot_id);
CREATE INDEX IF NOT EXISTS idx_team_rankings_team_id ON team_rankings(team_id);

CREATE INDEX IF NOT EXISTS idx_allocation_history_match_id ON allocation_history(match_id);
CREATE INDEX IF NOT EXISTS idx_allocation_history_user_id ON allocation_history(user_id);
CREATE INDEX IF NOT EXISTS idx_allocation_history_changed_at ON allocation_history(changed_at);

-- ============================================================================
-- Comments
-- ============================================================================
COMMENT ON TABLE match_series IS 'Groups of matches representing a round (e.g., Round 1, Quarter Finals)';
COMMENT ON TABLE matches IS 'Individual debate matches/rooms within a series';
COMMENT ON TABLE match_teams IS 'Teams within a match with their positions';
COMMENT ON TABLE allocations IS 'Participant assignments to matches (speakers, adjudicators, resources)';
COMMENT ON TABLE ballots IS 'Adjudicator ballot submissions for scoring';
COMMENT ON TABLE speaker_scores IS 'Individual speaker scores within a ballot';
COMMENT ON TABLE team_rankings IS 'Team rankings/results within a ballot';
COMMENT ON TABLE allocation_history IS 'Audit log of allocation changes';

COMMENT ON COLUMN matches.scores_released IS 'When true, speaker scores are visible to participants';
COMMENT ON COLUMN matches.rankings_released IS 'When true, team rankings are visible to participants';
COMMENT ON COLUMN allocations.is_chair IS 'Whether this adjudicator is the chair of the panel';
COMMENT ON COLUMN ballots.is_voting IS 'Voting adjudicators can submit scores; non-voting only submit notes';
