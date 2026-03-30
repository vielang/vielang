//! Bottom timeline panel — scrubber, playback controls, time-range picker.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    api::{AggregationType, HistoricalDataCache},
    events::FetchHistoryRequest,
    playback::{fmt_ts, PlaybackState, TimeRange},
    ui::SelectedDevice,
};

// ── System ────────────────────────────────────────────────────────────────────

pub fn render_timeline(
    mut ctx:        EguiContexts,
    mut playback:   ResMut<PlaybackState>,
    mut range:      ResMut<TimeRange>,
    _cache:         Res<HistoricalDataCache>,
    selected:       Res<SelectedDevice>,
    mut fetch_ev:   MessageWriter<FetchHistoryRequest>,
    time:           Res<Time>,
) {
    let ctx = ctx.ctx_mut().expect("egui context");

    egui::TopBottomPanel::bottom("timeline_panel")
        .min_height(90.0)
        .show(ctx, |ui| {
            // ── Playback controls row ──────────────────────────────────────
            ui.horizontal(|ui| {
                // Go Live
                let live_label = if playback.is_live() {
                    egui::RichText::new("● Live").color(egui::Color32::GREEN)
                } else {
                    egui::RichText::new("◀◀ Live").color(egui::Color32::LIGHT_GRAY)
                };
                if ui.button(live_label).clicked() {
                    *playback = PlaybackState::Live;
                }

                ui.separator();

                // Jump to start of range
                if ui.button("⏮").on_hover_text("Jump to start").clicked() {
                    *playback = PlaybackState::Paused { at_ts: range.start };
                }

                // Step back 5 minutes
                if ui.button("◀ 5m").clicked() {
                    let at = playback.current_ts() - 5 * 60_000;
                    *playback = PlaybackState::Paused { at_ts: at.max(range.start) };
                }

                // Play / Pause toggle
                let is_playing = matches!(*playback, PlaybackState::Playing { .. });
                let play_label = if is_playing { "⏸ Pause" } else { "▶ Play" };
                if ui.button(play_label).clicked() {
                    if is_playing {
                        let at_ts = playback.current_ts();
                        *playback = PlaybackState::Paused { at_ts };
                    } else {
                        let current_ts = match &*playback {
                            PlaybackState::Paused { at_ts } => *at_ts,
                            _                               => range.start,
                        };
                        *playback = PlaybackState::Playing {
                            current_ts,
                            speed:           1.0,
                            last_frame_secs: time.elapsed_secs_f64(),
                        };
                    }
                }

                // Step forward 5 minutes
                if ui.button("▶ 5m").clicked() {
                    let at = playback.current_ts() + 5 * 60_000;
                    *playback = PlaybackState::Paused { at_ts: at.min(range.end) };
                }

                // Jump to end of range
                if ui.button("⏭").on_hover_text("Jump to end").clicked() {
                    *playback = PlaybackState::Paused { at_ts: range.end };
                }

                ui.separator();

                // Speed selector — only meaningful while Playing
                ui.label("Speed:");
                for &speed in &[0.5f32, 1.0, 2.0, 5.0, 10.0] {
                    let cur_speed = if let PlaybackState::Playing { speed: s, .. } = &*playback {
                        *s
                    } else {
                        1.0
                    };
                    let selected = (cur_speed - speed).abs() < 0.01;
                    if ui.selectable_label(selected, format!("{speed}×")).clicked() {
                        if let PlaybackState::Playing { speed: s, .. } = playback.as_mut() {
                            *s = speed;
                        }
                    }
                }

                ui.separator();

                // Time range preset buttons
                ui.label("Range:");
                if ui.small_button("1h").clicked()  { *range = TimeRange::last_hour(); }
                if ui.small_button("24h").clicked() { *range = TimeRange::last_24h(); }
                if ui.small_button("7d").clicked()  { *range = TimeRange::last_7d(); }
            });

            // ── Timeline scrubber ──────────────────────────────────────────
            let mut current_f = playback.current_ts() as f64;
            let start_f = range.start as f64;
            let end_f   = range.end   as f64;

            let slider = egui::Slider::new(&mut current_f, start_f..=end_f)
                .show_value(false)
                .trailing_fill(true);

            if ui.add(slider).changed() && !playback.is_live() {
                *playback = PlaybackState::Paused { at_ts: current_f as i64 };
            }

            // Timestamp labels + fetch button
            ui.horizontal(|ui| {
                ui.small(fmt_ts(range.start));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Fetch button
                    if ui.button("📥 Fetch Range").clicked() {
                        if let Some(device_id) = selected.device_id {
                            fetch_ev.write(FetchHistoryRequest {
                                device_id,
                                keys:     vec!["temperature".into(), "humidity".into(), "wind_speed".into()],
                                start_ts: range.start,
                                end_ts:   range.end,
                                agg:      AggregationType::Avg,
                            });
                        } else {
                            tracing::warn!("No device selected for history fetch");
                        }
                    }

                    ui.small(fmt_ts(range.end));

                    // Current position label
                    ui.separator();
                    let cur_label = match &*playback {
                        PlaybackState::Live => "Live".into(),
                        _ => fmt_ts(playback.current_ts()),
                    };
                    ui.small(format!("▲ {cur_label}"));
                });
            });
        });
}
