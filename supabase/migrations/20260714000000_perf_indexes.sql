-- Performance indexes uncovered by the pre-launch audit. Each is scoped to a
-- real query pattern in src/lib/supabase.ts, not speculative.
--
-- Uses IF NOT EXISTS so re-running (or applying against a partially-migrated
-- staging DB) is safe.

-- getReviewedSessionIdsForStudent(): "which sessions has this student
-- reviewed already?" — filters `reviews WHERE student_id = ?`. Table already
-- has UNIQUE(session_id), so lookups by session are fast; by student_id they
-- were a sequential scan.
CREATE INDEX IF NOT EXISTS idx_reviews_student
  ON reviews(student_id, created_at DESC);

-- Waiting-room queue on the host view: session_admissions rows where the
-- host hasn't decided yet (admitted_at IS NULL AND denied_at IS NULL). The
-- PK (session_id, user_id) already handles WHERE session_id = ?, but this
-- partial index skips the resolved rows so the queue query stays fast even
-- for busy group rooms where most admissions have already been decided.
CREATE INDEX IF NOT EXISTS idx_session_admissions_pending
  ON session_admissions(session_id, requested_at)
  WHERE admitted_at IS NULL AND denied_at IS NULL;

-- session_attendance capacity gate — /api/livekit/token counts open seats
-- via "WHERE session_id = ? AND left_at IS NULL". The existing
-- idx_session_attendance_session_user(session_id, user_id) covers the
-- range but scans still touch closed rows. A partial index on the "still
-- inside the room" subset is small (< handful of rows per live session)
-- and lets the capacity check hit an index-only scan.
CREATE INDEX IF NOT EXISTS idx_session_attendance_open
  ON session_attendance(session_id)
  WHERE left_at IS NULL;
