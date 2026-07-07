import { type NextRequest, NextResponse } from 'next/server';
import { authenticate } from '@/lib/auth-server';

export const dynamic = 'force-dynamic';

/**
 * Returns the canonical user record for the calling session. The client
 * uses this right after Supabase login to hydrate AuthContext with the
 * resolved row from `public.users` (role, id).
 */
export async function GET(req: NextRequest) {
  const auth = await authenticate(req);
  if ('response' in auth) return auth.response;
  return NextResponse.json({
    id: auth.user.id,
    role: auth.user.role,
    email: auth.user.email,
  });
}
