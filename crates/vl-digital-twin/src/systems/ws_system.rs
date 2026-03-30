//! WebSocket Bevy systems — drains the event queue và fires Bevy events.

use bevy::prelude::*;

use crate::api::DeviceListCache;
use crate::events::{AlarmUpdate, AttributeUpdate, TelemetryUpdate, WsStatusEvent};
use crate::ws::{WsConfig, WsConnectionStatus, WsEventQueue, WsIncomingEvent, WsSubscriptions};

/// Startup system: khởi động WS background thread.
/// Chạy sau khi devices đã load để có danh sách subscriptions.
pub fn setup_ws_connection(
    config:       Res<WsConfig>,
    queue:        Res<WsEventQueue>,
    mut subs:     ResMut<WsSubscriptions>,
    device_cache: Res<DeviceListCache>,
) {
    for device in &device_cache.devices {
        subs.add(device.id.id);
    }

    let entries      = subs.entries.clone();
    let attr_entries = subs.attr_entries.clone();
    crate::ws::start_ws((*config).clone(), (*queue).clone(), entries, attr_entries);

    tracing::info!(
        server = %config.server_url,
        subs   = subs.entries.len(),
        "WS connection started",
    );
}

/// Update system: drain shared queue và emit Bevy events.
pub fn drain_ws_events(
    queue:       Res<WsEventQueue>,
    subs:        Res<WsSubscriptions>,
    mut status:  ResMut<WsConnectionStatus>,
    mut ts_ev:   MessageWriter<TelemetryUpdate>,
    _al_ev:      MessageWriter<AlarmUpdate>,
    mut st_ev:   MessageWriter<WsStatusEvent>,
    mut attr_ev: MessageWriter<AttributeUpdate>,
) {
    for event in queue.drain() {
        match event {
            WsIncomingEvent::Connected => {
                status.connected          = true;
                status.reconnecting       = false;
                status.error              = None;
                status.next_retry_ms      = None;
                status.disconnected_at_ms = None;
                st_ev.write(WsStatusEvent::Connected);
                tracing::info!("WS connected");
            }

            WsIncomingEvent::Disconnected(reason) => {
                status.connected           = false;
                status.reconnecting        = false;
                status.error               = Some(reason.clone());
                status.reconnect_attempts += 1;
                status.disconnected_at_ms  = Some(crate::components::current_time_ms());
                st_ev.write(WsStatusEvent::Disconnected(reason));
            }

            WsIncomingEvent::Reconnecting { attempt, delay_ms } => {
                status.reconnecting       = true;
                status.reconnect_attempts = attempt;
                status.next_retry_ms      = Some(delay_ms);
                tracing::debug!(attempt, delay_ms, "WS will reconnect");
            }

            WsIncomingEvent::Telemetry { subscription_id, data } => {
                let Some(device_id) = subs.device_for_cmd(subscription_id) else { continue };

                for (key, values) in &data {
                    if let Some(entry) = values.last() {
                        let ts = entry[0].as_i64().unwrap_or(0);
                        let value = entry[1]
                            .as_f64()
                            .or_else(|| entry[1].as_str().and_then(|s| s.parse::<f64>().ok()))
                            .unwrap_or(0.0);

                        ts_ev.write(TelemetryUpdate {
                            cmd_id: subscription_id,
                            device_id,
                            key: key.clone(),
                            value,
                            ts,
                        });
                    }
                }
            }

            WsIncomingEvent::AttributeUpdate { subscription_id, data } => {
                let Some(device_id) = subs.device_for_attr_cmd(subscription_id) else { continue };
                for (key, value) in data {
                    attr_ev.write(AttributeUpdate { device_id, key, value });
                }
            }
        }
    }
}
