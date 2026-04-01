import { useState, useEffect, useCallback } from 'react';
import { api } from '../lib/api';
import { apiUrl } from '../lib/config';
import type { PlayerProfile } from '../types/player';

export function useAuth() {
  const [player, setPlayer] = useState<PlayerProfile | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.auth
      .me()
      .then((data) => setPlayer(data as PlayerProfile))
      .catch(() => setPlayer(null))
      .finally(() => setLoading(false));
  }, []);

  const logout = useCallback(async () => {
    await api.auth.logout();
    setPlayer(null);
  }, []);

  const login = useCallback((provider: 'google' | 'discord' | 'github') => {
    // Navigate directly to the worker — OAuth redirect must hit the worker origin
    window.location.href = apiUrl(`/auth/${provider}`);
  }, []);

  return { player, loading, login, logout };
}
