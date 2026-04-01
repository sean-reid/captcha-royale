const TOKEN_KEY = 'captcha-royale-token';

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}

/** Call once on app startup to extract token from OAuth redirect hash */
export function extractTokenFromHash(): void {
  const hash = window.location.hash;
  const match = hash.match(/token=([^&]+)/);
  if (match) {
    setToken(match[1]);
    // Clean the hash from the URL
    window.history.replaceState(null, '', window.location.pathname + window.location.search);
  }
}
