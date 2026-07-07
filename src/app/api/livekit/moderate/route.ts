import { type NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { getSessionById } from '@/lib/supabase';
import { mutePublishedTrack, removeParticipant } from '@/lib/livekit-admin';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';
import { recordAudit, type AuditAction } from '@/lib/audit';

export const dynamic = 'force-dynamic';

const bodySchema = z
  .object({
    session_id: z.string().uuid(),
    target_identity: z.string().min(1),
    action: z.enum(['mute', 'unmute', 'remove']),
    track_sid: z.string().min(1).optional(),
  })
  .refine((v) => v.action === 'remove' || !!v.track_sid, {
    path: ['track_sid'],
    message: 'track_sid required for mute/unmute',
  });

/**
 * POST /api/livekit/moderate
 *
 * Host-only moderation actions inside a live session. Auth chain:
 *  1. Caller must be authenticated.
 *  2. Session must exist and have a provisioned LiveKit room.
 *  3. Caller must be the session's tutor OR a platform admin.
 *     - For group sessions, `sessions.tutor_id` holds the host tutor id
 *       (created that way in createGroupSession), so the same check applies.
 *  4. Caller can never target themselves; only admins can moderate the
 *     tutor, to prevent a co-host free-for-all once we add more roles.
 *
 * On success returns { ok: true }. Errors: 400 validation_failed,
 * 401 unauthenticated, 403 forbidden / self_target / tutor_protected,
 * 404 session_not_found / room_not_provisioned, 503 livekit_not_configured,
 * 502 for any RoomServiceClient failure surfaced from LiveKit.
 */
export async function POST(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const rl = await rateLimit('livekit-moderate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const parsed = bodySchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }
    const { session_id, target_identity, action, track_sid } = parsed.data;

    const session = await getSessionById(session_id);
    if (!session) return NextResponse.json({ error: 'session_not_found' }, { status: 404 });
    if (!session.livekit_room_name) {
      return NextResponse.json({ error: 'room_not_provisioned' }, { status: 500 });
    }

    const isTutor = auth.user.id === session.tutor_id;
    const isAdmin = auth.user.role === 'admin';
    if (!isTutor && !isAdmin) {
      return NextResponse.json({ error: 'forbidden' }, { status: 403 });
    }
    if (target_identity === auth.user.id) {
      return NextResponse.json({ error: 'self_target' }, { status: 403 });
    }
    // Only admin may moderate the tutor — a co-host or student can't nuke
    // the person actually teaching the class.
    if (target_identity === session.tutor_id && !isAdmin) {
      return NextResponse.json({ error: 'tutor_protected' }, { status: 403 });
    }

    try {
      if (action === 'remove') {
        await removeParticipant(session.livekit_room_name, target_identity);
      } else {
        await mutePublishedTrack(
          session.livekit_room_name,
          target_identity,
          track_sid!,
          action === 'mute',
        );
      }
    } catch (err) {
      const message = (err as Error)?.message || 'livekit_error';
      if (message === 'livekit_not_configured') {
        return NextResponse.json({ error: message }, { status: 503 });
      }
      // Bubble the raw LiveKit message as detail — usually "participant
      // does not exist" or "track not found". 502 = we spoke to the
      // upstream and it refused.
      return NextResponse.json({ error: 'livekit_error', detail: message }, { status: 502 });
    }

    const auditAction: AuditAction =
      action === 'remove'
        ? 'session.remove'
        : action === 'mute'
          ? 'session.mute'
          : 'session.unmute';
    void recordAudit({
      actor: { id: auth.user.id, email: auth.user.email, role: auth.user.role },
      action: auditAction,
      entity: 'sessions',
      entityId: session_id,
      metadata: {
        user_id: target_identity,
        ...(track_sid ? { track_sid } : {}),
      },
      request: req,
    });

    return NextResponse.json({ ok: true });
  } catch (err) {
    return apiError(err);
  }
}
