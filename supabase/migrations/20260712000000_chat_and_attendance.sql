-- Session chat persistence + attendance analytics.
--
-- Two tables that share a design principle: dedupe on the caller's stable ID
-- rather than trusting server-side deduplication of retries.
--
--   • session_messages.client_id — comes from LiveKit's ChatMessage.id (or a
--     timestamp+identity fallback). On network flakes the sender POSTs again;
--     the ON CONFLICT clause in /api/sessions/[id]/messages turns duplicate
--     inserts into no-ops so we don't get double-posted chat lines.
--
--   • session_attendance.event_id — every LiveKit webhook carries a unique
--     event id. The webhook retries on network failure with the same id, so
--     using it as a unique index keeps attendance rows exactly-once.
--
-- We store attendance one row per join event, closed on the corresponding
-- leave. duration_sec is a generated column so reporting queries never have
-- to compute it — and it's null while the participant is still in the room.

CREATE TABLE session_messages (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  sender_id  UUID NOT NULL REFERENCES users(id),
  body       TEXT NOT NULL CHECK (char_length(body) BETWEEN 1 AND 4000),
  sent_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  -- Stable id per outgoing message from the sender's client. NULL is allowed
  -- because a future SMS or agent-generated message might not have one, but
  -- the API always sets it for LiveKit chat.
  client_id  TEXT
);

-- The single index carries our two hot queries: history load for a session,
-- and the on-conflict dedupe for repeat POSTs from the same sender.
CREATE INDEX idx_session_messages_session ON session_messages(session_id, sent_at);
CREATE UNIQUE INDEX idx_session_messages_client_id
  ON session_messages(session_id, sender_id, client_id)
  WHERE client_id IS NOT NULL;

CREATE TABLE session_attendance (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id   UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  user_id      UUID NOT NULL REFERENCES users(id),
  joined_at    TIMESTAMPTZ NOT NULL,
  left_at      TIMESTAMPTZ,
  -- LiveKit webhook event id — dedupe key for the participant_joined event.
  -- The participant_left event carries its own id but we don't index on it,
  -- since a leave is always resolved by closing the most recent open row.
  event_id     TEXT NOT NULL,
  duration_sec INTEGER GENERATED ALWAYS AS (
    CASE
      WHEN left_at IS NULL THEN NULL
      ELSE EXTRACT(EPOCH FROM (left_at - joined_at))::INTEGER
    END
  ) STORED
);

CREATE UNIQUE INDEX idx_session_attendance_event ON session_attendance(event_id);
CREATE INDEX idx_session_attendance_session_user
  ON session_attendance(session_id, user_id, joined_at DESC);
