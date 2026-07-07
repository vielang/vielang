import { NextResponse } from 'next/server';
import * as Sentry from '@sentry/nextjs';
import { logger } from './logger';

/**
 * Log internal error server-side, return generic message to client.
 * Prevents leaking DB schema, stack traces, secrets via API error responses.
 */
/**
 * Coerce anything thrown into a readable string. Native `Error` has
 * `.message`; Supabase/PostgREST errors are plain objects with `.message`,
 * `.code`, `.details`, `.hint` and were previously being run through
 * `String(err)` → "[object Object]" which hid the real failure both in
 * server logs and (in dev) in the client debug field.
 */
function describeError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;
  if (err && typeof err === 'object') {
    const e = err as { message?: string; code?: string; details?: string; hint?: string };
    const parts: string[] = [];
    if (e.message) parts.push(e.message);
    if (e.code) parts.push(`code=${e.code}`);
    if (e.details) parts.push(`details=${e.details}`);
    if (e.hint) parts.push(`hint=${e.hint}`);
    if (parts.length > 0) return parts.join(' | ');
    try {
      return JSON.stringify(err);
    } catch {
      /* circular — fall through */
    }
  }
  return String(err);
}

export function apiError(err: unknown, scope?: string) {
  const message = describeError(err);
  const effectiveScope = scope ?? 'api';
  logger.error({ scope: effectiveScope, err }, message);
  // Sentry is a no-op when the DSN env var is unset (see sentry.*.config.ts)
  // so this is safe to call from any environment.
  Sentry.captureException(err, { tags: { scope: effectiveScope } });
  return NextResponse.json(
    {
      error: 'Internal server error',
      ...(process.env.NODE_ENV !== 'production' ? { debug: message } : {}),
    },
    { status: 500 },
  );
}
