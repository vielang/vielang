import { type NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { TrackSource } from 'livekit-server-sdk';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase, getSessionById } from '@/lib/supabase';
import {
  isParticipantNotFoundError,
  removeParticipant,
  updateParticipantAccess,
} from '@/lib/livekit-admin';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';
import { childLogger } from '@/lib/logger';
import { recordAudit } from '@/lib/audit';

const log = childLogger('livekit-admit');

export const dynamic = 'force-dynamic';

const bodySchema = z.object({
  session_id: z.string().uuid(),
  target_identity: z.string().uuid(),
  decision: z.enum(['admit', 'deny']),
});

// Same student grant we hand out at the top of /api/livekit/token — kept in
// sync manually rather than shared, since exposing the constant from the
// token file would drag its runtime-only imports (AccessToken) into every
// caller. Small enough to duplicate.
const STUDENT_SOURCES = [TrackSource.CAMERA, TrackSource.MICROPHONE];

// Postgres check_violation SQLSTATE. session_admissions has a XOR constraint
// (`admitted_at IS NULL OR denied_at IS NULL`), so two hosts firing opposing
// decisions on the same row race — one wins, the other's UPDATE trips this
// code. We translate it to 409 for the loser's client.
const PG_CHECK_VIOLATION = '23514';

