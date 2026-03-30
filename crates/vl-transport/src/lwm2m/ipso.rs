use std::collections::HashMap;

/// IPSO Smart Object registry: maps (object_id, resource_id) → telemetry key name.
/// Covers OMA IPSO Alliance standard objects commonly used in LwM2M devices.
///
/// Reference: http://www.openmobilealliance.org/wp/OMNA/LwM2M/LwM2MRegistry.html
pub struct IpsoRegistry {
    mappings: HashMap<(u16, u16), &'static str>,
}

impl IpsoRegistry {
    /// Build the standard IPSO registry with well-known object/resource mappings.
    pub fn standard() -> Self {
        let mut m: HashMap<(u16, u16), &'static str> = HashMap::new();

        // ── Object 3: Device ─────────────────────────────────────────────────
        m.insert((3, 0),  "manufacturer");
        m.insert((3, 1),  "model_number");
        m.insert((3, 2),  "serial_number");
        m.insert((3, 3),  "firmware_version");
        m.insert((3, 9),  "battery_level");     // 0..100 %
        m.insert((3, 10), "memory_free");        // KiB
        m.insert((3, 20), "battery_status");

        // ── Object 4: Connectivity Monitoring ────────────────────────────────
        m.insert((4, 0),  "network_bearer");
        m.insert((4, 2),  "radio_signal_strength");
        m.insert((4, 3),  "link_quality");
        m.insert((4, 4),  "ip_address");
        m.insert((4, 8),  "link_utilization");

        // ── Object 5: Firmware Update ─────────────────────────────────────────
        m.insert((5, 3),  "fw_update_state");
        m.insert((5, 5),  "fw_update_result");

        // ── Object 6: Location ────────────────────────────────────────────────
        m.insert((6, 0),  "latitude");
        m.insert((6, 1),  "longitude");
        m.insert((6, 2),  "altitude");
        m.insert((6, 3),  "radius");
        m.insert((6, 4),  "velocity");
        m.insert((6, 5),  "location_timestamp");
        m.insert((6, 6),  "speed");

        // ── Object 3300: Generic Sensor ───────────────────────────────────────
        m.insert((3300, 5700), "sensor_value");
        m.insert((3300, 5701), "sensor_units");
        m.insert((3300, 5702), "min_measured_value");
        m.insert((3300, 5703), "max_measured_value");

        // ── Object 3301: Illuminance ──────────────────────────────────────────
        m.insert((3301, 5700), "illuminance");
        m.insert((3301, 5701), "illuminance_units");

        // ── Object 3302: Presence ─────────────────────────────────────────────
        m.insert((3302, 5500), "presence");
        m.insert((3302, 5501), "presence_counter");

        // ── Object 3303: Temperature ──────────────────────────────────────────
        m.insert((3303, 5700), "temperature");
        m.insert((3303, 5701), "temperature_units");
        m.insert((3303, 5602), "temperature_min");
        m.insert((3303, 5603), "temperature_max");

        // ── Object 3304: Humidity ─────────────────────────────────────────────
        m.insert((3304, 5700), "humidity");
        m.insert((3304, 5701), "humidity_units");

        // ── Object 3305: Power Measurement ───────────────────────────────────
        m.insert((3305, 5800), "instant_active_power");
        m.insert((3305, 5801), "min_measured_active_power");
        m.insert((3305, 5802), "max_measured_active_power");
        m.insert((3305, 5805), "cumulative_active_power");
        m.insert((3305, 5820), "active_power_calibration");

        // ── Object 3306: Actuation ────────────────────────────────────────────
        m.insert((3306, 5850), "on_off");
        m.insert((3306, 5851), "dimmer");
        m.insert((3306, 5852), "on_time");

        // ── Object 3311: Light Control ────────────────────────────────────────
        m.insert((3311, 5850), "light_on");
        m.insert((3311, 5851), "light_dimmer");
        m.insert((3311, 5852), "light_on_time");
        m.insert((3311, 5706), "light_colour");

        // ── Object 3313: Accelerometer ────────────────────────────────────────
        m.insert((3313, 5702), "accel_x");
        m.insert((3313, 5703), "accel_y");
        m.insert((3313, 5704), "accel_z");

        // ── Object 3314: Magnetometer ─────────────────────────────────────────
        m.insert((3314, 5702), "mag_x");
        m.insert((3314, 5703), "mag_y");
        m.insert((3314, 5704), "mag_z");

        // ── Object 3315: Barometer ────────────────────────────────────────────
        m.insert((3315, 5700), "barometric_pressure");
        m.insert((3315, 5701), "barometric_pressure_units");

        // ── Object 3316: Voltage ──────────────────────────────────────────────
        m.insert((3316, 5700), "voltage");
        m.insert((3316, 5701), "voltage_units");

        // ── Object 3317: Current ──────────────────────────────────────────────
        m.insert((3317, 5700), "current");
        m.insert((3317, 5701), "current_units");

        // ── Object 3318: Frequency ────────────────────────────────────────────
        m.insert((3318, 5700), "frequency");

        // ── Object 3320: Direction ────────────────────────────────────────────
        m.insert((3320, 5705), "compass_direction");

        // ── Object 3321: Distance ─────────────────────────────────────────────
        m.insert((3321, 5700), "distance");

        // ── Object 3323: Pressure ─────────────────────────────────────────────
        m.insert((3323, 5700), "pressure");

        // ── Object 3325: Concentration ───────────────────────────────────────
        m.insert((3325, 5700), "concentration");
        m.insert((3325, 5701), "concentration_units");

        // ── Object 3326: Acidity ──────────────────────────────────────────────
        m.insert((3326, 5700), "acidity");

        // ── Object 3327: Conductivity ─────────────────────────────────────────
        m.insert((3327, 5700), "conductivity");

        // ── Object 3328: Power ────────────────────────────────────────────────
        m.insert((3328, 5700), "power");

        // ── Object 3329: Power Factor ─────────────────────────────────────────
        m.insert((3329, 5700), "power_factor");

        // ── Object 3330: Distance (sensor) ───────────────────────────────────
        m.insert((3330, 5700), "distance_sensor");

        Self { mappings: m }
    }

    /// Resolve an IPSO (object_id, resource_id) pair to a human-readable telemetry key.
    /// Falls back to `None` if no mapping exists — callers can then use the OID string.
    pub fn telemetry_key(&self, object_id: u16, resource_id: u16) -> Option<&'static str> {
        self.mappings.get(&(object_id, resource_id)).copied()
    }
}

/// Global lazy-initialised IPSO registry.
/// Created once and reused across all LwM2M connections.
use std::sync::OnceLock;
static IPSO: OnceLock<IpsoRegistry> = OnceLock::new();

pub fn ipso() -> &'static IpsoRegistry {
    IPSO.get_or_init(IpsoRegistry::standard)
}
