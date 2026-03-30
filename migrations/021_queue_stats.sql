CREATE TABLE IF NOT EXISTS queue_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    queue_name VARCHAR(255) NOT NULL,
    messages_total BIGINT NOT NULL DEFAULT 0,
    messages_per_second DOUBLE PRECISION NOT NULL DEFAULT 0,
    consumers_total INT NOT NULL DEFAULT 0,
    lag BIGINT NOT NULL DEFAULT 0,
    collected_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
    UNIQUE(queue_name, collected_at)
);
CREATE INDEX IF NOT EXISTS idx_queue_stats_name ON queue_stats(queue_name, collected_at DESC);
