import { apiUrl } from './config';
import { getToken } from './session';

async function fetchJson<T>(path: string, options?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options?.headers as Record<string, string>),
  };

  const token = getToken();
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const res = await fetch(apiUrl(path), {
    ...options,
    headers,
  });
  if (!res.ok) {
    throw new Error(`API error: ${res.status} ${res.statusText}`);
  }
  return res.json();
}

export const api = {
  auth: {
    me: () => fetchJson('/auth/me'),
    logout: () => fetchJson('/auth/logout', { method: 'POST' }),
  },
  profile: {
    get: (id: string) => fetchJson(`/profile/${id}`),
    update: (data: { display_name?: string }) =>
      fetchJson('/profile', { method: 'PATCH', body: JSON.stringify(data) }),
    history: (id: string, page = 1) => fetchJson(`/profile/${id}/history?page=${page}`),
    achievements: (id: string) => fetchJson(`/profile/${id}/achievements`),
  },
  leaderboard: {
    top: (season = 'current') => fetchJson(`/leaderboard?season=${season}`),
    around: (id: string) => fetchJson(`/leaderboard/around/${id}`),
  },
  match: {
    createPrivate: () =>
      fetchJson<{ roomCode: string }>('/match/private', { method: 'POST' }),
    joinPrivate: (code: string) =>
      fetchJson(`/match/join/${code}`, { method: 'POST' }),
  },
};
