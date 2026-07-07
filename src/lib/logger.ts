import 'server-only';
import pino, { type Logger } from 'pino';

/**
 * Server-side structured logger. Emits JSON in production (single-line, ready
 * for Vercel log drains → Axiom/Datadog/Grafana Loki), pretty-prints in dev.
 *
 * Design choices:
 *  - Singleton via a module-level constant. Route handlers grab a child via
 *    `logger.child({ scope: 'sessions' })` for tag-based filtering.
 *  - No transports in production — writing JSON to stdout is the cheapest,
 *    most reliable path on serverless. Log-shipping happens at the platform
 *    edge, not inside the process.
 *  - Level defaults to `info` in prod, `debug` in dev. Override with
 *    `LOG_LEVEL=trace|debug|info|warn|error|fatal` for one-off deep dives.
 *  - Redacts `authorization` / `cookie` headers + common secret-shaped fields
 *    so a stray req.headers or config dump can't leak credentials into logs.
 */

const isProd = process.env.NODE_ENV === 'production';
const level = process.env.LOG_LEVEL || (isProd ? 'info' : 'debug');

// pino-pretty is a devDependency — only require it when dev mode actually uses
// it. In prod we never touch the transport slot so the module isn't loaded.
const transport =
  !isProd && process.env.NODE_ENV !== 'test'
    ? {
        target: 'pino-pretty',
        options: {
          colorize: true,
          translateTime: 'HH:MM:ss.l',
          ignore: 'pid,hostname',
          singleLine: false,
        },
      }
    : undefined;

export const logger: Logger = pino({
  level,
  base: {
    // `env` + `service` make it trivial to filter across environments in
    // aggregated dashboards. Vercel exposes VERCEL_ENV; fall back to NODE_ENV.
    env: process.env.VERCEL_ENV || process.env.NODE_ENV || 'unknown',
    service: 'vielang-web',
  },
  redact: {
    paths: [
      'req.headers.authorization',
      'req.headers.cookie',
      'headers.authorization',
      'headers.cookie',
      '*.password',
      '*.token',
      '*.secret',
      '*.apiKey',
      '*.api_key',
      'user.password',
    ],
    censor: '[redacted]',
  },
  ...(transport ? { transport } : {}),
});

/**
 * Convenience helper — attach a stable `scope` to every log line from a given
 * module/route. Prefer this over `logger` directly so log filtering by scope
 * stays consistent across the codebase.
 *
 * ```ts
 * const log = childLogger('sessions');
 * log.info({ sessionId }, 'booking accepted');
 * ```
 */
export function childLogger(scope: string): Logger {
  return logger.child({ scope });
}
