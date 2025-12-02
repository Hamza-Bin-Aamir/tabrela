import { TABULATION_API_URL } from './config';
import { TokenManager } from './tokenManager';
import type {
  MatchSeries,
  SeriesListResponse,
  CreateSeriesRequest,
  UpdateSeriesRequest,
  MatchResponse,
  MatchListResponse,
  CreateMatchRequest,
  UpdateMatchRequest,
  ReleaseToggleRequest,
  AllocationPoolResponse,
  CreateAllocationRequest,
  UpdateAllocationRequest,
  SwapAllocationRequest,
  AllocationHistoryResponse,
  BallotResponse,
  SubmitBallotRequest,
  SubmitFeedbackRequest,
  PerformanceResponse,
  UpdateTeamRequest,
  MatchTeam,
  MatchStatus,
} from './types';

class TabulationHttpClient {
  private baseUrl: string;

  constructor() {
    this.baseUrl = TABULATION_API_URL;
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

  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}

const httpClient = new TabulationHttpClient();

// Tabulation Service
export class TabulationService {
  // ========================================================================
  // Series Methods
  // ========================================================================

  static async listSeries(
    eventId: string,
    page = 1,
    perPage = 20
  ): Promise<SeriesListResponse> {
    return httpClient.get<SeriesListResponse>(
      `/series?event_id=${eventId}&page=${page}&per_page=${perPage}`
    );
  }

  static async getSeries(seriesId: string): Promise<MatchSeries> {
    return httpClient.get<MatchSeries>(`/series/${seriesId}`);
  }

  static async createSeries(data: CreateSeriesRequest): Promise<{ message: string; series: MatchSeries }> {
    return httpClient.post<{ message: string; series: MatchSeries }>('/admin/series', data);
  }

  static async updateSeries(seriesId: string, data: UpdateSeriesRequest): Promise<{ message: string; series: MatchSeries }> {
    return httpClient.put<{ message: string; series: MatchSeries }>(`/admin/series/${seriesId}`, data);
  }

  static async deleteSeries(seriesId: string): Promise<{ message: string }> {
    return httpClient.delete<{ message: string }>(`/admin/series/${seriesId}`);
  }

  // ========================================================================
  // Match Methods
  // ========================================================================

  static async listMatches(
    options: {
      seriesId?: string;
      eventId?: string;
      status?: MatchStatus;
      page?: number;
      perPage?: number;
    }
  ): Promise<MatchListResponse> {
    const params = new URLSearchParams();
    if (options.seriesId) params.append('series_id', options.seriesId);
    if (options.eventId) params.append('event_id', options.eventId);
    if (options.status) params.append('status', options.status);
    params.append('page', String(options.page || 1));
    params.append('per_page', String(options.perPage || 20));
    
    return httpClient.get<MatchListResponse>(`/matches?${params.toString()}`);
  }

  static async getMatch(matchId: string): Promise<MatchResponse> {
    return httpClient.get<MatchResponse>(`/matches/${matchId}`);
  }

  static async createMatch(data: CreateMatchRequest): Promise<{ message: string; match: MatchResponse; teams: MatchTeam[] }> {
    return httpClient.post<{ message: string; match: MatchResponse; teams: MatchTeam[] }>('/admin/matches', data);
  }

  static async updateMatch(matchId: string, data: UpdateMatchRequest): Promise<{ message: string; match: MatchResponse }> {
    return httpClient.put<{ message: string; match: MatchResponse }>(`/admin/matches/${matchId}`, data);
  }

  static async deleteMatch(matchId: string): Promise<{ message: string }> {
    return httpClient.delete<{ message: string }>(`/admin/matches/${matchId}`);
  }

  static async toggleRelease(matchId: string, data: ReleaseToggleRequest): Promise<{ message: string; match: MatchResponse }> {
    return httpClient.post<{ message: string; match: MatchResponse }>(`/admin/matches/${matchId}/release`, data);
  }

  // ========================================================================
  // Team Methods
  // ========================================================================

  static async updateTeam(teamId: string, data: UpdateTeamRequest): Promise<{ message: string; team: MatchTeam }> {
    return httpClient.put<{ message: string; team: MatchTeam }>(`/admin/teams/${teamId}`, data);
  }

  // ========================================================================
  // Allocation Methods
  // ========================================================================

  static async getAllocationPool(seriesId: string): Promise<AllocationPoolResponse> {
    return httpClient.get<AllocationPoolResponse>(`/admin/series/${seriesId}/pool`);
  }

  static async createAllocation(data: CreateAllocationRequest): Promise<{ message: string; allocation: unknown }> {
    return httpClient.post<{ message: string; allocation: unknown }>('/admin/allocations', data);
  }

  static async updateAllocation(allocationId: string, data: UpdateAllocationRequest): Promise<{ message: string; allocation: unknown }> {
    return httpClient.put<{ message: string; allocation: unknown }>(`/admin/allocations/${allocationId}`, data);
  }

  static async deleteAllocation(allocationId: string): Promise<{ message: string }> {
    return httpClient.delete<{ message: string }>(`/admin/allocations/${allocationId}`);
  }

  static async swapAllocations(data: SwapAllocationRequest): Promise<{ message: string }> {
    return httpClient.post<{ message: string }>('/admin/allocations/swap', data);
  }

  static async getAllocationHistory(matchId: string, page = 1, perPage = 50): Promise<AllocationHistoryResponse> {
    return httpClient.get<AllocationHistoryResponse>(
      `/admin/matches/${matchId}/history?page=${page}&per_page=${perPage}`
    );
  }

  // ========================================================================
  // Ballot Methods
  // ========================================================================

  static async getMyBallot(matchId: string): Promise<BallotResponse> {
    return httpClient.get<BallotResponse>(`/matches/${matchId}/my-ballot`);
  }

  static async submitBallot(data: SubmitBallotRequest): Promise<{ message: string; ballot: BallotResponse }> {
    return httpClient.post<{ message: string; ballot: BallotResponse }>(
      `/matches/${data.match_id}/submit-ballot`,
      data
    );
  }

  static async submitFeedback(data: SubmitFeedbackRequest): Promise<{ message: string; ballot: BallotResponse }> {
    return httpClient.post<{ message: string; ballot: BallotResponse }>(
      `/matches/${data.match_id}/submit-feedback`,
      data
    );
  }

  static async getMatchBallots(matchId: string): Promise<BallotResponse[]> {
    return httpClient.get<BallotResponse[]>(`/admin/matches/${matchId}/ballots`);
  }

  // ========================================================================
  // Performance Methods
  // ========================================================================

  static async getUserPerformance(userId: string, eventId?: string): Promise<PerformanceResponse> {
    const params = eventId ? `?event_id=${eventId}` : '';
    return httpClient.get<PerformanceResponse>(`/users/${userId}/performance${params}`);
  }
}
