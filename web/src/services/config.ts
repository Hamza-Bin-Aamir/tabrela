// API Configuration
// All services are accessed via a single base URL with path prefixes:
// - /api/auth/* for auth service
// - /api/attendance/* for attendance service
// - /api/merit/* for merit service
// - /api/tabulation/* for tabulation service
//
// In development: Vite dev server proxies these routes to the backend services
// In production: nginx reverse proxy handles the routing
//
// Use empty string for API_BASE_URL in dev to use relative URLs (proxied by Vite)
// Set VITE_API_URL in production to point to the API gateway
export const API_BASE_URL = import.meta.env.VITE_API_URL || '';
export const AUTH_API_URL = `${API_BASE_URL}/api/auth`;
export const ATTENDANCE_API_URL = `${API_BASE_URL}/api/attendance`;
export const MERIT_API_URL = `${API_BASE_URL}/api/merit`;
export const TABULATION_API_URL = `${API_BASE_URL}/api/tabulation`;
