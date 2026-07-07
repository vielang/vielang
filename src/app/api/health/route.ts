import { NextResponse } from 'next/server';

export const dynamic = 'force-dynamic';
export const runtime = 'nodejs';

/**
 * Liveness probe — succeeds if the process is up and the runtime is happy.
 * Does NOT touch downstream deps. Used by:
 *  - Vercel platform to detect a hung deploy
 *  - Uptime monitors that only need "is the process alive?"
 *
 * For "can we serve real traffic?" use /api/ready which probes Supabase.
 */
export async function GET() {
  return NextResponse.json({
    status: 'ok',
    service: 'vielang-web',
    env: process.env.VERCEL_ENV || process.env.NODE_ENV || 'unknown',
    uptime: Math.round(process.uptime()),
    timestamp: new Date().toISOString(),
  });
}
