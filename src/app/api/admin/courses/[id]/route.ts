import { type NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { authenticate, requireRole } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { setCoursePublished, deleteCourse, getCourseById } from '@/lib/supabase';
import { recordAudit } from '@/lib/audit';

export const dynamic = 'force-dynamic';

const patchSchema = z.object({
  is_published: z.boolean(),
});

/**
 * PATCH /api/admin/courses/[id] — flip a course between draft and published.
 * Admin-only; the tutor's own course edit UI has its own path.
 */
export async function PATCH(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'admin');
    if (gate) return gate;

    const { id } = await params;
    const parsed = patchSchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }

    // Snapshot before-state for the audit log — helps a post-mortem see
    // whether an unpublish was an accident or a policy call.
    const before = await getCourseById(id);
    if (!before) return NextResponse.json({ error: 'course_not_found' }, { status: 404 });

    const course = await setCoursePublished(id, parsed.data.is_published);
    if (!course) return NextResponse.json({ error: 'course_not_found' }, { status: 404 });

    void recordAudit({
      actor: { id: auth.user.id, email: auth.user.email, role: auth.user.role },
      action: parsed.data.is_published ? 'course.publish' : 'course.unpublish',
      entity: 'courses',
      entityId: id,
      before: { is_published: before.is_published },
      after: { is_published: parsed.data.is_published },
      request: req,
    });

    return NextResponse.json({ course });
  } catch (err) {
    return apiError(err);
  }
}

/**
 * DELETE /api/admin/courses/[id] — hard delete. Foreign keys on sessions +
 * materials use ON DELETE SET NULL / CASCADE — the session history isn't
 * lost, just decoupled from the course.
 */
export async function DELETE(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;
    const gate = requireRole(auth.user, 'admin');
    if (gate) return gate;

    const { id } = await params;
    const before = await getCourseById(id);
    if (!before) return NextResponse.json({ error: 'course_not_found' }, { status: 404 });

    const removed = await deleteCourse(id);
    if (!removed) return NextResponse.json({ error: 'course_not_found' }, { status: 404 });

    void recordAudit({
      actor: { id: auth.user.id, email: auth.user.email, role: auth.user.role },
      action: 'course.delete',
      entity: 'courses',
      entityId: id,
      before: {
        title_en: before.title_en,
        tutor_id: before.tutor_id,
        is_published: before.is_published,
      },
      request: req,
    });

    return NextResponse.json({ ok: true });
  } catch (err) {
    return apiError(err);
  }
}
