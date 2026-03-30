use serde::{Deserialize, Serialize};

// ── Firebase FCM config ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FirebaseConfig {
    pub enabled:    bool,
    /// Firebase Legacy Server Key (from Firebase Console → Project Settings → Cloud Messaging)
    pub server_key: String,
}

impl Default for FirebaseConfig {
    fn default() -> Self {
        Self {
            enabled:    false,
            server_key: String::new(),
        }
    }
}

// ── Auth config (P4) ──────────────────────────────────────────────────────────
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// TTL for email activation tokens (hours)
    pub activation_token_ttl_hours: u64,
    /// How often to clean up expired activation tokens (seconds)
    pub session_cleanup_interval_secs: u64,
    /// LDAP periodic sync
    #[serde(default)]
    pub ldap: LdapSyncConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            activation_token_ttl_hours: 24,
            session_cleanup_interval_secs: 3600,
            ldap: LdapSyncConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LdapSyncConfig {
    pub sync_enabled: bool,
    pub sync_interval_secs: u64,
    pub deactivate_on_remove: bool,
}

impl Default for LdapSyncConfig {
    fn default() -> Self {
        Self {
            sync_enabled: false,
            sync_interval_secs: 300,
            deactivate_on_remove: false,
        }
    }
}

/// Root config — khớp với cấu trúc thingsboard.yml
/// Load theo thứ tự: config/vielang.toml → env vars (VL_ prefix)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VieLangConfig {
    pub server:       ServerConfig,
    pub database:     DatabaseConfig,
    pub cache:        CacheConfig,
    pub queue:        QueueConfig,
    pub security:     SecurityConfig,
    pub transport:    TransportConfig,
    #[serde(default)]
    pub cluster:      ClusterConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub housekeeper:  HousekeeperConfig,
    #[serde(default)]
    pub edge_grpc:    EdgeGrpcConfig,
    #[serde(default)]
    pub trendz:       TrendzConfig,
    #[serde(default)]
    pub firebase:     FirebaseConfig,
    #[serde(default)]
    pub stripe:       StripeConfig,
    #[serde(default)]
    pub ota:          OtaConfig,
    #[serde(default)]
    pub search:       SearchConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub geofence: GeofenceConfig,
    #[serde(default)]
    pub scheduler: SchedulerConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub simulator: SimulatorCfg,
}

// ── IoT Simulator config ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimulatorCfg {
    pub enabled: bool,
    pub max_simulations_per_tenant: usize,
    pub min_interval_ms: i64,
    #[serde(default = "default_arduino_cli_path")]
    pub arduino_cli_path: String,
    #[serde(default = "default_arduino_timeout")]
    pub arduino_compile_timeout_secs: u64,
    #[serde(default = "default_arduino_sketch_dir")]
    pub arduino_sketch_dir: String,
}

fn default_arduino_cli_path() -> String { "arduino-cli".into() }
fn default_arduino_timeout() -> u64 { 120 }
fn default_arduino_sketch_dir() -> String { "/tmp/vielang-arduino".into() }

impl Default for SimulatorCfg {
    fn default() -> Self {
        Self {
            enabled: true,
            max_simulations_per_tenant: 50,
            min_interval_ms: 1000,
            arduino_cli_path: default_arduino_cli_path(),
            arduino_compile_timeout_secs: default_arduino_timeout(),
            arduino_sketch_dir: default_arduino_sketch_dir(),
        }
    }
}

// ── OTA Firmware Distribution config ─────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OtaConfig {
    pub enabled:        bool,
    /// Chunk size in KB per MQTT firmware request
    pub chunk_size_kb:  usize,
    /// Maximum retry attempts for QUEUED states
    pub max_retries:    u32,
    /// Retry delay in seconds (re-notify device after this interval)
    pub retry_delay_s:  u64,
    /// Allow firmware download over HTTP (device token auth)
    pub http_download:  bool,
}

