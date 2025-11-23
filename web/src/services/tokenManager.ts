import type { AuthResponse, UserResponse } from './types';

// Token Management
export class TokenManager {
  private static ACCESS_TOKEN_KEY = 'access_token';
  private static REFRESH_TOKEN_KEY = 'refresh_token';
  private static CSRF_TOKEN_KEY = 'csrf_token';
  private static USER_KEY = 'user';

  static setTokens(auth: AuthResponse, csrfToken: string, user: UserResponse): void {
    localStorage.setItem(this.ACCESS_TOKEN_KEY, auth.access_token);
    localStorage.setItem(this.REFRESH_TOKEN_KEY, auth.refresh_token);
    localStorage.setItem(this.CSRF_TOKEN_KEY, csrfToken);
    localStorage.setItem(this.USER_KEY, JSON.stringify(user));
  }

  static getAccessToken(): string | null {
    return localStorage.getItem(this.ACCESS_TOKEN_KEY);
  }

  static getRefreshToken(): string | null {
    return localStorage.getItem(this.REFRESH_TOKEN_KEY);
  }

  static getCsrfToken(): string | null {
    return localStorage.getItem(this.CSRF_TOKEN_KEY);
  }

  static getUser(): UserResponse | null {
    const userStr = localStorage.getItem(this.USER_KEY);
    return userStr ? JSON.parse(userStr) : null;
  }

  static clearTokens(): void {
    localStorage.removeItem(this.ACCESS_TOKEN_KEY);
    localStorage.removeItem(this.REFRESH_TOKEN_KEY);
    localStorage.removeItem(this.CSRF_TOKEN_KEY);
    localStorage.removeItem(this.USER_KEY);
  }

  static updateAccessToken(accessToken: string): void {
    localStorage.setItem(this.ACCESS_TOKEN_KEY, accessToken);
  }

  static updateRefreshToken(refreshToken: string): void {
    localStorage.setItem(this.REFRESH_TOKEN_KEY, refreshToken);
  }
}
