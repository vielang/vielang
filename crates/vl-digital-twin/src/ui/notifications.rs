//! In-app toast notifications + desktop notifications for critical alarms.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::components::AlarmSeverity;
use crate::events::AlarmUpdate;

// ── Data ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Notification {
    pub id:           u32,
    pub severity:     AlarmSeverity,
    pub device:       String,
    pub message:      String,
    pub created_at:   std::time::Instant,
    /// Auto-dismiss after N seconds; `None` = manual dismiss required.
    pub auto_dismiss: Option<f32>,
}

// ── Resource ──────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct NotificationQueue {
    pub notifications: Vec<Notification>,
    next_id:           u32,
}

impl NotificationQueue {
    /// Add a notification to the queue (capped at 10).
    pub fn push(&mut self, severity: AlarmSeverity, device: String, message: String) {
        self.next_id += 1;
        let auto_dismiss = match severity {
            AlarmSeverity::Critical => None,         // Must dismiss manually
            AlarmSeverity::Major    => Some(30.0),
            _                       => Some(10.0),
        };
        self.notifications.push(Notification {
            id:           self.next_id,
            severity,
            device,
            message,
            created_at:   std::time::Instant::now(),
            auto_dismiss,
        });
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    /// Remove a notification by ID.
    pub fn dismiss(&mut self, id: u32) {
        self.notifications.retain(|n| n.id != id);
    }

    /// Expire auto-dismiss notifications that have timed out.
    pub fn tick(&mut self, _delta_secs: f32) {
        self.notifications.retain(|n| {
            match n.auto_dismiss {
                Some(timeout) => n.created_at.elapsed().as_secs_f32() < timeout,
                None          => true,
            }
        });
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Read incoming AlarmUpdate events and push a notification for active alarms.
/// Looks up device name from the ECS query.
pub fn push_alarm_notifications(
    mut events:   MessageReader<AlarmUpdate>,
    mut queue:    ResMut<NotificationQueue>,
    device_query: Query<&crate::components::DeviceEntity>,
) {
    for ev in events.read() {
        if !ev.active { continue; }

        let severity = AlarmSeverity::from_str(&ev.severity);
        let message  = ev.alarm_type.clone();

        // Look up device name by id
        let device_name = device_query
            .iter()
            .find(|d| d.device_id == ev.device_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| ev.device_id.to_string());

        // Desktop notification for Critical (native only)
        #[cfg(not(target_arch = "wasm32"))]
        if severity == AlarmSeverity::Critical {
            send_desktop_notification(&device_name, &message);
        }

        queue.push(severity, device_name, message);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn send_desktop_notification(device: &str, message: &str) {
    let mut notif = notify_rust::Notification::new();
    notif.summary(&format!("CRITICAL — {device}"));
    notif.body(message);
    // urgency / timeout are Linux-only; skip on other platforms
    #[cfg(target_os = "linux")]
    {
        notif.urgency(notify_rust::Urgency::Critical);
        notif.timeout(notify_rust::Timeout::Never);
    }
    let _ = notif.show();
}

/// Render toast notifications in the top-right corner of the screen.
pub fn render_notifications(
    mut contexts: EguiContexts,
    time:         Res<Time>,
    mut queue:    ResMut<NotificationQueue>,
) {
    queue.tick(time.delta_secs());
    if queue.notifications.is_empty() { return; }

    let ctx = match contexts.ctx_mut() {
        Ok(c)  => c,
        Err(_) => return,
    };

    let screen_rect  = ctx.viewport_rect();
    let mut y_offset = 10.0f32;

    let notifications: Vec<Notification> = queue.notifications.clone();
    let mut to_dismiss: Vec<u32>         = Vec::new();

    for notif in &notifications {
        let color = notif.severity.to_egui_color();
        let pos   = egui::pos2(screen_rect.right() - 310.0, y_offset);

        egui::Window::new(format!("notif_{}", notif.id))
            .title_bar(false)
            .fixed_pos(pos)
            .fixed_size([300.0, 80.0])
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 30, 230)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(color, format!("● {}", notif.severity));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if ui.small_button("✕").clicked() {
                            to_dismiss.push(notif.id);
                        }
                    });
                });
                ui.label(&notif.device);
                ui.small(&notif.message);

                // Progress bar for auto-dismiss
                if let Some(timeout) = notif.auto_dismiss {
                    let elapsed = notif.created_at.elapsed().as_secs_f32();
                    let frac    = 1.0 - (elapsed / timeout).min(1.0);
                    ui.add(
                        egui::ProgressBar::new(frac)
                            .desired_height(3.0)
                            .fill(color),
                    );
                }
            });

        y_offset += 90.0;
    }

    for id in to_dismiss {
        queue.dismiss(id);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_queue_push_caps_at_10() {
        let mut q = NotificationQueue::default();
        for i in 0..12 {
            q.push(AlarmSeverity::Warning, format!("Dev{i}"), "msg".into());
        }
        assert_eq!(q.notifications.len(), 10);
    }

    #[test]
    fn notification_queue_dismiss() {
        let mut q = NotificationQueue::default();
        q.push(AlarmSeverity::Minor, "DevA".into(), "test".into());
        let id = q.notifications[0].id;
        q.dismiss(id);
        assert!(q.notifications.is_empty());
    }

    #[test]
    fn critical_alarm_no_auto_dismiss() {
        let mut q = NotificationQueue::default();
        q.push(AlarmSeverity::Critical, "DevX".into(), "critical!".into());
        assert!(q.notifications[0].auto_dismiss.is_none());
    }

    #[test]
    fn warning_alarm_has_10s_dismiss() {
        let mut q = NotificationQueue::default();
        q.push(AlarmSeverity::Warning, "DevY".into(), "warn".into());
        assert_eq!(q.notifications[0].auto_dismiss, Some(10.0));
    }
}
