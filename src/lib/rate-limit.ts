import 'server-only';
import { NextResponse, type NextRequest } from 'next/server';
import { Ratelimit } from '@upstash/ratelimit';
import { Redis } from '@upstash/redis';
import { childLogger } from './logger';

/**
 * Rate limiting with graceful degradation.
 *
 *  - If UPSTASH_REDIS_REST_URL + UPSTASH_REDIS_REST_TOKEN are set → real
 *    distributed rate limit (production path). Same Redis works across all
 *    serverless instances → no bypass by scaling out.
 *  - Otherwise → in-memory sliding window (dev + preview). Not durable across
 *    restarts and per-instance, but gives realistic UX + tests behaviour
 *    without provisioning Redis first.
 *  - `RATE_LIMIT_DISABLED=true` → all limiters become no-ops. Used by CI + E2E
 *    where the run needs to blast the API without tripping guards.
 *
 * The identifier is derived per-request:
 *   1. `x-forwarded-for` (Vercel adds this) — the client IP.
 *   2. Falls back to `x-real-ip` or 'unknown'.
 * When a limiter targets an authenticated user, the caller can pass a
 * user-scoped identifier (e.g. `user:${authUser.id}`) instead.
 */

const log = childLogger('rate-limit');

const DISABLED = process.env.RATE_LIMIT_DISABLED === 'true';

interface LimiterConfig {
  /** Requests allowed inside the window. */
  requests: number;
  /** Sliding window duration for @upstash/ratelimit. */
  window: `${number} ${'s' | 'm' | 'h' | 'd'}`;
  /** Stable prefix keyed against Redis (or the in-memory map). */
  prefix: string;
}

type LimiterName =
  | 'session-create'
  | 'session-reserve'
  | 'session-mutate'
  | 'review-create'
  | 'livekit-token'
  | 'livekit-moderate'
  | 'chat-message'
  | 'auth-callback'
  | 'profile-mutate'
  | 'availability-mutate'
  | 'material-mutate';

const CONFIGS: Record<LimiterName, LimiterConfig> = {
  'session-create': { requests: 5, window: '1 m', prefix: 'rl:session-create' },
  'session-reserve': { requests: 15, window: '1 m', prefix: 'rl:session-reserve' },
  // Cancel / reschedule / confirm — legitimate students may retry once or twice
  // during a network hiccup; 10/min leaves plenty of headroom.
  'session-mutate': { requests: 10, window: '1 m', prefix: 'rl:session-mutate' },
  'review-create': { requests: 3, window: '1 m', prefix: 'rl:review-create' },
  'livekit-token': { requests: 10, window: '1 m', prefix: 'rl:livekit-token' },
  // Hosts may fire several mute/remove actions in quick succession when a
  // troll floods a group room, so we allow a burstier ceiling than a normal
  // write endpoint.
  'livekit-moderate': { requests: 30, window: '1 m', prefix: 'rl:livekit-moderate' },
  // Chat is chatty by definition. 60/min per user is ~1 msg/sec — enough for
  // a heated exchange, low enough that a bot in a group room can't drown
  // everyone else out.
  'chat-message': { requests: 60, window: '1 m', prefix: 'rl:chat-message' },
  'auth-callback': { requests: 20, window: '1 m', prefix: 'rl:auth-callback' },
  // Tutor edits their own profile — bio/rate/etc. Bursts of a few saves are
  // common when someone iterates on wording; 15/min covers that comfortably.
  'profile-mutate': { requests: 15, window: '1 m', prefix: 'rl:profile-mutate' },
  // Availability edits are chunkier (bulk replace + individual slot adds).
  // Tutors setting up their week may fire 20+ writes in a minute.
  'availability-mutate': { requests: 30, window: '1 m', prefix: 'rl:availability-mutate' },
  // Course-materials add / delete. Uploads are one-at-a-time but drag-drop
  // batches can arrive together; 20/min is the ceiling.
  'material-mutate': { requests: 20, window: '1 m', prefix: 'rl:material-mutate' },
};

// Single Redis client — Upstash uses HTTP transport, no persistent connection
// to leak. Lazy-init so a missing env doesn't crash cold-start.
let _redis: Redis | null = null;
function getRedis(): Redis | null {
  if (_redis) return _redis;
  const url = process.env.UPSTASH_REDIS_REST_URL;
  const token = process.env.UPSTASH_REDIS_REST_TOKEN;
  if (!url || !token) return null;
  _redis = new Redis({ url, token });
  return _redis;
}

