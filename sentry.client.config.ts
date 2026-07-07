import * as Sentry from '@sentry/nextjs';

/**
 * Browser-side Sentry. Runs in the client bundle — DSN is public (safe to
 * expose, same guarantees as any other analytics SDK). Missing DSN = no-op
 * (the SDK never phones home) so preview/dev never surface fake errors.
 */
const dsn = process.env.NEXT_PUBLIC_SENTRY_DSN;
if (dsn) {
  Sentry.init({
    dsn,
    environment: process.env.NEXT_PUBLIC_VERCEL_ENV || process.env.NODE_ENV,
    // Trace 10% of requests in prod, 100% in dev. Adjust based on quota.
    tracesSampleRate: process.env.NODE_ENV === 'production' ? 0.1 : 1.0,
    // Replay is heavy — only sample 1% of sessions, and 100% of sessions with
    // errors. Turned OFF entirely if the integration isn't loaded.
    replaysSessionSampleRate: 0.01,
    replaysOnErrorSampleRate: 1.0,
    integrations: [],
    sendDefaultPii: false,
    beforeSend(event, hint) {
      // Drop noisy `ResizeObserver loop limit exceeded` — a benign browser
      // hiccup that dominates Sentry inboxes if left unfiltered.
      const err = hint.originalException;
      if (err instanceof Error && /ResizeObserver loop/i.test(err.message)) return null;

      // Scrub the same cookie / auth-header surfaces we scrub server-side —
      // the browser SDK captures the request the page was fetched with, and
      // that includes the Supabase session cookie.
      if (event.request) {
        if (event.request.cookies) event.request.cookies = { redacted: '[Filtered]' };
        if (event.request.headers) {
          const h = event.request.headers as Record<string, string>;
          if (h.authorization) h.authorization = '[Filtered]';
          if (h.cookie) h.cookie = '[Filtered]';
        }
      }
      if (event.user) {
        event.user = { id: event.user.id };
      }
      return event;
    },
  });
}
