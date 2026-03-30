//! Playback systems — advance the timeline clock and fetch historical data.

use bevy::prelude::*;

use crate::{
    api::{ApiClient, ApiConfig, HistoricalDataCache},
    api::client::TimeseriesQuery,
    events::FetchHistoryRequest,
    playback::{PlaybackState, TimeRange},
};

// ── Playback clock ────────────────────────────────────────────────────────────

/// Advance `PlaybackState::Playing` by scaled delta time each frame.
/// Automatically transitions to `Paused` when the end of the range is reached.
pub fn update_playback(
    time:      Res<Time>,
    mut state: ResMut<PlaybackState>,
    range:     Res<TimeRange>,
) {
    let PlaybackState::Playing { current_ts, speed, last_frame_secs } = state.as_mut() else {
        return;
    };

    let now_secs = time.elapsed_secs_f64();
    if *last_frame_secs == 0.0 {
        // First frame after entering Playing — just record time, don't jump.
        *last_frame_secs = now_secs;
        return;
    }

    let delta_ms = ((now_secs - *last_frame_secs) * 1_000.0 * *speed as f64) as i64;
    *current_ts      += delta_ms;
    *last_frame_secs  = now_secs;

    if *current_ts >= range.end {
        let end = range.end;
        *state = PlaybackState::Paused { at_ts: end };
    }
}

// ── History fetch ─────────────────────────────────────────────────────────────

/// Listen for FetchHistoryRequest events and dispatch async fetch tasks.
/// Results are stored directly in HistoricalDataCache via shared Arc.
#[cfg(not(target_arch = "wasm32"))]
pub fn handle_fetch_history(
    mut events:    MessageReader<FetchHistoryRequest>,
    api_config:    Res<ApiConfig>,
    mut hist_cache: ResMut<HistoricalDataCache>,
) {
    for req in events.read() {
        // Skip if this range is already cached
        let pseudo_range = TimeRange { start: req.start_ts, end: req.end_ts };
        if hist_cache.is_fetched(req.device_id, &pseudo_range) {
            tracing::debug!(device = %req.device_id, "History already cached, skipping fetch");
            continue;
        }

        // Mark as fetched immediately to avoid duplicate in-flight requests
        hist_cache.mark_fetched(req.device_id, &pseudo_range);

        let config    = (*api_config).clone();
        let device_id = req.device_id;
        let keys      = req.keys.clone();
        let start_ts  = req.start_ts;
        let end_ts    = req.end_ts;
        let agg       = req.agg;

        // Use a channel to get results back on the Bevy thread.
        // We'll use a simple std::sync approach: Arc<Mutex<Option<result>>>.
        // The next fetch system invocation will see it.  A cleaner approach
        // (shared queue) would be identical to RpcResponseQueue.
        // For Phase 22 we spawn and let the next frame drain results via a
        // separate HistoryResultQueue resource (not implemented here for brevity).
        // Instead we use a simpler synchronous approach: background thread +
        // store results in a thread-local queue.
        let result_queue = HISTORY_QUEUE.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            let result = rt.block_on(async move {
                let client = ApiClient::new(config);
                let query = TimeseriesQuery {
                    entity_type: "DEVICE".into(),
                    entity_id:   device_id,
                    keys:        keys.clone(),
                    start_ts,
                    end_ts,
                    interval_ms: Some(60_000), // 1-minute buckets
                    limit:       Some(1_000),
                    agg,
                    order_asc:   true,
                };
                client.get_timeseries(&query).await.map(|data| (device_id, data))
            });

            if let Ok(mut q) = result_queue.lock() {
                q.push(result);
            }
        });
    }
}

