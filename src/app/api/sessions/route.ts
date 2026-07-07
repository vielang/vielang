import { type NextRequest, NextResponse } from 'next/server';
import { randomUUID } from 'crypto';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase, getSessionsForUser, getTutorById } from '@/lib/supabase';
import { isSlotStillOpen, resolveCourseForBooking } from '@/lib/booking';
import {
  createSessionSchema,
  type CreatePrivateSessionInput,
  type CreateGroupSessionInput,
} from '@/lib/schemas/session';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';
import { sendEmail } from '@/lib/email';
import { SessionConfirmationEmail } from '@/lib/emails/SessionConfirmation';

export const dynamic = 'force-dynamic';

/**
 * GET /api/sessions — list sessions for the current user.
 *
 * - user   → their bookings as a student
 * - tutor  → sessions they're teaching
 * - admin  → everything (optionally filtered by ?studentId/&tutorId)
 *
 * Optional `?status=` filter accepts any of the enum values.
 */
export async function GET(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const status = req.nextUrl.searchParams.get('status') || undefined;

    if (auth.user.role === 'admin') {
      let query = supabase.from('sessions').select('*').order('scheduled_at', { ascending: true });
      if (status) query = query.eq('status', status);
      const { data, error } = await query;
      if (error) throw error;
      return NextResponse.json({ sessions: data || [] });
    }

    const list = await getSessionsForUser(
      auth.user.id,
      auth.user.role === 'tutor' ? 'tutor' : 'user',
    );
    const filtered = status ? list.filter((s) => s.status === status) : list;
    return NextResponse.json({ sessions: filtered });
  } catch (err) {
    return apiError(err);
  }
}

/**
 * POST /api/sessions — create a session.
 *
 * Two shapes, disambiguated by `type`:
 *  - `private` (default): student books a 1-on-1 tutor slot. Concurrency:
 *    we validate the slot is still open right before insert AND rely on the
 *    partial unique index on (tutor_id, scheduled_at) to reject any insert
 *    that beats the check via a race. Duplicate-key errors surface as 409.
 *  - `group`: admin creates a group free-talk room. Requires admin auth,
 *    a topic, a capacity >= 2, and a duration. host_tutor_id is optional —
 *    if omitted the admin is the implicit host.
 */
export async function POST(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    // Rate limit AFTER auth so anonymous 401s don't burn quota, and so we can
    // scope the identifier per user (per-IP would let one user starve others
    // behind a shared NAT).
    const rl = await rateLimit('session-create', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const raw = await req.json().catch(() => ({}));
    const parsed = createSessionSchema.safeParse(raw);
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }

    if (parsed.data.type === 'group') {
      return createGroupSession(auth, parsed.data);
    }
    return createPrivateSession(auth, parsed.data);
  } catch (err) {
    return apiError(err);
  }
}

async function createPrivateSession(
  auth: { user: { id: string; role: string; email: string | null } },
  input: CreatePrivateSessionInput,
) {
  const { tutor_id, course_id, scheduled_at, student_notes } = input;

  const tutor = await getTutorById(tutor_id);
  if (!tutor || !tutor.profile?.is_approved) {
    return NextResponse.json({ error: 'tutor_not_found' }, { status: 404 });
  }
  const course = await resolveCourseForBooking(course_id, tutor_id);
  if (!course) {
    return NextResponse.json({ error: 'course_not_available' }, { status: 400 });
  }
  if (auth.user.id === tutor_id) {
    return NextResponse.json({ error: 'cannot_book_self' }, { status: 400 });
  }
  const start = new Date(scheduled_at);
  if (start.getTime() <= Date.now()) {
    return NextResponse.json({ error: 'slot_in_past' }, { status: 400 });
  }
  const open = await isSlotStillOpen({
    tutorId: tutor_id,
    scheduledAt: start,
    durationMin: course.duration_min,
  });
  if (!open) {
    return NextResponse.json({ error: 'slot_taken' }, { status: 409 });
  }

  const roomName = `session-${randomUUID()}`;
  const { data, error } = await supabase
    .from('sessions')
    .insert({
      type: 'private',
      student_id: auth.user.id,
      tutor_id,
      course_id,
      scheduled_at: start.toISOString(),
      duration_min: course.duration_min,
      status: 'pending',
      livekit_room_name: roomName,
      price_vnd: course.price_vnd,
      student_notes: student_notes ?? null,
      capacity: 1,
    })
    .select('*')
    .single();
  if (error) {
    if ((error as any).code === '23505') {
      return NextResponse.json({ error: 'slot_taken' }, { status: 409 });
    }
    throw error;
  }

  // Fire-and-forget confirmation email. Wrapped in void so a slow (or absent)
  // Resend API can never delay the 201 response.
  void sendConfirmationEmail({
    session: data as { id: string; scheduled_at: string; duration_min: number },
    studentId: auth.user.id,
    studentEmail: auth.user.email || null,
    tutorName: tutor.name,
    courseTitle: course.title_en,
  });

  return NextResponse.json({ session: data }, { status: 201 });
}

