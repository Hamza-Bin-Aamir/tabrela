import { AUTH_API_URL } from './config';
import { TokenManager } from './tokenManager';
import type { ApiError } from './types';

// HTTP Client with automatic token handling
// Note: This client is primarily used for auth-related requests
export class HttpClient {
  private static async refreshAccessToken(): Promise<boolean> {
    const refreshToken = TokenManager.getRefreshToken();
    if (!refreshToken) {
      return false;
    }

    try {
      const response = await fetch(`${AUTH_API_URL}/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ refresh_token: refreshToken }),
      });

      if (!response.ok) {
        TokenManager.clearTokens();
        return false;
      }

      const data = await response.json();
      TokenManager.updateAccessToken(data.access_token);
      TokenManager.updateRefreshToken(data.refresh_token);
      return true;
    } catch (error) {
      TokenManager.clearTokens();
      return false;
    }
  }

  static async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const accessToken = TokenManager.getAccessToken();
    let csrfToken = TokenManager.getCsrfToken();

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`;
    }

    // For non-GET requests, ensure we have a CSRF token
    if (options.method !== 'GET' && !csrfToken && accessToken) {
      try {
        // Fetch CSRF token if we don't have one but are authenticated
        const response = await fetch(`${AUTH_API_URL}/csrf-token`, {
          method: 'GET',
          headers: {
            'Authorization': `Bearer ${accessToken}`,
          },
        });
        if (response.ok) {
          const data = await response.json();
          TokenManager.updateCsrfToken(data.csrf_token);
          csrfToken = data.csrf_token;
        }
      } catch (error) {
        console.warn('Failed to fetch CSRF token:', error);
      }
    }

    if (csrfToken && options.method !== 'GET') {
      headers['X-CSRF-Token'] = csrfToken;
    }

    let response = await fetch(`${AUTH_API_URL}${endpoint}`, {
      ...options,
      headers,
    });

    // If token expired, try to refresh
    if (response.status === 401 && accessToken) {
      const refreshed = await this.refreshAccessToken();
      if (refreshed) {
        // Retry the original request with new token
        const newAccessToken = TokenManager.getAccessToken();
        if (newAccessToken) {
          headers['Authorization'] = `Bearer ${newAccessToken}`;
          response = await fetch(`${AUTH_API_URL}${endpoint}`, {
            ...options,
            headers,
          });
        }
      }
    }

    if (!response.ok) {
      const error: ApiError = await response.json().catch(() => ({
        error: 'An unexpected error occurred',
      }));
      throw new Error(error.error);
    }

    return response.json();
  }

  static async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'GET' });
  }

  static async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  static async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  static async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}
