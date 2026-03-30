pub mod editor;
pub mod layout;
pub mod renderer;
pub mod widget;

pub use editor::{render_dashboard_editor, toggle_dashboard_editor, DashboardEditorState};
pub use layout::{ActiveDashboard, DashboardLayout, GRID_COLS, GRID_ROWS};
pub use renderer::{grid_cell_to_px, render_dashboard};
pub use widget::{GridRect, Widget, WidgetId, WidgetKind};
