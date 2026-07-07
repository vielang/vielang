import { type NextRequest, NextResponse } from 'next/server';
import { supabase } from '@/lib/supabase';
import { createSupabaseServer } from '@/lib/supabase/server';
import { validateEnv } from '@/lib/env';

// Run boot-time env validation once, side-effectfully on first import of the
// authenticate() helper — every authenticated Route Handler goes through this
// module, so a fatal misconfig fails on the first request instead of silently
// 500-ing later. Idempotent inside validateEnv().
validateEnv();

export interface AuthUser {
  id: string;
  role: string;
  email: string;
  isDemo: boolean;
}

/**
 * Authenticate a request. Tries in order:
 *   1. Cookie-based Supabase session (via @supabase/ssr) — preferred.
 *   2. Authorization: Bearer <supabase_access_token> — legacy clients.
 *   3. X-Demo-User header — only honored outside production (E2E tests).
 */
export async function authenticate(
  req: NextRequest,
): Promise<{ user: AuthUser } | { response: NextResponse }> {
  // 1. Cookie-backed Supabase session — the standard SSR auth path.
  try {
    const cookieClient = await createSupabaseServer();
    const { data, error } = await cookieClient.auth.getUser();
    if (!error && data.user) {
      const userRow = await lookupUserByEmail(data.user.email || '');
      const blocked = disabledResponse(userRow);
      if (blocked) return { response: blocked };
      return {
        user: {
          id: userRow?.id || data.user.id,
          role: userRow?.role || 'user',
          email: data.user.email || '',
          isDemo: false,
        },
      };
    }
  } catch {
    /* fall through to header-based paths */
  }

  // 2. Authorization Bearer — Supabase access token (legacy clients)
  const authHeader = req.headers.get('authorization');
  if (authHeader?.startsWith('Bearer ')) {
    const token = authHeader.slice(7);
    try {
      const { data, error } = await supabase.auth.getUser(token);
      if (!error && data.user) {
        const userRow = await lookupUserByEmail(data.user.email || '');
        const blocked = disabledResponse(userRow);
        if (blocked) return { response: blocked };
        return {
          user: {
            id: userRow?.id || data.user.id,
            role: userRow?.role || 'user',
            email: data.user.email || '',
            isDemo: false,
          },
        };
      }
    } catch {
      /* fall through */
    }

    return { response: NextResponse.json({ error: 'Invalid token' }, { status: 401 }) };
  }

  // 3. Demo header — DEV ONLY. We previously allowed an opt-in
  // `ALLOW_DEMO_USER=true` escape hatch in production for E2E tests, but
  // that lets any caller forge X-Demo-User and become an admin (no
  // signature, no expiry). For E2E in prod-like envs, create a real test
  // account with a real session instead of relying on this header.
  const demoHeader = req.headers.get('x-demo-user');
  if (demoHeader) {
    if (process.env.NODE_ENV === 'production') {
      return {
        response: NextResponse.json(
          { error: 'Demo header not allowed in production' },
          { status: 401 },
        ),
      };
    }
    try {
      const demoUser = JSON.parse(demoHeader);
      if (!demoUser.id || !demoUser.role) {
        return {
          response: NextResponse.json({ error: 'Invalid demo user format' }, { status: 400 }),
        };
      }
      // If the demo id corresponds to a real users row that's been disabled,
      // honor it — otherwise admin can't actually verify the disable behavior
      // in dev. Demo ids without a matching row (pure E2E fixtures) pass
      // through unchanged.
      const { data: realRow } = await supabase
        .from('users')
        .select('enabled, disabled_reason')
        .eq('id', demoUser.id)
        .maybeSingle();
      const blocked = disabledResponse(realRow);
      if (blocked) return { response: blocked };
      return {
        user: {
          id: demoUser.id,
          role: demoUser.role,
          email: demoUser.email || '',
          isDemo: true,
        },
      };
    } catch {
      return {
        response: NextResponse.json({ error: 'Invalid X-Demo-User header' }, { status: 400 }),
      };
    }
  }

  return { response: NextResponse.json({ error: 'Authentication required' }, { status: 401 }) };
}

/**
 * Look up the canonical app user row by email. Email is used (not Supabase
 * UID) because seed rows exist before an auth user is provisioned.
 *
 * Returns `enabled` + `disabled_reason` too — the auth-gate response uses them
 * to 403 disabled users with a human-readable message.
 */
async function lookupUserByEmail(email: string) {
  if (!email) return null;
  const { data } = await supabase
    .from('users')
    .select('id, role, name, enabled, disabled_reason')
    .eq('email', email)
    .maybeSingle();
  return data;
}

/**
 * If the user row exists and is explicitly disabled, return a 403 response
 * carrying the admin-supplied reason. `code: 'ACCOUNT_DISABLED'` is a stable
 * identifier the login UI uses to distinguish "banned" from generic 403.
 */
function disabledResponse(row: any): NextResponse | null {
  if (row && row.enabled === false) {
    return NextResponse.json(
      {
        error: 'Account disabled',
        reason: row.disabled_reason || 'Please contact support.',
        code: 'ACCOUNT_DISABLED',
      },
      { status: 403 },
    );
  }
  return null;
}

/** Require specific roles. Admin always passes. */
export function requireRole(user: AuthUser, ...roles: string[]): NextResponse | null {
  if (user.role === 'admin' || roles.includes(user.role)) return null;
  return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 });
}

/**
 * Require ownership of a resource. Admin bypasses. Returns null on pass.
 *
 * `collection` is the Supabase table name. The ownership column is inferred:
 * `courses.tutor_id`, `sessions.tutor_id` OR `sessions.student_id`, etc.
 * For tables with a bespoke ownership shape, gate them inline in the route
 * instead of using this helper.
 */
export async function requireOwnership(
  user: AuthUser,
  table: string,
  ownershipColumn: string,
  docId: string,
): Promise<NextResponse | null> {
  if (user.role === 'admin') return null;
  if (!docId) return NextResponse.json({ error: 'Resource ID required' }, { status: 400 });

  try {
    const { data, error } = await supabase
      .from(table)
      .select(ownershipColumn)
      .eq('id', docId)
      .maybeSingle();
    if (error || !data) return NextResponse.json({ error: 'Resource not found' }, { status: 404 });
    if ((data as any)[ownershipColumn] && (data as any)[ownershipColumn] !== user.id) {
      return NextResponse.json({ error: 'You do not own this resource' }, { status: 403 });
    }
    return null;
  } catch {
    return NextResponse.json({ error: 'Failed to check ownership' }, { status: 500 });
  }
}
