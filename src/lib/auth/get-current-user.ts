import 'server-only';
import { cookies } from 'next/headers';
import { createSupabaseServer } from '@/lib/supabase/server';
import { supabase as supabaseAdmin } from '@/lib/supabase';
import type { CurrentUser } from './types';

export type { CurrentUser };

// Test bypass: a same-origin cookie set by the E2E harness (Playwright
// `loginAs`) so server-side route guards can find the demo user without a
// real Supabase session. Gated by ALLOW_DEMO_USER so prod can't be tricked
// into honoring a forged cookie.
const DEMO_COOKIE = 'vielang_demo_user';
async function readDemoCookieUser(): Promise<CurrentUser | null> {
  if (process.env.NODE_ENV === 'production' && process.env.ALLOW_DEMO_USER !== 'true') return null;
  try {
    const store = await cookies();
    const raw = store.get(DEMO_COOKIE)?.value;
    if (!raw) return null;
    const parsed = JSON.parse(decodeURIComponent(raw));
    if (!parsed?.id || !parsed?.role) return null;
    return {
      id: parsed.id,
      email: parsed.email || '',
      name: parsed.name || '',
      role: parsed.role,
    };
  } catch {
    return null;
  }
}

/**
 * Resolve the currently signed-in user on the server.
 *
 * Reads the Supabase session cookie (via @supabase/ssr), then resolves the
 * canonical app user from the `users` table by email. Email is used (not
 * Supabase UID) because the app keys ownership off legacy stable ids
 * ("owner-north", "admin-1") that match `courses.ownerId` etc.
 *
 * Returns null if no session OR the email isn't mapped in `users`.
 *
 * Use in Server Components, Server Actions, and Route Handlers when you need
 * authenticated user info on the server side — replaces client-side
 * `useAuth()` reads for SSR.
 */
export async function getCurrentUser(): Promise<CurrentUser | null> {
  // Demo bypass first (dev / E2E only) — avoids two Supabase round-trips when
  // tests are running. Cookie is JSON-encoded { id, role, email?, name? }.
  const demo = await readDemoCookieUser();
  if (demo) return demo;

  // Graceful degrade: no Supabase env → treat as logged-out. Lets `npm run dev`
  // render the app before P1 provisions the new Supabase project.
  if (!process.env.NEXT_PUBLIC_SUPABASE_URL && !process.env.SUPABASE_URL) return null;

  const supabase = await createSupabaseServer();
  const { data: sessionData } = await supabase.auth.getUser();
  const sbUser = sessionData.user;
  if (!sbUser) return null;

  const email = sbUser.email ?? '';
  if (!email) return null;

  // Look up canonical row by email. Uses the service-role admin client so RLS
  // doesn't block reads even if policies tighten later.
  const { data: row } = await supabaseAdmin
    .from('users')
    .select('id, role, name, enabled')
    .eq('email', email)
    .maybeSingle();

  // Account explicitly disabled by admin → treat as logged-out at SSR. The
  // client AuthContext's call to /api/users/me will also 403, force-signs the
  // Supabase session out, and redirects to /login?disabled=1 with the reason.
  // Returning null here prevents the Header / dashboards from flashing the
  // "logged in" state on first paint before the client-side sign-out fires.
  if (row && row.enabled === false) return null;

  return {
    id: row?.id || sbUser.id,
    email,
    name: row?.name || (sbUser.user_metadata?.name as string | undefined) || email,
    role: (row?.role as CurrentUser['role']) || 'user',
  };
}
