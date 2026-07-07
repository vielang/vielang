import { type NextRequest, NextResponse } from 'next/server';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import {
  addAvailabilitySlot,
  getAvailabilityForTutor,
  replaceAvailabilityForTutor,
} from '@/lib/supabase';
import { availabilitySchema, availabilityReplaceSchema } from '@/lib/schemas/tutor';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

// GET  /api/availability                  — list current tutor's own slots
// POST /api/availability { weekday, start, end } — add slot; overlaps rejected
//                                            by the (tutor_id, weekday,
//                                            start_time) unique constraint.
export async function GET(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'tutor');
    if (gate) return gate;
    const rows = await getAvailabilityForTutor(auth.user.id);
    return NextResponse.json({ availability: rows });
  } catch (err) {
    return apiError(err);
  }
}

// PUT /api/availability { slots: [...] } — atomic replace. Wipes the caller's
// existing schedule and re-inserts the payload. Used by the weekly grid
// editor where computing per-row diffs client-side would be more code than
// letting the server settle it all at once.
export async function PUT(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'tutor');
    if (gate) return gate;

    const rl = await rateLimit('availability-mutate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const parsed = availabilityReplaceSchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }

    try {
      const rows = await replaceAvailabilityForTutor(auth.user.id, parsed.data.slots);
      return NextResponse.json({ availability: rows });
    } catch (err) {
      if ((err as { code?: string } | null)?.code === '23505') {
        return NextResponse.json({ error: 'duplicate_slot' }, { status: 409 });
      }
      throw err;
    }
  } catch (err) {
    return apiError(err);
  }
}

export async function POST(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'tutor');
    if (gate) return gate;

    const rl = await rateLimit('availability-mutate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const parsed = availabilitySchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }

    try {
      const row = await addAvailabilitySlot({
        tutor_id: auth.user.id,
        weekday: parsed.data.weekday,
        start_time: parsed.data.start_time,
        end_time: parsed.data.end_time,
      });
      return NextResponse.json({ slot: row }, { status: 201 });
    } catch (err) {
      if ((err as { code?: string } | null)?.code === '23505') {
        return NextResponse.json({ error: 'slot_exists' }, { status: 409 });
      }
      throw err;
    }
  } catch (err) {
    return apiError(err);
  }
}
