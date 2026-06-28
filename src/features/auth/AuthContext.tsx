import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";

import { authApi } from "../../lib/api";
import type { AdminUser } from "../../types/common";

type AuthContextValue = {
  admin: AdminUser | null;
  hasAdmin: boolean | null;
  loading: boolean;
  setup: (payload: { full_name: string; email: string; password: string }) => Promise<void>;
  login: (payload: { email: string; password: string }) => Promise<void>;
  logout: () => Promise<void>;
  refresh: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [admin, setAdmin] = useState<AdminUser | null>(null);
  const [hasAdmin, setHasAdmin] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const [exists, current] = await Promise.all([authApi.hasAdmin(), authApi.current()]);
      setHasAdmin(exists);
      setAdmin(current);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const setup = useCallback(async (payload: { full_name: string; email: string; password: string }) => {
    const user = await authApi.setup(payload);
    setHasAdmin(true);
    setAdmin(user);
  }, []);

  const login = useCallback(async (payload: { email: string; password: string }) => {
    const user = await authApi.login(payload);
    setAdmin(user);
    setHasAdmin(true);
  }, []);

  const logout = useCallback(async () => {
    await authApi.logout();
    setAdmin(null);
  }, []);

  const value = useMemo(
    () => ({ admin, hasAdmin, loading, setup, login, logout, refresh }),
    [admin, hasAdmin, loading, login, logout, refresh, setup]
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const value = useContext(AuthContext);
  if (!value) {
    throw new Error("useAuth must be used inside AuthProvider");
  }
  return value;
}
