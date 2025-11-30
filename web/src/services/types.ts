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
