// API Configuration
// All services are accessed via a single base URL with path prefixes:
// - /api/auth/* for auth service
// - /api/attendance/* for attendance service
// - /api/merit/* for merit service
export const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';
export const AUTH_API_URL = `${API_BASE_URL}/api/auth`;
export const ATTENDANCE_API_URL = `${API_BASE_URL}/api/attendance`;
export const MERIT_API_URL = `${API_BASE_URL}/api/merit`;
