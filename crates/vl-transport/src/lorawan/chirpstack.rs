/// ChirpStack v4 uplink / join / status message types.
///
/// Reference: <https://www.chirpstack.io/docs/chirpstack/api/>
use std::collections::HashMap;

use serde::Deserialize;

// ── Uplink (event/up) ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChirpStackUplink {
    pub device_info: ChirpDeviceInfo,
    /// Base64-encoded application payload.
    pub data:        String,
    /// Frame counter.
    #[serde(rename = "fCnt", default)]
    pub f_cnt:       u32,
    /// ISO 8601 timestamp, e.g. "2024-01-01T00:00:00Z".
    #[serde(default)]
    pub time:        String,
    #[serde(rename = "rxInfo", default)]
    pub rx_info:     Vec<RxInfo>,
    #[serde(rename = "txInfo", default)]
    pub tx_info:     Option<TxInfo>,
    /// True if this is a confirmed uplink.
    #[serde(default)]
    pub confirmed:   bool,
    /// LoRaWAN f-port (application port).
    #[serde(rename = "fPort", default)]
    pub f_port:      u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChirpDeviceInfo {
    /// Device EUI (hex string, 16 chars), e.g. "0102030405060708".
    #[serde(rename = "devEui")]
    pub dev_eui:        String,
    #[serde(rename = "applicationId", default)]
    pub application_id: String,
    #[serde(rename = "deviceName", default)]
    pub device_name:    String,
    #[serde(default)]
    pub tags:           HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RxInfo {
    /// RSSI in dBm.
    #[serde(default)]
    pub rssi:    i32,
    /// Signal-to-noise ratio in dB.
    #[serde(rename = "snr", default)]
    pub snr:     f64,
    #[serde(rename = "gatewayId", default)]
    pub gateway_id: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TxInfo {
    #[serde(default)]
    pub frequency: u64,
}

// ── Join event (event/join) ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChirpStackJoin {
    pub device_info: ChirpDeviceInfo,
    #[serde(rename = "devAddr", default)]
    pub dev_addr:    String,
    #[serde(default)]
    pub time:        String,
}

// ── Device status (event/status) ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChirpStackStatus {
    pub device_info:   ChirpDeviceInfo,
    /// Device battery level: 0–254 = percentage, 255 = external power.
    #[serde(default)]
    pub battery_level: u8,
    /// True if battery level is not available.
    #[serde(default)]
    pub battery_level_unavailable: bool,
    /// External power source connected.
    #[serde(default)]
    pub external_power_source: bool,
    #[serde(default)]
    pub margin: i32,
}

// ── Topic helpers ─────────────────────────────────────────────────────────────

/// Parse the `dev_eui` and `event` from a ChirpStack v4 MQTT topic.
/// Topic format: `application/<app_id>/device/<dev_eui>/event/<event>`
pub fn parse_topic(topic: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = topic.split('/').collect();
    // application / <app_id> / device / <dev_eui> / event / <event>
    if parts.len() == 6 && parts[0] == "application" && parts[2] == "device" && parts[4] == "event" {
        Some((parts[3].to_string(), parts[5].to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chirpstack_topic() {
        let t = "application/abc/device/0102030405060708/event/up";
        let (eui, event) = parse_topic(t).unwrap();
        assert_eq!(eui,   "0102030405060708");
        assert_eq!(event, "up");
    }

    #[test]
    fn parse_unknown_topic_returns_none() {
        assert!(parse_topic("some/random/topic").is_none());
    }

    #[test]
    fn deserialize_uplink() {
        let json = r#"{
            "deviceInfo": { "devEui": "aabbccdd11223344", "applicationId": "1", "deviceName": "test" },
            "data": "AQID",
            "fCnt": 42,
            "rxInfo": [{"rssi": -80, "snr": 7.5, "gatewayId": "gw1"}]
        }"#;
        let uplink: ChirpStackUplink = serde_json::from_str(json).unwrap();
        assert_eq!(uplink.device_info.dev_eui, "aabbccdd11223344");
        assert_eq!(uplink.f_cnt, 42);
        assert_eq!(uplink.rx_info[0].rssi, -80);
    }
}
