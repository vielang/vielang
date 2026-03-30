-- Phase P11: Queue Message Persistence
-- Stores in-flight queue messages in PostgreSQL for crash-safe replay.
-- Consumer reads unacked messages after_offset; marks acked_time after processing.

CREATE SEQUENCE IF NOT EXISTS queue_offset_seq START 1;

CREATE TABLE IF NOT EXISTS queue_message (
    id           UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
    topic        TEXT    NOT NULL,
    partition_id INT     NOT NULL DEFAULT 0,
    offset_value BIGINT  NOT NULL DEFAULT nextval('queue_offset_seq'),
    payload      BYTEA   NOT NULL,
    headers      JSONB,
    created_time BIGINT  NOT NULL,
    acked_time   BIGINT,
    consumer_id  TEXT
);

CREATE INDEX IF NOT EXISTS idx_qmsg_topic_offset
    ON queue_message(topic, partition_id, offset_value)
    WHERE acked_time IS NULL;

CREATE INDEX IF NOT EXISTS idx_qmsg_created
    ON queue_message(created_time);

-- Extend queue_stats with pending-message counters (used by persistent backend)
ALTER TABLE queue_stats
    ADD COLUMN IF NOT EXISTS messages_pending BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS bytes_pending    BIGINT NOT NULL DEFAULT 0;
