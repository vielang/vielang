import type { Metadata } from 'next';
import Link from 'next/link';
import { SearchX } from 'lucide-react';
import { getApprovedTutors } from '@/lib/supabase';
import { TutorCard } from '@/components/tutor/TutorCard';
import { TutorFilterBar } from '@/components/tutor/TutorFilterBar';
import { EmptyState } from '@/components/shared/EmptyState';
import { buttonVariants } from '@/components/ui/button';

export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'English Tutors · VieLang',
  description:
    'Browse certified English tutors — IELTS, Business English, Kids, TOEIC and more. Book flexible 1-on-1 video sessions.',
};

type SearchParams = { specialty?: string; sort?: string };

export default async function TutorsPage({
  searchParams,
}: {
  searchParams: Promise<SearchParams>;
}) {
  const params = await searchParams;
  const hasFilter = !!params.specialty;
  const tutors = await getApprovedTutors({
    specialty: params.specialty,
    sort: (params.sort as any) || 'rating',
  }).catch(() => []);

  return (
    <div className="space-y-8">
      <header className="space-y-2">
        <h1 className="type-page text-slate-900 dark:text-slate-100">Meet our English tutors</h1>
        <p className="max-w-2xl text-sm text-slate-500 dark:text-slate-400">
          {tutors.length} {tutors.length === 1 ? 'certified tutor' : 'certified tutors'} available
          for live 1-on-1 video lessons. Filter by specialty and sort by what matters most to you.
        </p>
      </header>

      <TutorFilterBar />

      {tutors.length === 0 ? (
        <EmptyState
          icon={<SearchX className="size-6" />}
          title={hasFilter ? 'No tutors match those filters yet' : 'No approved tutors yet'}
          description={
            hasFilter
              ? 'Try clearing the specialty or picking a different sort.'
              : 'Once an admin approves a tutor they will appear here.'
          }
          action={
            hasFilter ? (
              <Link href="/tutors" className={buttonVariants({ size: 'sm' })}>
                Clear filters
              </Link>
            ) : undefined
          }
        />
      ) : (
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {tutors.map((t, i) => (
            <TutorCard key={t.id} tutor={t} index={i} />
          ))}
        </div>
      )}
    </div>
  );
}