impl Default for OtaConfig {
    fn default() -> Self {
        Self {
            enabled:       true,
            chunk_size_kb: 16,
            max_retries:   3,
            retry_delay_s: 1800,
            http_download: true,
        }
    }
}

// ── Full-Text Search config ───────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchConfig {
    pub enabled:             bool,
    /// PostgreSQL FTS language dictionary (e.g. "english", "simple")
    pub language:            String,
    /// Append :* to each term for prefix matching
    pub prefix_matching:     bool,
    /// Minimum query length — queries shorter than this return empty results
    pub min_query_length:    usize,
    /// Max results fetched per entity type before merging (caps internal fan-out)
    pub max_results_per_type: i64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enabled:              true,
            language:             "english".into(),
            prefix_matching:      true,
            min_query_length:     2,
            max_results_per_type: 100,
        }
    }
}

// ── Observability config (Phase P7) ──────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObservabilityConfig {
    /// Expose Prometheus /metrics endpoint.
    pub metrics_enabled: bool,

    /// Enable OpenTelemetry distributed tracing.
    pub tracing_enabled: bool,

    /// OTLP exporter endpoint (HTTP). Empty = disable OTLP.
    /// e.g. "http://localhost:4318"  (HTTP/proto, default Jaeger/Collector port)
    pub otlp_endpoint: String,

    /// Log output format: "json" (production) | "pretty" (development)
    pub log_format: String,

    /// Minimum log level: "error" | "warn" | "info" | "debug" | "trace"
    pub log_level: String,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            tracing_enabled: false,
            otlp_endpoint:   String::new(),
            log_format:      "json".into(),
            log_level:       "info".into(),
        }
    }
}

// ── Geofencing config (Phase P8) ─────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeofenceConfig {
    /// Enable PostGIS-backed geofencing (requires postgis extension).
    pub enabled: bool,
}

impl Default for GeofenceConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

// ── Job Scheduler config (Phase P10) ─────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SchedulerConfig {
    /// Run the background job scheduler loop.
    pub enabled: bool,
    /// How often to check for due jobs (seconds).
    pub check_interval_s: u64,
    /// Max number of jobs executing concurrently.
    pub max_concurrent_jobs: usize,
    /// How many days of execution history to keep.
    pub execution_history_days: u32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled:                true,
            check_interval_s:       60,
            max_concurrent_jobs:    10,
            execution_history_days: 30,
        }
    }
}

// ── Trendz Analytics config ───────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TrendzConfig {
    #[serde(default)]
    pub enabled:    bool,
    #[serde(default)]
    pub base_url:   String,
    #[serde(default)]
    pub secret_key: String,
}

// ── Notification config ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationConfig {
    #[serde(default)]
    pub smtp: SmtpConfig,
    #[serde(default)]
    pub sms: SmsConfig,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            smtp: SmtpConfig::default(),
            sms:  SmsConfig::default(),
        }
    }
}

/// SMS delivery config.
/// Set provider = "disabled" (default) to suppress sends without error.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmsConfig {
    /// "twilio" | "disabled"
    pub provider:            String,
    pub twilio_account_sid:  String,
    pub twilio_auth_token:   String,
    pub twilio_from_number:  String,
}

impl Default for SmsConfig {
    fn default() -> Self {
        Self {
            provider:           "disabled".into(),
            twilio_account_sid: String::new(),
            twilio_auth_token:  String::new(),
            twilio_from_number: String::new(),
        }
    }
}

/// SMTP config cho email delivery.
/// Nếu host để trống (""), EmailChannel sẽ chỉ log thay vì gửi thật.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpConfig {
    pub host:     String,
    pub port:     u16,
    pub username: String,
    pub password: String,
    pub from:     String,
    pub tls:      bool,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host:     String::new(),  // empty = disabled
            port:     587,
            username: String::new(),
            password: String::new(),
            from:     "noreply@vielang.local".into(),
            tls:      true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    /// Allowed CORS origins. Empty = deny all cross-origin requests (production default).
    /// Use ["*"] only for local development.
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Max messages per actor batch before yielding (default: 30).
    pub actor_throughput: Option<usize>,
    /// Max actor init retry attempts, 0 = unlimited (default: 10).
    pub max_actor_init_attempts: Option<u32>,
}

