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
