import * as Sentry from '@sentry/nextjs';

/**
 * Edge runtime Sentry — powers `middleware.ts` and any route handler that
 * opts in with `export const runtime = 'edge'`. Uses the same DSN as the
 * Node runtime; Sentry uses the SDK tag (`sdk.name`) to distinguish.
 */
const dsn = process.env.SENTRY_DSN || process.env.NEXT_PUBLIC_SENTRY_DSN;
if (dsn) {
  Sentry.init({
    dsn,
    environment: process.env.VERCEL_ENV || process.env.NODE_ENV,
    tracesSampleRate: process.env.NODE_ENV === 'production' ? 0.1 : 1.0,
  });
}
