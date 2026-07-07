import { NextResponse } from 'next/server';
import { supabase } from '@/lib/supabase';
import { childLogger } from '@/lib/logger';

export const dynamic = 'force-dynamic';
export const runtime = 'nodejs';

const log = childLogger('ready');

/**
 * Readiness probe — succeeds only when all critical downstream deps respond.
 *
 * Checked:
 *   - Supabase (cheap SELECT count on a small table)
 *
 * Returns 200 with `{ deps: {...} }` when everything's healthy, 503 with the
 * same shape when any dep is degraded. Response is intentionally verbose so
 * on-call can read the first failing hop from the alert body.
 *
 * A separate check for LiveKit lives client-side (the /session room hits the
 * signaling URL directly). Adding a server-side ping would require a token
 * mint per probe — too costly for a 30s-interval check.
 */
export async function GET() {
  const deps: Record<string, { ok: boolean; latencyMs?: number; error?: string }> = {};

  deps.supabase = await probeSupabase();

  const allOk = Object.values(deps).every((d) => d.ok);
  const body = {
    status: allOk ? 'ok' : 'degraded',
    service: 'vielang-web',
    timestamp: new Date().toISOString(),
    deps,
  };

  if (!allOk) log.warn({ deps }, 'readiness probe degraded');
  return NextResponse.json(body, { status: allOk ? 200 : 503 });
}

async function probeSupabase(): Promise<{ ok: boolean; latencyMs?: number; error?: string }> {
  const start = performance.now();
  try {
    // Cheapest possible sanity check — head+count against a small table. If
    // the DB is down or the service_role key is wrong this errors immediately.
    const { error } = await supabase
      .from('users')
      .select('*', { count: 'exact', head: true })
      .limit(1);
    const latencyMs = Math.round(performance.now() - start);
    if (error) return { ok: false, latencyMs, error: error.message };
    return { ok: true, latencyMs };
  } catch (err) {
    return {
      ok: false,
      latencyMs: Math.round(performance.now() - start),
      error: err instanceof Error ? err.message : String(err),
    };
  }
}
