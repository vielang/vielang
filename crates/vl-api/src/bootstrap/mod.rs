pub mod tasks;
pub mod infra;
pub mod queue;
pub mod services;
pub mod background;
pub mod transport;

pub use tasks::BackgroundTasks;
pub use infra::init_infra;
pub use services::init_core_services;
pub use background::start_background_services;
pub use transport::start_transports;

use tracing::info;

/// Shutdown signal handler (SIGINT + SIGTERM).
pub async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for Ctrl+C: {}", e);
        }
    };

    #[cfg(unix)]
    let sigterm = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => { sig.recv().await; }
            Err(e) => {
                tracing::error!("Failed to install SIGTERM handler: {}", e);
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c   => { info!("SIGINT received") }
        _ = sigterm  => { info!("SIGTERM received") }
    }

    info!("Graceful shutdown initiated, draining connections...");
}
