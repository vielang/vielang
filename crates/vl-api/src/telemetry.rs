//! Observability initialisation — structured logging + optional OTLP tracing.
//!
//! Call `init_tracing(&config)` early in `main()`, before the first `tracing::info!`.
//! Keep the returned `ShutdownGuard` alive for the process lifetime so the OTLP
//! exporter can flush pending spans on graceful shutdown.
//!
//! # Layer order
//!
//! The OpenTelemetryLayer must be added BEFORE the fmt layer in the subscriber
//! chain; adding it after a `JsonFields` formatter breaks the trait bounds because
//! `OpenTelemetryLayer<S,T>` requires `S` to use `DefaultFields`.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use vl_config::ObservabilityConfig;

/// Holds the OpenTelemetry TracerProvider so it can be flushed on drop.
/// When OTLP is disabled, this is a no-op.
pub struct ShutdownGuard {
    otel_enabled: bool,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        if self.otel_enabled {
            opentelemetry::global::shutdown_tracer_provider();
        }
    }
}

/// Initialise the global tracing subscriber.
///
/// - `log_format = "json"`   → structured JSON lines (production)
/// - `log_format = "pretty"` → coloured human-readable output (development)
/// - `tracing_enabled = true` + non-empty `otlp_endpoint` → OTLP HTTP export
///
/// The `RUST_LOG` env var overrides `log_level` from config.
pub fn init_tracing(config: &ObservabilityConfig) -> ShutdownGuard {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "vielang={level},tower_http=info",
            level = config.log_level
        ))
    });

    let want_otlp = config.tracing_enabled && !config.otlp_endpoint.is_empty();
    let pretty    = config.log_format == "pretty";

    if want_otlp {
        match build_otlp_tracer(&config.otlp_endpoint) {
            Ok(tracer) => {
                // IMPORTANT: add otel layer BEFORE the fmt layer so the
                // subscriber type seen by OpenTelemetryLayer uses DefaultFields.
                let otel = tracing_opentelemetry::layer().with_tracer(tracer);
                if pretty {
                    tracing_subscriber::registry()
                        .with(filter)
                        .with(otel)
                        .with(tracing_subscriber::fmt::layer())
                        .init();
                } else {
                    tracing_subscriber::registry()
                        .with(filter)
                        .with(otel)
                        .with(tracing_subscriber::fmt::layer().json())
                        .init();
                }
                tracing::info!(otlp_endpoint = %config.otlp_endpoint, "OTLP tracing enabled");
                return ShutdownGuard { otel_enabled: true };
            }
            Err(e) => {
                // Non-fatal: fall through to plain logging
                eprintln!("[vielang] OTLP setup failed ({e}), falling back to local logging");
            }
        }
    }

    // Plain logging only
    if pretty {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    }

    ShutdownGuard { otel_enabled: false }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn build_otlp_tracer(
    endpoint: &str,
) -> Result<opentelemetry_sdk::trace::Tracer, opentelemetry::trace::TraceError> {
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::{runtime::Tokio, trace as sdktrace, Resource};

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(endpoint),
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name",    "vielang"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ])),
        )
        .install_batch(Tokio)
}
