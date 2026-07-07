import type { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/auth/get-current-user';
import { getSessionListForUser, getReviewedSessionIdsForStudent } from '@/lib/supabase';
import { MySessionsClient } from './MySessionsClient';

export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'My sessions · VieLang',
  robots: { index: false, follow: false },
};

export default async function MySessionsPage() {
  const user = await getCurrentUser();
  if (!user) redirect('/login?redirect=/my-sessions');
  if (user.role !== 'user' && user.role !== 'tutor' && user.role !== 'admin') {
    redirect('/');
  }

  const role = user.role === 'tutor' ? 'tutor' : 'user';
  const [sessions, reviewedIds] = await Promise.all([
    getSessionListForUser(user.id, role).catch(() => []),
    role === 'user'
      ? getReviewedSessionIdsForStudent(user.id).catch(() => new Set<string>())
      : Promise.resolve(new Set<string>()),
  ]);

  return (
    <MySessionsClient
      initial={sessions}
      viewerRole={role}
      reviewedSessionIds={Array.from(reviewedIds)}
    />
  );
}
