import { type NextRequest, NextResponse } from 'next/server';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { updateTutorProfile } from '@/lib/supabase';
import { tutorProfilePatchSchema } from '@/lib/schemas/tutor';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

// PATCH /api/tutor/profile — tutor updates their own bio, avatar, rate,
// specialties, certs, etc. Admin-only fields (is_approved, rating_avg,
// session_count, approved_at) are intentionally not exposed here.
export async function PATCH(req: NextRequest) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'tutor');
    if (gate) return gate;

    const rl = await rateLimit('profile-mutate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const parsed = tutorProfilePatchSchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }

    const { name, bio, avatar, ...profilePatch } = parsed.data;
    const userPatch: { name?: string; bio?: string; avatar?: string | null } = {};
    if (name !== undefined) userPatch.name = name;
    if (bio !== undefined) userPatch.bio = bio ?? '';
    if (avatar !== undefined) userPatch.avatar = avatar;

    const result = await updateTutorProfile(auth.user.id, userPatch, profilePatch);
    return NextResponse.json(result);
  } catch (err) {
    return apiError(err);
  }
}
