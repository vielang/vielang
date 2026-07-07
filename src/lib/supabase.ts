import { createClient, type SupabaseClient } from '@supabase/supabase-js';
import type {
  Course,
  Material,
  MaterialType,
  NewsItem,
  Review,
  Session,
  Tutor,
  TutorProfile,
  User,
} from './types';

// Lazy client — createClient throws on empty key, which would break Next.js
// build-time prerendering (e.g. /_not-found) when env isn't loaded. Defer
// construction to first use.
let _client: SupabaseClient | null = null;
function getClient(): SupabaseClient {
  if (_client) return _client;
  const url = process.env.SUPABASE_URL || process.env.NEXT_PUBLIC_SUPABASE_URL || '';
  const key = process.env.SUPABASE_SERVICE_ROLE_KEY || '';
  _client = createClient(url, key);
  return _client;
}

export const supabase = new Proxy({} as SupabaseClient, {
  get(_t, prop) {
    // Proxy has to dispatch to whatever method is accessed at runtime; the
    // return type depends on the property so a fully typed indexer would
    // require conditional types over every SupabaseClient method. Kept
    // narrow-cast here to a plain index type instead of `any`.
    return (getClient() as unknown as Record<PropertyKey, unknown>)[prop];
  },
});

// ---- Users ----

export async function getUserById(id: string): Promise<User | null> {
  const { data } = await supabase.from('users').select('*').eq('id', id).maybeSingle();
  return data as User | null;
}

export async function getUserByEmail(email: string): Promise<User | null> {
  if (!email) return null;
  const { data } = await supabase.from('users').select('*').eq('email', email).maybeSingle();
  return data as User | null;
}

export async function upsertUserFromAuth(input: {
  supabase_uid: string;
  email: string;
  name: string;
  avatar?: string | null;
}): Promise<User> {
  // Look up by email first — accounts predate supabase_uid on seed rows and
  // shouldn't fork when the same person signs in via OAuth for the first time.
  const existing = await getUserByEmail(input.email);
  if (existing) {
    if (!existing.supabase_uid) {
      const { data } = await supabase
        .from('users')
        .update({ supabase_uid: input.supabase_uid, avatar: input.avatar ?? existing.avatar })
        .eq('id', existing.id)
        .select('*')
        .single();
      return data as User;
    }
    return existing;
  }
  const { data, error } = await supabase
    .from('users')
    .insert({
      supabase_uid: input.supabase_uid,
      email: input.email,
      name: input.name,
      avatar: input.avatar ?? null,
      role: 'user',
    })
    .select('*')
    .single();
  if (error) throw error;
  return data as User;
}

// ---- Tutors ----

export interface TutorListFilters {
  specialty?: string;
  minRating?: number;
  maxPriceVnd?: number;
  sort?: 'rating' | 'price_asc' | 'price_desc' | 'experience' | 'newest';
}

export async function getApprovedTutors(filters?: TutorListFilters): Promise<Tutor[]> {
  // Filters that live on the profile need to be applied post-fetch — PostgREST
  // supports embedded-table filtering but not sorting by embedded columns in
  // combination with the parent's WHERE. Doing it in JS keeps the code simple
  // and stays fast at the scale we ever expect a discovery grid to handle.
  const { data, error } = await supabase
    .from('users')
    .select('*, profile:tutor_profiles!inner(*)')
    .eq('role', 'tutor')
    .eq('enabled', true)
    .eq('tutor_profiles.is_approved', true);
  if (error) throw error;
  let tutors = (data || []) as Tutor[];

  if (filters?.specialty) {
    const needle = filters.specialty.toLowerCase();
    tutors = tutors.filter((t) => t.profile.specialties.some((s) => s.toLowerCase() === needle));
  }
  if (typeof filters?.minRating === 'number') {
    tutors = tutors.filter((t) => (t.profile.rating_avg ?? 0) >= filters.minRating!);
  }
  if (typeof filters?.maxPriceVnd === 'number') {
    tutors = tutors.filter((t) => t.profile.hourly_rate_vnd <= filters.maxPriceVnd!);
  }

  switch (filters?.sort) {
    case 'rating':
      tutors.sort((a, b) => (b.profile.rating_avg ?? 0) - (a.profile.rating_avg ?? 0));
      break;
    case 'price_asc':
      tutors.sort((a, b) => a.profile.hourly_rate_vnd - b.profile.hourly_rate_vnd);
      break;
    case 'price_desc':
      tutors.sort((a, b) => b.profile.hourly_rate_vnd - a.profile.hourly_rate_vnd);
      break;
    case 'experience':
      tutors.sort((a, b) => b.profile.years_experience - a.profile.years_experience);
      break;
    case 'newest':
    default:
      tutors.sort((a, b) => (b.created_at || '').localeCompare(a.created_at || ''));
      break;
  }
  return tutors;
}

