-- Group free-talk sessions — extend the existing sessions table to support
-- admin-created group rooms alongside 1-on-1 tutor bookings.
--
-- Design goal: single sessions table so /my-sessions, the admin listing, the
-- LiveKit token flow, and the status enum stay unified. A CHECK constraint
-- enforces the semantic difference between the two shapes; a small companion
-- table (session_participants) tracks group attendance without touching the
-- 1-on-1 code path.

-- 1. Columns for group support. Defaults keep pre-existing rows as 'private'.
ALTER TABLE sessions
  ADD COLUMN type TEXT NOT NULL DEFAULT 'private'
    CHECK (type IN ('private', 'group')),
  ADD COLUMN topic_en TEXT,
  ADD COLUMN topic_vn TEXT,
  ADD COLUMN description TEXT,
  ADD COLUMN level TEXT
    CHECK (level IS NULL OR level IN ('all', 'a1_a2', 'b1_b2', 'c1_c2')),
  ADD COLUMN capacity INTEGER NOT NULL DEFAULT 1,
  ADD COLUMN cover_emoji TEXT;

-- 2. Relax NOT NULLs — group sessions have no single student, no course, and
-- an admin can self-host so tutor_id is optional too. Private still requires
-- all three via the CHECK below.
ALTER TABLE sessions
  ALTER COLUMN student_id DROP NOT NULL,
  ALTER COLUMN tutor_id DROP NOT NULL,
  ALTER COLUMN price_vnd SET DEFAULT 0;

-- 3. Shape constraint. Belt-and-braces against a malformed insert bypassing
-- the API layer.
ALTER TABLE sessions ADD CONSTRAINT sessions_shape_check CHECK (
  (
    type = 'private'
    AND student_id IS NOT NULL
    AND tutor_id  IS NOT NULL
    AND course_id IS NOT NULL
    AND capacity  = 1
  )
  OR
  (
    type = 'group'
    AND student_id IS NULL
    AND course_id  IS NULL
    AND topic_en   IS NOT NULL
    AND capacity  >= 2
  )
);

-- 4. Participants — populated ONLY for group sessions. Private sessions
-- continue to carry their single student in sessions.student_id, so we don't
-- duplicate data or force the 1-on-1 flow through a join.
CREATE TABLE session_participants (
  session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  user_id    UUID NOT NULL REFERENCES users(id)   ON DELETE CASCADE,
  joined_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  left_at    TIMESTAMPTZ,
  PRIMARY KEY (session_id, user_id)
);
CREATE INDEX idx_session_participants_user ON session_participants(user_id);

-- 5. Speeds the public "/sessions" listing: upcoming + live group sessions
-- ordered by start time.
CREATE INDEX idx_sessions_group_upcoming
  ON sessions(scheduled_at)
  WHERE type = 'group' AND status IN ('pending', 'confirmed', 'live');

-- 6. Rating trigger stays untouched — it fires off the reviews table and
-- reviews are 1-per-session by design, so it naturally never runs for group
-- rows (which currently have no review path). If we wire reviews for group
-- hosts later we can revisit.
