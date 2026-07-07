import { createServerClient } from '@supabase/ssr';
import { cookies } from 'next/headers';
import type { SupabaseClient } from '@supabase/supabase-js';

/**
 * Server-side Supabase client (Server Components, Route Handlers, Server Actions).
 *
 * Reads & writes the session cookie via next/headers. Use this anywhere on the
 * server where you need the authenticated user's session — it returns the same
 * user the client sees, kept in sync via the middleware refresh.
 *
 * NOTE: in some contexts (Server Components rendered as part of the response
 * stream) cookie writes are no-ops; Next.js logs a warning. That's expected —
 * `middleware.ts` handles cookie writes for those cases. We try/catch the set
 * calls to suppress the warning when used inside RSC.
 */
export async function createSupabaseServer(): Promise<SupabaseClient> {
  const cookieStore = await cookies();

  const url = process.env.NEXT_PUBLIC_SUPABASE_URL || process.env.SUPABASE_URL || '';
  const key = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || process.env.SUPABASE_ANON_KEY || '';

  return createServerClient(url, key, {
    cookies: {
      getAll() {
        return cookieStore.getAll();
      },
      setAll(cookiesToSet) {
        try {
          cookiesToSet.forEach(({ name, value, options }) => {
            cookieStore.set(name, value, options);
          });
        } catch {
          // Server Components cannot set cookies — middleware handles refresh.
        }
      },
    },
  });
}
