/**
 * Next.js runtime instrumentation entry point.
 *
 * Called ONCE per runtime (nodejs or edge) at cold start, before any request
 * is served. We lazy-import the Sentry config here so the client bundle never
 * pays for it — the two config files are otherwise picked up by the Sentry
 * webpack plugin at build time.
 */
export async function register() {
  if (process.env.NEXT_RUNTIME === 'nodejs') {
    await import('../sentry.server.config');
  }
  if (process.env.NEXT_RUNTIME === 'edge') {
    await import('../sentry.edge.config');
  }
}
