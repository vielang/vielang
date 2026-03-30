use std::future::Future;
use tokio::task::JoinHandle;

/// Collects background task handles for tracked lifecycle and graceful shutdown.
pub struct BackgroundTasks {
    handles: Vec<(String, JoinHandle<()>)>,
}

impl BackgroundTasks {
    pub fn new() -> Self {
        Self { handles: Vec::new() }
    }

    /// Spawn a named background task.
    pub fn spawn(&mut self, name: impl Into<String>, future: impl Future<Output = ()> + Send + 'static) {
        let name = name.into();
        let handle = tokio::spawn(future);
        self.handles.push((name, handle));
    }

    /// Track an already-spawned task handle.
    pub fn track(&mut self, name: impl Into<String>, handle: JoinHandle<()>) {
        self.handles.push((name.into(), handle));
    }

    /// Abort all tracked background tasks (called during graceful shutdown).
    pub async fn shutdown_all(self) {
        for (name, handle) in self.handles {
            handle.abort();
            tracing::debug!("Aborted background task: {name}");
        }
    }
}
