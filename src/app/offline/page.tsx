import type { Metadata } from 'next';
import Link from 'next/link';

// Served by the service worker as the navigation fallback when both
// the network AND the runtime cache miss. Kept intentionally
// self-contained: no header, no footer, no client data hooks — those
// rely on bundles/APIs that may also be unreachable when this page is
// the one shown. Tri-lingual copy via static markup since the LangContext
// is also down-stream of the unreachable JS chunks.

export const metadata: Metadata = {
  title: 'Offline · VieLang',
  robots: { index: false, follow: false },
};

export default function OfflinePage() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-gradient-to-b from-slate-50 to-white px-6 py-12 dark:from-slate-950 dark:to-slate-900">
      <div className="max-w-md space-y-6 text-center">
        <div className="bg-brand text-accent-warm mx-auto flex size-20 items-center justify-center rounded-2xl text-3xl font-bold shadow-md">
          V
        </div>

        <div className="space-y-2">
          <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold">
            Mất kết nối · Offline
          </h1>
          <p className="text-sm leading-relaxed text-slate-500 dark:text-slate-400">
            Vui lòng kiểm tra kết nối và thử lại.
            <br />
            Please check your connection and try again.
          </p>
        </div>

        <div className="flex flex-col gap-2 pt-2">
          <Link
            href="/"
            className="bg-brand hover:bg-brand-hover inline-flex h-11 items-center justify-center rounded-lg px-6 text-sm font-semibold text-white shadow-sm transition-colors"
          >
            Thử lại · Retry
          </Link>
          <p className="text-[11px] text-slate-400 dark:text-slate-500">
            VieLang · Learn English Online
          </p>
        </div>
      </div>
    </main>
  );
}
