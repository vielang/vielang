import type { Metadata } from 'next';
import { getApprovedTutors, getUpcomingGroupSessions, getHomeStats } from '@/lib/supabase';
import { HomeClient } from './HomeClient';

export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'VieLang · Learn English 1-on-1 Online',
  description:
    'Book flexible 1-on-1 video sessions with certified English tutors, or drop into free-talk group rooms.',
  openGraph: {
    title: 'VieLang · Learn English Online',
    description: 'Live 1-on-1 English tutoring + community group sessions via video call.',
    type: 'website',
    locale: 'vi_VN',
    alternateLocale: ['en_US'],
  },
};

export default async function Page() {
  // All three queries are independent — parallelize so first paint doesn't
  // wait three times as long. Each failure degrades to a safe empty value so
  // the hero + how-it-works still render if the DB is unreachable, and any
  // section that has no data (e.g. group sessions before the migration lands)
  // just hides itself.
  const [featuredTutors, upcomingSessions, stats] = await Promise.all([
    getApprovedTutors({ sort: 'rating' })
      .then((t) => t.slice(0, 6))
      .catch(() => []),
    getUpcomingGroupSessions()
      .then((s) => s.slice(0, 6))
      .catch(() => []),
    getHomeStats().catch(() => ({ tutorsApproved: 0, sessionsCompleted: 0, ratingAvg: 0 })),
  ]);

  return (
    <HomeClient featuredTutors={featuredTutors} upcomingSessions={upcomingSessions} stats={stats} />
  );
}
