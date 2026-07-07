import { type NextRequest, NextResponse } from 'next/server';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase, getSessionById } from '@/lib/supabase';
import { isSlotStillOpen } from '@/lib/booking';
import { patchSessionSchema } from '@/lib/schemas/session';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

const RESCHEDULE_MIN_HOURS = 24;

/**
 * PATCH /api/sessions/[id] — cancel, reschedule or confirm a session.
 *
 *   action=cancel      : student, tutor of the session, or admin
 *   action=reschedule  : same actors, and >= 24h before scheduled_at
 *   action=confirm     : tutor of the session or admin (student can't
 *                        self-confirm — the tutor confirms once they've
 *                        acknowledged the pending booking)
 */
export async function PATCH(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const rl = await rateLimit('session-mutate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const { id } = await params;
    const session = await getSessionById(id);
    if (!session) return NextResponse.json({ error: 'not_found' }, { status: 404 });

    // Group session lifecycle (edit / cancel / reschedule) is admin-only and
    // lives under its own endpoints — reject here so a student can't cancel
    // a free-talk room they merely joined.
    if (session.type === 'group') {
      return NextResponse.json({ error: 'group_session_read_only_here' }, { status: 400 });
    }

    const raw = await req.json().catch(() => ({}));
    const parsed = patchSessionSchema.safeParse(raw);
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }
    const { action } = parsed.data;

    const isStudent = auth.user.id === session.student_id;
    const isTutor = auth.user.id === session.tutor_id;
    const isAdmin = auth.user.role === 'admin';

    if (!isStudent && !isTutor && !isAdmin) {
      return NextResponse.json({ error: 'forbidden' }, { status: 403 });
    }

    // Terminal statuses can't be mutated by anyone but admin — prevents a
    // student from cancelling a completed lesson to dodge a review.
    if (
      !isAdmin &&
      (session.status === 'completed' ||
        session.status === 'cancelled' ||
        session.status === 'no_show')
    ) {
      return NextResponse.json({ error: 'session_locked' }, { status: 400 });
    }

    if (action === 'confirm') {
      if (!isTutor && !isAdmin) {
        return NextResponse.json({ error: 'only_tutor_can_confirm' }, { status: 403 });
      }
      if (session.status !== 'pending') {
        return NextResponse.json({ error: 'not_pending' }, { status: 400 });
      }
      const { data, error } = await supabase
        .from('sessions')
        .update({ status: 'confirmed' })
        .eq('id', id)
        .select('*')
        .single();
      if (error) throw error;
      return NextResponse.json({ session: data });
    }

    if (action === 'cancel') {
      const patch: Record<string, unknown> = {
        status: 'cancelled',
        cancelled_by: auth.user.id,
        cancelled_reason: parsed.data.cancelled_reason ?? null,
      };
      const { data, error } = await supabase
        .from('sessions')
        .update(patch)
        .eq('id', id)
        .select('*')
        .single();
      if (error) throw error;
      return NextResponse.json({ session: data });
    }

    if (action === 'reschedule') {
      const currentStart = new Date(session.scheduled_at).getTime();
      const hoursUntil = (currentStart - Date.now()) / 3_600_000;
      if (!isAdmin && hoursUntil < RESCHEDULE_MIN_HOURS) {
        return NextResponse.json(
          { error: 'reschedule_window_closed', hoursUntil },
          { status: 400 },
        );
      }
      const nextStart = new Date(parsed.data.scheduled_at!);
      if (nextStart.getTime() <= Date.now()) {
        return NextResponse.json({ error: 'slot_in_past' }, { status: 400 });
      }
      if (!session.tutor_id) {
        return NextResponse.json({ error: 'session_missing_tutor' }, { status: 400 });
      }
      const open = await isSlotStillOpen({
        tutorId: session.tutor_id,
        scheduledAt: nextStart,
        durationMin: session.duration_min,
      });
      if (!open) return NextResponse.json({ error: 'slot_taken' }, { status: 409 });

      const { data, error } = await supabase
        .from('sessions')
        .update({ scheduled_at: nextStart.toISOString(), status: 'pending' })
        .eq('id', id)
        .select('*')
        .single();
      if (error) {
        if ((error as any).code === '23505') {
          return NextResponse.json({ error: 'slot_taken' }, { status: 409 });
        }
        throw error;
      }
      return NextResponse.json({ session: data });
    }

    return NextResponse.json({ error: 'unknown_action' }, { status: 400 });
  } catch (err) {
    return apiError(err);
  }
}
