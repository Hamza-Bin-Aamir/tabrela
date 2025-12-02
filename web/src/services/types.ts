// Authentication Types
export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
  reg_number: string;
  year_joined: number;
  phone_number: string;
}

export interface LoginRequest {
  username_or_email: string;
  password: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface UserResponse {
  id: string;
  username: string;
  email: string;
  reg_number: string;
  year_joined: number;
  phone_number: string;
  email_verified: boolean;
  created_at: string;
}

export interface RegisterResponse {
  message: string;
  email: string;
}

export interface VerifyOtpRequest {
  email: string;
  otp: string;
}

export interface VerifyOtpResponse {
  user: UserResponse;
  auth: AuthResponse;
  csrf_token: string;
}

export interface ResendOtpRequest {
  email: string;
}

export interface LoginResponse {
  user: UserResponse;
  auth: AuthResponse;
  csrf_token: string;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface RequestPasswordResetRequest {
  email: string;
}

export interface ResetPasswordRequest {
  email: string;
  otp: string;
  new_password: string;
}

export interface ApiError {
  error: string;
  attempts_remaining?: number;
}

// Admin Types
export interface AdminUserListItem {
  id: string;
  username: string;
  email: string;
  reg_number: string;
  year_joined: number;
  phone_number: string;
  email_verified: boolean;
  is_admin: boolean;
  created_at: string;
}

export interface AdminListUsersResponse {
  users: AdminUserListItem[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface PromoteToAdminRequest {
  user_id: string;
}

export interface AdminCheckResponse {
  is_admin: boolean;
}

// ============================================================================
// Attendance/Event Types
// ============================================================================

export type EventType = 'tournament' | 'weekly_match' | 'meeting' | 'other';

export interface Event {
  id: string;
  title: string;
  description: string | null;
  event_type: EventType;
  event_date: string;
  location: string | null;
  created_by: string;
  is_locked: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateEventRequest {
  title: string;
  description?: string;
  event_type: EventType;
  event_date: string;
  location?: string;
}

export interface UpdateEventRequest {
  title?: string;
  description?: string;
  event_type?: EventType;
  event_date?: string;
  location?: string;
}

export interface EventListResponse {
  events: Event[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface AttendanceRecord {
  id: string;
  event_id: string;
  user_id: string;
  username: string;
  is_available: boolean;
  is_checked_in: boolean;
  checked_in_by: string | null;
  checked_in_at: string | null;
  availability_set_at: string;
}

export interface AttendanceStats {
  total_available: number;
  total_checked_in: number;
  total_unavailable: number;
}

export interface EventAttendanceResponse {
  event: Event;
  attendance: AttendanceRecord[];
  stats: AttendanceStats;
}

export interface SetAvailabilityRequest {
  is_available: boolean;
}

export interface CheckInRequest {
  user_id: string;
  is_checked_in: boolean;
}

export interface RevokeAvailabilityRequest {
  user_id: string;
}

export interface LockEventRequest {
  is_locked: boolean;
}

// ============================================================================
// Attendance Matrix/Dashboard Types
// ============================================================================

export interface EventSummary {
  id: string;
  title: string;
  event_type: string;
  event_date: string;
  is_locked: boolean;
  total_available: number;
  total_checked_in: number;
}

export interface UserAttendanceSummary {
  user_id: string;
  username: string;
  events_available: number;
  events_checked_in: number;
  total_events: number;
  availability_rate: number;
  attendance_rate: number;
}

export type AttendanceCellStatus = 'no_response' | 'available' | 'checked_in' | 'unavailable';

export interface AttendanceMatrixRow {
  user: UserAttendanceSummary;
  cells: AttendanceCellStatus[];
}

export interface EventTypeStats {
  event_type: string;
  count: number;
  avg_attendance: number;
}

export interface AggregateStats {
  total_events: number;
  total_users: number;
  overall_availability_rate: number;
  overall_attendance_rate: number;
  avg_available_per_event: number;
  avg_checked_in_per_event: number;
  most_attended_event: EventSummary | null;
  least_attended_event: EventSummary | null;
  most_reliable_users: UserAttendanceSummary[];
  events_by_type: EventTypeStats[];
}

export interface AttendanceMatrixResponse {
  events: EventSummary[];
  rows: AttendanceMatrixRow[];
  aggregate_stats: AggregateStats;
}

// ============================================================================
// Merit Types
// ============================================================================

// Public profile - visible to everyone
export interface PublicProfileResponse {
  id: string;
  username: string;
  year_joined: number;
  created_at: string;
}

// Private profile - visible to self (includes merit)
export interface PrivateProfileResponse {
  id: string;
  username: string;
  email: string;
  reg_number: string;
  year_joined: number;
  phone_number: string;
  email_verified: boolean;
  merit_points: number;
  created_at: string;
}

// Admin profile - visible to admins (full details)
export interface AdminProfileResponse {
  id: string;
  username: string;
  email: string;
  reg_number: string;
  year_joined: number;
  phone_number: string;
  email_verified: boolean;
  merit_points: number;
  is_admin: boolean;
  created_at: string;
}

// Union type for profile responses
export type ProfileResponse = PublicProfileResponse | PrivateProfileResponse | AdminProfileResponse;

// Merit response
export interface MeritResponse {
  user_id: string;
  merit_points: number;
  updated_at: string;
}

// Merit history entry
export interface MeritHistoryEntry {
  id: string;
  user_id: string;
  admin_id: string | null;
  admin_username: string | null;
  change_amount: number;
  previous_total: number;
  new_total: number;
  reason: string;
  created_at: string;
}

// Merit history response
export interface MeritHistoryResponse {
  history: MeritHistoryEntry[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

// Admin update merit request
export interface UpdateMeritRequest {
  user_id: string;
  change_amount: number;
  reason: string;
}

// Admin merit list item
export interface UserMeritInfo {
  user_id: string;
  username: string;
  merit_points: number;
}

// Admin merit list response
export interface AdminMeritListResponse {
  users: UserMeritInfo[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

// ============================================================================
// Award Types
// ============================================================================

// Award tier enum
export type AwardTier = 'bronze' | 'silver' | 'gold';

// Award response (public)
export interface AwardResponse {
  id: string;
  title: string;
  description: string | null;
  tier: AwardTier;
  awarded_at: string;
}

// Award list response
export interface AwardListResponse {
  awards: AwardResponse[];
  total: number;
}

// Award with admin details (for admin views)
export interface AwardWithAdmin {
  id: string;
  user_id: string;
  title: string;
  description: string | null;
  tier: AwardTier;
  awarded_by: string | null;
  awarded_by_username: string | null;
  username: string;  // recipient's username
  awarded_at: string;
  created_at: string;
  updated_at: string;
}

// Award history entry
export interface AwardHistoryEntry {
  id: string;
  award_id: string;
  user_id: string;
  admin_id: string | null;
  admin_username: string | null;
  previous_tier: AwardTier | null;
  new_tier: AwardTier;
  reason: string;
  created_at: string;
}

// Award history response
export interface AwardHistoryResponse {
  history: AwardHistoryEntry[];
  total: number;
}

// Create award request (admin)
export interface CreateAwardRequest {
  user_id: string;
  title: string;
  description?: string;
  tier: AwardTier;
  reason: string;
}

// Upgrade award request (admin)
export interface UpgradeAwardRequest {
  new_tier: AwardTier;
  new_title?: string;
  reason: string;
}

// Edit award request (admin)
export interface EditAwardRequest {
  title: string;
  description?: string;
  tier: AwardTier;
}

// Admin award list response
export interface AdminAwardListResponse {
  awards: AwardWithAdmin[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

// ============================================================================
// Tabulation/Match Types
// ============================================================================

// Enums
export type TeamFormat = 'two_team' | 'four_team';
export type TwoTeamPosition = 'government' | 'opposition';
export type FourTeamPosition = 'opening_government' | 'opening_opposition' | 'closing_government' | 'closing_opposition';
export type TwoTeamSpeakerRole = 
  | 'prime_minister' 
  | 'deputy_prime_minister' 
  | 'government_whip'
  | 'leader_of_opposition'
  | 'deputy_leader_of_opposition'
  | 'opposition_whip'
  | 'government_reply'
  | 'opposition_reply';
export type FourTeamSpeakerRole =
  | 'prime_minister'
  | 'deputy_prime_minister'
  | 'leader_of_opposition'
  | 'deputy_leader_of_opposition'
  | 'member_of_government'
  | 'government_whip'
  | 'member_of_opposition'
  | 'opposition_whip';
export type AllocationRole = 'speaker' | 'resource' | 'voting_adjudicator' | 'non_voting_adjudicator';
export type MatchStatus = 'draft' | 'published' | 'in_progress' | 'completed' | 'cancelled';

// Match Series
export interface MatchSeries {
  id: string;
  event_id: string;
  name: string;
  description: string | null;
  round_number: number | null;  // Optional for friendly matches
  team_format: TeamFormat;
  allow_reply_speeches: boolean;
  is_break_round: boolean;
  match_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateSeriesRequest {
  event_id: string;
  name: string;
  description?: string;
  round_number?: number;  // Optional for friendly matches
  team_format: TeamFormat;
  allow_reply_speeches?: boolean;
  is_break_round?: boolean;
}

export interface UpdateSeriesRequest {
  name?: string;
  description?: string;
  allow_reply_speeches?: boolean;
  is_break_round?: boolean;
}

export interface SeriesListResponse {
  series: MatchSeries[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

// Match
export interface SpeakerInfo {
  allocation_id: string;
  user_id: string | null;  // Null for guest allocations
  guest_name: string | null;  // Name for guest participants
  username: string;  // Will be guest_name for non-users
  two_team_speaker_role: TwoTeamSpeakerRole | null;
  four_team_speaker_role: FourTeamSpeakerRole | null;
  score: number | null;
  was_checked_in: boolean;
}

export interface ResourceInfo {
  allocation_id: string;
  user_id: string | null;  // Null for guest allocations
  guest_name: string | null;  // Name for guest participants
  username: string;
  was_checked_in: boolean;
}

export interface MatchTeam {
  id: string;
  two_team_position: TwoTeamPosition | null;
  four_team_position: FourTeamPosition | null;
  team_name: string | null;
  institution: string | null;
  final_rank: number | null;
  total_speaker_points: number | null;
  speakers: SpeakerInfo[];
  resources: ResourceInfo[];
}

export interface AdjudicatorInfo {
  allocation_id: string;
  user_id: string | null;  // Null for guest allocations
  guest_name: string | null;  // Name for guest participants
  username: string;
  is_voting: boolean;
  is_chair: boolean;
  has_submitted: boolean;
  was_checked_in: boolean;
}

export interface MatchResponse {
  id: string;
  series_id: string;
  series_name: string;
  room_name: string | null;
  motion: string | null;
  info_slide: string | null;
  status: MatchStatus;
  scheduled_time: string | null;
  scores_released: boolean;
  rankings_released: boolean;
  teams: MatchTeam[];
  adjudicators: AdjudicatorInfo[];
  created_at: string;
  updated_at: string;
}

export interface CreateMatchRequest {
  series_id: string;
  room_name?: string;
  motion?: string;
  info_slide?: string;
  scheduled_time?: string;
}

export interface UpdateMatchRequest {
  room_name?: string;
  motion?: string;
  info_slide?: string;
  status?: MatchStatus;
  scheduled_time?: string;
}

export interface ReleaseToggleRequest {
  scores_released?: boolean;
  rankings_released?: boolean;
}

export interface MatchListResponse {
  matches: MatchResponse[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

// Allocation
export interface CurrentAllocationInfo {
  match_id: string;
  room_name: string | null;
  role: AllocationRole;
}

export interface CheckedInUserInfo {
  user_id: string;
  username: string;
  checked_in_at: string | null; // null if not checked in but available for allocation
  is_allocated: boolean;
  current_allocation: CurrentAllocationInfo | null;
}

export interface AllocationPoolResponse {
  event_id: string;
  series_id: string;
  checked_in_users: CheckedInUserInfo[];
  all_users?: CheckedInUserInfo[]; // all users in the system for friendly matches
  total_checked_in: number;
  total_allocated: number;
  total_available: number;
}

export interface CreateAllocationRequest {
  match_id: string;
  user_id?: string;  // Optional for guest allocations
  guest_name?: string;  // Required if user_id is not provided
  role: AllocationRole;
  team_id?: string;
  two_team_speaker_role?: TwoTeamSpeakerRole;
  four_team_speaker_role?: FourTeamSpeakerRole;
  is_chair?: boolean;
}

export interface UpdateAllocationRequest {
  role?: AllocationRole;
  team_id?: string;
  two_team_speaker_role?: TwoTeamSpeakerRole;
  four_team_speaker_role?: FourTeamSpeakerRole;
  is_chair?: boolean;
}

export interface SwapAllocationRequest {
  allocation_id_1: string;
  allocation_id_2: string;
}

export interface AllocationHistory {
  id: string;
  allocation_id: string | null;
  match_id: string;
  user_id: string;
  action: string;
  previous_role: AllocationRole | null;
  new_role: AllocationRole | null;
  previous_team_id: string | null;
  new_team_id: string | null;
  changed_by: string;
  changed_at: string;
  notes: string | null;
}

export interface AllocationHistoryResponse {
  history: AllocationHistory[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

// Ballot
export interface SpeakerScoreResponse {
  id: string;
  allocation_id: string;
  speaker_username: string;
  score: number;
  feedback: string | null;
}

export interface TeamRankingResponse {
  id: string;
  team_id: string;
  team_name: string | null;
  rank: number;
  is_winner: boolean | null;
}

export interface BallotResponse {
  id: string;
  match_id: string;
  adjudicator_id: string;
  adjudicator_username: string;
  is_voting: boolean;
  is_submitted: boolean;
  submitted_at: string | null;
  notes: string | null;
  speaker_scores: SpeakerScoreResponse[];
  team_rankings: TeamRankingResponse[];
}

export interface SpeakerScoreInput {
  allocation_id: string;
  score: number;
  feedback?: string;
}

export interface TeamRankingInput {
  team_id: string;
  rank: number;
  is_winner?: boolean;
}

export interface SubmitBallotRequest {
  match_id: string;
  notes?: string;
  speaker_scores: SpeakerScoreInput[];
  team_rankings: TeamRankingInput[];
}

export interface SubmitFeedbackRequest {
  match_id: string;
  notes: string;
}

// Performance
export interface RankingCount {
  rank: number;
  count: number;
}

export interface PerformanceResponse {
  user_id: string;
  username: string;
  total_rounds: number;
  rounds_as_speaker: number;
  rounds_as_adjudicator: number;
  average_speaker_score: number | null;
  total_wins: number;
  total_losses: number;
  win_rate: number | null;
  rankings: RankingCount[];
}

export interface PerformanceListResponse {
  performances: PerformanceResponse[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

// Update team request
export interface UpdateTeamRequest {
  team_name?: string;
  institution?: string;
}
