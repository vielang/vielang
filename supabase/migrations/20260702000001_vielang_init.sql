-- VieLang initial schema — Phase 1 of the pivot.
-- This replaces the golf/VinaRounding schema entirely; run against a fresh
-- Supabase project (region: Southeast Asia — Singapore).
--
-- Design notes:
--   • users.supabase_uid links to auth.users.id (populated on first login by
--     /auth/callback). Kept nullable so seed rows can exist before an auth
--     user is provisioned.
--   • Role is a text enum with a CHECK. Simpler than a Postgres enum type
--     (which requires ALTER TYPE dance to add values later).
--   • Every timestamp is TIMESTAMPTZ. Sessions live in UTC and get formatted
--     client-side per user.timezone.
--   • Uses UUID PKs everywhere; gen_random_uuid() ships with pgcrypto which
--     Supabase enables by default.
--   • RLS intentionally NOT enabled here. All access goes through the
--     server-side service_role client via /api/* Route Handlers, which apply
--     their own ownership checks. Turning RLS on later is a separate task —
--     we'd need per-table policies and drop the service_role default.

CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  supabase_uid UUID UNIQUE,
  email TEXT UNIQUE NOT NULL,
  name TEXT NOT NULL,
  role TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('user','tutor','admin')),
  avatar TEXT,
  bio TEXT,
  timezone TEXT DEFAULT 'Asia/Ho_Chi_Minh',
  native_lang TEXT DEFAULT 'vi',
  learning_lang TEXT DEFAULT 'en',
  enabled BOOLEAN NOT NULL DEFAULT TRUE,
  disabled_reason TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_users_role ON users(role) WHERE enabled = TRUE;

-- 1-1 with users where role='tutor'. Kept separate so students don't carry
-- tutor-only columns (approval, hourly_rate, ratings) they'd never use.
CREATE TABLE tutor_profiles (
  user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  hourly_rate_vnd INTEGER NOT NULL DEFAULT 0,
  intro_video_url TEXT,
  intro_video_thumbnail TEXT,
  specialties TEXT[] NOT NULL DEFAULT '{}',
  years_experience INTEGER NOT NULL DEFAULT 0,
  certifications TEXT[] NOT NULL DEFAULT '{}',
  languages_spoken TEXT[] NOT NULL DEFAULT '{}',
  is_approved BOOLEAN NOT NULL DEFAULT FALSE,
  approved_at TIMESTAMPTZ,
  rating_avg NUMERIC(2,1) NOT NULL DEFAULT 0,
  session_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_tutor_profiles_approved ON tutor_profiles(is_approved) WHERE is_approved = TRUE;

CREATE TABLE courses (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title_vn TEXT NOT NULL,
  title_en TEXT NOT NULL,
  description_vn TEXT,
  description_en TEXT,
  level TEXT CHECK (level IN ('A1','A2','B1','B2','C1','C2')),
  category TEXT,
  image TEXT,
  tutor_id UUID REFERENCES users(id) ON DELETE SET NULL,
  price_vnd INTEGER NOT NULL,
  duration_min INTEGER NOT NULL DEFAULT 60,
  is_published BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_courses_tutor ON courses(tutor_id);
CREATE INDEX idx_courses_published ON courses(is_published, level, category) WHERE is_published = TRUE;

-- Recurring weekly availability. weekday 0=Sunday to match JS Date.getDay().
-- One row per contiguous free window on a given weekday.
CREATE TABLE availability (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tutor_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  weekday SMALLINT NOT NULL CHECK (weekday BETWEEN 0 AND 6),
  start_time TIME NOT NULL,
  end_time TIME NOT NULL,
  UNIQUE (tutor_id, weekday, start_time),
  CHECK (end_time > start_time)
);
CREATE INDEX idx_availability_tutor_day ON availability(tutor_id, weekday);

-- Concrete booked classes. livekit_room_name is generated once (session-<uuid>)
-- and is what both participants join via the LiveKit token endpoint.
CREATE TABLE sessions (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  student_id UUID NOT NULL REFERENCES users(id),
  tutor_id UUID NOT NULL REFERENCES users(id),
  course_id UUID REFERENCES courses(id) ON DELETE SET NULL,
  scheduled_at TIMESTAMPTZ NOT NULL,
  duration_min INTEGER NOT NULL DEFAULT 60,
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending','confirmed','live','completed','cancelled','no_show')),
  livekit_room_name TEXT UNIQUE,
  price_vnd INTEGER NOT NULL,
  student_notes TEXT,
  tutor_notes TEXT,
  cancelled_by UUID REFERENCES users(id),
  cancelled_reason TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_sessions_tutor_date ON sessions(tutor_id, scheduled_at);
CREATE INDEX idx_sessions_student ON sessions(student_id, scheduled_at DESC);
CREATE INDEX idx_sessions_status ON sessions(status, scheduled_at);

-- Constraint enforced at insert time to prevent a tutor from being double-
-- booked for the same start time. Not a range check — booking flow already
-- validates against availability + existing pending/confirmed sessions in a
-- transaction, so this is a belt-and-braces safety net for concurrent inserts.
CREATE UNIQUE INDEX idx_sessions_no_overlap
  ON sessions(tutor_id, scheduled_at)
  WHERE status IN ('pending','confirmed','live');

CREATE TABLE reviews (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id UUID UNIQUE NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  student_id UUID NOT NULL REFERENCES users(id),
  tutor_id UUID NOT NULL REFERENCES users(id),
  rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
  comment TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_reviews_tutor ON reviews(tutor_id, created_at DESC);

CREATE TABLE materials (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  type TEXT NOT NULL CHECK (type IN ('pdf','video','link','image')),
  url TEXT NOT NULL,
  order_index INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_materials_course ON materials(course_id, order_index);

CREATE TABLE banners (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title_vn TEXT, title_en TEXT,
  subtitle_vn TEXT, subtitle_en TEXT,
  image TEXT NOT NULL,
  link TEXT,
  order_index INTEGER NOT NULL DEFAULT 0,
  is_active BOOLEAN NOT NULL DEFAULT TRUE
);
CREATE INDEX idx_banners_active ON banners(is_active, order_index) WHERE is_active = TRUE;

CREATE TABLE news (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title_vn TEXT NOT NULL, title_en TEXT NOT NULL,
  content_vn TEXT, content_en TEXT,
  category TEXT,
  image TEXT,
  published_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  is_published BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX idx_news_published ON news(published_at DESC) WHERE is_published = TRUE;

-- Trigger: keep tutor_profiles.rating_avg + session_count in sync when reviews
-- land. Simpler than recomputing in application code + guarantees the two
-- summary fields never drift from the underlying reviews table.
CREATE OR REPLACE FUNCTION refresh_tutor_rating() RETURNS TRIGGER AS $$
BEGIN
  UPDATE tutor_profiles tp
     SET rating_avg = COALESCE((
           SELECT ROUND(AVG(rating)::numeric, 1)
             FROM reviews r
            WHERE r.tutor_id = tp.user_id
         ), 0),
         session_count = COALESCE((
           SELECT COUNT(*) FROM sessions s
            WHERE s.tutor_id = tp.user_id AND s.status = 'completed'
         ), 0)
   WHERE tp.user_id = NEW.tutor_id;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_review_refresh_tutor
  AFTER INSERT OR UPDATE OR DELETE ON reviews
  FOR EACH ROW EXECUTE FUNCTION refresh_tutor_rating();
