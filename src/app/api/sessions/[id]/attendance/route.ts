import { type NextRequest, NextResponse } from 'next/server';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase, getSessionById } from '@/lib/supabase';

export const dynamic = 'force-dynamic';

/**
 * Same rule as /api/sessions/[id]/messages — inline instead of shared to
 * avoid pulling every messages import into this file. If a third caller
 * needs it we lift to a lib helper.
 */
async function isSessionParticipant(
  session: {
    id: string;
    type: 'private' | 'group';
    student_id: string | null;
    tutor_id: string | null;
  },
  uid: string,
  role: string,
): Promise<boolean> {
  if (role === 'admin') return true;
  if (uid === session.tutor_id) return true;
  if (session.type === 'private') return uid === session.student_id;
  const { data } = await supabase
    .from('session_participants')
    .select('user_id')
    .eq('session_id', session.id)
    .eq('user_id', uid)
    .maybeSingle();
  return !!data;
}

interface RawAttendanceRow {
  user_id: string;
  joined_at: string;
  left_at: string | null;
  duration_sec: number | null;
  user: { name: string | null; avatar: string | null } | null;
}

/**
 * GET /api/sessions/[id]/attendance
 *
 * Returns one aggregated row per user — first_joined_at, last_left_at,
 * total_duration_sec, event_count. Rows with a null left_at (participant is
 * still connected) contribute 0 to the total but stay in the list so the UI
 * can show "Still here" instead of hiding them.
 *
 * Aggregation is done in JS rather than SQL to keep the migration lightweight
 * — the row counts are always tiny (a handful of participants times a handful
 * of join events each). If it ever grows we can push into a view.
 */
export async function GET(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const { id } = await params;
    const session = await getSessionById(id);
    if (!session) return NextResponse.json({ error: 'session_not_found' }, { status: 404 });

    if (!(await isSessionParticipant(session, auth.user.id, auth.user.role))) {
      return NextResponse.json({ error: 'forbidden' }, { status: 403 });
    }

    const { data, error } = await supabase
      .from('session_attendance')
      .select(
        `
        user_id, joined_at, left_at, duration_sec,
        user:users!session_attendance_user_id_fkey(name, avatar)
      `,
      )
      .eq('session_id', id)
      .order('joined_at', { ascending: true });
    if (error) throw error;

    const rows = (data || []) as unknown as RawAttendanceRow[];
    const byUser = new Map<
      string,
      {
        user_id: string;
        user_name: string | null;
        avatar: string | null;
        first_joined_at: string;
        last_left_at: string | null;
        total_duration_sec: number;
        event_count: number;
        still_present: boolean;
      }
    >();
    for (const r of rows) {
      const existing = byUser.get(r.user_id);
      const dur = r.duration_sec ?? 0;
      const stillHere = r.left_at === null;
      if (!existing) {
        byUser.set(r.user_id, {
          user_id: r.user_id,
          user_name: r.user?.name ?? null,
          avatar: r.user?.avatar ?? null,
          first_joined_at: r.joined_at,
          last_left_at: r.left_at,
          total_duration_sec: dur,
          event_count: 1,
          still_present: stillHere,
        });
      } else {
        existing.total_duration_sec += dur;
        existing.event_count += 1;
        // joined_at is sorted ASC so the last row we see is the newest left_at.
        // A still-open row wins over any prior closed row for the "still_present"
        // flag — a rejoin without a leave means they're currently in the room.
        if (stillHere) existing.still_present = true;
        if (r.left_at) existing.last_left_at = r.left_at;
      }
    }

    return NextResponse.json({
      attendance: Array.from(byUser.values()).sort((a, b) =>
        a.first_joined_at.localeCompare(b.first_joined_at),
      ),
    });
  } catch (err) {
    return apiError(err);
  }
}
