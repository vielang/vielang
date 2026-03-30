use anyhow::Context;
use tracing::info;
use utoipa::OpenApi as _;

mod actors;
mod bootstrap;
mod error;
mod metrics;
mod middleware;
mod notification;
mod openapi;
mod routes;
mod services;
mod state;
mod telemetry;
mod util;
mod ws;

pub use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut tasks = bootstrap::BackgroundTasks::new();

    // Layer 1: Infrastructure (config, tracing, metrics, DB, migrations, KV warmup)
    let (infra, _tracing_guard) = bootstrap::init_infra(&mut tasks).await?;

    // Layer 2: Core services (cache, rule engine, queue, cluster, timeseries, AppState)
    let core = bootstrap::init_core_services(&infra, &mut tasks).await?;

    // Layer 3: Background services (housekeeper, scheduler, monitors, LDAP, token cleanup)
    bootstrap::start_background_services(&core.state, &infra.config, &infra.pool, &mut tasks);

    // Layer 4: Transport protocols (MQTT, HTTP, CoAP, LwM2M, SNMP, LoRaWAN, Edge gRPC)
    bootstrap::start_transports(&core, &infra.config, &infra.pool, &mut tasks);

    // Layer 5: HTTP management API router
    let metrics_router = {
        let handle = infra.prometheus_handle.clone();
        axum::Router::new().route(
            "/metrics",
            axum::routing::get(move || {
                let h = handle.clone();
                async move { h.render() }
            }),
        )
    };

    let actor_system = core.actor_system.clone();

    let app = routes::create_router(core.state)
        .merge(metrics_router)
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
        );

    // Bind & Serve
    let addr = format!("{}:{}", infra.config.server.host, infra.config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to {}", addr))?;

    info!("VieLang listening on http://{}", addr);
    info!("Swagger UI: http://{}/swagger-ui", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(bootstrap::shutdown_signal())
        .await
        .context("Server error")?;

    // Graceful shutdown: actor system + background tasks
    actor_system.shutdown().await;
    tasks.shutdown_all().await;

    Ok(())
}
