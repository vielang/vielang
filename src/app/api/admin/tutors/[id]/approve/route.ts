import { type NextRequest, NextResponse } from 'next/server';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { setTutorApproval } from '@/lib/supabase';
import { recordAudit } from '@/lib/audit';

export const dynamic = 'force-dynamic';

export async function PUT(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'admin');
    if (gate) return gate;

    const { id } = await params;
    const tutor = await setTutorApproval(id, true);
    if (!tutor) return NextResponse.json({ error: 'tutor_profile_not_found' }, { status: 404 });

    // Fire-and-forget — an audit-log write must never block the primary op.
    void recordAudit({
      actor: { id: auth.user.id, email: auth.user.email, role: auth.user.role },
      action: 'tutor.approve',
      entity: 'tutor_profiles',
      entityId: id,
      after: { is_approved: true },
      request: req,
    });

    return NextResponse.json({ tutor });
  } catch (err) {
    return apiError(err);
  }
}
