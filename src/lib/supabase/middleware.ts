import { createServerClient } from '@supabase/ssr';
import { NextResponse, type NextRequest } from 'next/server';

/**
 * Refresh the Supabase auth cookies on every matching request. Must be called
 * from the root `middleware.ts`. Without this, expired access tokens never get
 * refreshed and the server sees stale sessions until the client manually
 * re-authenticates.
 *
 * Returns the NextResponse with updated cookies — caller should return it.
 */
export async function updateSession(request: NextRequest): Promise<{
  response: NextResponse;
  user: { id: string; email: string | null } | null;
}> {
  let response = NextResponse.next({ request });

  const url = process.env.NEXT_PUBLIC_SUPABASE_URL || process.env.SUPABASE_URL || '';
  const key = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || process.env.SUPABASE_ANON_KEY || '';

  // Graceful degrade: when Supabase env is not configured (fresh clone / pre-P1),
  // skip auth work entirely so pages still render locally. The protected-route
  // gate in middleware.ts will bounce authenticated-only routes to /login.
  if (!url || !key) return { response, user: null };

  const supabase = createServerClient(url, key, {
    cookies: {
      getAll() {
        return request.cookies.getAll();
      },
      setAll(cookiesToSet) {
        cookiesToSet.forEach(({ name, value }) => request.cookies.set(name, value));
        response = NextResponse.next({ request });
        cookiesToSet.forEach(({ name, value, options }) => {
          response.cookies.set(name, value, options);
        });
      },
    },
  });

  // IMPORTANT: don't read or write to the response between createServerClient
  // and getUser — doing so risks closing the session early.
  const { data } = await supabase.auth.getUser();
  const user = data.user ? { id: data.user.id, email: data.user.email ?? null } : null;

  return { response, user };
}
