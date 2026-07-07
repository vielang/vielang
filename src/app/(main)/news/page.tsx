import type { Metadata } from 'next';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'News · VieLang',
  description: 'Study tips, product updates, and stories from VieLang tutors and students.',
};

// Placeholder — news list will be rebuilt in Phase 8 against the new
// bilingual news table (title_vn/en, content_vn/en, category, image).
export default function NewsPage() {
  return (
    <main className="flex min-h-[60vh] items-center justify-center px-6 py-16">
      <div className="max-w-lg space-y-4 text-center">
        <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold">
          News & Study Tips
        </h1>
        <p className="text-sm text-slate-500 dark:text-slate-400">
          Fresh content is on the way. In the meantime, check out our tutors and book your first
          lesson.
        </p>
        <Link
          href="/tutors"
          className="bg-brand hover:bg-brand-hover inline-flex h-10 items-center justify-center rounded-lg px-6 text-sm font-semibold text-white transition-colors"
        >
          Browse tutors
        </Link>
      </div>
    </main>
  );
}
