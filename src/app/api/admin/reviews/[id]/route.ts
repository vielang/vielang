import { type NextRequest, NextResponse } from 'next/server';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { deleteReview } from '@/lib/supabase';
import { recordAudit } from '@/lib/audit';

export const dynamic = 'force-dynamic';

/**
 * DELETE /api/admin/reviews/[id] — moderation. A hard delete rather than a
 * soft "hidden" flag: the schema doesn't carry a hidden column and hiding
 * without a paper trail is worse UX than removing. `audit_log` keeps the
 * before-state so we can trace who removed what.
 *
 * The AFTER-DELETE trigger on `reviews` recomputes the tutor's rating_avg +
 * session_count automatically — no follow-up write required.
 */
export async function DELETE(_req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(_req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'admin');
    if (gate) return gate;

    const { id } = await params;
    const removed = await deleteReview(id);
    if (!removed) {
      return NextResponse.json({ error: 'review_not_found' }, { status: 404 });
    }

    void recordAudit({
      actor: { id: auth.user.id, email: auth.user.email, role: auth.user.role },
      action: 'review.moderate',
      entity: 'reviews',
      entityId: id,
      metadata: { action: 'delete' },
      request: _req,
    });

    return NextResponse.json({ ok: true });
  } catch (err) {
    return apiError(err);
  }
}
