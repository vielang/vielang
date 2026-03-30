-- P16: LoRaWAN device identifier
-- lora_dev_eui: ChirpStack Device EUI (8-byte hex string, e.g. "0102030405060708")
-- Indexed for fast lookup on ChirpStack uplink messages.
ALTER TABLE device
    ADD COLUMN IF NOT EXISTS lora_dev_eui TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_device_lora_dev_eui
    ON device (lora_dev_eui)
    WHERE lora_dev_eui IS NOT NULL;
