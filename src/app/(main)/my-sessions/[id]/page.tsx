import type { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/auth/get-current-user';
import { supabase } from '@/lib/supabase';
import { SessionDetailClient, type SessionDetailContext } from './SessionDetailClient';

export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'Session details · VieLang',
  robots: { index: false, follow: false },
};

/**
 * Post-session review page. Shows the chat transcript persisted by
 * SessionChatPanel and the aggregated attendance derived from the
 * participant_joined/left webhooks.
 *
 * Access rules mirror /api/sessions/[id]/messages exactly:
 *   • admin can see any session
 *   • tutor / student can see a private session they were part of
 *   • group-session guests need a session_participants row
 * Anything else redirects back to /my-sessions rather than 403-ing —
 * the URL is often guessable so a wrong click shouldn't blow up.
 */
export default async function SessionDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const user = await getCurrentUser();
  if (!user) redirect(`/login?redirect=/my-sessions/${id}`);

  const { data } = await supabase
    .from('sessions')
    .select(
      `
      id, type, status, scheduled_at, duration_min,
      student_id, tutor_id, topic_en, topic_vn,
      tutor:users!sessions_tutor_id_fkey(id, name, avatar),
      student:users!sessions_student_id_fkey(id, name),
      course:courses(title_en, title_vn)
    `,
    )
    .eq('id', id)
    .maybeSingle();

  if (!data) redirect('/my-sessions');

  const row = data as unknown as {
    id: string;
    type: 'private' | 'group';
    status: string;
    scheduled_at: string;
    duration_min: number;
    student_id: string | null;
    tutor_id: string | null;
    topic_en: string | null;
    topic_vn: string | null;
    tutor: { name: string | null; avatar: string | null } | null;
    student: { name: string | null } | null;
    course: { title_en: string | null; title_vn: string | null } | null;
  };

  const isAdmin = user.role === 'admin';
  const isTutor = user.id === row.tutor_id;
  const isStudent = user.id === row.student_id;
  let hasAccess = isAdmin || isTutor || isStudent;
  if (!hasAccess && row.type === 'group') {
    const { data: membership } = await supabase
      .from('session_participants')
      .select('user_id')
      .eq('session_id', id)
      .eq('user_id', user.id)
      .maybeSingle();
    hasAccess = !!membership;
  }
  if (!hasAccess) redirect('/my-sessions');

  const context: SessionDetailContext = {
    sessionId: id,
    status: row.status,
    scheduledAt: row.scheduled_at,
    durationMin: row.duration_min,
    courseTitle:
      row.course?.title_en || row.course?.title_vn || row.topic_en || row.topic_vn || null,
    tutorName: row.tutor?.name || null,
    studentName: row.student?.name || null,
    viewerRole: isAdmin ? 'admin' : isTutor ? 'tutor' : 'student',
    myId: user.id,
  };

  return <SessionDetailClient context={context} />;
}
