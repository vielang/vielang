import { type NextRequest, NextResponse } from 'next/server';
import { AccessToken, TrackSource } from 'livekit-server-sdk';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase, getSessionById } from '@/lib/supabase';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';
import { childLogger } from '@/lib/logger';

const log = childLogger('livekit-token');

// Sources a student can publish: camera + mic only. Screen share is reserved
// for the tutor/admin so a disruptive learner can't hijack the classroom.
const STUDENT_SOURCES: TrackSource[] = [TrackSource.CAMERA, TrackSource.MICROPHONE];
const HOST_SOURCES: TrackSource[] = [
  TrackSource.CAMERA,
  TrackSource.MICROPHONE,
  TrackSource.SCREEN_SHARE,
  TrackSource.SCREEN_SHARE_AUDIO,
];

type RoomRole = 'tutor' | 'admin' | 'student';
type AdmissionState = 'admitted' | 'waiting';

export const dynamic = 'force-dynamic';

// Grace period after the scheduled end — participants can stay in the room
// briefly after the session officially ends before the webhook closes it.
// No early-join restriction: participants can enter any time before the end
// so they can verify LiveKit connectivity right after booking.
const JOIN_GRACE_MS = 15 * 60_000;

/**
 * POST /api/livekit/token
 *
 * Body: { session_id: string }
 * Returns: { token, url, room, identity }
 *
 * Contract:
 *  - Caller must be authenticated (Supabase cookie or X-Demo-User).
 *  - Caller must be the session's student, its tutor, or an admin.
 *  - Current time must be inside the join window.
 *  - The session must have a livekit_room_name — it's assigned on creation
 *    but this is a defensive check for older/hand-edited rows.
 *
 * On first join we bump the session status to 'live' (pending/confirmed
 * only). The webhook flips it to 'completed' when the room finishes.
 */
