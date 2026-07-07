import type { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/auth/get-current-user';
import {
  getAdminKpis,
  getAllUsers,
  getAllSessionsForAdmin,
  getPendingTutors,
  getAllCoursesForAdmin,
  getAllReviewsForAdmin,
} from '@/lib/supabase';
import { AdminShell } from '@/components/admin/AdminShell';

export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'Admin · VieLang',
  robots: { index: false, follow: false },
};

export default async function AdminPage() {
  const user = await getCurrentUser();
  if (!user) redirect('/login?redirect=/admin');
  if (user.role !== 'admin') redirect('/?denied=1');

  // Every payload the shell + its four sections need, in a single parallel
  // fetch. Any single failure degrades to an empty list so the rest of the
  // dashboard stays usable — admin can retry with a hard refresh.
  const [kpis, users, sessions, pendingTutors, courses, reviews] = await Promise.all([
    getAdminKpis().catch(() => ({
      usersTotal: 0,
      studentsTotal: 0,
      tutorsTotal: 0,
      tutorsApproved: 0,
      tutorsPending: 0,
      sessionsToday: 0,
      sessionsWeek: 0,
      sessionsMonth: 0,
      revenueWeekVnd: 0,
      revenueMonthVnd: 0,
      ratingAvg: 0,
      sessionsPerDay: [],
    })),
    getAllUsers().catch(() => []),
    getAllSessionsForAdmin().catch(() => []),
    getPendingTutors().catch(() => []),
    getAllCoursesForAdmin().catch(() => []),
    getAllReviewsForAdmin().catch(() => []),
  ]);

  return (
    <AdminShell
      kpis={kpis}
      users={users}
      sessions={sessions}
      pendingTutors={pendingTutors}
      courses={courses}
      reviews={reviews}
      currentUserId={user.id}
    />
  );
}
