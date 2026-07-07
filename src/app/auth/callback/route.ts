import { type NextRequest, NextResponse } from 'next/server';
import { createSupabaseServer } from '@/lib/supabase/server';
import { supabase as supabaseAdmin, upsertUserFromAuth } from '@/lib/supabase';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

/**
 * OAuth PKCE callback for @supabase/ssr.
 *
 * Google (and any future Supabase-native OAuth provider) redirects here with
 * `?code=<auth_code>`. We exchange the code server-side via
 * `exchangeCodeForSession`, which sets the HTTP-only auth cookies. Without
 * this route, the browser sometimes succeeds at the exchange via
 * `detectSessionInUrl` and sometimes loses the race (depending on when
 * Supabase JS initializes vs. when React hydrates).
 *
 * On success: mirror the auth user into public.users (role defaults to 'user')
 * and redirect to `?next=<path>` (default '/').
 * On failure: bounce to /login?error=... so the UI can surface it.
 */
export async function GET(req: NextRequest) {
  // Rate-limit by IP — pre-auth, so we cap what a hostile client can throw at
  // the OAuth exchange. 20/min/IP is high enough for legitimate retries (user
  // hits refresh, opens two tabs) but shuts down credential-spray attempts.
  const rl = await rateLimit('auth-callback', requestIdentifier(req));
  if (!rl.ok) return rl.response;

  const { searchParams, origin } = new URL(req.url);
  const code = searchParams.get('code');
  const next = searchParams.get('next') || '/';
  const errorParam = searchParams.get('error');
  const errorDescription = searchParams.get('error_description');

  if (errorParam) {
    const msg = encodeURIComponent(errorDescription || errorParam);
    return NextResponse.redirect(`${origin}/login?error=${msg}`);
  }
  if (!code) {
    return NextResponse.redirect(`${origin}/login?error=missing_code`);
  }

  const supabase = await createSupabaseServer();
  const { data, error } = await supabase.auth.exchangeCodeForSession(code);
  if (error) {
    return NextResponse.redirect(`${origin}/login?error=${encodeURIComponent(error.message)}`);
  }

  const u = data.user;
  if (u?.email) {
    try {
      await upsertUserFromAuth({
        supabase_uid: u.id,
        email: u.email,
        name:
          (u.user_metadata?.name as string) ||
          (u.user_metadata?.full_name as string) ||
          u.email.split('@')[0],
        avatar: (u.user_metadata?.avatar_url as string | undefined) ?? null,
      });
    } catch {
      // Best-effort — auth succeeded, we can retry the mirror on next request.
    }

    // If admin disabled this account, kill the session immediately.
    const { data: row } = await supabaseAdmin
      .from('users')
      .select('enabled, disabled_reason')
      .eq('email', u.email)
      .maybeSingle();
    if (row && row.enabled === false) {
      await supabase.auth.signOut().catch(() => {});
      const reason = encodeURIComponent(row.disabled_reason || '');
      return NextResponse.redirect(
        `${origin}/login?disabled=1${reason ? `&reason=${reason}` : ''}`,
      );
    }
  }

  return NextResponse.redirect(`${origin}${next}`);
}
