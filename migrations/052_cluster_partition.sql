-- P15: Cluster Raft — partition assignment tracking + leader election flag

-- Add leader-election columns to cluster_node (idempotent)
ALTER TABLE cluster_node
    ADD COLUMN IF NOT EXISTS is_leader    BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS grpc_port    INT     NOT NULL DEFAULT 9091,
    ADD COLUMN IF NOT EXISTS leader_epoch BIGINT  NOT NULL DEFAULT 0;

-- Only one node can be leader at a time
CREATE UNIQUE INDEX IF NOT EXISTS idx_cluster_node_leader
    ON cluster_node (is_leader)
    WHERE is_leader = TRUE;

-- Partition → node assignment table
CREATE TABLE IF NOT EXISTS cluster_partition (
    partition_id  INT          NOT NULL,
    node_id       VARCHAR(255) REFERENCES cluster_node(node_id) ON DELETE SET NULL,
    assigned_at   BIGINT       NOT NULL,
    PRIMARY KEY (partition_id)
);

CREATE INDEX IF NOT EXISTS idx_cluster_partition_node
    ON cluster_partition (node_id);