/// WASM: history fetch via gloo-net HTTP GET (spawn_local + push to HISTORY_QUEUE).
#[cfg(target_arch = "wasm32")]
pub fn handle_fetch_history(
    mut events:     MessageReader<FetchHistoryRequest>,
    api_config:     Res<ApiConfig>,
    mut hist_cache: ResMut<HistoricalDataCache>,
) {
    for req in events.read() {
        let pseudo_range = TimeRange { start: req.start_ts, end: req.end_ts };
        if hist_cache.is_fetched(req.device_id, &pseudo_range) { continue; }
        hist_cache.mark_fetched(req.device_id, &pseudo_range);

        let base_url  = api_config.base_url.clone();
        let jwt_token = api_config.jwt_token.clone();
        let device_id = req.device_id;
        let keys_str  = req.keys.join(",");
        let start_ts  = req.start_ts;
        let end_ts    = req.end_ts;
        let agg       = req.agg.as_str();
        let result_queue = HISTORY_QUEUE.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let url = format!(
                "{base_url}/api/plugins/telemetry/DEVICE/{device_id}/values/timeseries\
                 ?keys={keys_str}&startTs={start_ts}&endTs={end_ts}&agg={agg}\
                 &orderBy=ASC&limit=1000&useStrictDataTypes=true"
            );

            let resp_result = gloo_net::http::Request::get(&url)
                .header("Authorization", &format!("Bearer {jwt_token}"))
                .send()
                .await
                .map_err(|e| format!("History HTTP error: {e:?}"));

            match resp_result {
                Ok(resp) if resp.status() == 200 => {
                    match resp.text().await {
                        Ok(text) => {
                            // ThingsBoard format: { "key": [[ts_ms, "value"], ...], ... }
                            if let Ok(raw) = serde_json::from_str::<
                                HashMap<String, Vec<[serde_json::Value; 2]>>
                            >(&text) {
                                let data: HashMap<String, Vec<DataPoint>> = raw.into_iter()
                                    .map(|(key, entries)| {
                                        let pts = entries.into_iter()
                                            .map(|e| {
                                                let ts = e[0].as_i64().unwrap_or(0);
                                                let value = e[1].as_f64()
                                                    .or_else(|| e[1].as_str()
                                                        .and_then(|s| s.parse::<f64>().ok()))
                                                    .unwrap_or(0.0);
                                                DataPoint { ts, value }
                                            })
                                            .collect();
                                        (key, pts)
                                    })
                                    .collect();
                                if let Ok(mut q) = result_queue.lock() {
                                    q.push(Ok((device_id, data)));
                                }
                            }
                        }
                        Err(e) => {
                            if let Ok(mut q) = result_queue.lock() {
                                q.push(Err(format!("History text error: {e:?}")));
                            }
                        }
                    }
                }
                Ok(resp) => {
                    if let Ok(mut q) = result_queue.lock() {
                        q.push(Err(format!("History HTTP {}", resp.status())));
                    }
                }
                Err(e) => {
                    if let Ok(mut q) = result_queue.lock() {
                        q.push(Err(e));
                    }
                }
            }
        });
    }
}

/// Drain the history result queue and insert data points into HistoricalDataCache.
pub fn drain_history_results(mut hist_cache: ResMut<HistoricalDataCache>) {
    let results: Vec<_> = {
        let Ok(mut q) = HISTORY_QUEUE.lock() else { return };
        q.drain(..).collect()
    };

    for result in results {
        match result {
            Ok((device_id, data)) => {
                let total: usize = data.values().map(|v| v.len()).sum();
                tracing::info!(device = %device_id, points = total, "Historical data cached");
                for (key, points) in &data {
                    hist_cache.insert(device_id, key, points);
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Historical data fetch failed");
            }
        }
    }
}

// ── Internal result queue ─────────────────────────────────────────────────────

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::api::DataPoint;

type HistoryResult = Result<(Uuid, HashMap<String, Vec<DataPoint>>), String>;

static HISTORY_QUEUE: std::sync::LazyLock<Arc<Mutex<Vec<HistoryResult>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

// ── Cache eviction ────────────────────────────────────────────────────────────

/// Resource: repeating 5-minute timer that triggers historical cache eviction.
#[derive(bevy::prelude::Resource)]
pub struct CacheEvictionTimer(pub bevy::prelude::Timer);

impl Default for CacheEvictionTimer {
    fn default() -> Self {
        Self(bevy::prelude::Timer::from_seconds(300.0, bevy::prelude::TimerMode::Repeating))
    }
}

/// Evict historical data points older than 24 hours every 5 minutes.
/// Prevents unbounded memory growth when the app runs for long periods.
pub fn evict_old_cache(
    time:       bevy::prelude::Res<bevy::prelude::Time>,
    mut timer:  bevy::prelude::ResMut<CacheEvictionTimer>,
    mut cache:  bevy::prelude::ResMut<crate::api::HistoricalDataCache>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() { return; }

    // Keep last 24 hours of data
    let threshold_ms = crate::components::current_time_ms() - 24 * 60 * 60 * 1_000;
    cache.evict_older_than(threshold_ms);
    tracing::debug!("Historical cache evicted (older than 24h)");
}
