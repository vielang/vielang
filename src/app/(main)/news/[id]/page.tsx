import type { Metadata } from 'next';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'News · VieLang',
};

// Placeholder — news detail rebuilt in later phase (news content adapts to
// the new bilingual title_vn/en, content_vn/en schema).
export default function NewsItemPage() {
  return (
    <main className="flex min-h-[60vh] items-center justify-center px-6 py-16">
      <div className="max-w-lg space-y-4 text-center">
        <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold">
          Article coming soon
        </h1>
        <p className="text-sm text-slate-500 dark:text-slate-400">
          Blog content is being migrated to the VieLang schema.
        </p>
        <Link
          href="/news"
          className="bg-brand hover:bg-brand-hover inline-flex h-10 items-center justify-center rounded-lg px-6 text-sm font-semibold text-white transition-colors"
        >
          Back to news
        </Link>
      </div>
    </main>
  );
}
