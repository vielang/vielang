// Boot-time environment validation. Imported side-effectfully from the
// server-only auth helpers so any prod deploy missing a required secret
// blows up on the FIRST request (or during `next build` prerender), not
// silently 500s later. Never imported by client-side code — anything with
// NEXT_PUBLIC_ prefix is checked client-side by the code that consumes it.
//
// Not thrown in dev: local devs run against docker-compose LiveKit + optional
// Sentry / Resend / Upstash, and forcing every one of those to be set would
// break the fast-start-from-fresh-clone flow. Prod deploys must set them all.

import { logger } from './logger';

interface Requirement {
  name: string;
  purpose: string;
}

// Server-only secrets that MUST be present in production for the app to
// function safely. Empty string counts as "not set" so a `.env` with the key
// declared but blank is treated as missing (common Vercel deploy footgun).
const REQUIRED_PROD: Requirement[] = [
  { name: 'SUPABASE_URL', purpose: 'Supabase Postgres + Auth API endpoint' },
  { name: 'SUPABASE_SERVICE_ROLE_KEY', purpose: 'server-side DB writes (auth-bypass)' },
  { name: 'NEXT_PUBLIC_SUPABASE_URL', purpose: 'browser-side Supabase client' },
  { name: 'NEXT_PUBLIC_SUPABASE_ANON_KEY', purpose: 'browser-side Supabase client' },
  { name: 'LIVEKIT_API_KEY', purpose: 'LiveKit room token minting' },
  { name: 'LIVEKIT_API_SECRET', purpose: 'LiveKit room token signing' },
  { name: 'NEXT_PUBLIC_LIVEKIT_WS_URL', purpose: 'browser LiveKit connection' },
  { name: 'NEXT_PUBLIC_SITE_URL', purpose: 'absolute URLs in emails, OAuth callbacks' },
];

// Server-only secrets whose absence degrades the platform but doesn't break
// it (emails go to notification_log only, rate-limit uses in-memory, Sentry
// silently no-ops). Warn once at boot so operators notice on inspect.
const RECOMMENDED_PROD: Requirement[] = [
  {
    name: 'CRON_SECRET',
    purpose: 'authorization on /api/cron/* endpoints (unset → cron endpoints return 401)',
  },
  { name: 'RESEND_API_KEY', purpose: 'transactional email delivery' },
  { name: 'UPSTASH_REDIS_REST_URL', purpose: 'durable rate limits (falls back to in-memory)' },
  { name: 'UPSTASH_REDIS_REST_TOKEN', purpose: 'durable rate limits (falls back to in-memory)' },
  { name: 'SENTRY_DSN', purpose: 'server-side error reporting' },
  { name: 'NEXT_PUBLIC_SENTRY_DSN', purpose: 'browser error reporting' },
];

function isMissing(name: string): boolean {
  const v = process.env[name];
  return v === undefined || v === '';
}

let _validated = false;

/**
 * Validate that required env vars are present. Idempotent — safe to call from
 * multiple modules; only runs the check the first time.
 *
 * Behavior by environment:
 *   • production: throws if any REQUIRED_PROD var is missing (fails boot).
 *   • non-production: warns for each missing REQUIRED_PROD, does not throw
 *     (local dev may legitimately run without LiveKit, Sentry, etc.).
 *
 * Called side-effectfully from `src/lib/auth-server.ts` so it runs once on
 * the first request to any authenticated endpoint. Safe for cold-start.
 */
export function validateEnv(): void {
  if (_validated) return;
  _validated = true;

  const isProd = process.env.NODE_ENV === 'production';
  const missingRequired = REQUIRED_PROD.filter((r) => isMissing(r.name));
  const missingRecommended = RECOMMENDED_PROD.filter((r) => isMissing(r.name));

  if (missingRequired.length > 0) {
    const lines = missingRequired.map((r) => `  - ${r.name} (${r.purpose})`).join('\n');
    const message = `Missing required environment variables:\n${lines}`;
    if (isProd) {
      // In prod this is a fatal misconfiguration — throw so the first request
      // fails loudly and monitoring lights up, rather than silently
      // returning 500 on every DB / LiveKit / cron call.
      logger.fatal({ scope: 'env' }, message);
      throw new Error(message);
    }
    logger.warn({ scope: 'env' }, message);
  }

  if (missingRecommended.length > 0) {
    const lines = missingRecommended.map((r) => `  - ${r.name} (${r.purpose})`).join('\n');
    logger.warn(
      { scope: 'env' },
      `Optional env vars unset — feature will degrade gracefully:\n${lines}`,
    );
  }

  if (missingRequired.length === 0 && missingRecommended.length === 0) {
    logger.info({ scope: 'env' }, 'env validation passed');
  }
}
