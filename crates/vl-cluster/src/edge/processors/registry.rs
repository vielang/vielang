use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

use super::EdgeEventProcessor;

/// Central registry that holds all edge event processors and dispatches by entity type.
///
/// Built during bootstrap (in vl-api) and injected into the edge gRPC server.
pub struct EdgeProcessorRegistry {
    processors: HashMap<String, Arc<dyn EdgeEventProcessor>>,
}

impl EdgeProcessorRegistry {
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
        }
    }

    /// Register a processor. Its `entity_type()` is used as the dispatch key.
    /// If a processor for the same entity type already exists, it is replaced.
    pub fn register(&mut self, processor: Arc<dyn EdgeEventProcessor>) {
        let entity_type = processor.entity_type().to_string();
        self.processors.insert(entity_type, processor);
    }

    /// Look up the processor for the given entity type.
    pub fn get(&self, entity_type: &str) -> Option<&Arc<dyn EdgeEventProcessor>> {
        self.processors.get(entity_type)
    }

    /// Process a downlink event by dispatching to the correct processor.
    pub async fn dispatch_downlink(
        &self,
        entity_type: &str,
        entity_id: uuid::Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String> {
        match self.processors.get(entity_type) {
            Some(processor) => processor.process_downlink(entity_id, action).await,
            None => {
                warn!(entity_type, "No edge processor registered for entity type");
                Err(format!("No processor for entity type: {}", entity_type))
            }
        }
    }

    /// Process an uplink event by dispatching to the correct processor.
    pub async fn dispatch_uplink(
        &self,
        entity_type: &str,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        match self.processors.get(entity_type) {
            Some(processor) => processor.process_uplink(payload).await,
            None => {
                warn!(entity_type, "No edge processor registered for entity type");
                Err(format!("No processor for entity type: {}", entity_type))
            }
        }
    }

    /// List all registered entity types.
    pub fn registered_types(&self) -> Vec<&str> {
        self.processors.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for EdgeProcessorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
