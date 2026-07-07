-- M4.4 — Notification log.
--
-- Every attempted email delivery (success + drop) writes a row. Powers:
--   • "did the T-24h reminder fire for session X?" — dedupe key.
--   • "how many bounces last week?" — rollups.
--   • "why did the confirmation drop?" — reason + error fields.
--
-- RLS: not enabled here (yet). All writes come from the server-side
-- service_role client via lib/email.ts. When we open anon reads to the
-- table (e.g. an in-app "delivery history" widget) we'll add per-user
-- policies alongside.

CREATE TABLE notification_log (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  kind          TEXT NOT NULL,
  recipient     TEXT NOT NULL,
  user_id       UUID REFERENCES users(id) ON DELETE SET NULL,
  session_id    UUID REFERENCES sessions(id) ON DELETE SET NULL,
  delivered     BOOLEAN NOT NULL,
  provider_id   TEXT,                 -- e.g. Resend's message id
  reason        TEXT,                 -- 'no_provider' | 'no_recipient' | 'provider_error'
  error         TEXT,                 -- provider error message when delivered=false
  metadata      JSONB,                -- extra tags: template version, campaign, etc.
  sent_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Rolling-window queries: "was a reminder already fired for this session?"
CREATE INDEX idx_notification_log_session_kind
  ON notification_log(session_id, kind, sent_at DESC);

-- Cross-user rollups + admin dashboards.
CREATE INDEX idx_notification_log_kind_time
  ON notification_log(kind, sent_at DESC);

-- Bounce analytics.
CREATE INDEX idx_notification_log_delivered
  ON notification_log(delivered, sent_at DESC)
  WHERE delivered = FALSE;
