-- Waiting-room admission control for sessions.
--
-- Two moving parts:
--   1. sessions.require_admission — per-room flag. When TRUE, students who
--      join get a restricted LiveKit token (no publish, no subscribe) and
--      wait for a tutor/admin to click Admit. Defaults FALSE so every
--      pre-existing session keeps its previous "join straight in" behavior.
--   2. session_admissions — audit log of who asked to join, when, who
--      admitted them, and (if applicable) who denied them. One row per
--      (session_id, user_id) pair. Idempotency: the token endpoint upserts,
--      so refreshing the tab does not create duplicate rows.
--
-- The waiting-room lifecycle lives half in this table and half in LiveKit
-- (participant.metadata + permissions). We store here so a refresh preserves
-- the "already admitted" state without a second round-trip to LiveKit.

ALTER TABLE sessions
  ADD COLUMN require_admission BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE session_admissions (
  session_id   UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  user_id      UUID NOT NULL REFERENCES users(id)   ON DELETE CASCADE,
  requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  admitted_at  TIMESTAMPTZ,
  admitted_by  UUID REFERENCES users(id),
  denied_at    TIMESTAMPTZ,
  denied_by    UUID REFERENCES users(id),
  PRIMARY KEY (session_id, user_id),
  -- A row cannot be both admitted and denied. Whichever action fires first
  -- locks the outcome; a second click bails out at the API layer.
  CONSTRAINT session_admissions_terminal_xor CHECK (
    admitted_at IS NULL OR denied_at IS NULL
  )
);

-- Index used by the host controls poller (if we ever need one) and by the
-- token endpoint's "am I still waiting" lookup, which is per-request. Cheap
-- to maintain since a room has a handful of admissions at once.
CREATE INDEX idx_session_admissions_user ON session_admissions(user_id);
