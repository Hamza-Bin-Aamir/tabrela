import { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';
import { AuthService } from '../services/auth';
import type { UserResponse, LoginRequest, RegisterRequest, VerifyOtpRequest } from '../services/types';

interface AuthContextType {
  user: UserResponse | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (credentials: LoginRequest) => Promise<void>;
  register: (data: RegisterRequest) => Promise<{ email: string; message: string }>;
  verifyOtp: (data: VerifyOtpRequest) => Promise<void>;
  resendOtp: (email: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Check if user is already logged in
    const loadUser = async () => {
      try {
        if (AuthService.isAuthenticated()) {
          const currentUser = await AuthService.getCurrentUser();
          setUser(currentUser);
        }
      } catch (error) {
        console.error('Failed to load user:', error);
        // Clear invalid tokens without calling logout endpoint
        AuthService.clearLocalSession();
      } finally {
        setIsLoading(false);
      }
    };

    loadUser();
  }, []);

  const login = async (credentials: LoginRequest) => {
    const response = await AuthService.login(credentials);
    setUser(response.user);
  };

  const register = async (data: RegisterRequest) => {
    const response = await AuthService.register(data);
    // Registration no longer logs user in - they must verify email first
    return { email: response.email, message: response.message };
  };

  const verifyOtp = async (data: VerifyOtpRequest) => {
    const response = await AuthService.verifyOtp(data);
    setUser(response.user);
  };

  const resendOtp = async (email: string) => {
    await AuthService.resendOtp({ email });
  };

  const logout = async () => {
    await AuthService.logout();
    setUser(null);
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        isAuthenticated: user !== null,
        isLoading,
        login,
        register,
        verifyOtp,
        resendOtp,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
