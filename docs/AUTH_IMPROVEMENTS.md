# Authentication System Improvements

## Overview
This document describes the major authentication improvements made to the Tabrela application.

## Changes Made

### 1. Migrated from JWT to PASETO v4 Tokens

**Why PASETO over JWT?**
- **Security by default**: PASETO v4 uses modern, secure cryptographic algorithms (XChaCha20-Poly1305)
- **No algorithm confusion attacks**: Unlike JWT, PASETO has a versioned protocol that prevents algorithm substitution attacks
- **Simpler implementation**: PASETO has a more straightforward API with fewer security pitfalls
- **Built-in encryption**: PASETO v4 local tokens are encrypted by default, not just signed

**Backend Changes:**
- Replaced `jsonwebtoken` crate with `rusty_paseto` v0.7
- Created new `PasetoService` in `services/auth/src/paseto.rs`
- Updated all microservices (auth, attendance, merit, tabulation) to validate PASETO tokens
- Each service has a `paseto_utils.rs` module for token validation

**Token Format:**
- Old: `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...` (JWT)
- New: `v4.local.xxxxx...` (PASETO v4 local tokens)

### 2. Improved Axum Middleware Implementation

The authentication middleware now:
- Uses PASETO token validation instead of JWT
- Properly validates token type (access vs refresh)
- Checks token expiration
- Adds user information to request extensions for downstream handlers
- Handles both regular auth and admin-only routes

**Middleware Functions:**
- `auth_middleware`: Validates access tokens and adds user info to request
- `admin_middleware`: Additionally checks admin privileges
- `optional_auth_middleware`: Allows anonymous access but extracts user if token present

### 3. Exponential Backoff for Network Errors

**Problem Solved:**
Users were being logged out immediately when network connectivity was temporarily lost, even though their session was still valid.

**Solution:**
Implemented a comprehensive retry mechanism with exponential backoff:

**Frontend Changes:**
- Created `retryUtils.ts` with configurable retry logic
- Default: 3 retries with exponential backoff (1s, 2s, 4s)
- Added jitter to prevent thundering herd
- Maximum retry delay capped at 10 seconds

**Error Classification:**
- `NetworkError`: Temporary connectivity issues → Retry with backoff
- `AuthenticationError`: Invalid/expired tokens → Don't retry, prompt login
- Other errors: Propagated without retry

**Updated HTTP Clients:**
- `services/http.ts` (Auth API)
- `services/merit.ts` (Merit API)
- `services/attendance.ts` (Attendance API)
- `services/tabulation.ts` (Tabulation API)

### 4. Graceful Error Handling in Auth Context

**Before:**
Any error loading user data would immediately clear session and log out the user.

**After:**
- Network errors preserve the session and use cached user data
- Only authentication errors (expired/invalid tokens) trigger logout
- Users see stored user data while network recovers
- Proper error type checking prevents unnecessary logouts

**AuthContext Improvements:**
```typescript
// Network error - keep session
if (error?.name === 'NetworkError') {
  const storedUser = AuthService.getStoredUser();
  if (storedUser) setUser(storedUser);
}

// Auth error - clear session
if (error?.name === 'AuthenticationError') {
  AuthService.clearLocalSession();
}
```

## Migration Guide

### For Existing Users
**No action required!** The changes are backward compatible at the protocol level. When users log in again, they'll automatically receive PASETO tokens.

### For Developers

#### Testing Token Validation
```bash
# The token format has changed
# Old JWT: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
# New PASETO: v4.local.xxxxx...

# You can inspect PASETO tokens at: https://token.dev/
```

#### Environment Variables
No changes needed - the same `JWT_SECRET` environment variable is used for PASETO symmetric key.

**Important:** Ensure your secret is at least 32 bytes for proper PASETO v4 security. The implementation will pad or truncate as needed, but a properly-sized secret is recommended.

## Benefits

### Security
✅ Modern cryptographic algorithms (XChaCha20-Poly1305)
✅ No algorithm confusion attacks
✅ Encrypted tokens (not just signed)
✅ Versioned protocol prevents downgrade attacks

### Reliability
✅ Network errors don't log users out
✅ Automatic retry with exponential backoff
✅ Graceful degradation on temporary failures
✅ Better user experience during connectivity issues

### User Experience
✅ No more frustrating logouts due to network blips
✅ Transparent retry mechanism
✅ Session persists across temporary outages
✅ Clear error messages distinguish network vs auth issues

## Technical Details

### PASETO Token Structure
```
v4.local.[base64url-encoded-encrypted-payload]
```

**Claims included:**
- `sub`: User ID (UUID)
- `username`: Username string
- `exp`: Expiration timestamp
- `iat`: Issued at timestamp
- `jti`: Unique token ID (for revocation)
- `token_type`: "access" or "refresh"

### Retry Configuration
```typescript
const MAX_RETRIES = 3;
const INITIAL_RETRY_DELAY_MS = 1000;  // 1 second
const MAX_RETRY_DELAY_MS = 10000;     // 10 seconds

// Delay calculation: min(1000ms * 2^attempt + jitter, 10000ms)
```

### Error Flow
```
Network Error → Retry 1 (delay ~1s)
             → Retry 2 (delay ~2s)
             → Retry 3 (delay ~4s)
             → NetworkError thrown
             → User sees friendly message
             → Session preserved

Auth Error → No retry
          → AuthenticationError thrown
          → Session cleared
          → Redirect to login
```

## Testing

### Manual Testing Checklist
- [ ] Login with valid credentials
- [ ] Access protected routes
- [ ] Refresh page (should stay logged in)
- [ ] Disconnect network, try API call (should retry and show error)
- [ ] Reconnect network (should work normally)
- [ ] Let token expire (should refresh automatically)
- [ ] Try with expired refresh token (should logout)

### Backend Testing
```bash
cd services/auth
cargo test

cd ../attendance
cargo test

cd ../merit
cargo test

cd ../tabulation
cargo test
```

### Frontend Testing
```bash
cd web
npm test
```

## Troubleshooting

### "Invalid or expired token" immediately after login
- Check that all services are using the same `JWT_SECRET`
- Verify PASETO crate is properly compiled
- Check system time synchronization

### Continuous retries even with good network
- Check API server is running
- Verify CORS configuration
- Check for 5xx server errors in logs

### Users still being logged out on network errors
- Verify `retryUtils.ts` is imported in all HTTP clients
- Check error type detection logic
- Review browser console for error names

## Future Improvements

Potential enhancements for consideration:
- Token rotation/refresh on each request
- Rate limiting for retry attempts
- Offline mode with request queuing
- Token revocation list/blacklist
- Multi-factor authentication support
- Biometric authentication
- Session management dashboard

## References

- [PASETO RFC](https://github.com/paseto-standard/paseto-spec)
- [rusty_paseto documentation](https://docs.rs/rusty_paseto/)
- [Exponential Backoff Pattern](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)
