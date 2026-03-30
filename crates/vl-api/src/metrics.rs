/// Metric name constants — tránh typo và cho phép dùng trong middleware + transport.
/// Naming khớp ThingsBoard Java JMX metrics.

// ── HTTP API ──────────────────────────────────────────────────────────────────
pub const HTTP_REQUESTS_TOTAL:   &str = "vielang_http_requests_total";
pub const HTTP_REQUEST_DURATION: &str = "vielang_http_request_duration_seconds";

// ── MQTT Transport ────────────────────────────────────────────────────────────
pub const MQTT_CONNECTIONS_TOTAL:  &str = "vielang_mqtt_connections_total";
pub const MQTT_ACTIVE_CONNECTIONS: &str = "vielang_mqtt_active_connections";
pub const MQTT_MESSAGES_RECEIVED:  &str = "vielang_mqtt_messages_received_total";

// ── Database pool ─────────────────────────────────────────────────────────────
pub const DB_POOL_SIZE: &str = "vielang_db_pool_size";
pub const DB_POOL_IDLE: &str = "vielang_db_pool_idle";

// ── Rule Engine ───────────────────────────────────────────────────────────────
pub const RE_MESSAGES_PROCESSED: &str = "vielang_rule_engine_messages_processed_total";
pub const RE_PROCESSING_TIME:    &str = "vielang_rule_engine_processing_time_seconds";
pub const RE_ERRORS_TOTAL:       &str = "vielang_rule_engine_errors_total";

// ── Queue ─────────────────────────────────────────────────────────────────────
pub const QUEUE_PUBLISH_TOTAL:  &str = "vielang_queue_publish_total";
pub const QUEUE_CONSUME_TOTAL:  &str = "vielang_queue_consume_total";
pub const QUEUE_LAG:            &str = "vielang_queue_lag";

// ── Alarms ────────────────────────────────────────────────────────────────────
pub const ALARMS_CREATED_TOTAL: &str = "vielang_alarms_created_total";
pub const ALARMS_CLEARED_TOTAL: &str = "vielang_alarms_cleared_total";

/// Đăng ký mô tả cho tất cả metrics — Prometheus HELP lines.
/// Gọi một lần sau khi install_recorder() trong main.rs.
pub fn init_metrics() {
    metrics::describe_counter!(
        HTTP_REQUESTS_TOTAL,
        "Total HTTP requests processed, labeled by method, endpoint, and status"
    );
    metrics::describe_histogram!(
        HTTP_REQUEST_DURATION,
        "HTTP request duration in seconds"
    );
    metrics::describe_counter!(
        MQTT_CONNECTIONS_TOTAL,
        "Total MQTT device connections accepted"
    );
    metrics::describe_gauge!(
        MQTT_ACTIVE_CONNECTIONS,
        "Currently active MQTT device connections"
    );
    metrics::describe_counter!(
        MQTT_MESSAGES_RECEIVED,
        "Total MQTT messages received, labeled by message type"
    );
    metrics::describe_gauge!(DB_POOL_SIZE, "Total database connection pool size");
    metrics::describe_gauge!(DB_POOL_IDLE, "Idle database connections in pool");
    metrics::describe_counter!(
        RE_MESSAGES_PROCESSED,
        "Total messages processed by the rule engine"
    );
    metrics::describe_histogram!(
        RE_PROCESSING_TIME,
        "Rule engine message processing time in seconds"
    );
    metrics::describe_counter!(
        RE_ERRORS_TOTAL,
        "Total rule engine processing errors, labeled by node type"
    );
    metrics::describe_counter!(
        QUEUE_PUBLISH_TOTAL,
        "Total messages published to queue, labeled by topic and backend"
    );
    metrics::describe_counter!(
        QUEUE_CONSUME_TOTAL,
        "Total messages consumed from queue, labeled by topic and group"
    );
    metrics::describe_gauge!(
        QUEUE_LAG,
        "Consumer lag (pending messages) per topic and group"
    );
    metrics::describe_counter!(
        ALARMS_CREATED_TOTAL,
        "Total alarms created"
    );
    metrics::describe_counter!(
        ALARMS_CLEARED_TOTAL,
        "Total alarms cleared"
    );
}
