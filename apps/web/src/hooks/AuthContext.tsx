import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from 'react';
import { api } from '../lib/api';
import { apiUrl } from '../lib/config';
import { getToken, clearToken } from '../lib/session';
import type { PlayerProfile } from '../types/player';

interface AuthState {
  player: PlayerProfile | null;
  loading: boolean;
  login: (provider: 'google' | 'discord' | 'github') => void;
  logout: () => Promise<void>;
  refresh: () => Promise<void>;
}

const AuthContext = createContext<AuthState>({
  player: null,
  loading: true,
  login: () => {},
  logout: async () => {},
  refresh: async () => {},
});

export function AuthProvider({ children }: { children: ReactNode }) {
  const [player, setPlayer] = useState<PlayerProfile | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchProfile = useCallback(async () => {
    if (!getToken()) {
      setPlayer(null);
      setLoading(false);
      return;
    }
    try {
      const data = await api.auth.me();
      setPlayer(data as PlayerProfile);
    } catch {
      clearToken();
      setPlayer(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchProfile();
  }, [fetchProfile]);

  const logout = useCallback(async () => {
    try { await api.auth.logout(); } catch { /* ignore */ }
    clearToken();
    setPlayer(null);
  }, []);

  const login = useCallback((provider: 'google' | 'discord' | 'github') => {
    window.location.href = apiUrl(`/auth/${provider}`);
  }, []);

  return (
    <AuthContext.Provider value={{ player, loading, login, logout, refresh: fetchProfile }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext);
}