impl ServerConfig {
    /// Base URL for this server instance, e.g. "http://0.0.0.0:8080".
    /// Used to construct redirect URLs for Stripe Checkout and Billing Portal.
    /// In production, override via VL__SERVER__HOST / VL__SERVER__PORT env vars
    /// or configure stripe.success_url / stripe.cancel_url directly.
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 8080,
            workers: None,
            allowed_origins: vec![],
            actor_throughput: None,
            max_actor_init_attempts: None,
        }
    }
}

// ── Stripe Billing config ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct StripeConfig {
    /// Set true to enable Stripe Checkout + webhook handling.
    #[serde(default)]
    pub enabled: bool,
    /// Stripe secret key (sk_live_... or sk_test_...).
    /// Override in production via VL__STRIPE__SECRET_KEY env var.
    #[serde(default)]
    pub secret_key: String,
    /// Stripe webhook signing secret (whsec_...).
    /// Override in production via VL__STRIPE__WEBHOOK_SECRET env var.
    #[serde(default)]
    pub webhook_secret: String,
    /// URL the browser is redirected to after a successful Stripe Checkout.
    #[serde(default = "default_success_url")]
    pub success_url: String,
    /// URL the browser is redirected to if the user cancels Stripe Checkout.
    #[serde(default = "default_cancel_url")]
    pub cancel_url: String,
}

fn default_success_url() -> String { "http://localhost:4200/billing/success".to_string() }
fn default_cancel_url()  -> String { "http://localhost:4200/billing/cancel".to_string() }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub postgres: PostgresConfig,
    #[serde(default)]
    pub timeseries: TimeseriesBackendConfig,
}

/// Chọn backend cho timeseries storage (lịch sử + latest).
/// Khớp Java: database.ts.type và database.ts_latest.type
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeseriesBackendConfig {
    /// Backend cho timeseries lịch sử: "sql" (PostgreSQL) hoặc "cassandra"
    #[serde(default)]
    pub ts_type: TsBackendType,
    /// Backend cho latest values (có thể độc lập với ts_type)
    #[serde(default)]
    pub ts_latest_type: TsBackendType,
    /// Cassandra config — bắt buộc khi ts_type = "cassandra"
    pub cassandra: Option<CassandraConfig>,
}

