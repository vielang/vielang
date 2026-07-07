import { type NextRequest, NextResponse } from 'next/server';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase, deleteMaterial, getCourseById } from '@/lib/supabase';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

// DELETE /api/materials/[id] — tutor of the parent course or admin.
export async function DELETE(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const rl = await rateLimit('material-mutate', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const { id } = await params;
    const { data: material } = await supabase
      .from('materials')
      .select('course_id')
      .eq('id', id)
      .maybeSingle();
    if (!material) return NextResponse.json({ error: 'not_found' }, { status: 404 });

    const course = await getCourseById(material.course_id);
    if (!course) return NextResponse.json({ error: 'course_not_found' }, { status: 404 });

    const isCourseTutor = auth.user.id === course.tutor_id;
    const isAdmin = auth.user.role === 'admin';
    if (!isCourseTutor && !isAdmin) {
      return NextResponse.json({ error: 'not_course_owner' }, { status: 403 });
    }

    const removed = await deleteMaterial(id);
    if (removed === 0) return NextResponse.json({ error: 'not_found' }, { status: 404 });
    return NextResponse.json({ ok: true });
  } catch (err) {
    return apiError(err);
  }
}
