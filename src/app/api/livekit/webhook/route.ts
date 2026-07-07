import { type NextRequest, NextResponse } from 'next/server';
import { WebhookReceiver } from 'livekit-server-sdk';
import { supabase } from '@/lib/supabase';
import { apiError } from '@/lib/api-errors';
import { sendEmail } from '@/lib/email';
import { ReviewRequestEmail } from '@/lib/emails/ReviewRequest';
import { childLogger } from '@/lib/logger';

const log = childLogger('livekit-webhook');

export const dynamic = 'force-dynamic';

let cachedReceiver: WebhookReceiver | null = null;
function getReceiver(): WebhookReceiver | null {
  if (cachedReceiver) return cachedReceiver;
  const apiKey = process.env.LIVEKIT_API_KEY;
  const apiSecret = process.env.LIVEKIT_API_SECRET;
  if (!apiKey || !apiSecret) return null;
  cachedReceiver = new WebhookReceiver(apiKey, apiSecret);
  return cachedReceiver;
}

/**
 * POST /api/livekit/webhook
 *
 * LiveKit fires signed webhook events for room + participant lifecycle.
 * We only act on `room_finished` for now — the room name is
 * `session-<uuid>` (set at booking time) which lets us map it back to a
 * VieLang session row and mark it completed.
 *
 * Signature is verified via the `Authorization` header LiveKit sends. If
 * verification fails, we 401 and let LiveKit retry rather than silently
 * dropping the event.
 */
export async function POST(req: NextRequest) {
  try {
    const receiver = getReceiver();
    if (!receiver) {
      return NextResponse.json({ error: 'livekit_not_configured' }, { status: 503 });
    }

    const authHeader = req.headers.get('authorization') || '';
    const body = await req.text();
    let event;
    try {
      event = await receiver.receive(body, authHeader);
    } catch {
      return NextResponse.json({ error: 'invalid_signature' }, { status: 401 });
    }

    const roomName = event.room?.name;
    switch (event.event) {
      case 'room_finished':
        if (roomName) {
          // Only mark completed if the session actually reached 'live'. A
          // room that finishes without anyone ever joining stays in its
          // prior status (pending/confirmed) so cron/analytics can flag
          // it as a no-show separately.
          const { data: completed } = await supabase
            .from('sessions')
            .update({ status: 'completed' })
            .eq('livekit_room_name', roomName)
            .eq('status', 'live')
            .select('id, student_id, tutor_id')
            .maybeSingle();

          if (completed) {
            // Send the review-request email out-of-band. `void` so a slow
            // Resend never keeps LiveKit's webhook connection open.
            void requestReview(completed);
          } else {
            // 0-row update — the session was moved off `live` by another
            // path (cron marked no_show, admin cancelled, or the webhook
            // is replayed after we already completed). This isn't
            // necessarily a bug, but knowing which sessions this hits
            // helps catch the webhook-vs-cron race.
            log.info(
              { roomName },
              'room_finished: no live session matched (already completed/cancelled/no_show)',
            );
          }
        }
        break;
      case 'participant_joined':
        if (roomName && event.id && event.participant?.identity) {
          await recordJoin(roomName, event.id, event.participant.identity, event.createdAt);
        }
        break;
      case 'participant_left':
        if (roomName && event.participant?.identity) {
          await recordLeave(roomName, event.participant.identity, event.createdAt);
        }
        break;
      default:
        break;
    }

    return NextResponse.json({ ok: true });
  } catch (err) {
    return apiError(err);
  }
}

/**
 * LiveKit gives us `createdAt` as bigint microseconds since epoch (unix seconds
 * in most SDK versions — the field name is generic across message types).
 * Convert defensively: if the number looks like microseconds we scale, if it
 * looks like seconds we widen to milliseconds, if it's absent we fall back to
 * `now`. Landing an accurate timestamp matters because session_attendance
 * derives duration_sec from it.
 */
