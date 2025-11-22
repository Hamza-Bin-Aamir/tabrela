// Authentication Types
export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface LoginRequest {
  username: string;
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
  created_at: string;
}

export interface RegisterResponse {
  user: UserResponse;
  auth: AuthResponse;
  csrf_token: string;
}

export interface LoginResponse {
  user: UserResponse;
  auth: AuthResponse;
  csrf_token: string;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface ApiError {
  error: string;
}
