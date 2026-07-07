// Cross-runtime error helpers. Kept separate from `api-errors.ts` (which
// pulls Sentry + Pino and is server-only) so client components can narrow
// caught errors without dragging server-only deps into the browser bundle.

/**
 * Coerce anything thrown into a human-readable string. Handles native `Error`,
 * strings, PostgREST / Supabase error objects, and falls back to the safe
 * `String(err)` for exotic values so callers can pass the result straight to
 * a toast description.
 */
export function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === 'string') return err;
  if (err && typeof err === 'object') {
    const e = err as { message?: unknown; code?: unknown; details?: unknown; hint?: unknown };
    const parts: string[] = [];
    if (typeof e.message === 'string') parts.push(e.message);
    if (typeof e.code === 'string') parts.push(`code=${e.code}`);
    if (typeof e.details === 'string') parts.push(`details=${e.details}`);
    if (typeof e.hint === 'string') parts.push(`hint=${e.hint}`);
    if (parts.length > 0) return parts.join(' | ');
    try {
      return JSON.stringify(err);
    } catch {
      /* fall through */
    }
  }
  return String(err);
}
