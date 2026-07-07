import type { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/auth/get-current-user';
import {
  getTutorById,
  getTutorKpis,
  getSessionListForUser,
  getAvailabilityForTutor,
  getPublishedCourses,
} from '@/lib/supabase';
import { TutorShell } from '@/components/tutor/TutorShell';

export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'Tutor · VieLang',
  robots: { index: false, follow: false },
};

export default async function TutorPage() {
  const user = await getCurrentUser();
  if (!user) redirect('/login?redirect=/tutor');
  // Admins can peek at the tutor dashboard for support purposes; regular
  // users bounce.
  if (user.role !== 'tutor' && user.role !== 'admin') redirect('/?denied=1');

  // Admin visiting /tutor without being a tutor themselves gets a friendly
  // note instead of blank screens — send them back to /admin.
  const tutor = await getTutorById(user.id).catch(() => null);
  if (!tutor || !tutor.profile) {
    if (user.role === 'admin') redirect('/admin');
    redirect('/?denied=1');
  }

  const [kpis, sessions, availability, courses] = await Promise.all([
    getTutorKpis(tutor.id).catch(() => ({
      upcomingCount: 0,
      todayCount: 0,
      weekCount: 0,
      monthCount: 0,
      completedTotal: 0,
      revenueWeekVnd: 0,
      revenueMonthVnd: 0,
      ratingAvg: 0,
      reviewsCount: 0,
    })),
    getSessionListForUser(tutor.id, 'tutor').catch(() => []),
    getAvailabilityForTutor(tutor.id).catch(() => []),
    getPublishedCourses({ tutorId: tutor.id }).catch(() => []),
  ]);

  return (
    <TutorShell
      user={tutor}
      profile={tutor.profile}
      kpis={kpis}
      sessions={sessions}
      availability={availability as any}
      courses={courses}
    />
  );
}
