import { type NextRequest, NextResponse } from 'next/server';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase } from '@/lib/supabase';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

/**
 * POST /api/sessions/[id]/reserve — student reserves a seat in a group session.
 *
 * Idempotent: if the caller already has a row in session_participants, we
 * return { alreadyReserved: true } with 200 rather than a 409, so the client
 * can just refresh state without special-casing the error.
 *
 * Capacity gate here counts *reservations* (session_participants), not live
 * presence — reservations are the promise of a seat; the LiveKit token endpoint
 * enforces the second, presence-based gate at actual join time.
 *
 * The tutor host and admins never need to reserve — the token endpoint grants
 * them access implicitly — so we short-circuit those roles with 400.
 */
export async function POST(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const rl = await rateLimit('session-reserve', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const { id } = await params;
    const { data: session } = await supabase
      .from('sessions')
      .select('id, type, status, tutor_id, scheduled_at, duration_min, capacity')
      .eq('id', id)
      .maybeSingle();
    if (!session) return NextResponse.json({ error: 'not_found' }, { status: 404 });
    if (session.type !== 'group') {
      return NextResponse.json({ error: 'not_a_group_session' }, { status: 400 });
    }
    if (auth.user.id === session.tutor_id || auth.user.role === 'admin') {
      return NextResponse.json({ error: 'host_no_reservation_needed' }, { status: 400 });
    }
    if (
      session.status === 'cancelled' ||
      session.status === 'completed' ||
      session.status === 'no_show'
    ) {
      return NextResponse.json({ error: 'session_closed' }, { status: 400 });
    }
    // Block reservations after the session's own end-of-window so late-comers
    // who missed the room don't collect a phantom row that would then unlock
    // the token endpoint on a room LiveKit already tore down.
    const end = new Date(session.scheduled_at).getTime() + session.duration_min * 60_000;
    if (Date.now() > end) {
      return NextResponse.json({ error: 'session_ended' }, { status: 400 });
    }

    // Fast path: already reserved.
    const { data: existing } = await supabase
      .from('session_participants')
      .select('user_id')
      .eq('session_id', id)
      .eq('user_id', auth.user.id)
      .maybeSingle();
    if (existing) {
      return NextResponse.json({ reserved: true, alreadyReserved: true });
    }

    // Capacity check on reservations. Small race window vs the insert below —
    // if two students hit exactly at the boundary, both may pass this SELECT
    // and one will succeed on the INSERT (primary key is (session_id, user_id)
    // per user, so the race can genuinely overshoot by one). We accept that
    // ±1 gap; the join-time gate in /api/livekit/token counts presence and
    // will refuse the extra body if it becomes a real problem.
    const { count } = await supabase
      .from('session_participants')
      .select('user_id', { count: 'exact', head: true })
      .eq('session_id', id);
    if ((count ?? 0) >= session.capacity) {
      return NextResponse.json({ error: 'at_capacity' }, { status: 409 });
    }

    const { error: insertError } = await supabase
      .from('session_participants')
      .insert({ session_id: id, user_id: auth.user.id });
    if (insertError) {
      // Duplicate key = raced with our own idempotent check; treat as success.
      if ((insertError as any).code === '23505') {
        return NextResponse.json({ reserved: true, alreadyReserved: true });
      }
      throw insertError;
    }

    return NextResponse.json({ reserved: true }, { status: 201 });
  } catch (err) {
    return apiError(err, 'sessions.reserve.post');
  }
}

/**
 * DELETE /api/sessions/[id]/reserve — student releases their reserved seat.
 *
 * Only permitted before the session has started. Once it's live we don't want
 * a student to un-reserve mid-lesson and re-open a seat that the token flow
 * has already handed out; the room lifecycle owns that state instead.
 */
export async function DELETE(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const { id } = await params;
    const { data: session } = await supabase
      .from('sessions')
      .select('id, type, scheduled_at, status')
      .eq('id', id)
      .maybeSingle();
    if (!session) return NextResponse.json({ error: 'not_found' }, { status: 404 });
    if (session.type !== 'group') {
      return NextResponse.json({ error: 'not_a_group_session' }, { status: 400 });
    }
    const start = new Date(session.scheduled_at).getTime();
    if (session.status === 'live' || Date.now() >= start) {
      return NextResponse.json({ error: 'session_started' }, { status: 400 });
    }

    const { error } = await supabase
      .from('session_participants')
      .delete()
      .eq('session_id', id)
      .eq('user_id', auth.user.id);
    if (error) throw error;

    return NextResponse.json({ released: true });
  } catch (err) {
    return apiError(err, 'sessions.reserve.delete');
  }
}
