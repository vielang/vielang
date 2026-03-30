-- Phase 39: Cluster Node Discovery & Health Tracking

CREATE TABLE IF NOT EXISTS cluster_node (
    node_id        VARCHAR(255) PRIMARY KEY,
    host           VARCHAR(255) NOT NULL,
    port           INT          NOT NULL,
    status         VARCHAR(32)  NOT NULL DEFAULT 'ACTIVE',
    service_type   VARCHAR(64)  NOT NULL DEFAULT 'MONOLITH',
    last_heartbeat BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
    joined_at      BIGINT       NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()) * 1000,
    metadata       JSONB        NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_cluster_node_status      ON cluster_node(status);
CREATE INDEX IF NOT EXISTS idx_cluster_node_heartbeat   ON cluster_node(last_heartbeat);
