-- Dead-Letter Queue: messages that failed Rule Engine processing
-- Created in Phase P6 — Queue Backends

CREATE TABLE dlq_messages (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    topic         TEXT        NOT NULL,
    payload       BYTEA       NOT NULL,
    error_message TEXT,
    retry_count   INT         NOT NULL DEFAULT 0,
    -- PENDING | REPLAYED | DISCARDED
    status        TEXT        NOT NULL DEFAULT 'PENDING',
    created_at    BIGINT      NOT NULL,
    updated_at    BIGINT      NOT NULL
);

CREATE INDEX idx_dlq_topic  ON dlq_messages (topic);
CREATE INDEX idx_dlq_status ON dlq_messages (status) WHERE status = 'PENDING';
