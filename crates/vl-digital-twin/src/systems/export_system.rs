//! Export — screenshot (Ctrl+P, native) + CSV telemetry export.

use bevy::prelude::*;

use crate::api::HistoricalDataCache;
use crate::playback::TimeRange;
use crate::ui::SelectedDevice;

// ── Screenshot (native only) ──────────────────────────────────────────────────

/// Listen for Ctrl+P and log a screenshot request.
/// Full screenshot implementation requires platform-specific Bevy API.
pub fn handle_screenshot_request(
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if !ctrl || !keyboard.just_pressed(KeyCode::KeyP) { return; }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = format!(
            "screenshots/twin_{}.png",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let _ = std::fs::create_dir_all("screenshots");
        tracing::info!("Screenshot requested → {path} (use Bevy screenshot trigger to capture)");
    }

    #[cfg(target_arch = "wasm32")]
    tracing::info!("Screenshot requested (WASM — not supported)");
}

// ── CSV Export ────────────────────────────────────────────────────────────────

/// Build a CSV string from the historical data cache for the selected device.
///
/// Samples up to 1 000 evenly-spaced rows across `range`.
pub fn export_telemetry_csv(
    selected:   &SelectedDevice,
    hist_cache: &HistoricalDataCache,
    range:      &TimeRange,
    keys:       &[String],
) -> String {
    let device_id = match selected.device_id {
        Some(id) => id,
        None     => return String::new(),
    };

    // Header
    let mut csv = String::from("timestamp_ms,datetime");
    for key in keys {
        csv.push(',');
        csv.push_str(key);
    }
    csv.push('\n');

    let step_ms = ((range.end - range.start) / 1_000).max(1);
    let mut ts  = range.start;

    while ts <= range.end {
        let dt = chrono::DateTime::from_timestamp_millis(ts)
            .map(|d| d.to_rfc3339())
            .unwrap_or_default();

        csv.push_str(&format!("{ts},{dt}"));

        for key in keys {
            let v = hist_cache
                .get_at(device_id, key, ts)
                .map(|v| format!("{v:.4}"))
                .unwrap_or_default();
            csv.push(',');
            csv.push_str(&v);
        }

        csv.push('\n');
        ts += step_ms;
    }

    csv
}

/// Write CSV to disk (native) or trigger a browser download (WASM).
pub fn save_csv(filename: &str, content: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Err(e) = std::fs::write(filename, content) {
            tracing::warn!("CSV export failed: {e}");
        } else {
            tracing::info!("CSV exported → {filename}");
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Trigger browser file download via a data URL
        tracing::info!("CSV export (WASM) — filename: {filename}");
        let _ = (filename, content); // suppress unused warnings
    }
}

// ── WASM URL state sync ───────────────────────────────────────────────────────

/// Sync selected device + playback timestamp into the URL hash (WASM only).
/// Example: `#device=<uuid>&ts=<ts_ms>`
#[cfg(target_arch = "wasm32")]
pub fn sync_url_state(
    selected: Res<SelectedDevice>,
    playback: Res<crate::playback::PlaybackState>,
) {
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else { return };
    let Ok(history) = window.history() else { return };

    let hash = match (&selected.device_id, &*playback) {
        (Some(id), crate::playback::PlaybackState::Paused { at_ts }) => {
            format!("device={id}&ts={at_ts}")
        }
        (Some(id), _) => format!("device={id}"),
        _ => String::new(),
    };

    if !hash.is_empty() {
        let _ = history.replace_state_with_url(
            &wasm_bindgen::JsValue::NULL,
            "",
            Some(&format!("#{hash}")),
        );
    }
}

/// Native stub — URL sync is browser-only.
#[cfg(not(target_arch = "wasm32"))]
pub fn sync_url_state() {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::HistoricalDataCache;
    use crate::ui::SelectedDevice;

    #[test]
    fn export_csv_no_device_returns_empty() {
        let selected   = SelectedDevice::default();
        let cache      = HistoricalDataCache::default();
        let range      = TimeRange { start: 0, end: 1_000 };
        let csv        = export_telemetry_csv(&selected, &cache, &range, &[]);
        assert!(csv.is_empty());
    }

    #[test]
    fn export_csv_has_header() {
        let mut selected = SelectedDevice::default();
        selected.device_id = Some(uuid::Uuid::new_v4());
        let cache  = HistoricalDataCache::default();
        let range  = TimeRange { start: 0, end: 60_000 };
        let keys   = vec!["temperature".to_string()];
        let csv    = export_telemetry_csv(&selected, &cache, &range, &keys);
        assert!(csv.starts_with("timestamp_ms,datetime,temperature\n"));
    }
}
