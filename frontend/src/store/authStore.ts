/**
 * Centralized authentication store.
 * Persists token in localStorage, supports role-based access control,
 * and auto-refreshes the token before expiry.
 */

export type Role = 'admin' | 'merchant' | 'user';

export interface AuthUser {
  id: string;
  email: string;
  roles: Role[];
  permissions: string[];
}

export interface AuthState {
  user: AuthUser | null;
  token: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

const TOKEN_KEY = 'pulsar_token';
const REFRESH_TOKEN_KEY = 'pulsar_refresh_token';
const USER_KEY = 'pulsar_user';

// ── Persistence helpers ────────────────────────────────────────────────────

function persist(token: string, refreshToken: string, user: AuthUser): void {
  localStorage.setItem(TOKEN_KEY, token);
  localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
  localStorage.setItem(USER_KEY, JSON.stringify(user));
}

function clearStorage(): void {
  localStorage.removeItem(TOKEN_KEY);
  localStorage.removeItem(REFRESH_TOKEN_KEY);
  localStorage.removeItem(USER_KEY);
}

export function loadStoredAuth(): Partial<AuthState> {
  const token = localStorage.getItem(TOKEN_KEY);
  const refreshToken = localStorage.getItem(REFRESH_TOKEN_KEY);
  const raw = localStorage.getItem(USER_KEY);
  if (!token || !raw) return {};
  try {
    const user: AuthUser = JSON.parse(raw);
    return { token, refreshToken, user, isAuthenticated: true };
  } catch {
    clearStorage();
    return {};
  }
}

// ── JWT helpers ────────────────────────────────────────────────────────────

function getTokenExpiry(token: string): number | null {
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    return typeof payload.exp === 'number' ? payload.exp * 1000 : null;
  } catch {
    return null;
  }
}

// ── Auth store ─────────────────────────────────────────────────────────────

type Listener = (state: AuthState) => void;

class AuthStore {
  private state: AuthState = {
    user: null,
    token: null,
    refreshToken: null,
    isAuthenticated: false,
    isLoading: true,
    ...loadStoredAuth(),
  };

  private listeners = new Set<Listener>();
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;

  getState(): AuthState {
    return this.state;
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private setState(patch: Partial<AuthState>): void {
    this.state = { ...this.state, ...patch };
    this.listeners.forEach((l) => l(this.state));
  }

  private scheduleRefresh(token: string, onRefresh: () => Promise<void>): void {
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    const expiry = getTokenExpiry(token);
    if (!expiry) return;
    // Refresh 60 s before expiry
    const delay = expiry - Date.now() - 60_000;
    if (delay <= 0) {
      onRefresh();
      return;
    }
    this.refreshTimer = setTimeout(onRefresh, delay);
  }

  login(token: string, refreshToken: string, user: AuthUser, onRefresh: () => Promise<void>): void {
    persist(token, refreshToken, user);
    this.setState({ token, refreshToken, user, isAuthenticated: true, isLoading: false });
    this.scheduleRefresh(token, onRefresh);
  }

  updateToken(token: string, onRefresh: () => Promise<void>): void {
    localStorage.setItem(TOKEN_KEY, token);
    this.setState({ token });
    this.scheduleRefresh(token, onRefresh);
  }

  logout(): void {
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    clearStorage();
    this.setState({
      user: null,
      token: null,
      refreshToken: null,
      isAuthenticated: false,
      isLoading: false,
    });
  }

  setLoading(isLoading: boolean): void {
    this.setState({ isLoading });
  }

  hasRole(role: Role): boolean {
    return this.state.user?.roles.includes(role) ?? false;
  }

  hasPermission(permission: string): boolean {
    return this.state.user?.permissions.includes(permission) ?? false;
  }
}

export const authStore = new AuthStore();