impl Default for TimeseriesBackendConfig {
    fn default() -> Self {
        Self {
            ts_type: TsBackendType::Sql,
            ts_latest_type: TsBackendType::Sql,
            cassandra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TsBackendType {
    /// PostgreSQL (mặc định) — dùng ts_kv + ts_kv_latest tables
    Sql,
    /// Apache Cassandra / ScyllaDB — dùng ts_kv_cf + ts_kv_latest_cf tables
    Cassandra,
}

impl Default for TsBackendType {
    fn default() -> Self { TsBackendType::Sql }
}

/// Cấu hình kết nối Cassandra / ScyllaDB.
/// Khớp Java: cassandra.* trong thingsboard.yml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CassandraConfig {
    /// Địa chỉ node Cassandra, ví dụ: "127.0.0.1:9042"
    pub url: String,
    /// Keyspace name — mặc định "thingsboard"
    #[serde(default = "default_keyspace")]
    pub keyspace: String,
    /// Local datacenter — cần thiết cho policy LoadBalancing
    #[serde(default = "default_datacenter")]
    pub local_datacenter: String,
    /// Granularity phân vùng thời gian: "MINUTES" | "HOURS" | "DAYS" | "MONTHS" | "YEARS"
    #[serde(default = "default_partition_granularity")]
    pub partition_granularity: String,
    /// TTL tính bằng giây. -1 = không TTL (giữ vĩnh viễn)
    #[serde(default = "default_ttl")]
    pub ttl_seconds: i64,
    /// Cache size cho partition tracking (mặc định 100_000)
    #[serde(default = "default_partition_cache_size")]
    pub partition_cache_size: usize,
    /// Cassandra username (None = no auth)
    pub username: Option<String>,
    /// Cassandra password (None = no auth)
    pub password: Option<String>,
}

impl Default for CassandraConfig {
    fn default() -> Self {
        Self {
            url: "127.0.0.1:9042".into(),
            keyspace: default_keyspace(),
            local_datacenter: default_datacenter(),
            partition_granularity: default_partition_granularity(),
            ttl_seconds: default_ttl(),
            partition_cache_size: default_partition_cache_size(),
            username: None,
            password: None,
        }
    }
}

fn default_keyspace() -> String { "thingsboard".into() }
fn default_datacenter() -> String { "datacenter1".into() }
fn default_partition_granularity() -> String { "MONTHS".into() }
fn default_ttl() -> i64 { -1 }
fn default_partition_cache_size() -> usize { 100_000 }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    /// Max seconds a connection may sit idle before being closed. None = no limit.
    pub idle_timeout_secs: Option<u64>,
    /// Max seconds a connection may live (regardless of activity). None = no limit.
    pub max_lifetime_secs: Option<u64>,
    /// SSL mode: "disable" | "prefer" | "require" | "verify-ca" | "verify-full"
    /// Production nên dùng "require" hoặc "verify-full"
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    /// Tên hiển thị trong pg_stat_activity — giúp DBA phân biệt kết nối
    #[serde(default = "default_application_name")]
    pub application_name: String,
    /// Hủy query sau N ms nếu chưa xong. None = không giới hạn.
    /// Production nên set ~30_000 (30s) để tránh slow query treo pool.
    #[serde(default)]
    pub statement_timeout_ms: Option<u64>,
}

fn default_ssl_mode() -> String { "prefer".into() }
fn default_application_name() -> String { "vielang".into() }

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: "postgres://vielang:vielang@localhost:5432/vielang".into(),
            max_connections: 20,
            min_connections: 2,
            connect_timeout_secs: 10,
            idle_timeout_secs: Some(600),
            max_lifetime_secs: Some(1800),
            ssl_mode: default_ssl_mode(),
            application_name: default_application_name(),
            statement_timeout_ms: None,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            postgres: PostgresConfig::default(),
            timeseries: TimeseriesBackendConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    InMemory,
    Redis,
}

