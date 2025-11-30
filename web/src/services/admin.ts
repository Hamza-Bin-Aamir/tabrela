import { HttpClient } from './http';
import type {
  AdminListUsersResponse,
  AdminCheckResponse,
  PromoteToAdminRequest,
} from './types';

// Admin Service
export class AdminService {
  static async checkAdminStatus(): Promise<AdminCheckResponse> {
    return HttpClient.get<AdminCheckResponse>('/admin/check');
  }

  static async listUsers(page = 1, perPage = 20): Promise<AdminListUsersResponse> {
    return HttpClient.get<AdminListUsersResponse>(`/admin/users?page=${page}&per_page=${perPage}`);
  }

  static async promoteToAdmin(userId: string): Promise<{ message: string }> {
    const request: PromoteToAdminRequest = { user_id: userId };
    return HttpClient.post<{ message: string }>('/admin/promote', request);
  }

  static async demoteAdmin(userId: string): Promise<{ message: string }> {
    const request: PromoteToAdminRequest = { user_id: userId };
    return HttpClient.post<{ message: string }>('/admin/demote', request);
  }
}
