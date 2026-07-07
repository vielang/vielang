import { type NextRequest, NextResponse } from 'next/server';
import { supabase } from '@/lib/supabase';
import { childLogger } from '@/lib/logger';

export const dynamic = 'force-dynamic';
export const runtime = 'nodejs';

const log = childLogger('cron-no-show');

// A session is eligible for the no-show check once it's been over for at
// least this long — mirrors the same 15-minute grace the token endpoint
// gives a late joiner. Anything shorter and we'd race against people
// straggling in during the grace window.
const OVERDUE_GRACE_MS = 15 * 60_000;

/**
 * Vercel Cron endpoint — marks sessions as no_show when nobody actually
 * joined. Complements /api/livekit/webhook which flips live sessions to
 * completed on room_finished but leaves untouched-status sessions in
 * pending/confirmed limbo.
 *
 * Runs on the schedule declared in vercel.json (every 10 minutes). Each tick:
 *   1. Find sessions with status IN ('pending','confirmed') whose end time
 *      passed more than OVERDUE_GRACE_MS ago.
 *   2. For each, look at session_attendance:
 *        • Private session — mark no_show if neither the student nor the
 *          tutor has an attendance row.
 *        • Group session — mark no_show if the tutor never joined (guests
 *          straggling into an empty group room isn't itself a session).
 *      A session where only one party made it stays untouched — the manual
 *      admin flow decides refunds / rescheduling in that case.
 *   3. Update the session row's status + record a `cancelled_reason` note
 *      so ops can distinguish no-show from an admin cancel later.
 *
 * Auth: Vercel Cron adds `Authorization: Bearer <CRON_SECRET>`. Missing or
 * mismatched → 401; missing CRON_SECRET env var also 401 (fail-closed).
 */
export async function GET(req: NextRequest) {
  const secret = process.env.CRON_SECRET;
  const authz = req.headers.get('authorization');
  if (!secret || authz !== `Bearer ${secret}`) {
    return NextResponse.json({ error: 'unauthorized' }, { status: 401 });
  }

  const now = Date.now();

  // Ideally we'd filter server-side on `scheduled_at + duration_min` but
  // Supabase's PostgREST doesn't compose the two columns arithmetically.
  // A single-column lower bound (7 days back) keeps the batch cheap and we
  // finish the check in JS.
  const scanFromIso = new Date(now - 7 * 24 * 3600_000).toISOString();
  const { data: sessions, error } = await supabase
    .from('sessions')
    .select('id, type, tutor_id, student_id, scheduled_at, duration_min, status')
    .in('status', ['pending', 'confirmed'])
    .gte('scheduled_at', scanFromIso);
  if (error) {
    log.error({ err: error }, 'no-show scan failed');
    return NextResponse.json({ error: 'scan_failed' }, { status: 500 });
  }
  if (!sessions || sessions.length === 0) {
    return NextResponse.json({ ok: true, marked: 0, checked: 0 });
  }

  // Second pass: keep only sessions whose end time is comfortably past.
  const overdue = sessions.filter((s) => {
    const start = new Date(s.scheduled_at).getTime();
    const end = start + s.duration_min * 60_000;
    return end + OVERDUE_GRACE_MS < now;
  });
  if (overdue.length === 0) {
    return NextResponse.json({ ok: true, marked: 0, checked: sessions.length });
  }

  const ids = overdue.map((s) => s.id);
  // Two independent "someone was here" signals — attendance (written by
  // LiveKit's participant_joined webhook) and session_messages (written
  // synchronously by the chat client). If either has rows for a session,
  // we know the room wasn't empty. Belt-and-braces: if the LiveKit webhook
  // ever misfires we still credit real attendance from chat activity.
  const [attRes, msgRes] = await Promise.all([
    supabase.from('session_attendance').select('session_id, user_id').in('session_id', ids),
    supabase.from('session_messages').select('session_id, sender_id').in('session_id', ids),
  ]);
  if (attRes.error) {
    log.error({ err: attRes.error }, 'attendance lookup failed');
    return NextResponse.json({ error: 'attendance_lookup_failed' }, { status: 500 });
  }
  if (msgRes.error) {
    log.warn({ err: msgRes.error }, 'messages lookup failed — proceeding with attendance only');
  }

  // Index: session_id → Set<user_id> that either joined OR posted chat.
  const attendedBySession = new Map<string, Set<string>>();
  const record = (sessionId: string, userId: string) => {
    let set = attendedBySession.get(sessionId);
    if (!set) {
      set = new Set();
      attendedBySession.set(sessionId, set);
    }
    set.add(userId);
  };
  for (const row of attRes.data ?? []) record(row.session_id, row.user_id);
  for (const row of msgRes.data ?? []) record(row.session_id, row.sender_id);

  let marked = 0;
  for (const s of overdue) {
    const attended = attendedBySession.get(s.id) ?? new Set<string>();
    const tutorPresent = !!s.tutor_id && attended.has(s.tutor_id);
    const studentPresent = !!s.student_id && attended.has(s.student_id);

    // Skip if any real attendance was captured — the split-fault case is
    // handled by admin flow. Group sessions only need the tutor to not
    // show; empty group rooms are a no_show regardless of whether random
    // guests wandered in.
    const isNoShow = s.type === 'group' ? !tutorPresent : !tutorPresent && !studentPresent;
    if (!isNoShow) continue;

    const { error: updErr } = await supabase
      .from('sessions')
      .update({
        status: 'no_show',
        cancelled_reason: 'auto: no attendance recorded within grace window',
      })
      .eq('id', s.id)
      // Belt-and-braces: another tick or a manual admin action might have
      // moved the row in the meantime. Filter on the two statuses we
      // originally selected so we can't clobber a completed/cancelled row.
      .in('status', ['pending', 'confirmed']);
    if (updErr) {
      log.warn({ sessionId: s.id, err: updErr }, 'no-show update failed');
      continue;
    }
    marked += 1;
    log.info({ sessionId: s.id, type: s.type }, 'marked session as no_show');
  }

  log.info({ marked, checked: overdue.length }, 'no-show cron tick complete');
  return NextResponse.json({ ok: true, marked, checked: overdue.length });
}