impl Default for CacheType {
    fn default() -> Self {
        CacheType::InMemory
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalCacheConfig {
    pub max_size: u64,
    pub ttl_seconds: u64,
}

impl Default for LocalCacheConfig {
    fn default() -> Self {
        Self { max_size: 100_000, ttl_seconds: 900 }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisCacheConfig {
    pub url: String,
    pub ttl_seconds: u64,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self { url: "redis://localhost:6379".into(), ttl_seconds: 900 }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub cache_type: CacheType,
    #[serde(default)]
    pub local: LocalCacheConfig,
    #[serde(default)]
    pub redis: RedisCacheConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: CacheType::InMemory,
            local: LocalCacheConfig::default(),
            redis: RedisCacheConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueueConfig {
    pub queue_type:    QueueType,
    pub kafka:         Option<KafkaConfig>,
    #[serde(default)]
    pub rabbitmq:      Option<RabbitMqConfig>,
    #[serde(default)]
    pub redis_streams: Option<RedisStreamsConfig>,
    #[serde(default)]
    pub sqs:           Option<SqsConfig>,
    #[serde(default)]
    pub persistent:    Option<PersistentQueueConfig>,
    /// Số parallel consumer tasks cho Rule Engine topic. Default = 1.
    #[serde(default = "default_consumer_threads")]
    pub consumer_threads: usize,
    /// Timeout (giây) cho mỗi message khi Rule Engine xử lý.
    /// Quá thời gian → message bị route sang DLQ. Default = 30.
    #[serde(default = "default_processing_timeout_secs")]
    pub processing_timeout_secs: u64,
    /// Tên topic cho Dead-Letter Queue.
    #[serde(default = "default_dlq_topic")]
    pub dlq_topic: String,
    /// Số giờ giữ lại acked messages trước khi cleanup. Default = 72.
    #[serde(default = "default_message_retention_hours")]
    pub message_retention_hours: u64,
}

fn default_consumer_threads() -> usize { 1 }
fn default_processing_timeout_secs() -> u64 { 30 }
fn default_dlq_topic() -> String { "vl.rule-engine.dlq".into() }
fn default_message_retention_hours() -> u64 { 72 }

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueType {
    InMemory,
    Kafka,
    RabbitMq,
    RedisStreams,
    Sqs,
    /// PostgreSQL-backed persistent queue — survives server restarts.
    Persistent,
}

// ── Persistent queue ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PersistentQueueConfig {
    /// How often (in hours) to run cleanup of acked messages.
    pub cleanup_interval_h: u64,
    /// How long (in hours) to retain acked messages before deleting them.
    pub message_ttl_h: u64,
    /// Max messages returned per poll call.
    pub batch_size: i32,
    /// How long (in ms) a consumer waits between polls when the topic is empty.
    pub poll_interval_ms: u64,
}

impl Default for PersistentQueueConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_h: 1,
            message_ttl_h:      24,
            batch_size:         500,
            poll_interval_ms:   100,
        }
    }
}

// ── Kafka ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub acks: String,
    pub retries: u32,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".into(),
            acks: "all".into(),
            retries: 3,
        }
    }
}

// ── RabbitMQ ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RabbitMqConfig {
    /// AMQP connection URL: amqp://user:pass@host:5672/vhost
    pub url:            String,
    /// Exchange name — "" = default exchange (direct routing by queue name)
    pub exchange:       String,
    /// QoS prefetch count per consumer channel
    pub prefetch_count: u16,
    /// Enable dead-letter exchange for rejected/expired messages
    pub dlx_enabled:    bool,
    /// Name of the dead-letter exchange
    pub dlx_exchange:   String,
}

impl Default for RabbitMqConfig {
    fn default() -> Self {
        Self {
            url:            "amqp://guest:guest@localhost:5672/%2F".into(),
            exchange:       "thingsboard".into(),
            prefetch_count: 100,
            dlx_enabled:    true,
            dlx_exchange:   "thingsboard.dlx".into(),
        }
    }
}

// ── Redis Streams ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisStreamsConfig {
    /// Redis connection URL: redis://[:password@]host[:port][/db]
    pub url:            String,
    /// Consumer group name (shared across all VieLang nodes)
    pub group:          String,
    /// Unique consumer name within the group (auto-appended with node ID)
    pub consumer_name:  String,
    /// Max messages per XREADGROUP call
    pub batch_size:     usize,
    /// MAXLEN per stream — approximate cap to limit memory usage
    pub max_len:        usize,
    /// Seconds before a pending (unacknowledged) message is reclaimed via XAUTOCLAIM
    pub pending_ttl_s:  u64,
}

impl Default for RedisStreamsConfig {
    fn default() -> Self {
        Self {
            url:           "redis://localhost:6379".into(),
            group:         "thingsboard".into(),
            consumer_name: "vielang".into(),
            batch_size:    500,
            max_len:       1_000_000,
            pending_ttl_s: 30,
        }
    }
}