export async function getTutorById(id: string): Promise<Tutor | null> {
  const { data } = await supabase
    .from('users')
    .select('*, profile:tutor_profiles(*)')
    .eq('id', id)
    .eq('role', 'tutor')
    .maybeSingle();
  return data as Tutor | null;
}

export async function getAvailabilityForTutor(tutorId: string) {
  const { data, error } = await supabase
    .from('availability')
    .select('*')
    .eq('tutor_id', tutorId)
    .order('weekday', { ascending: true })
    .order('start_time', { ascending: true });
  if (error) throw error;
  return data || [];
}

// ---- Courses ----

export async function getPublishedCourses(filters?: {
  level?: string;
  category?: string;
  tutorId?: string;
}): Promise<Course[]> {
  let query = supabase.from('courses').select('*').eq('is_published', true);
  if (filters?.level) query = query.eq('level', filters.level);
  if (filters?.category) query = query.eq('category', filters.category);
  if (filters?.tutorId) query = query.eq('tutor_id', filters.tutorId);
  const { data, error } = await query.order('created_at', { ascending: false });
  if (error) throw error;
  return (data || []) as Course[];
}

export async function getCourseById(id: string): Promise<Course | null> {
  const { data } = await supabase.from('courses').select('*').eq('id', id).maybeSingle();
  return data as Course | null;
}

// Public course detail with the assigned tutor's summary joined in one round-
// trip. Returns null when the course doesn't exist OR isn't published (public
// route shouldn't reveal drafts). Admin views load via getCourseById + the
// admin-only listing.
export interface CourseDetail extends Course {
  tutor_name: string | null;
  tutor_avatar: string | null;
  tutor_bio: string | null;
  tutor_rating: number | null;
  tutor_hourly_rate_vnd: number | null;
}

export async function getPublishedCourseDetail(id: string): Promise<CourseDetail | null> {
  const { data } = await supabase
    .from('courses')
    .select(
      `
      *,
      tutor:users!courses_tutor_id_fkey(id, name, avatar, bio, profile:tutor_profiles(rating_avg, hourly_rate_vnd))
    `,
    )
    .eq('id', id)
    .eq('is_published', true)
    .maybeSingle();
  if (!data) return null;
  // Bio lives on `users.bio` (single column — no VN/EN split). The tutor
  // profile edit form writes it there; earlier queries here referenced
  // non-existent `tutor_profiles.bio_en/vn` columns and silently 404'd the
  // whole detail page.
  type CourseDetailRow = Course & {
    tutor: {
      name: string | null;
      avatar: string | null;
      bio: string | null;
      profile: {
        rating_avg: number | null;
        hourly_rate_vnd: number | null;
      } | null;
    } | null;
  };
  const row = data as CourseDetailRow;
  return {
    ...(row as Course),
    tutor_name: row.tutor?.name ?? null,
    tutor_avatar: row.tutor?.avatar ?? null,
    tutor_bio: row.tutor?.bio ?? null,
    tutor_rating: row.tutor?.profile?.rating_avg ?? null,
    tutor_hourly_rate_vnd: row.tutor?.profile?.hourly_rate_vnd ?? null,
  };
}

/**
 * All courses for a tutor — published AND draft. Powers the tutor's
 * MyCoursesSection where they need to see their own drafts before publishing.
 */
export async function getAllCoursesForTutor(tutorId: string): Promise<Course[]> {
  const { data, error } = await supabase
    .from('courses')
    .select('*')
    .eq('tutor_id', tutorId)
    .order('created_at', { ascending: false });
  if (error) throw error;
  return (data || []) as Course[];
}

/**
 * Admin course listing — joins the tutor name so the moderation table doesn't
 * need a follow-up round-trip. Includes draft (is_published=false) courses.
 */
export interface CourseForAdmin extends Course {
  tutor_name: string;
}

export async function getAllCoursesForAdmin(limit = 500): Promise<CourseForAdmin[]> {
  const { data, error } = await supabase
    .from('courses')
    .select(`*, tutor:users!courses_tutor_id_fkey(name)`)
    .order('created_at', { ascending: false })
    .limit(limit);
  if (error) throw error;
  type CourseAdminRow = Course & { tutor: { name: string | null } | null };
  return ((data ?? []) as CourseAdminRow[]).map((row) => ({
    ...(row as Course),
    tutor_name: row.tutor?.name ?? '',
  }));
}

/**
 * Toggle publish state. Returns the updated row or null when the course
 * doesn't exist.
 */
export async function setCoursePublished(
  courseId: string,
  is_published: boolean,
): Promise<Course | null> {
  const { data, error } = await supabase
    .from('courses')
    .update({ is_published })
    .eq('id', courseId)
    .select('*')
    .maybeSingle();
  if (error) throw error;
  return data as Course | null;
}

export async function deleteCourse(courseId: string): Promise<boolean> {
  const { error, count } = await supabase
    .from('courses')
    .delete({ count: 'exact' })
    .eq('id', courseId);
  if (error) throw error;
  return (count ?? 0) > 0;
}