export async function POST(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const rl = await rateLimit('livekit-token', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const apiKey = process.env.LIVEKIT_API_KEY;
    const apiSecret = process.env.LIVEKIT_API_SECRET;
    const wsUrl = process.env.NEXT_PUBLIC_LIVEKIT_WS_URL;
    if (!apiKey || !apiSecret || !wsUrl) {
      return NextResponse.json({ error: 'livekit_not_configured' }, { status: 503 });
    }

    const body = await req.json().catch(() => ({}) as any);
    const sessionId = body?.session_id;
    if (typeof sessionId !== 'string' || !sessionId) {
      return NextResponse.json({ error: 'session_id_required' }, { status: 400 });
    }

    const session = await getSessionById(sessionId);
    if (!session) return NextResponse.json({ error: 'session_not_found' }, { status: 404 });
    if (!session.livekit_room_name) {
      return NextResponse.json({ error: 'room_not_provisioned' }, { status: 500 });
    }

    const uid = auth.user.id;
    let isParticipant = false;
    if (session.type === 'group') {
      // Group rooms: host tutor + admins get automatic access; anyone else
      // must have a row in session_participants (created by the join flow).
      if (uid === session.tutor_id || auth.user.role === 'admin') {
        isParticipant = true;
      } else {
        const { data: membership } = await supabase
          .from('session_participants')
          .select('user_id')
          .eq('session_id', session.id)
          .eq('user_id', uid)
          .maybeSingle();
        isParticipant = !!membership;
      }
    } else {
      isParticipant = uid === session.student_id || uid === session.tutor_id;
    }
    if (!isParticipant && auth.user.role !== 'admin') {
      return NextResponse.json({ error: 'not_a_participant' }, { status: 403 });
    }

    // Capacity enforcement for group sessions. Counts active LiveKit seats
    // (session_attendance rows where `left_at IS NULL`) — session_participants
    // is the *eligibility* list, not the presence list, so counting there
    // would treat everyone who was ever allowed in as still-here. Tutor and
    // admin bypass since they're hosts, not seat consumers. A user who's
    // already in the room (their own attendance row is open) always gets a
    // fresh token so tab reload / reconnect works even at capacity.
    if (session.type === 'group' && uid !== session.tutor_id && auth.user.role !== 'admin') {
      const capacity = session.capacity ?? 0;
      if (capacity > 0) {
        const [totalRes, mineRes] = await Promise.all([
          supabase
            .from('session_attendance')
            .select('*', { count: 'exact', head: true })
            .eq('session_id', session.id)
            .is('left_at', null),
          supabase
            .from('session_attendance')
            .select('id')
            .eq('session_id', session.id)
            .eq('user_id', uid)
            .is('left_at', null)
            .limit(1)
            .maybeSingle(),
        ]);
        const active = totalRes.count ?? 0;
        const alreadyInside = !!mineRes.data;
        if (active >= capacity && !alreadyInside) {
          return NextResponse.json({ error: 'session_full', capacity, active }, { status: 409 });
        }
      }
    }

    // Terminal states: nobody joins a cancelled/no-show room. Completed
    // rooms would be re-openable via LiveKit even after the webhook, so
    // reject early to keep post-mortem history clean.
    if (
      session.status === 'cancelled' ||
      session.status === 'no_show' ||
      session.status === 'completed'
    ) {
      return NextResponse.json(
        { error: 'session_closed', status: session.status },
        { status: 400 },
      );
    }

    const now = Date.now();
    const start = new Date(session.scheduled_at).getTime();
    const end = start + session.duration_min * 60_000;
    // Only block joining after the session has fully ended (+ grace window).
    // No lower bound: participants can join at any time before the end so
    // they can test the LiveKit connection immediately after booking.
    if (now > end + JOIN_GRACE_MS) {
      return NextResponse.json(
        {
          error: 'outside_join_window',
          scheduledAt: session.scheduled_at,
          durationMin: session.duration_min,
        },
        { status: 400 },
      );
    }

    // Resolve the caller's role in *this* session. Admin trumps tutor
    // (an admin joining a session they teach is still an admin for
    // moderation purposes). Everyone else is a student, including
    // group-session guests who joined via session_participants.
    let role: RoomRole = 'student';
    if (auth.user.role === 'admin') role = 'admin';
    else if (uid === session.tutor_id) role = 'tutor';

    // Waiting-room gate. Hosts always skip — they need to be inside the room
    // to admit anyone in the first place. Students land in `waiting` state
    // until a tutor admits them via /api/livekit/admit; the host client
    // filters remote participants by metadata.state to show the waiting
    // section in the host controls drawer.
    let state: AdmissionState = 'admitted';
    let queue: { rank: number; total: number } | null = null;
    if (role === 'student' && session.require_admission) {
      const { data: admission } = await supabase
        .from('session_admissions')
        .select('admitted_at, denied_at')
        .eq('session_id', session.id)
        .eq('user_id', uid)
        .maybeSingle();
      if (admission?.denied_at) {
        return NextResponse.json({ error: 'admission_denied' }, { status: 403 });
      }
      if (!admission?.admitted_at) {
        // Upsert the waiting row and AWAIT it — the queue snapshot below
        // reads back from this table, so a fire-and-forget write would
        // race the SELECT and mis-report the rank. If the write fails,
        // let the request fail loudly rather than issuing a token with
        // a bogus queue payload; the client can retry.
        const { error: admissionErr } = await supabase.from('session_admissions').upsert(
          {
            session_id: session.id,
            user_id: uid,
            requested_at: new Date().toISOString(),
          },
          { onConflict: 'session_id,user_id' },
        );
        if (admissionErr) throw admissionErr;
        state = 'waiting';

        // Snapshot the waiting queue for the client. Rank is 1-based ("you
        // are #3 of 5"). Cheap SELECT — the table is tiny and the index
        // idx_session_admissions_user covers this filter.
        const { data: pending } = await supabase
          .from('session_admissions')
          .select('user_id, requested_at')
          .eq('session_id', session.id)
          .is('admitted_at', null)
          .is('denied_at', null)
          .order('requested_at', { ascending: true });
        if (pending && pending.length > 0) {
          const idx = pending.findIndex((r: { user_id: string }) => r.user_id === uid);
          // -1 shouldn't happen (we just upserted our own row) but guard
          // anyway: report queue only when the caller is actually in the
          // list, otherwise skip rather than lie about position.
          if (idx >= 0) {
            queue = { rank: idx + 1, total: pending.length };
          }
        }
      }
    }

    const token = new AccessToken(apiKey, apiSecret, {
      identity: uid,
      name: auth.user.email || uid,
      ttl: '2h',
      // Metadata propagates to every other participant via participant.metadata
      // — the client uses it to badge roles and to hide/show host controls.
      metadata: JSON.stringify({ role, state }),
    });
    if (state === 'waiting') {
      // Waiting participants join the LiveKit room but are muted end-to-end:
      // they can't publish, can't subscribe. Their presence is only visible
      // to hosts (via metadata.state='waiting') so hosts can admit them.
      token.addGrant({
        room: session.livekit_room_name,
        roomJoin: true,
        canPublish: false,
        canPublishSources: [],
        canSubscribe: false,
        canPublishData: false,
      });
    } else {
      token.addGrant({
        room: session.livekit_room_name,
        roomJoin: true,
        canPublish: true,
        canPublishSources: role === 'student' ? STUDENT_SOURCES : HOST_SOURCES,
        canSubscribe: true,
        canPublishData: true,
      });
    }
    const jwt = await token.toJwt();

    // Best-effort status bump — but only once someone is actually admitted.
    // A student sitting in the waiting room shouldn't flip the session live
    // yet; that would confuse the no-show cron.
    if (state === 'admitted') {
      await supabase
        .from('sessions')
        .update({ status: 'live' })
        .eq('id', sessionId)
        .in('status', ['pending', 'confirmed'])
        .then(
          (result) => {
            if (result.error) {
              // Non-fatal: the token still went out, LiveKit's webhook will
              // move the row to `completed` when the room ends. Log so the
              // operator can spot a systemic issue (RLS misfire, deadlock).
              log.warn({ err: result.error, sessionId }, 'session status → live bump failed');
            }
          },
          (err) => log.warn({ err, sessionId }, 'session status → live bump threw'),
        );
    }

    return NextResponse.json({
      token: jwt,
      url: wsUrl,
      room: session.livekit_room_name,
      identity: uid,
      role,
      state,
      queue,
    });
  } catch (err) {
    return apiError(err);
  }
}
