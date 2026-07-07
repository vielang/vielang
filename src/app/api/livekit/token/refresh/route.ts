import { type NextRequest, NextResponse } from 'next/server';
import { AccessToken, TrackSource } from 'livekit-server-sdk';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase, getSessionById } from '@/lib/supabase';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

// Same source lists as /api/livekit/token; kept in sync manually rather than
// shared so this route's imports stay minimal and independent.
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

/**
 * POST /api/livekit/token/refresh
 *
 * Body: { session_id }
 * Returns: { token, url, room, identity, role, state }
 *
 * Called by the client after ~1h50min so a session that runs past the 2h
 * TTL doesn't disconnect participants mid-lesson. Diverges from
 * /api/livekit/token in two ways:
 *
 *   1. **No join-window check.** By the time this endpoint is called the
 *      participant is already inside the LiveKit room — enforcing the
 *      15-minute window would kick them out of a long-running class the
 *      moment the token was reissued.
 *   2. **No status bump.** The session is already `live`; the initial
 *      token endpoint already did the pending → live transition.
 *
 * All the other checks (participant membership, terminal-status guard,
 * admission-denied lock) still apply — someone who was denied between
 * their initial token and this refresh should not get a new grant.
 */
export async function POST(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    // Same limiter bucket as the initial mint — refresh is rare per user
    // (once per ~2h) so it can't push anyone into throttle territory.
    const rl = await rateLimit('livekit-token', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const apiKey = process.env.LIVEKIT_API_KEY;
    const apiSecret = process.env.LIVEKIT_API_SECRET;
    const wsUrl = process.env.NEXT_PUBLIC_LIVEKIT_WS_URL;
    if (!apiKey || !apiSecret || !wsUrl) {
      return NextResponse.json({ error: 'livekit_not_configured' }, { status: 503 });
    }

    const body = (await req.json().catch(() => ({}))) as { session_id?: string };
    if (typeof body.session_id !== 'string' || !body.session_id) {
      return NextResponse.json({ error: 'session_id_required' }, { status: 400 });
    }

    const session = await getSessionById(body.session_id);
    if (!session) return NextResponse.json({ error: 'session_not_found' }, { status: 404 });
    if (!session.livekit_room_name) {
      return NextResponse.json({ error: 'room_not_provisioned' }, { status: 500 });
    }

    const uid = auth.user.id;
    let isParticipant = false;
    if (session.type === 'group') {
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

    // Terminal statuses shouldn't be refreshable. A completed / cancelled /
    // no_show session should force the client to fall out of the room, not
    // hand it a new token to keep churning.
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

    let role: RoomRole = 'student';
    if (auth.user.role === 'admin') role = 'admin';
    else if (uid === session.tutor_id) role = 'tutor';

    // Re-resolve admission state — if a host denied this student between
    // the last mint and this refresh, we want to bounce them out.
    let state: AdmissionState = 'admitted';
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
      if (!admission?.admitted_at) state = 'waiting';
    }

    const token = new AccessToken(apiKey, apiSecret, {
      identity: uid,
      name: auth.user.email || uid,
      ttl: '2h',
      metadata: JSON.stringify({ role, state }),
    });
    if (state === 'waiting') {
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

    return NextResponse.json({
      token: jwt,
      url: wsUrl,
      room: session.livekit_room_name,
      identity: uid,
      role,
      state,
    });
  } catch (err) {
    return apiError(err);
  }
}