// ---- Sessions ----

export async function getSessionsForUser(
  userId: string,
  role: 'user' | 'tutor',
): Promise<Session[]> {
  if (role === 'tutor') {
    // Tutors: everything they host (private + group they're assigned to).
    const { data, error } = await supabase
      .from('sessions')
      .select('*')
      .eq('tutor_id', userId)
      .order('scheduled_at', { ascending: true });
    if (error) throw error;
    return (data || []) as Session[];
  }

  // Students: their private bookings (student_id match) UNION any group
  // sessions they've booked (via session_participants). Two round-trips is
  // simpler and clearer than a stored function; performance is fine for the
  // per-user session count we expect.
  const [privateRes, participantRes] = await Promise.all([
    supabase
      .from('sessions')
      .select('*')
      .eq('student_id', userId)
      .order('scheduled_at', { ascending: true }),
    supabase.from('session_participants').select('session_id').eq('user_id', userId),
  ]);
  if (privateRes.error) throw privateRes.error;
  if (participantRes.error) throw participantRes.error;

  const groupIds = ((participantRes.data ?? []) as { session_id: string }[]).map(
    (p) => p.session_id,
  );
  let groupSessions: Session[] = [];
  if (groupIds.length > 0) {
    const { data, error } = await supabase
      .from('sessions')
      .select('*')
      .in('id', groupIds)
      .order('scheduled_at', { ascending: true });
    if (error) throw error;
    groupSessions = (data || []) as Session[];
  }

  const merged = [...((privateRes.data || []) as Session[]), ...groupSessions];
  merged.sort((a, b) => a.scheduled_at.localeCompare(b.scheduled_at));
  return merged;
}

// Sessions denormalized with the tutor/student/course names the list UI
// needs. Powers /my-sessions, the tutor + admin dashboards, and the public
// group session listing. For group rows student_name is empty; the list UI
// should render topic_en/topic_vn + participant_count instead.
export interface SessionListItem extends Session {
  tutor_name: string;
  tutor_avatar: string | null;
  student_name: string;
  course_title_vn: string | null;
  course_title_en: string | null;
  participant_count: number;
}

const SESSION_SELECT = `
  *,
  tutor:users!sessions_tutor_id_fkey(id, name, avatar),
  student:users!sessions_student_id_fkey(id, name),
  course:courses(title_vn, title_en),
  participants:session_participants(user_id)
`;

// Row shape returned by SESSION_SELECT (Session + joined nullable relations).
// Kept narrow so the surface we assert to is smaller than SupabaseClient's
// dynamic return; anything not in the select above simply isn't observable.
type SessionSelectRow = Session & {
  tutor: { id: string; name: string | null; avatar: string | null } | null;
  student: { id: string; name: string | null } | null;
  course: { title_vn: string | null; title_en: string | null } | null;
  participants: { user_id: string }[] | null;
};

const denormalize = (row: SessionSelectRow): SessionListItem => ({
  ...(row as Session),
  tutor_name: row.tutor?.name ?? '',
  tutor_avatar: row.tutor?.avatar ?? null,
  student_name: row.student?.name ?? '',
  course_title_vn: row.course?.title_vn ?? null,
  course_title_en: row.course?.title_en ?? null,
  participant_count: Array.isArray(row.participants) ? row.participants.length : 0,
});

export async function getSessionListForUser(
  userId: string,
  role: 'user' | 'tutor',
): Promise<SessionListItem[]> {
  if (role === 'tutor') {
    const { data, error } = await supabase
      .from('sessions')
      .select(SESSION_SELECT)
      .eq('tutor_id', userId)
      .order('scheduled_at', { ascending: true });
    if (error) throw error;
    return ((data ?? []) as SessionSelectRow[]).map(denormalize);
  }

  // Students: private bookings + group sessions they've joined.
  const [privateRes, participantRes] = await Promise.all([
    supabase.from('sessions').select(SESSION_SELECT).eq('student_id', userId),
    supabase.from('session_participants').select('session_id').eq('user_id', userId),
  ]);
  if (privateRes.error) throw privateRes.error;
  if (participantRes.error) throw participantRes.error;

  const groupIds = ((participantRes.data ?? []) as { session_id: string }[]).map(
    (p) => p.session_id,
  );
  let groupRows: SessionSelectRow[] = [];
  if (groupIds.length > 0) {
    const { data, error } = await supabase
      .from('sessions')
      .select(SESSION_SELECT)
      .in('id', groupIds);
    if (error) throw error;
    groupRows = (data ?? []) as SessionSelectRow[];
  }
  const merged = [...((privateRes.data ?? []) as SessionSelectRow[]), ...groupRows].map(
    denormalize,
  );
  merged.sort((a, b) => a.scheduled_at.localeCompare(b.scheduled_at));
  return merged;
}

export async function getSessionById(id: string): Promise<Session | null> {
  const { data } = await supabase.from('sessions').select('*').eq('id', id).maybeSingle();
  return data as Session | null;
}

