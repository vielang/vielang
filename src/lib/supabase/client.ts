'use client';

import { createBrowserClient } from '@supabase/ssr';
import type { SupabaseClient } from '@supabase/supabase-js';

// Lazy singleton: createBrowserClient throws if URL/key are empty, which would
// break Next.js build-time prerender before env is guaranteed to be loaded.
let _client: SupabaseClient | null = null;

function getClient(): SupabaseClient {
  if (_client) return _client;
  const url = process.env.NEXT_PUBLIC_SUPABASE_URL || '';
  const key = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY || '';
  if (typeof window !== 'undefined' && (!url || !key)) {
    console.error(
      'Missing NEXT_PUBLIC_SUPABASE_URL or NEXT_PUBLIC_SUPABASE_ANON_KEY. Auth will not work.',
    );
  }
  _client = createBrowserClient(url, key);
  return _client;
}

/**
 * Browser-side Supabase client backed by HTTP-only cookies (via @supabase/ssr).
 * Sessions are shared with the server through cookies — no localStorage handoff
 * needed. Read on client + server stay in sync because both layers parse the
 * same cookie source.
 *
 * Replaces the older localStorage-based createClient in `lib/supabase-client.ts`.
 */
export const supabaseBrowser = new Proxy({} as SupabaseClient, {
  get(_t, prop) {
    // Proxy dispatches to whatever method is accessed at runtime; a fully
    // typed indexer would require conditional types over every SupabaseClient
    // method, so we cast to a plain index type instead of `any`.
    return (getClient() as unknown as Record<PropertyKey, unknown>)[prop];
  },
});
