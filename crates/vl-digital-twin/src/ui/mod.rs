pub mod alarm_panel;
pub mod alarm_rules;
pub mod asset_tree;
pub mod background_settings;
pub mod context_menu;
pub mod device_list;
pub mod device_panel;
pub mod file_picker;
pub mod layout_manager;
pub mod login;
pub mod map_view;
pub mod notifications;
pub mod rpc_log;
pub mod timeline;

pub use alarm_panel::{render_alarm_panel, AlarmPanelState};
pub use alarm_rules::{render_alarm_rules, toggle_alarm_rules, AlarmRulesState};
pub use asset_tree::render_asset_tree;
pub use background_settings::render_background_settings;
pub use file_picker::{emit_load_from_picker, open_file_picker_native, render_file_picker_input, FilePicker};
pub use context_menu::{handle_entity_right_click, render_context_menu, ContextMenuState};
pub use device_list::{render_device_list, DeviceListFilter};
pub use device_panel::{render_device_panel, DevicePanelState, SelectedDevice, TelemetryHistory};
pub use layout_manager::{render_layout_manager, LayoutManager};
pub use login::{handle_login_task, render_login_screen, LoginFormState};
pub use map_view::{render_map_panel, MapViewState};
pub use notifications::{push_alarm_notifications, render_notifications, NotificationQueue};
pub use rpc_log::{collect_rpc_results, render_rpc_log, RpcLogState};
pub use timeline::render_timeline;

// ── Layout mode ───────────────────────────────────────────────────────────────

/// Controls which egui panel arrangement is active.
#[derive(bevy::prelude::Resource, Default, Debug, Clone, PartialEq)]
pub enum LayoutMode {
    /// Default: device list on the left, device panel on the right (egui side panels).
    #[default]
    SideBySide,
    /// Device list + charts in a bottom panel; 3D view gets more vertical space.
    BottomDashboard,
    /// All egui panels hidden — only the 3D scene + minimal HUD visible.
    FullscreenScene,
}