// ── AWS SQS ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SqsConfig {
    /// AWS region (e.g. "us-east-1")
    pub region:        String,
    /// AWS access key ID (prefer env var AWS_ACCESS_KEY_ID in production)
    pub access_key:    String,
    /// AWS secret access key (prefer env var AWS_SECRET_ACCESS_KEY in production)
    pub secret_key:    String,
    /// Prefix prepended to all queue names (e.g. "thingsboard-")
    pub queue_prefix:  String,
    /// Use FIFO queues (.fifo suffix) — enables exactly-once delivery
    pub fifo:          bool,
    /// Max messages per ReceiveMessage call (1–10)
    pub max_messages:  i32,
    /// Long-polling wait time in seconds (0 = short poll, 1–20 = long poll)
    pub wait_seconds:  i32,
}

impl Default for SqsConfig {
    fn default() -> Self {
        Self {
            region:       "us-east-1".into(),
            access_key:   String::new(),
            secret_key:   String::new(),
            queue_prefix: "thingsboard-".into(),
            fifo:         true,
            max_messages: 10,
            wait_seconds: 20,
        }
    }
}

// ── QueueConfig ───────────────────────────────────────────────────────────────

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            queue_type:               QueueType::InMemory,
            kafka:                    None,
            rabbitmq:                 None,
            redis_streams:            None,
            sqs:                      None,
            persistent:               None,
            consumer_threads:         default_consumer_threads(),
            processing_timeout_secs:  default_processing_timeout_secs(),
            dlq_topic:                default_dlq_topic(),
            message_retention_hours:  default_message_retention_hours(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    /// Secret key cho HS512 (current/new key after rotation)
    pub secret: String,
    /// Previous secret key — tokens signed with this key remain valid during rotation window.
    /// Set to old `secret` value when rotating, then remove after all old tokens expire.
    #[serde(default)]
    pub previous_signing_key: Option<String>,
    /// Thời gian sống token (giây) — TB Java default: 9000s (2.5h)
    pub expiration_secs: u64,
    /// Refresh token TTL (giây) — TB Java default: 604800s (7 ngày)
    pub refresh_expiration_secs: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "vielang-change-this-secret-in-production".into(),
            previous_signing_key: None,
            expiration_secs: 9000,
            refresh_expiration_secs: 604_800,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self { jwt: JwtConfig::default() }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransportConfig {
    pub mqtt:    MqttTransportConfig,
    pub http:    HttpTransportConfig,
    pub coap:    CoapTransportConfig,
    #[serde(default)]
    pub lwm2m:   Lwm2mTransportConfig,
    #[serde(default)]
    pub snmp:    SnmpConfig,
    #[serde(default)]
    pub lorawan: LoRaWanConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Lwm2mTransportConfig {
    pub enabled: bool,
    pub bind:    String,
    pub port:    u16,
}

impl Default for Lwm2mTransportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind:    "0.0.0.0".into(),
            port:    5783,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MqttTransportConfig {
    pub enabled: bool,
    pub bind: String,
    pub port: u16,
    #[serde(default)]
    pub ws_enabled: bool,
    #[serde(default = "default_ws_port")]
    pub ws_port: u16,
    #[serde(default = "default_ws_path")]
    pub ws_path: String,
    #[serde(default = "default_mqtt_max_clients")]
    pub max_clients: usize,
}

fn default_mqtt_max_clients() -> usize { 10_000 }

fn default_ws_port() -> u16  { 8083 }
fn default_ws_path() -> String { "/mqtt".into() }

impl Default for MqttTransportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind: "0.0.0.0".into(),
            port: 1883,
            ws_enabled: false,
            ws_port: 8083,
            ws_path: "/mqtt".into(),
            max_clients: 10_000,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpTransportConfig {
    pub enabled: bool,
    pub bind: String,
    pub port: u16,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind: "0.0.0.0".into(),
            port: 8081,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoapTransportConfig {
    pub enabled: bool,
    pub bind: String,
    pub port: u16,
    #[serde(default = "default_coap_observe_enabled")]
    pub observe_enabled: bool,
    #[serde(default = "default_coap_max_observe_rels")]
    pub max_observe_rels: usize,
    #[serde(default = "default_coap_notify_timeout_s")]
    pub notify_timeout_s: u64,
}

fn default_coap_observe_enabled() -> bool  { true }
fn default_coap_max_observe_rels() -> usize { 50_000 }
fn default_coap_notify_timeout_s() -> u64   { 30 }

impl Default for CoapTransportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: "0.0.0.0".into(),
            port: 5683,
            observe_enabled: true,
            max_observe_rels: 50_000,
            notify_timeout_s: 30,
        }
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            mqtt:    MqttTransportConfig::default(),
            http:    HttpTransportConfig::default(),
            coap:    CoapTransportConfig::default(),
            lwm2m:   Lwm2mTransportConfig::default(),
            snmp:    SnmpConfig::default(),
            lorawan: LoRaWanConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnmpConfig {
    pub enabled:    bool,
    /// UDP port to listen for SNMP traps (default 10162 — non-privileged alternative to 162)
    pub bind_port:  u16,
    /// SNMPv2c community string (default "public")
    pub community:  String,
}

impl Default for SnmpConfig {
    fn default() -> Self {
        Self {
            enabled:   false,
            bind_port: 10162,
            community: "public".into(),
        }
    }
}

/// LoRaWAN ChirpStack bridge config (P16).
/// Connects to a ChirpStack MQTT broker and ingests uplink messages.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoRaWanConfig {
    pub enabled:         bool,
    /// MQTT broker URL, e.g. "mqtt://localhost:1883"
    pub chirpstack_url:  String,
    pub username:        String,
    pub password:        String,
    /// If empty, subscribe to all applications; otherwise filter by these IDs.
    #[serde(default)]
    pub application_ids: Vec<String>,
    /// Payload codec: "none" | "cayenne_lpp"
    #[serde(default = "LoRaWanConfig::default_codec")]
    pub payload_codec:   String,
}

impl LoRaWanConfig {
    fn default_codec() -> String { "none".into() }
}

impl Default for LoRaWanConfig {
    fn default() -> Self {
        Self {
            enabled:         false,
            chirpstack_url:  "mqtt://localhost:1883".into(),
            username:        String::new(),
            password:        String::new(),
            application_ids: Vec::new(),
            payload_codec:   "none".into(),
        }
    }
}

/// Cluster coordination config (Phase 11).
/// Default: single-node mode (enabled = false) — etcd/RPC are not started.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClusterConfig {
    /// false = single-node mode (no etcd, no inter-node RPC)
    pub enabled:  bool,
    /// etcd endpoint
    pub etcd_url: String,
    /// This node's ID — auto-generated UUID if empty
    pub node_id:  String,
    /// gRPC / RPC listen host (this node's advertised IP)
    pub rpc_host: String,
    /// gRPC / RPC listen port
    pub rpc_port: u16,
    /// Number of virtual partitions for consistent hash ring (default: 12)
    #[serde(default = "ClusterConfig::default_num_partitions")]
    pub num_partitions: u32,
    /// Leader election timeout in milliseconds (default: 10 000)
    #[serde(default = "ClusterConfig::default_election_timeout_ms")]
    pub election_timeout_ms: u64,
}

