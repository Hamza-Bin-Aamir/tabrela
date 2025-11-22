import { HttpClient } from './http';
import { TokenManager } from './tokenManager';
import type {
  RegisterRequest,
  LoginRequest,
  RegisterResponse,
  LoginResponse,
  UserResponse,
} from './types';

// Authentication Service
export class AuthService {
  static async register(data: RegisterRequest): Promise<RegisterResponse> {
    const result = await HttpClient.post<RegisterResponse>('/register', data);
    TokenManager.setTokens(result.auth, result.csrf_token, result.user);
    return result;
  }

  static async login(data: LoginRequest): Promise<LoginResponse> {
    const result = await HttpClient.post<LoginResponse>('/login', data);
    TokenManager.setTokens(result.auth, result.csrf_token, result.user);
    return result;
  }

  static async logout(): Promise<void> {
    try {
      await HttpClient.post<{ message: string }>('/logout');
    } finally {
      TokenManager.clearTokens();
    }
  }

  static async getCurrentUser(): Promise<UserResponse> {
    return HttpClient.get<UserResponse>('/me');
  }

  static async getCsrfToken(): Promise<string> {
    const data = await HttpClient.get<{ csrf_token: string }>('/csrf-token');
    return data.csrf_token;
  }

  static isAuthenticated(): boolean {
    return TokenManager.getAccessToken() !== null;
  }

  static getStoredUser(): UserResponse | null {
    return TokenManager.getUser();
  }
}
