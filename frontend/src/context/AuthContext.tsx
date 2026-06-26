import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  type ReactNode,
} from 'react';
import { authStore, type AuthState, type AuthUser, type Role } from '../store/authStore';

interface AuthContextValue extends AuthState {
  login: (token: string, refreshToken: string, user: AuthUser) => void;
  logout: () => void;
  hasRole: (role: Role) => boolean;
  hasPermission: (permission: string) => boolean;
}

const AuthContext = createContext<AuthContextValue | null>(null);

interface AuthProviderProps {
  children: ReactNode;
  /** Called when a token refresh is needed. Must return new token or throw. */
  onRefreshToken?: (refreshToken: string) => Promise<string>;
  /** Called on logout or unauthorized state — redirect to login. */
  onUnauthorized?: () => void;
}

export function AuthProvider({ children, onRefreshToken, onUnauthorized }: AuthProviderProps) {
  const [state, setState] = useState<AuthState>(authStore.getState);

  useEffect(() => {
    const unsub = authStore.subscribe(setState);
    authStore.setLoading(false);
    return unsub;
  }, []);

  // Redirect to login when unauthenticated and no longer loading
  useEffect(() => {
    if (!state.isLoading && !state.isAuthenticated) {
      onUnauthorized?.();
    }
  }, [state.isAuthenticated, state.isLoading, onUnauthorized]);

  const buildRefresh = useCallback(
    (refreshToken: string) => async () => {
      if (!onRefreshToken) return;
      try {
        const newToken = await onRefreshToken(refreshToken);
        authStore.updateToken(newToken, buildRefresh(refreshToken));
      } catch {
        authStore.logout();
        onUnauthorized?.();
      }
    },
    [onRefreshToken, onUnauthorized],
  );

  const login = useCallback(
    (token: string, refreshToken: string, user: AuthUser) => {
      authStore.login(token, refreshToken, user, buildRefresh(refreshToken));
    },
    [buildRefresh],
  );

  const logout = useCallback(() => {
    authStore.logout();
    onUnauthorized?.();
  }, [onUnauthorized]);

  const value: AuthContextValue = {
    ...state,
    login,
    logout,
    hasRole: authStore.hasRole.bind(authStore),
    hasPermission: authStore.hasPermission.bind(authStore),
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used inside <AuthProvider>');
  return ctx;
}
