-- M3.4 — Enable Row-Level Security + add audit_log table.
--
-- Design:
--   • RLS is on for every user-visible table. Default is DENY — a policy has
--     to explicitly say who can read/write.
--   • Application traffic still runs through the server-side service_role
--     client, which BYPASSES RLS. Enabling RLS here doesn't change the API
--     behavior; it's a defense-in-depth net for two cases:
--       (a) A future switch to the anon key on the client — RLS becomes the
--           only gate stopping cross-user reads.
--       (b) Direct SQL access via the anon key (e.g. debug console, someone
--           embedding the anon key in a mobile app) — RLS holds the line.
--   • Public read policies are limited to explicitly-public data:
--       users        → nothing (email is PII)
--       tutor_profiles → only approved tutors (public directory)
--       courses      → only published
--       reviews      → all rows (they show on tutor profile pages)
--       news, banners → all rows (marketing content)
--   • Authenticated reads:
--       users        → own row (via supabase_uid = auth.uid())
--       sessions     → own rows (as student OR tutor)
--       materials    → students who have a confirmed/completed session on
--                      the linked course.
--   • Writes: NONE via anon or authenticated. Every mutation goes through
--     /api/* which uses service_role. This is intentional — application-layer
--     validation (zod, ownership checks, rate limits) is the source of truth.
--
-- audit_log tracks sensitive mutations (role changes, tutor approvals,
-- session cancellations) so we can reconstruct what happened when something
-- goes sideways. Populated by the application via lib/audit.ts.

--------------------------------------------------------------------------------
-- 1. audit_log table
--------------------------------------------------------------------------------

CREATE TABLE audit_log (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  actor_id     UUID REFERENCES users(id) ON DELETE SET NULL,
  actor_email  TEXT,
  actor_role   TEXT,
  action       TEXT NOT NULL,          -- e.g. 'session.cancel', 'user.role_change'
  entity       TEXT NOT NULL,          -- e.g. 'sessions', 'users', 'tutor_profiles'
  entity_id    TEXT,                   -- id of the affected row (text so we can log external ids too)
  before       JSONB,                  -- snapshot BEFORE the change (nullable for CREATE)
  after        JSONB,                  -- snapshot AFTER the change (nullable for DELETE)
  ip           TEXT,
  user_agent   TEXT,
  metadata     JSONB,                  -- arbitrary extra context (route, request id, etc.)
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_actor ON audit_log(actor_id, created_at DESC);
CREATE INDEX idx_audit_log_entity ON audit_log(entity, entity_id, created_at DESC);
CREATE INDEX idx_audit_log_action ON audit_log(action, created_at DESC);

-- audit_log itself: only service_role reads (anon/authenticated can't peek).
ALTER TABLE audit_log ENABLE ROW LEVEL SECURITY;
-- No policies = deny all under RLS. service_role bypasses RLS entirely.

--------------------------------------------------------------------------------
-- 2. Enable RLS on every user-visible table
--------------------------------------------------------------------------------

ALTER TABLE users            ENABLE ROW LEVEL SECURITY;
ALTER TABLE tutor_profiles   ENABLE ROW LEVEL SECURITY;
ALTER TABLE courses          ENABLE ROW LEVEL SECURITY;
ALTER TABLE availability     ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions         ENABLE ROW LEVEL SECURITY;
ALTER TABLE reviews          ENABLE ROW LEVEL SECURITY;
ALTER TABLE materials        ENABLE ROW LEVEL SECURITY;
ALTER TABLE banners          ENABLE ROW LEVEL SECURITY;
ALTER TABLE news             ENABLE ROW LEVEL SECURITY;

--------------------------------------------------------------------------------
-- 3. SELECT policies (deny by default; whitelist below)
--------------------------------------------------------------------------------

-- users: authenticated users can only see their own row.
CREATE POLICY "users_read_self"
  ON users FOR SELECT
  TO authenticated
  USING (supabase_uid = auth.uid());

-- tutor_profiles: public reads for approved tutors only.
CREATE POLICY "tutor_profiles_public_read_approved"
  ON tutor_profiles FOR SELECT
  TO anon, authenticated
  USING (is_approved = true);

-- courses: public reads for published courses only.
CREATE POLICY "courses_public_read_published"
  ON courses FOR SELECT
  TO anon, authenticated
  USING (is_published = true);

-- availability: public reads (used to render tutor calendar previews).
CREATE POLICY "availability_public_read"
  ON availability FOR SELECT
  TO anon, authenticated
  USING (true);

-- sessions: student, tutor, or participant of the group room.
CREATE POLICY "sessions_read_own"
  ON sessions FOR SELECT
  TO authenticated
  USING (
    -- 1-on-1: student or tutor
    student_id IN (SELECT id FROM users WHERE supabase_uid = auth.uid())
    OR tutor_id  IN (SELECT id FROM users WHERE supabase_uid = auth.uid())
    -- Group: participant row
    OR EXISTS (
      SELECT 1 FROM session_participants sp
      WHERE sp.session_id = sessions.id
        AND sp.user_id IN (SELECT id FROM users WHERE supabase_uid = auth.uid())
    )
  );

-- reviews: fully public (they show on tutor profile pages).
CREATE POLICY "reviews_public_read"
  ON reviews FOR SELECT
  TO anon, authenticated
  USING (true);

-- materials: only students with a booked/completed session on the linked
-- course, plus the tutor who owns the course.
CREATE POLICY "materials_read_participants"
  ON materials FOR SELECT
  TO authenticated
  USING (
    course_id IN (
      SELECT c.id FROM courses c WHERE c.tutor_id IN (
        SELECT id FROM users WHERE supabase_uid = auth.uid()
      )
    )
    OR EXISTS (
      SELECT 1 FROM sessions s
      WHERE s.course_id = materials.course_id
        AND s.student_id IN (SELECT id FROM users WHERE supabase_uid = auth.uid())
        AND s.status IN ('confirmed', 'live', 'completed')
    )
  );

-- banners: public reads for active banners.
CREATE POLICY "banners_public_read_active"
  ON banners FOR SELECT
  TO anon, authenticated
  USING (is_active = true);

-- news: public reads for published entries.
CREATE POLICY "news_public_read_published"
  ON news FOR SELECT
  TO anon, authenticated
  USING (is_published = true);

--------------------------------------------------------------------------------
-- 4. Session participants — support table for group sessions
--
-- Only run this block if the table exists (it was added in the group-sessions
-- migration). Guarded so this migration is idempotent across environments
-- where the table order might differ.
--------------------------------------------------------------------------------

DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = 'session_participants'
  ) THEN
    EXECUTE 'ALTER TABLE session_participants ENABLE ROW LEVEL SECURITY';
    EXECUTE '
      CREATE POLICY "session_participants_read_own"
        ON session_participants FOR SELECT
        TO authenticated
        USING (
          user_id IN (SELECT id FROM users WHERE supabase_uid = auth.uid())
        )
    ';
  END IF;
END $$;

--------------------------------------------------------------------------------
-- 5. INSERT / UPDATE / DELETE policies
--
-- None. All mutations go through service_role via /api/*. Adding client-side
-- write policies here would create two sources of truth — resist the urge.
--------------------------------------------------------------------------------