/**
 * POST /api/livekit/admit
 *
 * Host-only endpoint for waiting-room decisions.
 *
 * Ordering (both actions): LiveKit call FIRST, DB commit SECOND.
 * Rationale: if LiveKit fails, we want the DB row untouched so the host can
 * retry cleanly. If the DB write fails after LiveKit succeeded, the DB is
 * the more recoverable side — the host can retry to write the same row.
 *
 * Missing-participant handling: if the target has disconnected before the
 * decision lands, LiveKit responds with a `not found` error. We treat this
 * as benign — for admit, the DB row is still written so the student can
 * rejoin from `state=admitted`; for deny, the DB row is written so the
 * student's next token attempt is rejected. Both surface a `was_disconnected`
 * flag on the response so the UI can toast the difference.
 *
 * Concurrent decisions on the same row: the XOR check constraint traps a
 * losing UPDATE at commit time. We map PG error 23514 to a friendly 409
 * `admission_already_decided` instead of a bare 500.
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
    const { session_id, target_identity, decision } = parsed.data;

    const session = await getSessionById(session_id);
    if (!session) return NextResponse.json({ error: 'session_not_found' }, { status: 404 });
    if (!session.livekit_room_name) {
      return NextResponse.json({ error: 'room_not_provisioned' }, { status: 500 });
    }
    if (!session.require_admission) {
      return NextResponse.json({ error: 'admission_not_required' }, { status: 400 });
    }

    const isTutor = auth.user.id === session.tutor_id;
    const isAdmin = auth.user.role === 'admin';
    if (!isTutor && !isAdmin) {
      return NextResponse.json({ error: 'forbidden' }, { status: 403 });
    }
    if (target_identity === auth.user.id) {
      return NextResponse.json({ error: 'self_target' }, { status: 403 });
    }

    const { data: current } = await supabase
      .from('session_admissions')
      .select('admitted_at, denied_at')
      .eq('session_id', session_id)
      .eq('user_id', target_identity)
      .maybeSingle();
    if (!current) {
      return NextResponse.json({ error: 'no_pending_admission' }, { status: 404 });
    }
    if (current.admitted_at || current.denied_at) {
      return NextResponse.json({ error: 'admission_already_decided' }, { status: 409 });
    }

    const actor = { id: auth.user.id, email: auth.user.email, role: auth.user.role };
    return decision === 'admit'
      ? admit({ session, target_identity, admittedBy: auth.user.id, actor, req })
      : deny({ session, target_identity, deniedBy: auth.user.id, actor, req });
  } catch (err) {
    return apiError(err);
  }
}

async function admit(input: {
  session: { id: string; livekit_room_name: string | null };
  target_identity: string;
  admittedBy: string;
  actor: { id: string; email?: string | null; role: string };
  req: NextRequest;
}) {
  const roomName = input.session.livekit_room_name!;
  let wasDisconnected = false;

  try {
    await updateParticipantAccess(roomName, input.target_identity, {
      metadata: JSON.stringify({ role: 'student', state: 'admitted' }),
      permission: {
        canPublish: true,
        canSubscribe: true,
        canPublishData: true,
        canPublishSources: STUDENT_SOURCES,
      },
    });
  } catch (err) {
    if (isParticipantNotFoundError(err)) {
      // Student left before we could grant them access. Fine — we still mark
      // them admitted so they can rejoin without waiting again.
      wasDisconnected = true;
    } else {
      const message = (err as Error)?.message || 'livekit_error';
      if (message === 'livekit_not_configured') {
        return NextResponse.json({ error: message }, { status: 503 });
      }
      return NextResponse.json({ error: 'livekit_error', detail: message }, { status: 502 });
    }
  }

  const nowIso = new Date().toISOString();
  const { error: updErr } = await supabase
    .from('session_admissions')
    .update({ admitted_at: nowIso, admitted_by: input.admittedBy })
    .eq('session_id', input.session.id)
    .eq('user_id', input.target_identity)
    // Belt-and-braces: only touch a row that's still undecided. Prevents a
    // last-writer-wins overwrite if the row was denied in the sliver between
    // our own read and this UPDATE.
    .is('admitted_at', null)
    .is('denied_at', null);
  if (updErr) {
    const code = (updErr as { code?: string }).code;
    if (code === PG_CHECK_VIOLATION) {
      return NextResponse.json({ error: 'admission_already_decided' }, { status: 409 });
    }
    throw updErr;
  }

  void recordAudit({
    actor: input.actor,
    action: 'session.admit',
    entity: 'sessions',
    entityId: input.session.id,
    metadata: {
      user_id: input.target_identity,
      was_disconnected: wasDisconnected,
    },
    request: input.req,
  });

  // Bump session to 'live' only if it was still pre-live. Ignore failures —
  // this is a status hint, not the source of truth (LiveKit webhook does the
  // authoritative transition to 'completed').
  await supabase
    .from('sessions')
    .update({ status: 'live' })
    .eq('id', input.session.id)
    .in('status', ['pending', 'confirmed'])
    .then(
      (result) => {
        if (result.error) {
          log.warn(
            { err: result.error, sessionId: input.session.id },
            'session status → live bump failed after admit',
          );
        }
      },
      (err) =>
        log.warn(
          { err, sessionId: input.session.id },
          'session status → live bump threw after admit',
        ),
    );

  return NextResponse.json({ ok: true, decision: 'admit', was_disconnected: wasDisconnected });
}

async function deny(input: {
  session: { id: string; livekit_room_name: string | null };
  target_identity: string;
  deniedBy: string;
  actor: { id: string; email?: string | null; role: string };
  req: NextRequest;
}) {
  const roomName = input.session.livekit_room_name!;
  let wasDisconnected = false;

  try {
    await removeParticipant(roomName, input.target_identity);
  } catch (err) {
    if (isParticipantNotFoundError(err)) {
      wasDisconnected = true;
    } else {
      const message = (err as Error)?.message || 'livekit_error';
      if (message === 'livekit_not_configured') {
        return NextResponse.json({ error: message }, { status: 503 });
      }
      return NextResponse.json({ error: 'livekit_error', detail: message }, { status: 502 });
    }
  }

  const nowIso = new Date().toISOString();
  const { error: denyErr } = await supabase
    .from('session_admissions')
    .update({ denied_at: nowIso, denied_by: input.deniedBy })
    .eq('session_id', input.session.id)
    .eq('user_id', input.target_identity)
    .is('admitted_at', null)
    .is('denied_at', null);
  if (denyErr) {
    const code = (denyErr as { code?: string }).code;
    if (code === PG_CHECK_VIOLATION) {
      return NextResponse.json({ error: 'admission_already_decided' }, { status: 409 });
    }
    throw denyErr;
  }

  void recordAudit({
    actor: input.actor,
    action: 'session.deny',
    entity: 'sessions',
    entityId: input.session.id,
    metadata: {
      user_id: input.target_identity,
      was_disconnected: wasDisconnected,
    },
    request: input.req,
  });

  return NextResponse.json({ ok: true, decision: 'deny', was_disconnected: wasDisconnected });
}
