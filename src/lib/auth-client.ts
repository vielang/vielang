import { supabaseBrowser as supabaseClient } from '@/lib/supabase/client';

// ---- Google Auth ----
export async function loginWithGoogle() {
  // With @supabase/ssr PKCE flow, the code_verifier lives in HTTP-only
  // cookies. The browser cannot exchange `?code=` itself reliably — route
  // through our server callback which does the exchange + sets the cookie
  // + upserts public.users, then redirects to `next`.
  const params = new URLSearchParams(window.location.search);
  const fromQuery = params.get('redirect');
  const fromPath = window.location.pathname.startsWith('/login') ? null : window.location.pathname;
  const next = encodeURIComponent(fromQuery || fromPath || '/');
  const { data, error } = await supabaseClient.auth.signInWithOAuth({
    provider: 'google',
    options: { redirectTo: `${window.location.origin}/auth/callback?next=${next}` },
  });
  if (error) throw error;
  return data;
}

// ---- Email/Password Auth ----
// The users row is mirrored by /auth/callback for OAuth sign-ins. For
// email sign-up the callback isn't hit; the server upserts on the first
// /api/users/me call instead.
export async function registerWithEmail(email: string, password: string, name: string) {
  const { data, error } = await supabaseClient.auth.signUp({
    email,
    password,
    options: { data: { name, provider: 'email' } },
  });
  if (error) throw error;
  return data.user;
}

export async function loginWithEmail(email: string, password: string) {
  const { data, error } = await supabaseClient.auth.signInWithPassword({ email, password });
  if (error) throw error;
  return data.user;
}

export async function resetPassword(email: string) {
  // Supabase sends a recovery email whose link points at `redirectTo`. That
  // page (/auth/reset-password) picks up the auth event with a temporary
  // session and lets the user submit a new password. Without redirectTo the
  // link falls back to whatever's configured in the Supabase dashboard, which
  // is fine for dev but leaves a footgun on fresh deploys.
  const redirectTo = `${window.location.origin}/auth/reset-password`;
  const { error } = await supabaseClient.auth.resetPasswordForEmail(email, { redirectTo });
  if (error) throw error;
}

export async function logout() {
  await supabaseClient.auth.signOut();
}
