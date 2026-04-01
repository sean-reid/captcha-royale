import { useState, useEffect, useCallback } from 'react';
import { api } from '../lib/api';
import { apiUrl } from '../lib/config';
import { getToken, clearToken } from '../lib/session';
import type { PlayerProfile } from '../types/player';

export function useAuth() {
  const [player, setPlayer] = useState<PlayerProfile | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Only try to fetch profile if we have a token
    if (!getToken()) {
      setLoading(false);
      return;
    }
    api.auth
      .me()
      .then((data) => setPlayer(data as PlayerProfile))
      .catch(() => {
        clearToken();
        setPlayer(null);
      })
      .finally(() => setLoading(false));
  }, []);

  const logout = useCallback(async () => {
    try { await api.auth.logout(); } catch { /* ignore */ }
    clearToken();
    setPlayer(null);
  }, []);

  const login = useCallback((provider: 'google' | 'discord' | 'github') => {
    window.location.href = apiUrl(`/auth/${provider}`);
  }, []);

  return { player, loading, login, logout };
}
