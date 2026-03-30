-- VieLang Phase 62 — Partitioned event tables (matching ThingsBoard Java schema)
-- Replaces the generic `event` table with type-specific partitioned tables
-- for better query performance and time-based data retention.

-- ── Rule Node Debug Event ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS rule_node_debug_event (
    id               UUID        NOT NULL DEFAULT gen_random_uuid(),
    tenant_id        UUID        NOT NULL,
    ts               BIGINT      NOT NULL,
    entity_id        UUID        NOT NULL,
    service_id       VARCHAR,
    e_type           VARCHAR,
    e_entity_id      UUID,
    e_entity_type    VARCHAR,
    e_msg_id         UUID,
    e_msg_type       VARCHAR,
    e_data_type      VARCHAR,
    e_relation_type  VARCHAR,
    e_data           VARCHAR,
    e_metadata       VARCHAR,
    e_error          VARCHAR
) PARTITION BY RANGE (ts);

CREATE INDEX IF NOT EXISTS idx_rule_node_debug_event_tenant_entity
    ON rule_node_debug_event (tenant_id, entity_id, ts DESC);

-- ── Rule Chain Debug Event ─────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS rule_chain_debug_event (
    id               UUID        NOT NULL DEFAULT gen_random_uuid(),
    tenant_id        UUID        NOT NULL,
    ts               BIGINT      NOT NULL,
    entity_id        UUID        NOT NULL,
    service_id       VARCHAR     NOT NULL,
    e_message        VARCHAR,
    e_error          VARCHAR
) PARTITION BY RANGE (ts);

CREATE INDEX IF NOT EXISTS idx_rule_chain_debug_event_tenant_entity
    ON rule_chain_debug_event (tenant_id, entity_id, ts DESC);

-- ── Stats Event ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS stats_event (
    id                    UUID        NOT NULL DEFAULT gen_random_uuid(),
    tenant_id             UUID        NOT NULL,
    ts                    BIGINT      NOT NULL,
    entity_id             UUID        NOT NULL,
    service_id            VARCHAR     NOT NULL,
    e_messages_processed  BIGINT      NOT NULL DEFAULT 0,
    e_errors_occurred     BIGINT      NOT NULL DEFAULT 0
) PARTITION BY RANGE (ts);

CREATE INDEX IF NOT EXISTS idx_stats_event_tenant_entity
    ON stats_event (tenant_id, entity_id, ts DESC);

-- ── Lifecycle Event ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS lc_event (
    id               UUID        NOT NULL DEFAULT gen_random_uuid(),
    tenant_id        UUID        NOT NULL,
    ts               BIGINT      NOT NULL,
    entity_id        UUID        NOT NULL,
    service_id       VARCHAR     NOT NULL,
    e_type           VARCHAR     NOT NULL,
    e_success        BOOLEAN     NOT NULL,
    e_error          VARCHAR
) PARTITION BY RANGE (ts);

CREATE INDEX IF NOT EXISTS idx_lc_event_tenant_entity
    ON lc_event (tenant_id, entity_id, ts DESC);

-- ── Error Event ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS error_event (
    id               UUID        NOT NULL DEFAULT gen_random_uuid(),
    tenant_id        UUID        NOT NULL,
    ts               BIGINT      NOT NULL,
    entity_id        UUID        NOT NULL,
    service_id       VARCHAR     NOT NULL,
    e_method         VARCHAR     NOT NULL,
    e_error          VARCHAR
) PARTITION BY RANGE (ts);

CREATE INDEX IF NOT EXISTS idx_error_event_tenant_entity
    ON error_event (tenant_id, entity_id, ts DESC);

-- ── Calculated Field Debug Event ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS cf_debug_event (
    id               UUID        NOT NULL DEFAULT gen_random_uuid(),
    tenant_id        UUID        NOT NULL,
    ts               BIGINT      NOT NULL,
    entity_id        UUID        NOT NULL,
    service_id       VARCHAR,
    cf_id            UUID        NOT NULL,
    e_entity_id      UUID,
    e_entity_type    VARCHAR,
    e_msg_id         UUID,
    e_msg_type       VARCHAR,
    e_args           VARCHAR,
    e_result         VARCHAR,
    e_error          VARCHAR
) PARTITION BY RANGE (ts);

CREATE INDEX IF NOT EXISTS idx_cf_debug_event_tenant_entity
    ON cf_debug_event (tenant_id, entity_id, ts DESC);

-- ── Monthly partitions: 2026-01 through 2027-12 ───────────────────────────────
-- Timestamps are milliseconds since epoch.
-- 2026-01-01 00:00:00 UTC = 1767225600000
-- Each month is ~2592000000 ms (30 days) but we use exact boundaries.

DO $$
DECLARE
    tbl TEXT;
    y   INT;
    m   INT;
    ts_start BIGINT;
    ts_end   BIGINT;
    part_name TEXT;
BEGIN
    FOREACH tbl IN ARRAY ARRAY[
        'rule_node_debug_event',
        'rule_chain_debug_event',
        'stats_event',
        'lc_event',
        'error_event',
        'cf_debug_event'
    ] LOOP
        FOR y IN 2026..2027 LOOP
            FOR m IN 1..12 LOOP
                -- Calculate epoch ms for start of month
                ts_start := (EXTRACT(EPOCH FROM make_date(y, m, 1)::TIMESTAMP AT TIME ZONE 'UTC') * 1000)::BIGINT;
                -- Calculate epoch ms for start of next month
                IF m = 12 THEN
                    ts_end := (EXTRACT(EPOCH FROM make_date(y + 1, 1, 1)::TIMESTAMP AT TIME ZONE 'UTC') * 1000)::BIGINT;
                ELSE
                    ts_end := (EXTRACT(EPOCH FROM make_date(y, m + 1, 1)::TIMESTAMP AT TIME ZONE 'UTC') * 1000)::BIGINT;
                END IF;

                part_name := tbl || '_' || y || '_' || LPAD(m::TEXT, 2, '0');

                EXECUTE format(
                    'CREATE TABLE IF NOT EXISTS %I PARTITION OF %I FOR VALUES FROM (%s) TO (%s)',
                    part_name, tbl, ts_start, ts_end
                );
            END LOOP;
        END LOOP;
    END LOOP;
END $$;
