import { getToken } from './session';

// Worker base URL — set VITE_API_URL for production (e.g. https://captcha-royale-worker.seanreid.workers.dev)
// Leave empty for local dev with Vite proxy
export const WORKER_URL = import.meta.env.VITE_API_URL || '';

export function apiUrl(path: string): string {
  return `${WORKER_URL}/api${path}`;
}

export function wsUrl(path: string): string {
  let base: string;
  if (WORKER_URL) {
    base = WORKER_URL.replace(/^http/, 'ws');
  } else {
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    base = `${proto}//${window.location.host}`;
  }

  // WebSocket can't set Authorization header, so pass token as query param
  const token = getToken();
  const sep = path.includes('?') ? '&' : '?';
  const tokenParam = token ? `${sep}token=${token}` : '';
  return `${base}/api${path}${tokenParam}`;
}
