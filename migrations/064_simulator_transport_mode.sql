-- Add transport_mode to simulator_config
-- 'direct' = bypass transport (fast), 'http' = HTTP Device API, 'mqtt' = MQTT broker
ALTER TABLE simulator_config ADD COLUMN IF NOT EXISTS transport_mode VARCHAR(16) NOT NULL DEFAULT 'direct';
