import { type NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { TrackSource } from '@livekit/protocol';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { getSessionById, supabase } from '@/lib/supabase';
import { deleteRoom, listRoomParticipants, mutePublishedTrack } from '@/lib/livekit-admin';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';
import { recordAudit, type AuditAction } from '@/lib/audit';
import { childLogger } from '@/lib/logger';

const log = childLogger('livekit-room-action');

export const dynamic = 'force-dynamic';

const bodySchema = z.object({
  session_id: z.string().uuid(),
  action: z.enum(['mute_all', 'end_room']),
});

/**
 * POST /api/livekit/room-action
 *
 * Bulk moderation actions that operate on the whole room rather than a
 * single participant. Host-only. Two actions today:
 *
 *   • mute_all — iterate LiveKit's authoritative participant list, mute
 *     every microphone track that isn't the caller's own. The mute is a
 *     "hard mute" via RoomServiceClient.mutePublishedTrack — the target
 *     client can unmute themselves afterwards; if we ever want a
 *     force-mute-and-forbid we'd flip canPublish=false on top.
 *
 *   • end_room — RoomServiceClient.deleteRoom. Every connected client
 *     sees DisconnectReason.ROOM_DELETED, and LiveKit fires a
 *     `room_finished` webhook. Our webhook handler bumps
 *     `sessions.status='completed'` and closes open attendance rows, so
 *     this is atomic. No local DB write needed here.
 *
 * Neither action needs a per-participant confirm dialog — that's the UI's
 * job. The endpoint is idempotent-ish: mute_all on an already-quiet room
 * is a no-op; end_room on an already-deleted room throws NotFound which
 * we swallow to 200.
 */
export async function POST(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    // Reuse the moderation limiter — same burst profile (a host mashing
    // "mute all" during a troll incident) and same audit trail expectations.
    const rl = await rateLimit('livekit-moderate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const parsed = bodySchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }
    const { session_id, action } = parsed.data;

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

    let details: Record<string, unknown> = {};
    try {
      if (action === 'mute_all') {
        details = await muteAll(session.livekit_room_name, auth.user.id);
      } else {
        await endRoom(session.livekit_room_name);
        // Write completed immediately — don't wait for the webhook. The
        // webhook's room_finished handler also tries this update but its
        // WHERE status='live' filter makes it idempotent; whichever arrives
        // first wins. Writing here avoids the latency gap between room
        // deletion and webhook delivery (often 3-10s on local Docker).
        await supabase
          .from('sessions')
          .update({ status: 'completed' })
          .eq('id', session_id)
          .eq('status', 'live');
      }
    } catch (err) {
      const message = (err as Error)?.message || 'livekit_error';
      if (message === 'livekit_not_configured') {
        return NextResponse.json({ error: message }, { status: 503 });
      }
      log.warn({ err, action, sessionId: session_id }, 'room action failed');
      return NextResponse.json({ error: 'livekit_error', detail: message }, { status: 502 });
    }

    const auditAction: AuditAction =
      action === 'mute_all' ? 'session.mute_all' : 'session.end_room';
    void recordAudit({
      actor: { id: auth.user.id, email: auth.user.email, role: auth.user.role },
      action: auditAction,
      entity: 'sessions',
      entityId: session_id,
      metadata: details,
      request: req,
    });

    return NextResponse.json({ ok: true, ...details });
  } catch (err) {
    return apiError(err);
  }
}

/**
 * Iterate every participant in the room and mute their microphone track(s).
 * We skip the caller so a host who wants to speak while everyone else is
 * silenced isn't cut off, and we skip already-muted tracks to avoid a
 * round-trip that turns into a no-op.
 *
 * Returns a summary { targeted, muted, skipped, errors } for the audit
 * metadata so a later dispute can reconstruct what actually happened.
 */
async function muteAll(roomName: string, callerId: string): Promise<Record<string, unknown>> {
  const participants = await listRoomParticipants(roomName);
  let muted = 0;
  let skipped = 0;
  let errors = 0;
  const targets: string[] = [];
  for (const p of participants) {
    if (p.identity === callerId) continue;
    for (const track of p.tracks) {
      if (track.source !== TrackSource.MICROPHONE) continue;
      if (track.muted) {
        skipped++;
        continue;
      }
      try {
        await mutePublishedTrack(roomName, p.identity, track.sid, true);
        muted++;
        targets.push(p.identity);
      } catch (err) {
        errors++;
        log.warn({ err, identity: p.identity, sid: track.sid }, 'mute_all: track mute failed');
      }
    }
  }
  return { targeted: participants.length - 1, muted, skipped, errors, targets };
}

/**
 * Deletes the LiveKit room. If the room doesn't exist (already ended, or
 * webhook already ran), swallow as success — the caller's intent was "make
 * it be over", and it already is.
 */
async function endRoom(roomName: string): Promise<void> {
  try {
    await deleteRoom(roomName);
  } catch (err) {
    const msg = (err as Error)?.message ?? '';
    if (/does not exist|not[_\s-]?found/i.test(msg)) return;
    throw err;
  }
}
