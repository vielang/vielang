import { type NextRequest, NextResponse } from 'next/server';
import { apiError } from '@/lib/api-errors';
import { supabase } from '@/lib/supabase';
import { authenticate, requireRole } from '@/lib/auth-server';
import {
  ADMIN_USER,
  TUTORS,
  STUDENTS,
  AVAILABILITY,
  COURSES,
  BANNERS,
  GROUP_SESSIONS,
  NEWS,
} from '@/lib/seed-data';

export const dynamic = 'force-dynamic';

// Populate a fresh VieLang Supabase project with demo data. Idempotent —
// re-running upserts by primary key so it's safe to invoke repeatedly during
// development. Not intended for production data loads.
export async function POST(req: NextRequest) {
  // Auth-gate. In dev the admin@vielang.com row may not exist yet; allow the
  // seed to run if we can't authenticate and the DB has zero users (bootstrap).
  const auth = await authenticate(req);
  if ('response' in auth) {
    const { count } = await supabase.from('users').select('*', { count: 'exact', head: true });
    if ((count ?? 0) > 0) return auth.response;
  } else {
    const roleErr = requireRole(auth.user, 'admin');
    if (roleErr) return roleErr;
  }

  try {
    // 1. Users (admin + tutors + students)
    const userRows = [
      { ...ADMIN_USER },
      ...TUTORS.map((t) => ({
        id: t.id,
        email: t.email,
        name: t.name,
        role: 'tutor' as const,
        avatar: t.avatar,
        bio: t.bio,
      })),
      ...STUDENTS.map((s) => ({
        id: s.id,
        email: s.email,
        name: s.name,
        role: 'user' as const,
      })),
    ];
    const { error: usersErr } = await supabase.from('users').upsert(userRows, { onConflict: 'id' });
    if (usersErr) throw usersErr;

    // 2. Tutor profiles
    const profileRows = TUTORS.map((t) => ({ user_id: t.id, ...t.profile }));
    const { error: profErr } = await supabase
      .from('tutor_profiles')
      .upsert(profileRows, { onConflict: 'user_id' });
    if (profErr) throw profErr;

    // 3. Availability — wipe + re-insert (no natural PK to upsert against
    //    without hashing the composite key, and there are only ~8 rows)
    for (const t of TUTORS) {
      await supabase.from('availability').delete().eq('tutor_id', t.id);
    }
    const { error: availErr } = await supabase.from('availability').insert(AVAILABILITY);
    if (availErr) throw availErr;

    // 4. Courses
    const { error: coursesErr } = await supabase
      .from('courses')
      .upsert(COURSES, { onConflict: 'id' });
    if (coursesErr) throw coursesErr;

    // 5. Banners
    const { error: bannersErr } = await supabase
      .from('banners')
      .upsert(BANNERS, { onConflict: 'id' });
    if (bannersErr) throw bannersErr;

    // 6. News
    const { error: newsErr } = await supabase.from('news').upsert(NEWS, { onConflict: 'id' });
    if (newsErr) throw newsErr;

    // 7. Group free-talk sessions. Times are computed relative to "now" so the
    //    demo always renders "upcoming" rows on the homepage. Uses upsert-by-id
    //    so re-seeding just refreshes scheduled_at without duplicating.
    const now = Date.now();
    const groupRows = GROUP_SESSIONS.map((g) => {
      const tutorId = g.tutor_index === null ? null : (TUTORS[g.tutor_index]?.id ?? null);
      return {
        id: g.id,
        type: 'group' as const,
        student_id: null,
        tutor_id: tutorId,
        course_id: null,
        topic_en: g.topic_en,
        topic_vn: g.topic_vn,
        description: g.description,
        level: g.level,
        capacity: g.capacity,
        cover_emoji: g.cover_emoji,
        scheduled_at: new Date(now + g.offset_hours * 3_600_000).toISOString(),
        duration_min: g.duration_min,
        status: 'confirmed' as const,
        livekit_room_name: `demo-${g.id}`,
        price_vnd: 0,
      };
    });
    const { error: groupErr } = await supabase
      .from('sessions')
      .upsert(groupRows, { onConflict: 'id' });
    if (groupErr) throw groupErr;

    return NextResponse.json({
      ok: true,
      counts: {
        users: userRows.length,
        tutor_profiles: profileRows.length,
        availability: AVAILABILITY.length,
        courses: COURSES.length,
        banners: BANNERS.length,
        group_sessions: groupRows.length,
        news: NEWS.length,
      },
    });
  } catch (err) {
    return apiError(err);
  }
}
