'use client';

import { useEffect } from 'react';
import Link from 'next/link';
import * as Sentry from '@sentry/nextjs';
import { AlertTriangle, RotateCcw, Home } from 'lucide-react';
import { Button } from '@/components/ui/button';

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    // No-op when DSN is unset (see sentry.client.config.ts). digest is the
    // Next.js server-side error id — attach so client + server reports link
    // to the same incident in Sentry.
    Sentry.captureException(error, { tags: { digest: error.digest } });
    // Only echo to the browser console in dev — in prod it's noise, and
    // Sentry already carries the full stack.
    if (process.env.NODE_ENV !== 'production') {
      console.error(error);
    }
  }, [error]);

  return (
    <div className="flex min-h-[60vh] flex-col items-center justify-center px-4 py-16 text-center">
      <div className="mb-6 flex size-16 items-center justify-center rounded-2xl bg-red-50 text-red-500 dark:bg-red-950/30">
        <AlertTriangle className="size-8" />
      </div>
      <p className="mb-2 text-[10px] font-bold tracking-widest text-red-500 uppercase">Error</p>
      <h1 className="text-brand dark:text-accent-warm mb-3 font-serif text-2xl font-bold md:text-3xl">
        Đã xảy ra lỗi
      </h1>
      <p className="mb-2 max-w-md text-sm text-slate-500 dark:text-slate-400">
        Hệ thống gặp sự cố. Vui lòng thử lại · Something went wrong on our end. Please try again.
      </p>
      {error.digest && (
        <p className="mb-8 font-mono text-[10px] text-slate-500 dark:text-slate-400">
          ref: {error.digest}
        </p>
      )}
      <div className="mt-4 flex flex-col gap-3 sm:flex-row">
        <Button onClick={reset} className="h-10 gap-2 rounded-lg shadow-sm">
          <RotateCcw className="size-4" /> Thử lại
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
