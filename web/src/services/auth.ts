import { HttpClient } from './http';
import { TokenManager } from './tokenManager';
import type {
  RegisterRequest,
  LoginRequest,
  RegisterResponse,
  LoginResponse,
  UserResponse,
  VerifyOtpRequest,
  VerifyOtpResponse,
  ResendOtpRequest,
  RequestPasswordResetRequest,
  ResetPasswordRequest,
} from './types';

// Authentication Service
export class AuthService {
  static async register(data: RegisterRequest): Promise<RegisterResponse> {
    const result = await HttpClient.post<RegisterResponse>('/register', data);
    // No tokens returned on registration - user must verify email first
    return result;
  }

  static async verifyOtp(data: VerifyOtpRequest): Promise<VerifyOtpResponse> {
    const result = await HttpClient.post<VerifyOtpResponse>('/verify-otp', data);
    TokenManager.setTokens(result.auth, result.csrf_token, result.user);
    return result;
  }

  static async resendOtp(data: ResendOtpRequest): Promise<{ message: string }> {
    return HttpClient.post<{ message: string }>('/resend-verification', data);
  }

  static async login(data: LoginRequest): Promise<LoginResponse> {
    const result = await HttpClient.post<LoginResponse>('/login', data);
    TokenManager.setTokens(result.auth, result.csrf_token, result.user);
    return result;
  }

  static async logout(): Promise<void> {
    try {
      // Only call logout endpoint if we have a CSRF token
      if (TokenManager.getCsrfToken()) {
        await HttpClient.post<{ message: string }>('/logout');
      }
    } catch (error) {
      console.error('Logout failed:', error);
    } finally {
      TokenManager.clearTokens();
    }
  }

  static clearLocalSession(): void {
    TokenManager.clearTokens();
  }

  static async getCurrentUser(): Promise<UserResponse> {
    return HttpClient.get<UserResponse>('/me');
  }

  static async getCsrfToken(): Promise<string> {
    const data = await HttpClient.get<{ csrf_token: string }>('/csrf-token');
    return data.csrf_token;
  }

  static async requestPasswordReset(data: RequestPasswordResetRequest): Promise<{ message: string }> {
    return HttpClient.post<{ message: string }>('/request-password-reset', data);
  }

  static async resetPassword(data: ResetPasswordRequest): Promise<{ message: string }> {
    return HttpClient.post<{ message: string }>('/reset-password', data);
  }

  static isAuthenticated(): boolean {
    return TokenManager.getAccessToken() !== null;
  }

  static getStoredUser(): UserResponse | null {
    return TokenManager.getUser();
  }
}
