-- P2: OTA Device State tracking
-- Tracks per-device firmware update progress

CREATE TABLE IF NOT EXISTS ota_device_state (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id       UUID NOT NULL REFERENCES device(id) ON DELETE CASCADE,
    ota_package_id  UUID NOT NULL REFERENCES ota_package(id),
    status          TEXT NOT NULL DEFAULT 'QUEUED',
    error           TEXT,
    created_time    BIGINT NOT NULL,
    updated_time    BIGINT NOT NULL,
    UNIQUE (device_id, ota_package_id)
);

CREATE INDEX idx_ota_state_device  ON ota_device_state(device_id);
CREATE INDEX idx_ota_state_package ON ota_device_state(ota_package_id);
CREATE INDEX idx_ota_state_status  ON ota_device_state(status) WHERE status != 'UPDATED';
