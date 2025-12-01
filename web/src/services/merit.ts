import { MERIT_API_URL } from './config';
import { TokenManager } from './tokenManager';
import type {
  ProfileResponse,
  PublicProfileResponse,
  PrivateProfileResponse,
  AdminProfileResponse,
  MeritResponse,
  MeritHistoryResponse,
  UpdateMeritRequest,
  AdminMeritListResponse,
  AwardListResponse,
  AwardHistoryResponse,
  CreateAwardRequest,
  UpgradeAwardRequest,
  EditAwardRequest,
  AdminAwardListResponse,
} from './types';

class MeritHttpClient {
  private baseUrl: string;

  constructor() {
    this.baseUrl = MERIT_API_URL;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;

    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    const accessToken = TokenManager.getAccessToken();
    if (accessToken) {
      (headers as Record<string, string>)['Authorization'] = `Bearer ${accessToken}`;
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Request failed' }));
      throw new Error(errorData.error || `HTTP error! status: ${response.status}`);
    }

    return response.json();
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'GET' });
  }

  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async patch<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PATCH',
      body: data ? JSON.stringify(data) : undefined,
    });
  }
}

const httpClient = new MeritHttpClient();

// ============================================================================
// Profile Service
// ============================================================================

export class ProfileService {
  /**
   * Get user profile by username.
   * Returns different data based on who's viewing:
   * - Public: limited info (username, year_joined)
   * - Self: includes merit points
   * - Admin: full details including admin status
   */
  static async getProfile(username: string): Promise<ProfileResponse> {
    return httpClient.get<ProfileResponse>(`/users/${username}`);
  }

  /**
   * Type guard to check if profile is private (includes merit)
   */
  static isPrivateProfile(profile: ProfileResponse): profile is PrivateProfileResponse {
    return 'merit_points' in profile && 'email' in profile && !('is_admin' in profile);
  }

  /**
   * Type guard to check if profile is admin view
   */
  static isAdminProfile(profile: ProfileResponse): profile is AdminProfileResponse {
    return 'is_admin' in profile;
  }

  /**
   * Type guard to check if profile is public only
   */
  static isPublicProfile(profile: ProfileResponse): profile is PublicProfileResponse {
    return !('email' in profile);
  }
}

// ============================================================================
// Merit Service
// ============================================================================

export class MeritService {
  /**
   * Get current user's merit points
   */
  static async getMyMerit(): Promise<MeritResponse> {
    return httpClient.get<MeritResponse>('/merit/me');
  }

  /**
   * Get current user's merit history
   */
  static async getMyMeritHistory(page = 1, perPage = 20): Promise<MeritHistoryResponse> {
    return httpClient.get<MeritHistoryResponse>(
      `/merit/me/history?page=${page}&per_page=${perPage}`
    );
  }
}

// ============================================================================
// Admin Merit Service
// ============================================================================

export class AdminMeritService {
  /**
   * List all users with their merit points (admin only)
   */
  static async listAllMerits(page = 1, perPage = 50): Promise<AdminMeritListResponse> {
    return httpClient.get<AdminMeritListResponse>(
      `/admin/merit?page=${page}&per_page=${perPage}`
    );
  }

  /**
   * Update merit points for a user (admin only)
   */
  static async updateMerit(
    data: UpdateMeritRequest
  ): Promise<{
    message: string;
    user_id: string;
    username: string;
    previous_merit: number;
    new_merit: number;
    change_amount: number;
    reason: string;
  }> {
    return httpClient.post('/admin/merit', data);
  }

  /**
   * Get merit for a specific user (admin only)
   */
  static async getUserMerit(userId: string): Promise<{
    user_id: string;
    username: string;
    merit_points: number;
    is_admin: boolean;
  }> {
    return httpClient.get(`/admin/merit/${userId}`);
  }

  /**
   * Get merit history for a specific user (admin only)
   */
  static async getUserMeritHistory(
    userId: string,
    page = 1,
    perPage = 20
  ): Promise<MeritHistoryResponse> {
    return httpClient.get<MeritHistoryResponse>(
      `/admin/merit/${userId}/history?page=${page}&per_page=${perPage}`
    );
  }
}

// ============================================================================
// Award Service (Public)
// ============================================================================

export class AwardService {
  /**
   * Get awards for any user by username (public - awards are visible to all)
   */
  static async getUserAwards(username: string): Promise<AwardListResponse> {
    return httpClient.get<AwardListResponse>(`/users/${username}/awards`);
  }

  /**
   * Get current user's awards
   */
  static async getMyAwards(): Promise<AwardListResponse> {
    return httpClient.get<AwardListResponse>('/awards/me');
  }

  /**
   * Get current user's award history (tier upgrades)
   */
  static async getMyAwardsHistory(): Promise<AwardHistoryResponse> {
    return httpClient.get<AwardHistoryResponse>('/awards/me/history');
  }
}

// ============================================================================
// Admin Award Service
// ============================================================================

export class AdminAwardService {
  /**
   * List all awards (admin only)
   */
  static async listAllAwards(page = 1, perPage = 50): Promise<AdminAwardListResponse> {
    return httpClient.get<AdminAwardListResponse>(
      `/admin/awards?page=${page}&per_page=${perPage}`
    );
  }

  /**
   * Create a new award for a user (admin only)
   */
  static async createAward(
    data: CreateAwardRequest
  ): Promise<{
    message: string;
    award: {
      id: string;
      user_id: string;
      title: string;
      tier: string;
    };
  }> {
    return httpClient.post('/admin/awards', data);
  }

  /**
   * Edit an award (admin only)
   */
  static async editAward(
    awardId: string,
    data: EditAwardRequest
  ): Promise<{
    message: string;
    award: {
      id: string;
      title: string;
      description: string | null;
      tier: string;
    };
  }> {
    return httpClient.put(`/admin/awards/${awardId}`, data);
  }

  /**
   * Upgrade an award's tier (admin only)
   */
  static async upgradeAward(
    awardId: string,
    data: UpgradeAwardRequest
  ): Promise<{
    message: string;
    award: {
      id: string;
      title: string;
      previous_tier: string;
      new_tier: string;
    };
  }> {
    return httpClient.patch(`/admin/awards/${awardId}/upgrade`, data);
  }

  /**
   * Get award history for a user (admin only)
   */
  static async getUserAwardHistory(userId: string): Promise<AwardHistoryResponse> {
    return httpClient.get<AwardHistoryResponse>(`/admin/awards/${userId}/history`);
  }
}
