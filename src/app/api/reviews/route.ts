import { type NextRequest, NextResponse } from 'next/server';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { addReview, getReviewForSession, getSessionById } from '@/lib/supabase';
import { createReviewSchema } from '@/lib/schemas/review';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

/**
 * POST /api/reviews
 *
 * Only the student who booked the session can review it, and only after the
 * session is completed. One review per session is enforced by both the
 * UNIQUE session_id constraint (defensive) and an up-front lookup here (nice
 * error message + shortcut before we call the DB).
 *
 * The AFTER-INSERT trigger on `reviews` recomputes tutor_profiles.rating_avg
 * + session_count automatically, so no follow-up write is needed here.
 */
export async function POST(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const rl = await rateLimit('review-create', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const parsed = createReviewSchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }
    const { session_id, rating, comment } = parsed.data;

    const session = await getSessionById(session_id);
    if (!session) return NextResponse.json({ error: 'session_not_found' }, { status: 404 });

    if (auth.user.id !== session.student_id) {
      return NextResponse.json({ error: 'only_student_can_review' }, { status: 403 });
    }
    if (session.status !== 'completed') {
      return NextResponse.json({ error: 'session_not_completed' }, { status: 400 });
    }
    if (!session.student_id || !session.tutor_id) {
      // Schema declares NOT NULL for both — if this row lost its participants
      // somehow, the review is meaningless. Guard so downstream code has
      // narrowed strings, and the DB stays in a coherent state.
      return NextResponse.json({ error: 'session_missing_participants' }, { status: 400 });
    }
    const existing = await getReviewForSession(session_id);
    if (existing) {
      return NextResponse.json({ error: 'already_reviewed' }, { status: 409 });
    }

    try {
      const review = await addReview({
        session_id,
        student_id: session.student_id,
        tutor_id: session.tutor_id,
        rating,
        comment: (comment ?? '').trim() || null,
      });
      return NextResponse.json({ review }, { status: 201 });
    } catch (err) {
      // Race: someone else won the unique constraint.
      if ((err as { code?: string } | null)?.code === '23505') {
        return NextResponse.json({ error: 'already_reviewed' }, { status: 409 });
      }
      throw err;
    }
  } catch (err) {
    return apiError(err);
  }
}