// ---- News ----

export async function getPublishedNews(): Promise<NewsItem[]> {
  const { data, error } = await supabase
    .from('news')
    .select('*')
    .eq('is_published', true)
    .order('published_at', { ascending: false });
  if (error) throw error;
  return (data || []) as NewsItem[];
}

// ---- Reviews ----

export async function getReviewsForTutor(tutorId: string, limit = 20): Promise<Review[]> {
  const { data, error } = await supabase
    .from('reviews')
    .select('*')
    .eq('tutor_id', tutorId)
    .order('created_at', { ascending: false })
    .limit(limit);
  if (error) throw error;
  return (data || []) as Review[];
}

// Reviews joined with the student's public display name — powers the public
// tutor detail page so we don't leak private user fields but still show
// attribution.
export interface ReviewWithStudent extends Review {
  student_name: string;
  student_avatar: string | null;
}

export async function getReviewsWithStudentForTutor(
  tutorId: string,
  limit = 20,
): Promise<ReviewWithStudent[]> {
  const { data, error } = await supabase
    .from('reviews')
    .select(
      `
      *,
      student:users!reviews_student_id_fkey(name, avatar)
    `,
    )
    .eq('tutor_id', tutorId)
    .order('created_at', { ascending: false })
    .limit(limit);
  if (error) throw error;
  type ReviewWithStudentRow = Review & {
    student: { name: string | null; avatar: string | null } | null;
  };
  return ((data ?? []) as ReviewWithStudentRow[]).map((row) => ({
    ...(row as Review),
    student_name: row.student?.name ?? '',
    student_avatar: row.student?.avatar ?? null,
  }));
}

/** Existing review for the pair (session, student) — 1-per-session by design. */
export async function getReviewForSession(sessionId: string): Promise<Review | null> {
  const { data } = await supabase
    .from('reviews')
    .select('*')
    .eq('session_id', sessionId)
    .maybeSingle();
  return data as Review | null;
}

export async function addReview(input: {
  session_id: string;
  student_id: string;
  tutor_id: string;
  rating: number;
  comment: string | null;
}): Promise<Review> {
  const { data, error } = await supabase.from('reviews').insert(input).select('*').single();
  if (error) throw error;
  return data as Review;
}

/**
 * Admin-scoped review listing — joins the student + tutor display names so
 * moderation queues render without follow-up round-trips. Ordered newest
 * first; capped to a generous 500 rows (the moderation UI paginates client
 * side after that).
 */
export interface ReviewForAdmin extends Review {
  student_name: string;
  student_avatar: string | null;
  tutor_name: string;
}

export async function getAllReviewsForAdmin(limit = 500): Promise<ReviewForAdmin[]> {
  const { data, error } = await supabase
    .from('reviews')
    .select(
      `
      *,
      student:users!reviews_student_id_fkey(name, avatar),
      tutor:users!reviews_tutor_id_fkey(name)
    `,
    )
    .order('created_at', { ascending: false })
    .limit(limit);
  if (error) throw error;
  type ReviewForAdminRow = Review & {
    student: { name: string | null; avatar: string | null } | null;
    tutor: { name: string | null } | null;
  };
  return ((data ?? []) as ReviewForAdminRow[]).map((row) => ({
    ...(row as Review),
    student_name: row.student?.name ?? '',
    student_avatar: row.student?.avatar ?? null,
    tutor_name: row.tutor?.name ?? '',
  }));
}

/**
 * Hard-delete a review. The AFTER-DELETE trigger on `reviews` recomputes the
 * tutor_profiles.rating_avg + session_count automatically, so no follow-up
 * write is needed here.
 */
export async function deleteReview(reviewId: string): Promise<boolean> {
  const { error, count } = await supabase
    .from('reviews')
    .delete({ count: 'exact' })
    .eq('id', reviewId);
  if (error) throw error;
  return (count ?? 0) > 0;
}

/**
 * IDs of sessions the caller has already reviewed. Used by /my-sessions to
 * decide whether to show "Leave review" — cheaper than fetching every review
 * and joining in the browser.
 */
export async function getReviewedSessionIdsForStudent(studentId: string): Promise<Set<string>> {
  const { data } = await supabase.from('reviews').select('session_id').eq('student_id', studentId);
  const rows = (data ?? []) as { session_id: string }[];
  return new Set(rows.map((r) => r.session_id));
}

// ---- Materials ----

export async function getMaterialsForCourse(courseId: string): Promise<Material[]> {
  const { data, error } = await supabase
    .from('materials')
    .select('*')
    .eq('course_id', courseId)
    .order('order_index', { ascending: true });
  if (error) throw error;
  return (data || []) as Material[];
}

export async function addMaterial(input: {
  course_id: string;
  title: string;
  type: MaterialType;
  url: string;
  order_index?: number;
}): Promise<Material> {
  const { data, error } = await supabase.from('materials').insert(input).select('*').single();
  if (error) throw error;
  return data as Material;
}

