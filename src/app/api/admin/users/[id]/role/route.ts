import { type NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { setUserRole } from '@/lib/supabase';

export const dynamic = 'force-dynamic';

const bodySchema = z.object({ role: z.enum(['user', 'tutor', 'admin']) });

export async function PUT(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'admin');
    if (gate) return gate;

    const { id } = await params;
    const parsed = bodySchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }
    // Prevent an admin from demoting themselves out of the admin role,
    // which is an easy way to lock everyone out. Requires a second admin
    // (which we'll add via SQL for now).
    if (id === auth.user.id && parsed.data.role !== 'admin') {
      return NextResponse.json({ error: 'cannot_self_demote' }, { status: 400 });
    }
    const user = await setUserRole(id, parsed.data.role);
    return NextResponse.json({ user });
  } catch (err) {
    return apiError(err);
  }
}
