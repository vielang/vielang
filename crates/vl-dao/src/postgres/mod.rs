pub mod ts_dao;
pub mod device_activity;
pub mod admin_settings;
pub mod alarm;
pub mod mobile_app;
pub mod edge;
pub mod entity_query;
pub mod api_key;
pub mod asset;
pub mod asset_profile;
pub mod audit_log;
pub mod customer;
pub mod device_profile;
pub mod entity_view;
pub mod tenant_profile;
pub mod dashboard;
pub mod device;
pub mod event;
pub mod kv;
pub mod ldap_config;
pub mod notification_channel_settings;
pub mod saml_config;
pub mod notification_request;
pub mod notification_rule;
pub mod notification_target;
pub mod notification_template;
pub mod oauth2_registration;
pub mod ota_package;
pub mod ota_state;
pub mod relation;
pub mod rpc;
pub mod rule_chain;
pub mod tenant;
pub mod two_factor_auth;
pub mod resource;
pub mod user;
pub mod widget_type;
pub mod widgets_bundle;
pub mod component_descriptor;
pub mod housekeeper;
pub mod calculated_field;
pub mod entity_version;
pub mod scheduled_job;
pub mod cluster_node;
pub mod queue_stats;
pub mod ai_model;
pub mod domain;
pub mod oauth2_template;
pub mod partition_manager;
pub mod rbac;
pub mod mobile_session;
pub mod notification_delivery;
pub mod notification_inbox;
pub mod subscription;
pub mod api_usage;
pub mod analytics;
pub mod search;
pub mod geofence;
pub mod queue_message;
pub mod backup;
pub mod cluster_partition;
pub mod activation_token;
pub mod dlq;
pub mod simulator;
pub mod device_template;
pub mod rule_node;
pub mod rule_node_state;
pub mod entity_alarm;
pub mod user_settings;

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use vl_config::PostgresConfig;
use tracing::info;

pub type DbPool = PgPool;

/// Khởi tạo connection pool — gọi một lần khi startup.
///
/// Cấu hình industry-standard:
/// - SSL mode có thể chọn (disable/prefer/require/verify-ca/verify-full)
/// - `application_name` set trong pg_stat_activity để DBA nhận dạng
/// - `statement_timeout` tránh slow query treo pool
/// - Password không bao giờ xuất hiện trong log
pub async fn init_pool(config: &PostgresConfig) -> Result<DbPool, sqlx::Error> {
    let connect_opts = build_connect_options(config)?;

    info!(
        "Connecting to PostgreSQL: {}:{}/{} (ssl={}, app={})",
        connect_opts.get_host(),
        connect_opts.get_port(),
        connect_opts.get_database().unwrap_or("postgres"),
        config.ssl_mode,
        config.application_name,
    );

    let statement_timeout_ms = config.statement_timeout_ms;
    let application_name    = config.application_name.clone();

    let mut pool_opts = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.connect_timeout_secs))
        .after_connect(move |conn, _meta| {
            let app = application_name.clone();
            let timeout_ms = statement_timeout_ms;
            Box::pin(async move {
                // Đặt application_name — hiện trong pg_stat_activity
                sqlx::query(&format!("SET application_name = '{app}'"))
                    .execute(&mut *conn)
                    .await?;
                // Đặt statement_timeout nếu được cấu hình
                if let Some(ms) = timeout_ms {
                    sqlx::query(&format!("SET statement_timeout = {ms}"))
                        .execute(&mut *conn)
                        .await?;
                }
                Ok(())
            })
        });

    if let Some(idle_secs) = config.idle_timeout_secs {
        pool_opts = pool_opts.idle_timeout(std::time::Duration::from_secs(idle_secs));
    }
    if let Some(lifetime_secs) = config.max_lifetime_secs {
        pool_opts = pool_opts.max_lifetime(std::time::Duration::from_secs(lifetime_secs));
    }

    let pool = pool_opts.connect_with(connect_opts).await?;

    // Chạy migrations tự động
    sqlx::migrate!("../../migrations").run(&pool).await?;
    info!("Database migrations applied successfully");

    Ok(pool)
}

/// Parse URL → PgConnectOptions rồi override SSL mode.
/// Password không bao giờ logged vì chúng ta log từng field riêng.
fn build_connect_options(config: &PostgresConfig) -> Result<PgConnectOptions, sqlx::Error> {
    let ssl_mode = match config.ssl_mode.as_str() {
        "disable"     => PgSslMode::Disable,
        "allow"       => PgSslMode::Allow,
        "prefer"      => PgSslMode::Prefer,
        "require"     => PgSslMode::Require,
        "verify-ca"   => PgSslMode::VerifyCa,
        "verify-full" => PgSslMode::VerifyFull,
        other => {
            tracing::warn!("Unknown ssl_mode '{}', falling back to 'prefer'", other);
            PgSslMode::Prefer
        }
    };

    let opts: PgConnectOptions = config.url.parse()?;
    Ok(opts.ssl_mode(ssl_mode))
}