function tsToDate(raw: bigint | number | undefined): Date {
  if (raw === undefined || raw === null) return new Date();
  const n = typeof raw === 'bigint' ? Number(raw) : raw;
  if (!Number.isFinite(n) || n === 0) return new Date();
  // Heuristic: 1e15+ = microseconds; 1e12+ = milliseconds; smaller = seconds.
  if (n > 1e15) return new Date(n / 1000);
  if (n > 1e12) return new Date(n);
  return new Date(n * 1000);
}

/**
 * Insert an attendance row for a participant_joined webhook. Idempotent via
 * the unique index on event_id — if LiveKit retries the same event, the
 * second insert is a no-op and we don't get ghost rows.
 *
 * `identity` in our tokens is the users.id UUID — so no lookup is needed.
 * A row is only inserted for a session we know about (the join against
 * sessions.livekit_room_name resolves the session_id).
 */
async function recordJoin(
  roomName: string,
  eventId: string,
  identity: string,
  createdAt: bigint | number | undefined,
): Promise<void> {
  const { data: session } = await supabase
    .from('sessions')
    .select('id')
    .eq('livekit_room_name', roomName)
    .maybeSingle();
  if (!session) return;
  await supabase
    .from('session_attendance')
    .upsert(
      {
        session_id: session.id,
        user_id: identity,
        joined_at: tsToDate(createdAt).toISOString(),
        event_id: eventId,
      },
      { onConflict: 'event_id', ignoreDuplicates: true },
    )
    .then(
      (result) => {
        if (result.error) {
          log.warn({ err: result.error, roomName }, 'session_attendance write failed');
        }
      },
      (err) => log.warn({ err, roomName }, 'session_attendance write threw'),
    );
}

/**
 * Close the open attendance row for a participant_left event. We look up the
 * most recent row for (session_id, user_id) with `left_at IS NULL` — if a
 * participant somehow got two joins in a row (browser refresh mid-flight),
 * this still only closes the latest, leaving the earlier row untouched so
 * the audit trail is honest.
 */
async function recordLeave(
  roomName: string,
  identity: string,
  createdAt: bigint | number | undefined,
): Promise<void> {
  const { data: session } = await supabase
    .from('sessions')
    .select('id')
    .eq('livekit_room_name', roomName)
    .maybeSingle();
  if (!session) return;
  const { data: row } = await supabase
    .from('session_attendance')
    .select('id')
    .eq('session_id', session.id)
    .eq('user_id', identity)
    .is('left_at', null)
    .order('joined_at', { ascending: false })
    .limit(1)
    .maybeSingle();
  if (!row) return;
  await supabase
    .from('session_attendance')
    .update({ left_at: tsToDate(createdAt).toISOString() })
    .eq('id', row.id)
    .then(
      (result) => {
        if (result.error) {
          log.warn({ err: result.error, roomName }, 'session_attendance write failed');
        }
      },
      (err) => log.warn({ err, roomName }, 'session_attendance write threw'),
    );
}

async function requestReview(session: {
  id: string;
  student_id: string | null;
  tutor_id: string | null;
}) {
  try {
    if (!session.student_id || !session.tutor_id) return;
    const [{ data: student }, { data: tutor }] = await Promise.all([
      supabase.from('users').select('name, email').eq('id', session.student_id).maybeSingle(),
      supabase.from('users').select('name').eq('id', session.tutor_id).maybeSingle(),
    ]);
    if (!student?.email) return;

    const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || 'https://vielang.com';
    await sendEmail({
      kind: 'session.review_request',
      to: student.email,
      subject: `How was your VieLang session?`,
      template: ReviewRequestEmail({
        studentName: student.name || student.email.split('@')[0] || 'there',
        tutorName: tutor?.name || 'your tutor',
        reviewUrl: `${siteUrl}/my-sessions?review=${session.id}`,
      }),
      sessionId: session.id,
      userId: session.student_id,
    });
  } catch {
    /* email is out-of-band; notification_log tracks failures */
  }
}
