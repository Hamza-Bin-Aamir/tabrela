import { useState, useEffect } from 'react';
import type { FormEvent } from 'react';
import { useNavigate, useLocation, Link } from 'react-router-dom';
import { AuthService } from '../services/auth';

export default function ResetPasswordPage() {
  const [email, setEmail] = useState('');
  const [otp, setOtp] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [attemptsRemaining, setAttemptsRemaining] = useState<number | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    // Get email from navigation state
    const stateEmail = location.state?.email;
    if (stateEmail) {
      setEmail(stateEmail);
    }
  }, [location]);

  const handleResend = async () => {
    if (!email) {
      setError('Please enter your email address');
      return;
    }

    try {
      await AuthService.requestPasswordReset({ email });
      setError('');
      setAttemptsRemaining(null);
      alert('A new reset code has been sent to your email!');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to resend code');
    }
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError('');

    if (!email) {
      setError('Email is required');
      return;
    }

    if (otp.length !== 6) {
      setError('Please enter a valid 6-digit code');
      return;
    }

    if (newPassword.length < 8) {
      setError('Password must be at least 8 characters long');
      return;
    }

    if (newPassword !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    setIsLoading(true);

    try {
      await AuthService.resetPassword({
        email,
        otp,
        new_password: newPassword,
      });

      // Success - redirect to login
      alert('Password reset successfully! Please login with your new password.');
      navigate('/login');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to reset password';
      setError(errorMessage);

      // Try to extract attempts remaining from error
      try {
        const errorObj = JSON.parse(errorMessage);
        if (errorObj.attempts_remaining !== undefined) {
          setAttemptsRemaining(errorObj.attempts_remaining);
        }
      } catch {
        // Error is not JSON, ignore
      }
    } finally {
      setIsLoading(false);
    }
  };

  const formatOtp = (value: string) => {
    // Only allow digits and limit to 6 characters
    return value.replace(/\D/g, '').slice(0, 6);
  };

  return (
    <section className="flex items-center justify-center min-h-[calc(100vh-var(--header-height)-200px)]">
      <div className="w-full max-w-md">
        <div className="bg-white rounded-2xl shadow-lg p-8">
          <div className="text-center mb-8">
            <h1 className="text-3xl font-bold text-gray-900">Reset Password</h1>
            <p className="mt-2 text-gray-600">Enter the code sent to your email</p>
          </div>

          {error && (
            <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-sm text-red-800">{error}</p>
              {attemptsRemaining !== null && attemptsRemaining > 0 && (
                <p className="text-xs text-red-600 mt-1">
                  {attemptsRemaining} {attemptsRemaining === 1 ? 'attempt' : 'attempts'} remaining
                </p>
              )}
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-6">
            <div>
              <label htmlFor="email" className="block text-sm font-medium text-gray-700 mb-2">
                Email Address
              </label>
              <input
                id="email"
                type="email"
                required
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all outline-none"
                placeholder="Enter your email"
                disabled={isLoading}
              />
            </div>

            <div>
              <label htmlFor="otp" className="block text-sm font-medium text-gray-700 mb-2">
                Verification Code
              </label>
              <input
                id="otp"
                type="text"
                required
                value={otp}
                onChange={(e) => setOtp(formatOtp(e.target.value))}
                className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all outline-none text-center text-2xl tracking-widest font-mono"
                placeholder="000000"
                maxLength={6}
                disabled={isLoading}
              />
              <p className="mt-2 text-xs text-gray-500 text-center">
                Enter the 6-digit code sent to your email
              </p>
            </div>

            <div>
              <label htmlFor="newPassword" className="block text-sm font-medium text-gray-700 mb-2">
                New Password
              </label>
              <input
                id="newPassword"
                type="password"
                required
                value={newPassword}
                onChange={(e) => setNewPassword(e.target.value)}
                className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all outline-none"
                placeholder="Enter new password"
                disabled={isLoading}
                minLength={8}
              />
            </div>

            <div>
              <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-700 mb-2">
                Confirm Password
              </label>
              <input
                id="confirmPassword"
                type="password"
                required
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition-all outline-none"
                placeholder="Confirm new password"
                disabled={isLoading}
                minLength={8}
              />
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="w-full bg-indigo-600 text-white py-3 px-4 rounded-lg font-medium hover:bg-indigo-700 focus:ring-4 focus:ring-indigo-300 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isLoading ? (
                <span className="flex items-center justify-center">
                  <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Resetting Password...
                </span>
              ) : (
                'Reset Password'
              )}
            </button>
          </form>

          <div className="mt-6 text-center space-y-2">
            <button
              type="button"
              onClick={handleResend}
              disabled={isLoading}
              className="text-sm text-indigo-600 hover:text-indigo-700 font-medium disabled:opacity-50"
            >
              Didn't receive the code? Resend
            </button>
            <p className="text-sm text-gray-600">
              Remember your password?{' '}
              <Link to="/login" className="text-indigo-600 hover:text-indigo-700 font-medium">
                Sign in
              </Link>
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
