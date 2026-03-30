-- VieLang Phase 1 — Time Series Schema
-- Dùng PostgreSQL partition by range trên ts để scale
-- (TimescaleDB là optional extension, có thể dùng sau)

CREATE TABLE IF NOT EXISTS ts_kv (
    entity_id   UUID    NOT NULL,
    key         INTEGER NOT NULL,
    ts          BIGINT  NOT NULL,
    bool_v      BOOLEAN,
    str_v       VARCHAR(10000000),
    long_v      BIGINT,
    dbl_v       DOUBLE PRECISION,
    json_v      JSON,
    CONSTRAINT fk_ts_kv_key FOREIGN KEY (key) REFERENCES key_dictionary(key_id)
) PARTITION BY RANGE (ts);

-- Partition 2023 (cho tests với timestamp cố định)
CREATE TABLE IF NOT EXISTS ts_kv_2023
    PARTITION OF ts_kv
    FOR VALUES FROM (1672531200000) TO (1704067200000);  -- 2023-01-01 → 2024-01-01

-- Partition 2024
CREATE TABLE IF NOT EXISTS ts_kv_2024
    PARTITION OF ts_kv
    FOR VALUES FROM (1704067200000) TO (1735689600000);  -- 2024-01-01 → 2025-01-01

-- Partition 2025
CREATE TABLE IF NOT EXISTS ts_kv_2025
    PARTITION OF ts_kv
    FOR VALUES FROM (1735689600000) TO (1767225600000);  -- 2025-01-01 → 2026-01-01

-- Partition 2026 Q1 (Jan - Apr)
CREATE TABLE IF NOT EXISTS ts_kv_2026_q1
    PARTITION OF ts_kv
    FOR VALUES FROM (1767225600000) TO (1775001600000);  -- 2026-01-01 → 2026-04-01

-- Partition 2026 Q2 (Apr - Jul)
CREATE TABLE IF NOT EXISTS ts_kv_2026_q2
    PARTITION OF ts_kv
    FOR VALUES FROM (1775001600000) TO (1782864000000);  -- 2026-04-01 → 2026-07-01

-- Partition 2026 Q3 (Jul - Oct)
CREATE TABLE IF NOT EXISTS ts_kv_2026_q3
    PARTITION OF ts_kv
    FOR VALUES FROM (1782864000000) TO (1790784000000);  -- 2026-07-01 → 2026-10-01

-- Partition 2026 Q4 (Oct - 2027)
CREATE TABLE IF NOT EXISTS ts_kv_2026_q4
    PARTITION OF ts_kv
    FOR VALUES FROM (1790784000000) TO (1798761600000);  -- 2026-10-01 → 2027-01-01

-- Partition 2027
CREATE TABLE IF NOT EXISTS ts_kv_2027
    PARTITION OF ts_kv
    FOR VALUES FROM (1798761600000) TO (1830297600000);  -- 2027-01-01 → 2028-01-01

CREATE INDEX IF NOT EXISTS idx_ts_kv_entity_key_ts
    ON ts_kv (entity_id, key, ts DESC);
