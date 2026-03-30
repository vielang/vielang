-- Device Templates — predefined simulation configurations with Wokwi diagrams
CREATE TABLE IF NOT EXISTS device_template (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name             VARCHAR(255) NOT NULL,
    description      TEXT NOT NULL DEFAULT '',
    category         VARCHAR(64) NOT NULL,
    telemetry_schema JSONB NOT NULL DEFAULT '[]',
    diagram          JSONB NOT NULL DEFAULT '{}',
    is_builtin       BOOLEAN NOT NULL DEFAULT false,
    tenant_id        UUID,
    created_time     BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT * 1000
);

CREATE INDEX IF NOT EXISTS idx_device_template_tenant ON device_template(tenant_id);
CREATE INDEX IF NOT EXISTS idx_device_template_builtin ON device_template(is_builtin) WHERE is_builtin = true;

-- Seed builtin templates
INSERT INTO device_template (name, description, category, telemetry_schema, diagram, is_builtin) VALUES
(
    'Weather Station',
    'DHT22 temperature & humidity sensor with LED indicator and buzzer alarm',
    'environment',
    '[{"key":"temperature","dataType":"DOUBLE","generator":{"type":"SINE_WAVE","amplitude":10.0,"offset":22.0,"periodMs":60000}},{"key":"humidity","dataType":"DOUBLE","generator":{"type":"RANDOM","min":30.0,"max":90.0}},{"key":"alarm","dataType":"BOOLEAN","generator":{"type":"CONSTANT","value":false}}]',
    '{"version":1,"parts":[{"id":"dht22-1","type":"wokwi-dht22","x":100,"y":150,"attrs":{}},{"id":"led-1","type":"wokwi-led","x":250,"y":150,"attrs":{"color":"green"}},{"id":"buzzer-1","type":"wokwi-buzzer","x":250,"y":250,"attrs":{}}],"connections":[]}',
    true
),
(
    'Smart Light',
    'LED with brightness control via sine wave pattern',
    'lighting',
    '[{"key":"brightness","dataType":"LONG","generator":{"type":"SINE_WAVE","amplitude":127.0,"offset":128.0,"periodMs":30000}}]',
    '{"version":1,"parts":[{"id":"led-1","type":"wokwi-led","x":200,"y":200,"attrs":{"color":"white"}}],"connections":[]}',
    true
),
(
    'Motion Detector',
    'PIR motion sensor with alarm LED indicator',
    'security',
    '[{"key":"motion","dataType":"BOOLEAN","generator":{"type":"RANDOM","min":0.0,"max":1.0}},{"key":"alarm_led","dataType":"BOOLEAN","generator":{"type":"CONSTANT","value":false}}]',
    '{"version":1,"parts":[{"id":"pir-1","type":"wokwi-pir-motion-sensor","x":100,"y":200,"attrs":{}},{"id":"led-1","type":"wokwi-led","x":250,"y":200,"attrs":{"color":"red"}}],"connections":[]}',
    true
),
(
    'Servo Controller',
    'Servo motor with angle controlled by sine wave',
    'robotics',
    '[{"key":"angle","dataType":"DOUBLE","generator":{"type":"SINE_WAVE","amplitude":90.0,"offset":90.0,"periodMs":10000}}]',
    '{"version":1,"parts":[{"id":"servo-1","type":"wokwi-servo","x":200,"y":200,"attrs":{}}],"connections":[]}',
    true
),
(
    'Distance Monitor',
    'HC-SR04 ultrasonic distance sensor with NeoPixel indicator',
    'measurement',
    '[{"key":"distance","dataType":"DOUBLE","generator":{"type":"RANDOM","min":5.0,"max":200.0}},{"key":"color_r","dataType":"LONG","generator":{"type":"RANDOM","min":0.0,"max":255.0}}]',
    '{"version":1,"parts":[{"id":"hcsr04-1","type":"wokwi-hc-sr04","x":100,"y":150,"attrs":{}},{"id":"neo-1","type":"wokwi-neopixel","x":250,"y":150,"attrs":{}}],"connections":[]}',
    true
);
