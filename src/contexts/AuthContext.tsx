'use client';

import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  useEffect,
  useLayoutEffect,
} from 'react';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';
import { supabaseBrowser } from '@/lib/supabase/client';
import { logout as authLogout } from '@/lib/auth-client';

export interface AppUser {
  id: string;
  email: string;
  name: string;
  role: string;
}

// Landing route per role after login. Kept in one place so the login page,
// Header dashboard shortcut, and post-login navigation stay in sync.
export const HOME_FOR_ROLE: Record<string, string> = {
  admin: '/admin',
  tutor: '/tutor',
  user: '/',
};

interface AuthContextType {
  user: AppUser | null;
  /**
   * Auth state is always settled — server hydration provides the initial user
   * via cookies and middleware. Kept for backward compatibility with consumers
   * that still gate on it.
   */
  ready: boolean;
  login: (userData: AppUser) => void;
  /**
   * Dev-only quick sign-in as one of the seeded roles. Sets a signed-free
   * cookie + localStorage so the server middleware and client both see the
   * demo user without hitting Supabase. Blocked in production unless
   * ALLOW_DEMO_USER=true is set (opt-in escape hatch for E2E).
   */
  loginAsDemo: (role: 'user' | 'tutor' | 'admin') => void;
  logout: () => Promise<void>;
}

// Safe default so `const { user } = useAuth()` never destructures null when
// something renders outside <AuthProvider> (Turbopack HMR blip, mistaken
// refactor). Login-like methods are no-ops in that context — the provider
// always overrides these with the real implementations.
const DEFAULT_AUTH_CTX: AuthContextType = {
  user: null,
  ready: false,
  login: () => {},
  loginAsDemo: () => {},
  logout: async () => {},
};

const AuthContext = createContext<AuthContextType>(DEFAULT_AUTH_CTX);

// Module-level state for use outside the React tree (e.g. apiCall helpers).
// Kept for backward compat with code that imports getCurrentUser/getAuthHeaders.
// New code should prefer reading from useAuth() inside React.
let _currentUser: AppUser | null = null;
let _serverJwt: string | null = null;

const SERVER_JWT_KEY = 'vielang_server_jwt';
const SERVER_JWT_USER_KEY = 'vielang_server_jwt_user';
const DEMO_USER_KEY = 'vielang_demo_user';

function readServerJwt(): string | null {
  if (_serverJwt) return _serverJwt;
  if (typeof window === 'undefined') return null;
  _serverJwt = localStorage.getItem(SERVER_JWT_KEY);
  return _serverJwt;
}

export function getCurrentUser() {
  return _currentUser;
}

export function setServerJwt(token: string, userData: any) {
  _serverJwt = token;
  localStorage.setItem(SERVER_JWT_KEY, token);
  localStorage.setItem(SERVER_JWT_USER_KEY, JSON.stringify(userData));
}

function clearServerJwt() {
  _serverJwt = null;
  if (typeof window === 'undefined') return;
  localStorage.removeItem(SERVER_JWT_KEY);
  localStorage.removeItem(SERVER_JWT_USER_KEY);
}

function writeDemoCookie(user: AppUser) {
  if (typeof document === 'undefined') return;
  // 8h lifetime, path=/ so middleware sees it on every route. Same-site Lax
  // keeps it stripped from cross-origin fetches, and NOT HttpOnly because the
  // client owns the demo flow. This is a dev/E2E convenience — never rely on
  // it for real users.
  const value = encodeURIComponent(JSON.stringify(user));
  document.cookie = `${DEMO_USER_KEY}=${value}; Path=/; Max-Age=${8 * 60 * 60}; SameSite=Lax`;
}

function clearDemoCookie() {
  if (typeof document === 'undefined') return;
  document.cookie = `${DEMO_USER_KEY}=; Path=/; Max-Age=0; SameSite=Lax`;
}

function clearDemoUser() {
  if (typeof window === 'undefined') return;
  try {
    localStorage.removeItem(DEMO_USER_KEY);
  } catch {
    /* ignore */
  }
  clearDemoCookie();
}

function readDemoUser(): AppUser | null {
  if (typeof window === 'undefined') return null;
  try {
    const raw = localStorage.getItem(DEMO_USER_KEY);
    return raw ? JSON.parse(raw) : null;
  } catch {
    return null;
  }
}

/**
 * Get auth headers for API calls. Supabase sessions live in HTTP-only cookies
 * (sent automatically by same-origin fetch). We only add headers for:
 *  - Demo users (E2E tests, dev quick-login) → X-Demo-User
 *  - Legacy JWT (kept for now; safe no-op once storage stops being written)
 */