export async function deleteMaterial(materialId: string): Promise<number> {
  const { error, count } = await supabase
    .from('materials')
    .delete({ count: 'exact' })
    .eq('id', materialId);
  if (error) throw error;
  return count ?? 0;
}

/**
 * True if `studentId` has a confirmed/live/completed session for `courseId`.
 * Gates student access to a course's materials — a canceled or no-show
 * booking doesn't unlock the lesson kit.
 */
export async function studentHasSessionForCourse(
  studentId: string,
  courseId: string,
): Promise<boolean> {
  const { count } = await supabase
    .from('sessions')
    .select('id', { count: 'exact', head: true })
    .eq('student_id', studentId)
    .eq('course_id', courseId)
    .in('status', ['confirmed', 'live', 'completed']);
  return (count ?? 0) > 0;
}

// ---- Admin ----

export async function getAllUsers(): Promise<User[]> {
  const { data, error } = await supabase
    .from('users')
    .select('*')
    .order('created_at', { ascending: false });
  if (error) throw error;
  return (data || []) as User[];
}

// Sessions for admin come denormalized like getSessionListForUser but without
// any WHERE clause on student/tutor. Bounded query — no filter path stays fast
// while the platform is small; add pagination when the row count justifies.
export async function getAllSessionsForAdmin(filters?: {
  status?: string;
  from?: Date;
  to?: Date;
  type?: 'private' | 'group';
}): Promise<SessionListItem[]> {
  let query = supabase
    .from('sessions')
    .select(SESSION_SELECT)
    .order('scheduled_at', { ascending: false });
  if (filters?.status) query = query.eq('status', filters.status);
  if (filters?.type) query = query.eq('type', filters.type);
  if (filters?.from) query = query.gte('scheduled_at', filters.from.toISOString());
  if (filters?.to) query = query.lt('scheduled_at', filters.to.toISOString());
  const { data, error } = await query;
  if (error) throw error;
  return ((data ?? []) as SessionSelectRow[]).map(denormalize);
}

// Full group-session detail used by the public /sessions/[id] page. Includes
// tutor summary + a small participant preview (avatars for social proof).
// Returns null if the row doesn't exist or isn't a group session — the page
// then 404s rather than showing a broken private-session view.
export interface GroupSessionDetail extends Session {
  tutor_name: string | null;
  tutor_avatar: string | null;
  tutor_rating: number | null;
  tutor_bio: string | null;
  participant_count: number;
  participants: Array<{ user_id: string; name: string | null; avatar: string | null }>;
}

export async function getGroupSessionDetail(id: string): Promise<GroupSessionDetail | null> {
  const { data, error } = await supabase
    .from('sessions')
    .select(
      `
      *,
      tutor:users!sessions_tutor_id_fkey(id, name, avatar, bio, profile:tutor_profiles(rating_avg)),
      participants:session_participants(user_id, user:users(id, name, avatar))
    `,
    )
    .eq('id', id)
    .maybeSingle();
  if (error || !data) return null;
  // Bio lives on `users.bio`, not `tutor_profiles` — same story as
  // getPublishedCourseDetail above.
  type GroupDetailRow = Session & {
    tutor: {
      id: string;
      name: string | null;
      avatar: string | null;
      bio: string | null;
      profile: {
        rating_avg: number | null;
      } | null;
    } | null;
    participants:
      | {
          user_id: string;
          user: { id: string; name: string | null; avatar: string | null } | null;
        }[]
      | null;
  };
  const row = data as GroupDetailRow;
  if (row.type !== 'group') return null;
  const participants = Array.isArray(row.participants)
    ? row.participants.map((p) => ({
        user_id: p.user_id,
        name: p.user?.name ?? null,
        avatar: p.user?.avatar ?? null,
      }))
    : [];
  return {
    ...(row as Session),
    tutor_name: row.tutor?.name ?? null,
    tutor_avatar: row.tutor?.avatar ?? null,
    tutor_rating: row.tutor?.profile?.rating_avg ?? null,
    tutor_bio: row.tutor?.bio ?? null,
    participant_count: participants.length,
    participants,
  };
}

// Cheap check: does this user already have a session_participants row for the
// given group session? Powers the /sessions/[id] page's CTA state.
export async function hasReservedGroupSession(sessionId: string, userId: string): Promise<boolean> {
  const { data } = await supabase
    .from('session_participants')
    .select('user_id')
    .eq('session_id', sessionId)
    .eq('user_id', userId)
    .maybeSingle();
  return !!data;
}

/**
 * Public listing of group free-talk sessions. Only shows what a student can
 * potentially still join — scheduled/live in the future, or already-live
 * rooms that haven't exceeded their duration yet.
 */
