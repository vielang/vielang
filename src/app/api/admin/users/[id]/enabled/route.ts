import { type NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { setUserEnabled } from '@/lib/supabase';
import { recordAudit } from '@/lib/audit';

export const dynamic = 'force-dynamic';

// Reason is optional but capped to keep the login-page banner readable.
const bodySchema = z.object({
  enabled: z.boolean(),
  reason: z.string().trim().max(300).optional().nullable(),
});

/**
 * PUT /api/admin/users/[id]/enabled — flip a user account on or off.
 *
 * Disabled users see a 403 on every subsequent /api/* request and get a
 * login-page banner with the admin-supplied `reason`. Admins can't disable
 * themselves; that's an easy accidental lockout.
 */
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

    if (id === auth.user.id && !parsed.data.enabled) {
      return NextResponse.json({ error: 'cannot_self_disable' }, { status: 400 });
    }

    const user = await setUserEnabled(id, parsed.data.enabled, parsed.data.reason);

    // Trace the enable/disable for compliance — same table as other admin
    // mutations, actor identified by session.
    void recordAudit({
      actor: { id: auth.user.id, email: auth.user.email, role: auth.user.role },
      action: parsed.data.enabled ? 'user.enable' : 'user.disable',
      entity: 'users',
      entityId: id,
      after: {
        enabled: parsed.data.enabled,
        disabled_reason: parsed.data.enabled ? null : (parsed.data.reason ?? null),
      },
      request: req,
    });

    return NextResponse.json({ user });
  } catch (err) {
    return apiError(err);
  }
}
