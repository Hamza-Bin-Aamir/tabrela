import { ATTENDANCE_API_URL } from './config';
import { TokenManager } from './tokenManager';
import type {
  Event,
  EventListResponse,
  EventAttendanceResponse,
  CreateEventRequest,
  UpdateEventRequest,
  AttendanceRecord,
  SetAvailabilityRequest,
  CheckInRequest,
  RevokeAvailabilityRequest,
  LockEventRequest,
  AttendanceMatrixResponse,
} from './types';

class AttendanceHttpClient {
  private baseUrl: string;

  constructor() {
    this.baseUrl = ATTENDANCE_API_URL;
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

  async patch<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PATCH',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}

const httpClient = new AttendanceHttpClient();

// Attendance/Event Service
export class AttendanceService {
  // ========================================================================
  // Event Methods
  // ========================================================================

  static async listEvents(
    page = 1,
    perPage = 20,
    eventType?: string,
    upcomingOnly = false
  ): Promise<EventListResponse> {
    let url = `/events?page=${page}&per_page=${perPage}`;
    if (eventType) url += `&event_type=${eventType}`;
    if (upcomingOnly) url += `&upcoming_only=true`;
    return httpClient.get<EventListResponse>(url);
  }

  static async getEvent(eventId: string): Promise<Event> {
    return httpClient.get<Event>(`/events/${eventId}`);
  }

  static async createEvent(data: CreateEventRequest): Promise<{ message: string; event: Event }> {
    return httpClient.post<{ message: string; event: Event }>('/events', data);
  }

  static async updateEvent(eventId: string, data: UpdateEventRequest): Promise<{ message: string; event: Event }> {
    return httpClient.patch<{ message: string; event: Event }>(`/events/${eventId}`, data);
  }

  static async deleteEvent(eventId: string): Promise<{ message: string }> {
    return httpClient.delete<{ message: string }>(`/events/${eventId}`);
  }

  static async lockEvent(eventId: string, isLocked: boolean): Promise<{ message: string; event: Event }> {
    const data: LockEventRequest = { is_locked: isLocked };
    return httpClient.post<{ message: string; event: Event }>(`/events/${eventId}/lock`, data);
  }

  // ========================================================================
  // Attendance Methods
  // ========================================================================

  static async getEventAttendance(eventId: string): Promise<EventAttendanceResponse> {
    return httpClient.get<EventAttendanceResponse>(`/events/${eventId}/attendance`);
  }

  static async getMyAttendance(eventId: string): Promise<AttendanceRecord> {
    return httpClient.get<AttendanceRecord>(`/events/${eventId}/my-attendance`);
  }

  static async setAvailability(eventId: string, isAvailable: boolean): Promise<{ message: string; attendance: AttendanceRecord }> {
    const data: SetAvailabilityRequest = { is_available: isAvailable };
    return httpClient.post<{ message: string; attendance: AttendanceRecord }>(`/events/${eventId}/availability`, data);
  }

  static async checkInUser(eventId: string, userId: string, isCheckedIn: boolean): Promise<{ message: string; attendance: AttendanceRecord }> {
    const data: CheckInRequest = { user_id: userId, is_checked_in: isCheckedIn };
    return httpClient.post<{ message: string; attendance: AttendanceRecord }>(`/events/${eventId}/check-in`, data);
  }

  static async revokeAvailability(eventId: string, userId: string): Promise<{ message: string; attendance: AttendanceRecord }> {
    const data: RevokeAvailabilityRequest = { user_id: userId };
    return httpClient.post<{ message: string; attendance: AttendanceRecord }>(`/events/${eventId}/revoke`, data);
  }

  static async adminSetAvailability(eventId: string, userId: string, isAvailable: boolean): Promise<{ message: string; attendance: AttendanceRecord }> {
    return httpClient.post<{ message: string; attendance: AttendanceRecord }>(`/events/${eventId}/set-availability`, {
      user_id: userId,
      is_available: isAvailable,
    });
  }

  // ========================================================================
  // Dashboard/Matrix Methods (Admin only)
  // ========================================================================

  static async getAttendanceMatrix(): Promise<AttendanceMatrixResponse> {
    return httpClient.get<AttendanceMatrixResponse>('/attendance/matrix');
  }
}