export async function getUpcomingGroupSessions(): Promise<SessionListItem[]> {
  const now = new Date();
  const cutoff = new Date(now.getTime() - 30 * 60_000).toISOString();
  const { data, error } = await supabase
    .from('sessions')
    .select(SESSION_SELECT)
    .eq('type', 'group')
    .in('status', ['pending', 'confirmed', 'live'])
    .gte('scheduled_at', cutoff)
    .order('scheduled_at', { ascending: true });
  if (error) throw error;
  return ((data ?? []) as SessionSelectRow[]).map(denormalize);
}

/**
 * Small stat bundle for the public homepage trust line. Cheap counts + one
 * average — avoids pulling the full AdminKpis with its per-day chart series.
 * All numbers gracefully degrade to 0 if the underlying query fails.
 */
export interface HomeStats {
  tutorsApproved: number;
  sessionsCompleted: number;
  ratingAvg: number;
}

export async function getHomeStats(): Promise<HomeStats> {
  const [approved, completed, ratings] = await Promise.all([
    supabase
      .from('tutor_profiles')
      .select('user_id', { count: 'exact', head: true })
      .eq('is_approved', true),
    supabase
      .from('sessions')
      .select('id', { count: 'exact', head: true })
      .eq('status', 'completed'),
    supabase.from('reviews').select('rating'),
  ]);
  const ratingRows = (ratings?.data || []) as { rating: number }[];
  const ratingAvg = ratingRows.length
    ? Number((ratingRows.reduce((a, r) => a + r.rating, 0) / ratingRows.length).toFixed(1))
    : 0;
  return {
    tutorsApproved: approved?.count ?? 0,
    sessionsCompleted: completed?.count ?? 0,
    ratingAvg,
  };
}

export type PendingTutor = Tutor;

export async function getPendingTutors(): Promise<PendingTutor[]> {
  const { data, error } = await supabase
    .from('users')
    .select('*, profile:tutor_profiles!inner(*)')
    .eq('role', 'tutor')
    .eq('tutor_profiles.is_approved', false)
    .order('created_at', { ascending: false });
  if (error) throw error;
  return (data || []) as PendingTutor[];
}

export interface AdminKpis {
  usersTotal: number;
  studentsTotal: number;
  tutorsTotal: number;
  tutorsApproved: number;
  tutorsPending: number;
  sessionsToday: number;
  sessionsWeek: number;
  sessionsMonth: number;
  revenueWeekVnd: number;
  revenueMonthVnd: number;
  ratingAvg: number;
  sessionsPerDay: { date: string; count: number }[];
}

/**
 * Aggregate every KPI the overview needs in one shot. Uses Supabase
 * `head+count` for cheap COUNTs and a bounded lookback for the daily chart.
 * All queries run in parallel — total wall time ≈ slowest single query.
 */
export async function getAdminKpis(): Promise<AdminKpis> {
  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const startOfWeek = new Date(startOfToday.getTime() - 7 * 24 * 60 * 60_000);
  const startOfMonth = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());
  const chartStart = new Date(startOfToday.getTime() - 30 * 24 * 60 * 60_000);

  const [
    students,
    tutorsAll,
    tutorsApproved,
    sessionsToday,
    sessionsWeek,
    sessionsMonth,
    revenueWeek,
    revenueMonth,
    ratingRows,
    seriesRows,
  ] = await Promise.all([
    supabase.from('users').select('id', { count: 'exact', head: true }).eq('role', 'user'),
    supabase.from('users').select('id', { count: 'exact', head: true }).eq('role', 'tutor'),
    supabase
      .from('tutor_profiles')
      .select('user_id', { count: 'exact', head: true })
      .eq('is_approved', true),
    supabase
      .from('sessions')
      .select('id', { count: 'exact', head: true })
      .gte('scheduled_at', startOfToday.toISOString()),
    supabase
      .from('sessions')
      .select('id', { count: 'exact', head: true })
      .gte('scheduled_at', startOfWeek.toISOString()),
    supabase
      .from('sessions')
      .select('id', { count: 'exact', head: true })
      .gte('scheduled_at', startOfMonth.toISOString()),
    supabase
      .from('sessions')
      .select('price_vnd')
      .eq('status', 'completed')
      .gte('scheduled_at', startOfWeek.toISOString()),
    supabase
      .from('sessions')
      .select('price_vnd')
      .eq('status', 'completed')
      .gte('scheduled_at', startOfMonth.toISOString()),
    supabase.from('reviews').select('rating'),
    supabase
      .from('sessions')
      .select('scheduled_at')
      .gte('scheduled_at', chartStart.toISOString())
      .order('scheduled_at', { ascending: true }),
  ]);

  const sum = (rows: { data: { price_vnd: number | null }[] | null } | null | undefined) =>
    (rows?.data ?? []).reduce((s, r) => s + (r.price_vnd ?? 0), 0);

  const ratings = (ratingRows?.data || []) as { rating: number }[];
  const ratingAvg = ratings.length
    ? Number((ratings.reduce((a, r) => a + r.rating, 0) / ratings.length).toFixed(1))
    : 0;

  // Bucket sessions into per-day counts covering the full 30-day window,
  // including days with zero sessions so the chart doesn't have holes.
  const bucket = new Map<string, number>();
  for (let d = new Date(chartStart); d <= startOfToday; d.setDate(d.getDate() + 1)) {
    const key = d.toISOString().slice(0, 10);
    bucket.set(key, 0);
  }
  for (const row of (seriesRows?.data || []) as { scheduled_at: string }[]) {
    const key = row.scheduled_at.slice(0, 10);
    if (bucket.has(key)) bucket.set(key, (bucket.get(key) || 0) + 1);
  }

  const studentsTotal = students?.count ?? 0;
  const tutorsTotal = tutorsAll?.count ?? 0;
  const tutorsApprovedCount = tutorsApproved?.count ?? 0;

  return {
    usersTotal: studentsTotal + tutorsTotal,
    studentsTotal,
    tutorsTotal,
    tutorsApproved: tutorsApprovedCount,
    tutorsPending: Math.max(tutorsTotal - tutorsApprovedCount, 0),
    sessionsToday: sessionsToday?.count ?? 0,
    sessionsWeek: sessionsWeek?.count ?? 0,
    sessionsMonth: sessionsMonth?.count ?? 0,
    revenueWeekVnd: sum(revenueWeek),
    revenueMonthVnd: sum(revenueMonth),
    ratingAvg,
    sessionsPerDay: Array.from(bucket, ([date, count]) => ({ date, count })),
  };
}

