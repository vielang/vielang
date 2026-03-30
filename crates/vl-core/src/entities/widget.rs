use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Khớp với bảng `widgets_bundle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetsBundle {
    pub id: Uuid,
    pub created_time: i64,
    /// NULL = system bundle
    pub tenant_id: Option<Uuid>,
    pub alias: String,
    pub title: String,
    pub image: Option<String>,
    pub scada: bool,
    pub description: Option<String>,
    pub order_index: Option<i32>,
    pub external_id: Option<Uuid>,
    pub version: i64,
}

/// Khớp với bảng `widget_type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetType {
    pub id: Uuid,
    pub created_time: i64,
    /// NULL = system widget
    pub tenant_id: Option<Uuid>,
    pub fqn: String,
    pub name: String,
    pub descriptor: serde_json::Value,
    pub deprecated: bool,
    pub scada: bool,
    pub image: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub external_id: Option<Uuid>,
    pub version: i64,
}

/// Lightweight info — BaseWidgetType + image/description/tags + widgetType discriminator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetTypeInfo {
    #[serde(flatten)]
    pub widget_type: WidgetType,
    /// "LATEST_VALUE" | "TIME_SERIES" | "ALARM" | "STATIC" | "CONTROL" | "EDGE"
    pub widget_type_discriminator: Option<String>,
}