impl ClusterConfig {
    fn default_num_partitions() -> u32 { 12 }
    fn default_election_timeout_ms() -> u64 { 10_000 }
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            enabled:              false,
            etcd_url:             "http://localhost:2379".into(),
            node_id:              String::new(),
            rpc_host:             "localhost".into(),
            rpc_port:             9090,
            num_partitions:       Self::default_num_partitions(),
            election_timeout_ms:  Self::default_election_timeout_ms(),
        }
    }
}

impl Default for VieLangConfig {
    fn default() -> Self {
        Self {
            server:       ServerConfig::default(),
            database:     DatabaseConfig::default(),
            cache:        CacheConfig::default(),
            queue:        QueueConfig::default(),
            security:     SecurityConfig::default(),
            transport:    TransportConfig::default(),
            cluster:      ClusterConfig::default(),
            notification: NotificationConfig::default(),
            housekeeper:  HousekeeperConfig::default(),
            edge_grpc:    EdgeGrpcConfig::default(),
            trendz:       TrendzConfig::default(),
            firebase:     FirebaseConfig::default(),
            stripe:       StripeConfig::default(),
            ota:          OtaConfig::default(),
            search:        SearchConfig::default(),
            observability: ObservabilityConfig::default(),
            geofence:      GeofenceConfig::default(),
            scheduler:     SchedulerConfig::default(),
            auth:          AuthConfig::default(),
            simulator:     SimulatorCfg::default(),
        }
    }
}

