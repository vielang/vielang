// VieLang domain types — mirror the Supabase schema in
// supabase/migrations/20260702000001_vielang_init.sql.

export type Language = 'KR' | 'VN' | 'EN';
export type Role = 'user' | 'tutor' | 'admin';
export type SessionStatus =
  'pending' | 'confirmed' | 'live' | 'completed' | 'cancelled' | 'no_show';
export type SessionType = 'private' | 'group';
export type GroupLevel = 'all' | 'a1_a2' | 'b1_b2' | 'c1_c2';
export type Level = 'A1' | 'A2' | 'B1' | 'B2' | 'C1' | 'C2';
export type MaterialType = 'pdf' | 'video' | 'link' | 'image';

export interface User {
  id: string;
  supabase_uid: string | null;
  email: string;
  name: string;
  role: Role;
  avatar: string | null;
  bio: string | null;
  timezone: string;
  native_lang: string;
  learning_lang: string;
  enabled: boolean;
  disabled_reason: string | null;
  created_at: string;
}

export interface TutorProfile {
  user_id: string;
  hourly_rate_vnd: number;
  intro_video_url: string | null;
  intro_video_thumbnail: string | null;
  specialties: string[];
  years_experience: number;
  certifications: string[];
  languages_spoken: string[];
  is_approved: boolean;
  approved_at: string | null;
  rating_avg: number;
  session_count: number;
}

// Convenience shape returned by /api/tutors — user row + profile fields
// flattened so the frontend doesn't have to join client-side.
export interface Tutor extends User {
  profile: TutorProfile;
}

export interface Course {
  id: string;
  title_vn: string;
  title_en: string;
  description_vn: string | null;
  description_en: string | null;
  level: Level | null;
  category: string | null;
  image: string | null;
  tutor_id: string | null;
  price_vnd: number;
  duration_min: number;
  is_published: boolean;
  created_at: string;
}

export interface Availability {
  id: string;
  tutor_id: string;
  weekday: number; // 0 = Sunday
  start_time: string; // 'HH:MM:SS'
  end_time: string;
}

// Sessions cover both 1-on-1 tutor bookings (type='private') and admin-created
// group free-talk rooms (type='group'). Nullable fields carry the semantic
// difference — the DB-level CHECK in the group_sessions migration enforces
// which combinations are legal.
export interface Session {
  id: string;
  type: SessionType;
  student_id: string | null; // required for private, always null for group
  tutor_id: string | null; // required for private; nullable for admin-hosted group
  course_id: string | null; // required for private, always null for group
  topic_en: string | null; // required for group
  topic_vn: string | null;
  description: string | null;
  level: GroupLevel | null;
  capacity: number; // 1 for private, >=2 for group
  cover_emoji: string | null;
  scheduled_at: string; // ISO UTC
  duration_min: number;
  status: SessionStatus;
  livekit_room_name: string | null;
  price_vnd: number;
  student_notes: string | null;
  tutor_notes: string | null;
  cancelled_by: string | null;
  cancelled_reason: string | null;
  require_admission: boolean;
  created_at: string;
}

export interface SessionParticipant {
  session_id: string;
  user_id: string;
  joined_at: string;
  left_at: string | null;
}

export interface SessionAdmission {
  session_id: string;
  user_id: string;
  requested_at: string;
  admitted_at: string | null;
  admitted_by: string | null;
  denied_at: string | null;
  denied_by: string | null;
}

export interface Review {
  id: string;
  session_id: string;
  student_id: string;
  tutor_id: string;
  rating: number; // 1-5
  comment: string | null;
  created_at: string;
}

export interface Material {
  id: string;
  course_id: string;
  title: string;
  type: MaterialType;
  url: string;
  order_index: number;
}

export interface Banner {
  id: string;
  title_vn: string | null;
  title_en: string | null;
  subtitle_vn: string | null;
  subtitle_en: string | null;
  image: string;
  link: string | null;
  order_index: number;
  is_active: boolean;
}

export interface NewsItem {
  id: string;
  title_vn: string;
  title_en: string;
  content_vn: string | null;
  content_en: string | null;
  category: string | null;
  image: string | null;
  published_at: string;
  is_published: boolean;
}
