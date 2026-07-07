import { type NextRequest, NextResponse } from 'next/server';
import { getAvailableSlots, resolveCourseForBooking } from '@/lib/booking';
import { apiError } from '@/lib/api-errors';
import { supabase } from '@/lib/supabase';

export const dynamic = 'force-dynamic';

/**
 * Compute bookable slots for a tutor + course over a date range.
 *
 * Query params:
 *   courseId  (required) — needed to know the session length
 *   from      ISO date, default = today
 *   to        ISO date, default = from + 14 days
 */
export async function GET(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const { id: tutorId } = await params;
    const sp = req.nextUrl.searchParams;
    const courseId = sp.get('courseId');
    if (!courseId) {
      return NextResponse.json({ error: 'courseId required' }, { status: 400 });
    }

    const course = await resolveCourseForBooking(courseId, tutorId);
    if (!course) {
      return NextResponse.json({ error: 'course not found for tutor' }, { status: 404 });
    }

    const now = new Date();
    const from = sp.get('from') ? new Date(sp.get('from')!) : now;
    const to = sp.get('to')
      ? new Date(sp.get('to')!)
      : new Date(from.getTime() + 14 * 24 * 60 * 60_000);
    // Clamp the range so a rogue caller can't tie up the DB with a year-long
    // slot generation. 60 days is comfortably beyond any reasonable UX.
    const maxTo = new Date(from.getTime() + 60 * 24 * 60 * 60_000);
    const clampedTo = to > maxTo ? maxTo : to;

    // Resolve the tutor's own timezone — rules were authored in it, so slot
    // generation must interpret them the same way. Callers can override the
    // display timezone via `?tz=` (student's browser zone) for label
    // formatting; when omitted, labels stay in the tutor's calendar.
    const { data: tutorRow } = await supabase
      .from('users')
      .select('timezone')
      .eq('id', tutorId)
      .maybeSingle();
    const tutorTimezone = (tutorRow?.timezone as string | undefined)?.trim() || 'Asia/Ho_Chi_Minh';
    const displayTz = sp.get('tz')?.trim() || tutorTimezone;

    const slots = await getAvailableSlots({
      tutorId,
      from,
      to: clampedTo,
      durationMin: course.duration_min,
      tutorTimezone,
      displayTimezone: displayTz,
    });

    return NextResponse.json({
      courseId,
      durationMin: course.duration_min,
      slots,
    });
  } catch (err) {
    return apiError(err);
  }
}
