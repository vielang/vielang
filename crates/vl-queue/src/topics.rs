/// Topic constants — VíeLang message bus topics

/// Core service topic — entity events (CREATED, UPDATED, DELETED)
pub const VL_CORE: &str = "vl.core";

/// Rule engine processing topic — TbMsg for rule chain evaluation
pub const VL_RULE_ENGINE: &str = "vl.rule-engine";

/// Transport API requests — device → server (telemetry, attributes, RPC response)
pub const VL_TRANSPORT_API_REQUESTS: &str = "vl.transport.api.requests";

/// Transport API responses — server → device (RPC request, attribute subscription)
pub const VL_TRANSPORT_API_RESPONSES: &str = "vl.transport.api.responses";

/// Notifications — alarm/subscription pushes from core to transport nodes
pub const VL_NOTIFICATIONS: &str = "vl.notifications";

/// Version update events
pub const VL_VERSION_CONTROL: &str = "vl.vc.queue";
