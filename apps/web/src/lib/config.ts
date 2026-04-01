// Worker base URL — set VITE_API_URL for production (e.g. https://captcha-royale-worker.seanreid.workers.dev)
// Leave empty for local dev with Vite proxy
export const WORKER_URL = import.meta.env.VITE_API_URL || '';

export function apiUrl(path: string): string {
  return `${WORKER_URL}/api${path}`;
}

export function wsUrl(path: string): string {
  if (WORKER_URL) {
    // Convert https:// to wss://
    const base = WORKER_URL.replace(/^http/, 'ws');
    return `${base}/api${path}`;
  }
  // Local dev — use current host
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${proto}//${window.location.host}/api${path}`;
}