/// Edge gRPC server config (Phase 56).
/// TB Edge client kết nối tới đây để sync entities và push telemetry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EdgeGrpcConfig {
    /// false = không khởi động gRPC server (default)
    pub enabled: bool,
    pub bind:    String,
    pub port:    u16,
}

impl Default for EdgeGrpcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind:    "0.0.0.0".into(),
            port:    7070,  // ThingsBoard Edge default gRPC port
        }
    }
}

// ── Housekeeper config ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HousekeeperConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub ts_ttl_days: i64,
    pub events_ttl_days: i64,
    pub alarms_ttl_days: i64,
    pub rpc_ttl_days: i64,
    pub batch_size: i64,
}

impl Default for HousekeeperConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 3600,
            ts_ttl_days: 365,
            events_ttl_days: 7,
            alarms_ttl_days: 30,
            rpc_ttl_days: 1,
            batch_size: 10_000,
        }
    }
}

impl VieLangConfig {
    /// Load config từ file TOML + env vars
    /// Env vars override file (prefix VL_)
    pub fn load() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("config/vielang").required(false))
            .add_source(
                config::Environment::with_prefix("VL")
                    .separator("__")
                    .list_separator(",")
                    .with_list_parse_key("server.allowed_origins")
                    .with_list_parse_key("mobile.firebase.application_ids")
                    .try_parsing(true),
            )
            .build()?
            .try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = VieLangConfig::default();
        assert_eq!(config.server.port, 8080);
        assert!(matches!(config.cache.cache_type, CacheType::InMemory));
        assert!(matches!(config.queue.queue_type, QueueType::InMemory));
        assert!(!config.cluster.enabled);
    }

    #[test]
    fn default_security_config() {
        let config = SecurityConfig::default();
        assert!(!config.jwt.secret.is_empty());
        assert!(config.jwt.expiration_secs > 0);
        assert!(config.jwt.refresh_expiration_secs > config.jwt.expiration_secs);
    }

    #[test]
    fn default_transport_config() {
        let config = TransportConfig::default();
        assert!(config.mqtt.enabled);
        assert_eq!(config.mqtt.port, 1883);
    }

    #[test]
    fn default_database_config() {
        let config = DatabaseConfig::default();
        assert!(!config.postgres.url.is_empty());
        assert!(config.postgres.max_connections > 0);
    }

    #[test]
    fn default_housekeeper_config() {
        let config = HousekeeperConfig::default();
        assert!(config.interval_secs > 0);
    }

    #[test]
    fn default_ota_config() {
        let config = OtaConfig::default();
        assert!(config.enabled);
        assert!(config.chunk_size_kb > 0);
    }

    #[test]
    fn default_scheduler_config() {
        let config = SchedulerConfig::default();
        assert!(config.enabled);
        assert!(config.check_interval_s > 0);
        assert!(config.max_concurrent_jobs > 0);
    }

    #[test]
    fn config_serializes_to_json() {
        let config = VieLangConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.is_empty());
        // Round-trip
        let deserialized: VieLangConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.server.port, config.server.port);
    }
}
