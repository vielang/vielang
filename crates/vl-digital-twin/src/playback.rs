//! Playback state machine — Live / Paused / Playing modes for time-travel.

use bevy::prelude::*;

use crate::components::current_time_ms;

// ── PlaybackState ─────────────────────────────────────────────────────────────

/// Controls whether the twin shows live data or replays historical data.
#[derive(Resource, Debug, Clone, PartialEq)]
pub enum PlaybackState {
    /// Streaming real-time data via WebSocket.
    Live,
    /// Frozen at a specific historical timestamp.
    Paused {
        /// Timestamp being displayed (Unix ms).
        at_ts: i64,
    },
    /// Advancing through history automatically.
    Playing {
        /// Current position in the timeline (Unix ms).
        current_ts:      i64,
        /// Playback multiplier: 1.0 = real-time, 2.0 = 2× speed.
        speed:           f32,
        /// Bevy elapsed seconds at the last frame — used for delta calculation.
        last_frame_secs: f64,
    },
}

impl Default for PlaybackState {
    fn default() -> Self { Self::Live }
}

impl PlaybackState {
    /// Return the "current" timestamp regardless of mode.
    /// Live mode returns `now`; Paused/Playing return the scrubber position.
    pub fn current_ts(&self) -> i64 {
        match self {
            Self::Live                           => current_time_ms(),
            Self::Paused { at_ts }               => *at_ts,
            Self::Playing { current_ts, .. }     => *current_ts,
        }
    }

    pub fn is_live(&self) -> bool { matches!(self, Self::Live) }
}

// ── TimeRange ─────────────────────────────────────────────────────────────────

/// The time window used for historical queries and the timeline scrubber.
#[derive(Resource, Debug, Clone)]
pub struct TimeRange {
    pub start: i64,
    pub end:   i64,
}

impl Default for TimeRange {
    fn default() -> Self { Self::last_hour() }
}

impl TimeRange {
    pub fn last_hour() -> Self {
        let now = current_time_ms();
        Self { start: now - 3_600_000, end: now }
    }

    pub fn last_24h() -> Self {
        let now = current_time_ms();
        Self { start: now - 86_400_000, end: now }
    }

    pub fn last_7d() -> Self {
        let now = current_time_ms();
        Self { start: now - 604_800_000, end: now }
    }

    pub fn duration_hours(&self) -> f64 {
        (self.end - self.start) as f64 / 3_600_000.0
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Format a Unix-ms timestamp as "HH:MM:SS" using chrono.
pub fn fmt_ts(ts_ms: i64) -> String {
    use chrono::{DateTime, Utc};
    DateTime::from_timestamp_millis(ts_ms)
        .map(|dt: DateTime<Utc>| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "--:--:--".into())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_state_defaults_to_live() {
        assert!(PlaybackState::default().is_live());
    }

    #[test]
    fn time_range_last_hour_is_one_hour() {
        let r = TimeRange::last_hour();
        assert!((r.duration_hours() - 1.0).abs() < 0.001);
    }

    #[test]
    fn playback_current_ts_paused() {
        let state = PlaybackState::Paused { at_ts: 12345 };
        assert_eq!(state.current_ts(), 12345);
    }
}
