import { type NextRequest, NextResponse } from 'next/server';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { setTutorApproval } from '@/lib/supabase';

export const dynamic = 'force-dynamic';

export async function PUT(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'admin');
    if (gate) return gate;

    const { id } = await params;
    const tutor = await setTutorApproval(id, false);
    if (!tutor) return NextResponse.json({ error: 'tutor_profile_not_found' }, { status: 404 });
    return NextResponse.json({ tutor });
  } catch (err) {
    return apiError(err);
  }
}
