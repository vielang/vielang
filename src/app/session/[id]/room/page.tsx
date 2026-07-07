import type { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/auth/get-current-user';
import { supabase } from '@/lib/supabase';
import { RoomClient } from './RoomClient';

export const metadata: Metadata = {
  title: 'Session · VieLang',
  robots: { index: false, follow: false },
};

/**
 * Server-side guard + context resolver for the LiveKit room page.
 *
 *  - Unauthenticated → /login?redirect=…
 *  - Fetches session + tutor/student/course info so the RoomClient can
 *    render a proper top bar (course title, other party's name) without a
 *    second network round-trip on top of the LiveKit token call.
 *  - The token endpoint enforces the "participant + join window" rules
 *    again from the client fetch, so the shape here is best-effort UX
 *    context, not a security boundary.
 */
export default async function SessionRoomPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  const user = await getCurrentUser();
  if (!user) redirect(`/login?redirect=/session/${id}/room`);

  const { data } = await supabase
    .from('sessions')
    .select(
      `
      id, type, student_id, tutor_id, status, scheduled_at, duration_min,
      tutor:users!sessions_tutor_id_fkey(id, name, avatar),
      student:users!sessions_student_id_fkey(id, name),
      course:courses(title_en, title_vn)
    `,
    )
    .eq('id', id)
    .maybeSingle();

  // For 1-on-1 sessions "the student" is the exact paired user. For group
  // sessions `student_id` is null so we fall back to "everyone who isn't the
  // tutor and isn't a platform admin viewing" — that's who should see the
  // waiting-room / denied / review flow on the room page.
  const isTutor = !!data && user.id === data.tutor_id;
  const isAdmin = user.role === 'admin';
  const isStudent =
    !!data && (data.type === 'private' ? user.id === data.student_id : !isTutor && !isAdmin);

  const context = data
    ? {
        sessionId: id,
        courseTitle: (data.course as any)?.title_en || (data.course as any)?.title_vn || null,
        tutorName: (data.tutor as any)?.name || 'Your tutor',
        studentName: (data.student as any)?.name || 'Your student',
        isStudent,
        isTutor,
        scheduledAt: data.scheduled_at,
        durationMin: data.duration_min,
      }
    : {
        sessionId: id,
        courseTitle: null,
        tutorName: 'Your tutor',
        studentName: 'Your student',
        isStudent: false,
        isTutor: false,
        scheduledAt: null,
        durationMin: null,
      };

  return <RoomClient context={context} />;
}
