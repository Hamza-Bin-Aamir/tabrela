// Retry configuration
const MAX_RETRIES = 3;
const INITIAL_RETRY_DELAY_MS = 1000; // 1 second
const MAX_RETRY_DELAY_MS = 10000; // 10 seconds

export class NetworkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NetworkError';
  }
}

export class AuthenticationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AuthenticationError';
  }
}

async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function calculateBackoffDelay(attempt: number): number {
  const exponentialDelay = INITIAL_RETRY_DELAY_MS * Math.pow(2, attempt);
  const jitter = Math.random() * 1000; // Add random jitter up to 1 second
  return Math.min(exponentialDelay + jitter, MAX_RETRY_DELAY_MS);
}

export function isNetworkError(error: unknown): boolean {
  if (error instanceof TypeError) {
    // TypeError is thrown by fetch for network failures
    const message = error.message.toLowerCase();
    return message.includes('fetch') ||
           message.includes('network') ||
           message.includes('connection') ||
           message.includes('failed to fetch');
  }
  return false;
}

export async function fetchWithRetry<T>(
  fetchFn: () => Promise<T>,
  retryCount = 0
): Promise<T> {
  try {
    return await fetchFn();
  } catch (error) {
    // Handle network errors with retry logic
    if (isNetworkError(error) && retryCount < MAX_RETRIES) {
      const delay = calculateBackoffDelay(retryCount);
      console.warn(
        `Network error on attempt ${retryCount + 1}/${MAX_RETRIES + 1}. ` +
        `Retrying in ${Math.round(delay / 1000)}s...`,
        error
      );

      await sleep(delay);
      return fetchWithRetry(fetchFn, retryCount + 1);
    }

    // If it's an auth error, propagate it
    if (error instanceof AuthenticationError) {
      throw error;
    }

    // If we've exhausted retries or it's not a network error
    if (isNetworkError(error)) {
      throw new NetworkError(
        'Network connection failed. Please check your internet connection and try again.'
      );
    }

    // Re-throw other errors
    throw error;
  }
}