// In-memory fallback: single sliding window per key. Not durable, not shared
// across instances. Good enough for local dev and single-region preview.
const memBuckets = new Map<string, number[]>();

function limitInMemory(key: string, requests: number, windowMs: number, now: number) {
  const arr = memBuckets.get(key) || [];
  const cutoff = now - windowMs;
  const recent = arr.filter((t) => t > cutoff);
  const allowed = recent.length < requests;
  if (allowed) recent.push(now);
  memBuckets.set(key, recent);
  const remaining = Math.max(0, requests - recent.length);
  // Reset time = when the oldest request in the window exits.
  const reset = recent.length > 0 ? recent[0] + windowMs : now + windowMs;
  return { success: allowed, remaining, reset };
}

function windowToMs(window: LimiterConfig['window']): number {
  const [n, unit] = window.split(' ') as [string, 's' | 'm' | 'h' | 'd'];
  const mult = { s: 1_000, m: 60_000, h: 3_600_000, d: 86_400_000 }[unit];
  return Number(n) * mult;
}

// Upstash Ratelimit instances — one per config, memoized so we don't recreate
// the sliding-window state on every request.
const _cache = new Map<LimiterName, Ratelimit>();
function getLimiter(name: LimiterName): Ratelimit | null {
  if (_cache.has(name)) return _cache.get(name)!;
  const redis = getRedis();
  if (!redis) return null;
  const cfg = CONFIGS[name];
  const rl = new Ratelimit({
    redis,
    limiter: Ratelimit.slidingWindow(cfg.requests, cfg.window),
    prefix: cfg.prefix,
    analytics: false,
  });
  _cache.set(name, rl);
  return rl;
}

export interface RateLimitOk {
  ok: true;
  remaining: number;
  reset: number;
}
export interface RateLimitBlocked {
  ok: false;
  response: NextResponse;
}
export type RateLimitResult = RateLimitOk | RateLimitBlocked;

/**
 * Consume a rate-limit token for `name`. `identifier` should be a stable per-
 * user or per-IP string. Returns `{ ok: true }` when the request is allowed,
 * or `{ ok: false, response }` — the caller should short-circuit and return
 * `response` immediately.
 */
export async function rateLimit(name: LimiterName, identifier: string): Promise<RateLimitResult> {
  if (DISABLED) return { ok: true, remaining: Number.POSITIVE_INFINITY, reset: 0 };

  const cfg = CONFIGS[name];
  const key = `${cfg.prefix}:${identifier}`;

  const limiter = getLimiter(name);
  let success: boolean;
  let remaining: number;
  let reset: number;

  if (limiter) {
    const res = await limiter.limit(identifier);
    success = res.success;
    remaining = res.remaining;
    reset = res.reset;
  } else {
    const res = limitInMemory(key, cfg.requests, windowToMs(cfg.window), Date.now());
    success = res.success;
    remaining = res.remaining;
    reset = res.reset;
  }

  if (!success) {
    const retryAfter = Math.max(1, Math.ceil((reset - Date.now()) / 1000));
    log.warn({ name, identifier, retryAfter }, 'rate limit exceeded');
    const response = NextResponse.json(
      { error: 'rate_limited', retryAfter },
      {
        status: 429,
        headers: {
          'Retry-After': String(retryAfter),
          'X-RateLimit-Limit': String(cfg.requests),
          'X-RateLimit-Remaining': '0',
          'X-RateLimit-Reset': String(reset),
        },
      },
    );
    return { ok: false, response };
  }

  return { ok: true, remaining, reset };
}

/**
 * Extract a stable rate-limit identifier from a request. Prefer authenticated
 * user id (passed by the caller); fall back to `x-forwarded-for` / `x-real-ip`.
 * Never returns an empty string — a null-ish IP falls back to 'unknown' so all
 * anonymous requests share a single bucket rather than bypassing the limiter.
 */
export function requestIdentifier(req: NextRequest, userId?: string): string {
  if (userId) return `user:${userId}`;
  const forwarded = req.headers.get('x-forwarded-for');
  const realIp = req.headers.get('x-real-ip');
  const ip = (forwarded?.split(',')[0].trim() || realIp || '').trim();
  return `ip:${ip || 'unknown'}`;
}