export async function setTutorApproval(
  tutorId: string,
  is_approved: boolean,
): Promise<PendingTutor | null> {
  const patch = is_approved
    ? { is_approved: true, approved_at: new Date().toISOString() }
    : { is_approved: false, approved_at: null };
  const { data, error } = await supabase
    .from('tutor_profiles')
    .update(patch)
    .eq('user_id', tutorId)
    .select('*')
    .maybeSingle();
  if (error) throw error;
  if (!data) return null;
  const { data: fresh } = await supabase
    .from('users')
    .select('*, profile:tutor_profiles(*)')
    .eq('id', tutorId)
    .maybeSingle();
  return fresh as PendingTutor | null;
}

export async function setUserRole(userId: string, role: 'user' | 'tutor' | 'admin') {
  const { data, error } = await supabase
    .from('users')
    .update({ role })
    .eq('id', userId)
    .select('*')
    .single();
  if (error) throw error;
  return data as User;
}

/**
 * Enable or disable a user account. Setting enabled=false immediately gates
 * every /api/* call (see disabledResponse in auth-server.ts) and — for auth
 * sessions — surfaces a "you are disabled" message on next request.
 *
 * `reason` is stored so students see the admin-supplied explanation on the
 * login page. Pass null when re-enabling to clear it.
 */
export async function setUserEnabled(userId: string, enabled: boolean, reason?: string | null) {
  const patch: { enabled: boolean; disabled_reason: string | null } = {
    enabled,
    disabled_reason: enabled ? null : (reason ?? null),
  };
  const { data, error } = await supabase
    .from('users')
    .update(patch)
    .eq('id', userId)
    .select('*')
    .single();
  if (error) throw error;
  return data as User;
}

// ---- Tutor self-service ----

export interface TutorKpis {
  upcomingCount: number;
  todayCount: number;
  weekCount: number;
  monthCount: number;
  completedTotal: number;
  revenueWeekVnd: number;
  revenueMonthVnd: number;
  ratingAvg: number;
  reviewsCount: number;
}

/**
 * KPIs scoped to a single tutor. Every count is filtered by tutor_id in the
 * DB so we never scan sessions rows the tutor shouldn't see.
 */
