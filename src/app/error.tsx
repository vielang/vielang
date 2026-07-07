'use client';

import { useEffect } from 'react';
import Link from 'next/link';
import { AlertTriangle, Home, RefreshCw } from 'lucide-react';
import * as Sentry from '@sentry/nextjs';
import { Button } from '@/components/ui/button';

/**
 * Root error boundary — catches any uncaught throw from a Server Component,
 * Server Action, or the initial render of a Client Component below the app
 * root. Renders a small friendly panel and forwards the error to Sentry so
 * we don't just eat production issues.
 *
 * Notes:
 *  • Next.js requires this file to be a Client Component.
 *  • `reset()` re-tries the render tree — good for transient failures (a
 *    Supabase timeout, a temporary CSP mismatch during deploy). If the
 *    render just fails again the user sees the same panel and can bail.
 *  • `error.digest` is Next.js's stable, non-PII identifier — safe to show.
 *    We never render the raw error message in prod because it may leak the
 *    stack / DB structure / secrets that made it into the exception body.
 */
interface Props {
  error: Error & { digest?: string };
  reset: () => void;
}

export default function GlobalError({ error, reset }: Props) {
  useEffect(() => {
    Sentry.captureException(error, { tags: { boundary: 'app-root' } });
  }, [error]);

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-slate-50 px-4 text-center font-sans dark:bg-slate-950">
      <div className="mb-6 flex size-16 items-center justify-center rounded-2xl bg-red-50 text-red-600 dark:bg-red-950/40 dark:text-red-400">
        <AlertTriangle className="size-8" />
      </div>
      <p className="mb-2 text-[10px] font-bold tracking-widest text-red-500 uppercase dark:text-red-400">
        Something went wrong
      </p>
      <h1 className="text-brand dark:text-accent-warm mb-3 text-2xl font-bold md:text-3xl">
        Đã có lỗi xảy ra
      </h1>
      <p className="mb-2 max-w-md text-sm text-slate-500 dark:text-slate-400">
        We&apos;ve logged the issue and are looking into it. Try again in a moment, or head back to
        the home page.
      </p>
      {error.digest && (
        <p className="mb-8 text-[11px] text-slate-400 tabular-nums dark:text-slate-500">
          Reference: <span className="font-mono">{error.digest}</span>
        </p>
      )}
      <div className="flex flex-wrap items-center justify-center gap-2">
        <Button onClick={reset} className="h-10 gap-2 rounded-lg">
          <RefreshCw className="size-4" /> Try again
        </Button>
        <Link href="/">
          <Button variant="outline" className="h-10 gap-2 rounded-lg">
            <Home className="size-4" /> Trang chủ
          </Button>
        </Link>
      </div>
    </div>
  );
}