async function sendConfirmationEmail(input: {
  session: { id: string; scheduled_at: string; duration_min: number };
  studentId: string;
  studentEmail: string | null;
  tutorName: string;
  courseTitle: string;
}) {
  try {
    // Look up the student's display name for the greeting. Fall back to first
    // half of the email if the row is missing — better than "Hi ,".
    const { data: student } = await supabase
      .from('users')
      .select('name, email, timezone')
      .eq('id', input.studentId)
      .maybeSingle();
    const recipient = input.studentEmail || student?.email || '';
    if (!recipient) return; // sendEmail also guards, but skip the render round-trip

    const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || 'https://vielang.com';
    const joinUrl = `${siteUrl}/session/${input.session.id}/room`;
    const manageUrl = `${siteUrl}/my-sessions?highlight=${input.session.id}`;

    // Format in the student's own timezone — the confirmation email should
    // read "11:00 ICT" not "04:00 UTC" for a Ho Chi Minh user.
    const studentTz = student?.timezone || 'Asia/Ho_Chi_Minh';
    const scheduledAt = new Date(input.session.scheduled_at).toLocaleString('en-GB', {
      weekday: 'short',
      day: '2-digit',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit',
      timeZoneName: 'short',
      timeZone: studentTz,
    });

    await sendEmail({
      kind: 'session.confirmation',
      to: recipient,
      subject: `Booked: ${input.courseTitle} with ${input.tutorName}`,
      template: SessionConfirmationEmail({
        studentName: student?.name || recipient.split('@')[0] || 'there',
        tutorName: input.tutorName,
        courseTitle: input.courseTitle,
        scheduledAt,
        durationMin: input.session.duration_min,
        joinUrl,
        manageUrl,
      }),
      sessionId: input.session.id,
      userId: input.studentId,
    });
  } catch {
    // Never surface — email is out-of-band. The notification_log write inside
    // sendEmail already tracks failures.
  }
}

async function createGroupSession(
  auth: { user: { id: string; role: string; email: string | null } },
  input: CreateGroupSessionInput,
) {
  if (auth.user.role !== 'admin') {
    return NextResponse.json({ error: 'forbidden' }, { status: 403 });
  }

  const start = new Date(input.scheduled_at);
  if (start.getTime() <= Date.now()) {
    return NextResponse.json({ error: 'slot_in_past' }, { status: 400 });
  }

  // If admin assigns a host tutor, they must exist and be approved. Blocking
  // the create here beats surfacing a foreign-key error to the UI later.
  if (input.host_tutor_id) {
    const tutor = await getTutorById(input.host_tutor_id);
    if (!tutor || !tutor.profile?.is_approved) {
      return NextResponse.json({ error: 'host_tutor_not_found' }, { status: 404 });
    }
  }

  const roomName = `group-${randomUUID()}`;
  const { data, error } = await supabase
    .from('sessions')
    .insert({
      type: 'group',
      tutor_id: input.host_tutor_id ?? null,
      student_id: null,
      course_id: null,
      topic_en: input.topic_en,
      topic_vn: input.topic_vn ?? null,
      description: input.description ?? null,
      level: input.level,
      capacity: input.capacity,
      cover_emoji: input.cover_emoji ?? null,
      scheduled_at: start.toISOString(),
      duration_min: input.duration_min,
      // Admin-created rooms skip the pending → confirmed dance; they're open
      // for bookings the moment they exist.
      status: 'confirmed',
      livekit_room_name: roomName,
      price_vnd: 0,
      require_admission: input.require_admission,
    })
    .select('*')
    .single();
  if (error) {
    if ((error as any).code === '23505') {
      return NextResponse.json({ error: 'slot_taken' }, { status: 409 });
    }
    throw error;
  }
  return NextResponse.json({ session: data }, { status: 201 });
}