export async function getTutorKpis(tutorId: string): Promise<TutorKpis> {
  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const startOfWeek = new Date(startOfToday.getTime() - 7 * 24 * 60 * 60_000);
  const startOfMonth = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());

  const [upcoming, today, week, month, completed, revenueWeek, revenueMonth, reviews, profile] =
    await Promise.all([
      supabase
        .from('sessions')
        .select('id', { count: 'exact', head: true })
        .eq('tutor_id', tutorId)
        .in('status', ['pending', 'confirmed', 'live'])
        .gte('scheduled_at', now.toISOString()),
      supabase
        .from('sessions')
        .select('id', { count: 'exact', head: true })
        .eq('tutor_id', tutorId)
        .gte('scheduled_at', startOfToday.toISOString()),
      supabase
        .from('sessions')
        .select('id', { count: 'exact', head: true })
        .eq('tutor_id', tutorId)
        .gte('scheduled_at', startOfWeek.toISOString()),
      supabase
        .from('sessions')
        .select('id', { count: 'exact', head: true })
        .eq('tutor_id', tutorId)
        .gte('scheduled_at', startOfMonth.toISOString()),
      supabase
        .from('sessions')
        .select('id', { count: 'exact', head: true })
        .eq('tutor_id', tutorId)
        .eq('status', 'completed'),
      supabase
        .from('sessions')
        .select('price_vnd')
        .eq('tutor_id', tutorId)
        .eq('status', 'completed')
        .gte('scheduled_at', startOfWeek.toISOString()),
      supabase
        .from('sessions')
        .select('price_vnd')
        .eq('tutor_id', tutorId)
        .eq('status', 'completed')
        .gte('scheduled_at', startOfMonth.toISOString()),
      supabase.from('reviews').select('rating').eq('tutor_id', tutorId),
      supabase.from('tutor_profiles').select('rating_avg').eq('user_id', tutorId).maybeSingle(),
    ]);

  const sum = (rows: { data: { price_vnd: number | null }[] | null } | null | undefined) =>
    (rows?.data ?? []).reduce((s, r) => s + (r.price_vnd ?? 0), 0);

  const ratingsRows = (reviews?.data || []) as { rating: number }[];
  const profileRow = profile?.data as { rating_avg: number | null } | null;
  const ratingAvg =
    profileRow?.rating_avg ??
    (ratingsRows.length ? ratingsRows.reduce((a, r) => a + r.rating, 0) / ratingsRows.length : 0);

  return {
    upcomingCount: upcoming?.count ?? 0,
    todayCount: today?.count ?? 0,
    weekCount: week?.count ?? 0,
    monthCount: month?.count ?? 0,
    completedTotal: completed?.count ?? 0,
    revenueWeekVnd: sum(revenueWeek),
    revenueMonthVnd: sum(revenueMonth),
    ratingAvg: Number((ratingAvg || 0).toFixed(1)),
    reviewsCount: ratingsRows.length,
  };
}

/**
 * Tutor updates their own profile. Only accepts fields the tutor is allowed
 * to change themselves — is_approved / approved_at / rating_avg / session_count
 * stay under admin + trigger control.
 */
export interface TutorProfilePatch {
  hourly_rate_vnd?: number;
  bio?: string | null;
  intro_video_url?: string | null;
  specialties?: string[];
  years_experience?: number;
  certifications?: string[];
  languages_spoken?: string[];
}

export async function updateTutorProfile(
  tutorId: string,
  userPatch: { name?: string; bio?: string; avatar?: string | null },
  profilePatch: TutorProfilePatch,
): Promise<{ user: User; profile: TutorProfile }> {
  if (Object.keys(userPatch).length > 0) {
    const { error } = await supabase.from('users').update(userPatch).eq('id', tutorId);
    if (error) throw error;
  }
  if (Object.keys(profilePatch).length > 0) {
    const { error } = await supabase
      .from('tutor_profiles')
      .update(profilePatch)
      .eq('user_id', tutorId);
    if (error) throw error;
  }
  const [{ data: user }, { data: profile }] = await Promise.all([
    supabase.from('users').select('*').eq('id', tutorId).single(),
    supabase.from('tutor_profiles').select('*').eq('user_id', tutorId).single(),
  ]);
  return { user: user as User, profile: profile as TutorProfile };
}

export async function addAvailabilitySlot(input: {
  tutor_id: string;
  weekday: number;
  start_time: string;
  end_time: string;
}) {
  const { data, error } = await supabase.from('availability').insert(input).select('*').single();
  if (error) throw error;
  return data as {
    id: string;
    tutor_id: string;
    weekday: number;
    start_time: string;
    end_time: string;
  };
}

export async function deleteAvailabilitySlot(slotId: string, tutorId: string) {
  // Ownership guard: the tutor_id equality is redundant with API-layer auth,
  // but belt-and-braces means a stray internal call can't wipe someone else's
  // schedule.
  const { error, count } = await supabase
    .from('availability')
    .delete({ count: 'exact' })
    .eq('id', slotId)
    .eq('tutor_id', tutorId);
  if (error) throw error;
  return count ?? 0;
}

/**
 * Atomically replace a tutor's whole weekly schedule with the provided set.
 * Used by the grid editor — computing per-row diffs client-side would be more
 * code + more requests than letting the server settle it in two statements.
 *
 * Note: not wrapped in a Postgres transaction (Supabase JS lacks the primitive
 * without an RPC). The DELETE runs first; if the INSERT half fails the tutor
 * ends up with an empty schedule until they retry. Consider promoting to an
 * RPC (`replace_availability(tutor_id uuid, slots jsonb)`) once we have real
 * tutor volume — for MVP the retry loop in the client is enough.
 */
export async function replaceAvailabilityForTutor(
  tutorId: string,
  slots: { weekday: number; start_time: string; end_time: string }[],
) {
  await supabase.from('availability').delete().eq('tutor_id', tutorId);
  if (slots.length === 0) return [];
  const rows = slots.map((s) => ({ ...s, tutor_id: tutorId }));
  const { data, error } = await supabase.from('availability').insert(rows).select('*');
  if (error) throw error;
  return data as {
    id: string;
    tutor_id: string;
    weekday: number;
    start_time: string;
    end_time: string;
  }[];
}