export async function getAuthHeaders(): Promise<Record<string, string>> {
  const jwt = readServerJwt();
  if (jwt) return { Authorization: `Bearer ${jwt}` };
  const demo = readDemoUser();
  if (demo) return { 'X-Demo-User': JSON.stringify(_currentUser ?? demo) };
  return {};
}

export function AuthProvider({
  initialUser,
  children,
}: {
  initialUser?: AppUser | null;
  children: React.ReactNode;
}) {
  const router = useRouter();
  const navigate = useCallback((path: string) => router.push(path), [router]);

  // Server hydrated us with `initialUser` (from a cookie-backed session). It's
  // null for anonymous visits — the Header then immediately shows Login. No
  // Skeleton, no flash, no async wait.
  const [user, setUser] = useState<AppUser | null>(initialUser ?? null);
  // Mirror to module-level state for non-React consumers.
  if (typeof window !== 'undefined' && initialUser && !_currentUser) {
    _currentUser = initialUser;
  }

  const applyUser = useCallback(
    (userData: AppUser, opts?: { skipNavigate?: boolean }) => {
      _currentUser = userData;
      setUser(userData);
      if (opts?.skipNavigate) return;

      // Honor an explicit `?redirect=` query param FIRST — set by the Header
      // when the user clicked Login from somewhere like /cart. Falls back to
      // the older sessionStorage key for any legacy call sites still using it.
      // Either takes precedence over role-based defaults: returning the user
      // to where they were is more important than dumping admins on /admin
      // when they were trying to do something else.
      let stashed: string | null = null;
      if (typeof window !== 'undefined') {
        stashed =
          new URLSearchParams(window.location.search).get('redirect') ||
          sessionStorage.getItem('postLoginRedirect');
        if (stashed) sessionStorage.removeItem('postLoginRedirect');
      }
      if (stashed && stashed.startsWith('/') && !stashed.startsWith('//')) {
        navigate(stashed);
        return;
      }

      navigate(HOME_FOR_ROLE[userData?.role] || '/');
    },
    [navigate],
  );

  // After client-side Supabase sign-in, the SDK fires SIGNED_IN. We resolve
  // canonical app user via /api/users/me — the server uses the cookie to
  // identify us and reply with the row from `users`.
  // Returns null when the account is disabled — caller signs out + shows the
  // server-supplied reason.
  async function buildSessionUser(session: {
    user: { id: string; email?: string | null; user_metadata?: any };
  }): Promise<AppUser | { disabled: true; reason: string }> {
    let canonical: { id?: string; role?: string; name?: string } = {};
    try {
      const res = await fetch('/api/users/me');
      if (res.status === 403) {
        const body = await res.json().catch(() => ({}) as any);
        if (body?.code === 'ACCOUNT_DISABLED') {
          return { disabled: true, reason: body.reason || 'Account disabled' };
        }
      }
      if (res.ok) canonical = await res.json();
    } catch {
      /* ignore */
    }
    return {
      id: canonical.id || session.user.id,
      email: session.user.email || '',
      name: canonical.name || session.user.user_metadata?.name || session.user.email || '',
      role: canonical.role || 'user',
    };
  }

  // Hydrate from localStorage (legacy JWT or demo user) before paint, but
  // ONLY when the server didn't already give us a user. Keeps non-Supabase
  // sessions visible on the first frame without flashing the Login button.
  const useIsomorphicLayoutEffect = typeof window !== 'undefined' ? useLayoutEffect : useEffect;
  useIsomorphicLayoutEffect(() => {
    if (initialUser || user) return;
    const savedJwtUser =
      typeof window !== 'undefined' ? localStorage.getItem(SERVER_JWT_USER_KEY) : null;
    if (savedJwtUser && readServerJwt()) {
      try {
        applyUser(JSON.parse(savedJwtUser) as AppUser, { skipNavigate: true });
        return;
      } catch {
        /* ignore */
      }
    }
    const demo = readDemoUser();
    if (demo) applyUser(demo, { skipNavigate: true });
  }, [initialUser, user, applyUser]);

  // Listen for client-side Supabase auth changes (sign-in via email/password,
  // sign-out). The cookie set by signIn is also read by middleware on the next
  // navigation, keeping server + client in sync.
  //
  // SIGNED_IN fires on real login AND on session restore / token refresh.
  // Navigate when EITHER (a) caller was logged out (fresh login) OR (b) the
  // page is /login or /register (user submitted the form). Background token
  // refresh on other pages must not yank admin/tutor away.
  useEffect(() => {
    const { data: sub } = supabaseBrowser.auth.onAuthStateChange(async (event, session) => {
      if (event === 'SIGNED_IN' && session?.user) {
        const wasLoggedOut = _currentUser === null;
        const onLoginPage =
          typeof window !== 'undefined' && /^\/(login|register)/.test(window.location.pathname);
        const shouldNavigate = wasLoggedOut || onLoginPage;
        const u = await buildSessionUser(session);
        // Account disabled: force sign out + surface admin's reason. Returning
        // here keeps the failed session from being persisted into our state.
        if ('disabled' in u) {
          await supabaseBrowser.auth.signOut().catch(() => {});
          _currentUser = null;
          setUser(null);
          toast.error(u.reason, { duration: 8000, id: 'account-disabled' });
          if (typeof window !== 'undefined' && !/^\/login/.test(window.location.pathname)) {
            // Carry reason through so the persistent banner on /login can
            // re-render it after the sonner toast times out.
            navigate(`/login?disabled=1&reason=${encodeURIComponent(u.reason)}`);
          }
          return;
        }
        applyUser(u, shouldNavigate ? undefined : { skipNavigate: true });
      } else if (event === 'SIGNED_OUT') {
        _currentUser = null;
        setUser(null);
      }
    });
    return () => sub.subscription.unsubscribe();
  }, [applyUser]);

  const login = useCallback(
    (userData: AppUser) => {
      applyUser(userData);
    },
    [applyUser],
  );

  const loginAsDemo = useCallback(
    (role: 'user' | 'tutor' | 'admin') => {
      // Prod-safety: refuse unless explicitly opted-in via env. Matches the
      // server-side X-Demo-User + demo cookie gate in auth-server.ts and
      // get-current-user.ts.
      if (
        process.env.NODE_ENV === 'production' &&
        process.env.NEXT_PUBLIC_ALLOW_DEMO_USER !== 'true'
      ) {
        toast.error('Demo login disabled in production');
        return;
      }
      // Match seed rows in src/lib/seed-data.ts so downstream API queries that
      // look up by id return the seeded data instead of 404-ing.
      const demoIds: Record<
        'user' | 'tutor' | 'admin',
        { id: string; email: string; name: string }
      > = {
        admin: {
          id: '00000000-0000-4000-8000-000000000001',
          email: 'admin@vielang.com',
          name: 'VieLang Admin',
        },
        tutor: {
          id: '10000000-0000-4000-8000-000000000001',
          email: 'alice@vielang.com',
          name: 'Alice Nguyen',
        },
        user: {
          id: '20000000-0000-4000-8000-000000000001',
          email: 'student1@demo.vielang.com',
          name: 'Minh Le',
        },
      };
      const demoUser: AppUser = { ...demoIds[role], role };
      try {
        localStorage.setItem(DEMO_USER_KEY, JSON.stringify(demoUser));
      } catch {
        /* ignore */
      }
      writeDemoCookie(demoUser);
      applyUser(demoUser);
    },
    [applyUser],
  );

  const logout = useCallback(async () => {
    // Local cleanup runs FIRST and synchronously so the UI always reflects
    // the logged-out state, even if the server-side signOut hangs or rejects.
    // Earlier this all happened after `await authLogout()` — if Supabase's
    // /auth/v1/logout request stalled (rate limit, slow network), nothing
    // below it would run and the admin appeared stuck signed in.
    clearServerJwt();
    clearDemoUser();
    _currentUser = null;
    setUser(null);

    // Compute the post-logout target before kicking off the navigation.
    // Pages like /admin/*, /tutor/* that require auth would bounce back to
    // /login via middleware anyway, so skip the `?redirect=` there.
    let target = '/login';
    if (typeof window !== 'undefined') {
      const path = window.location.pathname;
      const skip =
        path === '/login' ||
        path === '/register' ||
        path.startsWith('/auth') ||
        path.startsWith('/admin') ||
        path.startsWith('/tutor');
      if (!skip) {
        target = `/login?redirect=${encodeURIComponent(path + window.location.search)}`;
      }
    }

    // Best-effort server signOut with a hard timeout so a hanging network
    // call can't block UI logout. The .catch swallows rejections; the race
    // against setTimeout handles the "promise never resolves" case.
    const signOutPromise = authLogout().catch(() => {
      /* ignore */
    });
    const timeoutPromise = new Promise<void>((resolve) => setTimeout(resolve, 3_000));
    await Promise.race([signOutPromise, timeoutPromise]);

    // Hard navigation (full page reload) guarantees the entire React tree,
    // any in-memory Supabase client cache, cart context, etc. are reset.
    // Soft router.push could leave a server-rendered admin shell visible
    // for a flicker, or fail entirely if a transition error boundary
    // intercepts the navigation.
    if (typeof window !== 'undefined') {
      window.location.replace(target);
    } else {
      navigate(target);
    }
  }, [navigate]);

  return (
    <AuthContext.Provider value={{ user, ready: true, login, loginAsDemo, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => useContext(AuthContext);
