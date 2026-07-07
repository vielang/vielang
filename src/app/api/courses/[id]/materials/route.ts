import { type NextRequest, NextResponse } from 'next/server';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import {
  addMaterial,
  getCourseById,
  getMaterialsForCourse,
  studentHasSessionForCourse,
} from '@/lib/supabase';
import { createMaterialSchema } from '@/lib/schemas/material';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

/**
 * GET /api/courses/[id]/materials
 *
 * Access rules:
 *   - Course tutor + admin: always allowed
 *   - Any other student: only if they have a confirmed/live/completed
 *     session tying them to the course
 *   - Anonymous: 401
 */
export async function GET(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const { id } = await params;
    const course = await getCourseById(id);
    if (!course) return NextResponse.json({ error: 'course_not_found' }, { status: 404 });

    const isCourseTutor = auth.user.id === course.tutor_id;
    const isAdmin = auth.user.role === 'admin';
    if (!isCourseTutor && !isAdmin) {
      const hasAccess = await studentHasSessionForCourse(auth.user.id, id);
      if (!hasAccess) return NextResponse.json({ error: 'no_access' }, { status: 403 });
    }

    const materials = await getMaterialsForCourse(id);
    return NextResponse.json({ materials });
  } catch (err) {
    return apiError(err);
  }
}

/**
 * POST /api/courses/[id]/materials — tutor of the course or admin only.
 */
export async function POST(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const rl = await rateLimit('material-mutate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const { id } = await params;
    const course = await getCourseById(id);
    if (!course) return NextResponse.json({ error: 'course_not_found' }, { status: 404 });

    const isCourseTutor = auth.user.id === course.tutor_id;
    const isAdmin = auth.user.role === 'admin';
    if (!isCourseTutor && !isAdmin) {
      return NextResponse.json({ error: 'not_course_owner' }, { status: 403 });
    }

    const parsed = createMaterialSchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }

    const material = await addMaterial({
      course_id: id,
      title: parsed.data.title,
      type: parsed.data.type,
      url: parsed.data.url,
      order_index: parsed.data.order_index,
    });
    return NextResponse.json({ material }, { status: 201 });
  } catch (err) {
    return apiError(err);
  }
}
