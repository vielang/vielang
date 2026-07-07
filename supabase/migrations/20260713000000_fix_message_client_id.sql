-- Fix: /api/sessions/[id]/messages uses ON CONFLICT (session_id, sender_id,
-- client_id) but Postgres won't match ON CONFLICT (column_list) against a
-- partial unique index — it needs a full unique constraint on those columns.
--
-- Our POST endpoint validates client_id with zod min(1), so in practice it's
-- always set. Flip the column to NOT NULL and drop the partial predicate; the
-- dedupe semantics stay identical (rows without a client_id can no longer be
-- inserted, which matches API reality anyway).

ALTER TABLE session_messages ALTER COLUMN client_id SET NOT NULL;
DROP INDEX IF EXISTS idx_session_messages_client_id;
CREATE UNIQUE INDEX idx_session_messages_client_id
  ON session_messages(session_id, sender_id, client_id);
