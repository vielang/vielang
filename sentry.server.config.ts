import * as Sentry from '@sentry/nextjs';

/**
 * Server-side (Node runtime) Sentry — powers /api/* route handlers and the
 * server rendered pages. Uses the SAME DSN as the client build; Sentry
 * distinguishes by SDK, not project.
 */
const dsn = process.env.SENTRY_DSN || process.env.NEXT_PUBLIC_SENTRY_DSN;
if (dsn) {
  Sentry.init({
    dsn,
    environment: process.env.VERCEL_ENV || process.env.NODE_ENV,
    tracesSampleRate: process.env.NODE_ENV === 'production' ? 0.1 : 1.0,
    // Sentry has a built-in `sendDefaultPii: false` (the default) but the
    // Next.js SDK still forwards cookies + auth headers on request objects.
    // Scrub them explicitly so a Supabase session cookie or a bearer token
    // never lands in an issue payload.
    sendDefaultPii: false,
    // The server has no user session — set once at capture time via
    // Sentry.setUser() inside authenticated route handlers if per-user
    // grouping is useful.
    beforeSend(event, hint) {
      // 401/403/404 are user errors, not app failures — the API returns them
      // by design. Drop so they don't dominate the inbox.
      const err = hint.originalException as { status?: number } | undefined;
      if (err?.status && [401, 403, 404, 429].includes(err.status)) return null;

      // Scrub session cookies + auth headers in case they leaked past the
      // SDK's default filters.
      if (event.request) {
        if (event.request.cookies) event.request.cookies = { redacted: '[Filtered]' };
        if (event.request.headers) {
          const h = event.request.headers as Record<string, string>;
          if (h.authorization) h.authorization = '[Filtered]';
          if (h.cookie) h.cookie = '[Filtered]';
          if (h['x-demo-user']) h['x-demo-user'] = '[Filtered]';
        }
      }
      // Only send `id` for the user — never email/username/ip_address.
      if (event.user) {
        event.user = { id: event.user.id };
      }
      return event;
    },
  });
}
