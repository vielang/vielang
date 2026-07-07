import { type NextRequest, NextResponse } from 'next/server';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { deleteAvailabilitySlot } from '@/lib/supabase';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

// DELETE /api/availability/[id] — remove one of the caller's own slots.
// The helper double-guards ownership so no one deletes another tutor's row.
export async function DELETE(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'tutor');
    if (gate) return gate;

    const rl = await rateLimit('availability-mutate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const { id } = await params;
    const removed = await deleteAvailabilitySlot(id, auth.user.id);
    if (removed === 0) {
      return NextResponse.json({ error: 'not_found_or_not_owner' }, { status: 404 });
    }
    return NextResponse.json({ ok: true });
  } catch (err) {
    return apiError(err);
  }
}
