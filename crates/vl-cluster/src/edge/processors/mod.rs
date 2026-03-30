/// Edge event processor framework — trait + registry for entity sync between cloud and edge.
///
/// The `EdgeEventProcessor` trait is defined here (in vl-cluster) so the gRPC server
/// can dispatch by entity type. Concrete implementations live in vl-api where DAOs
/// are available (same pattern as `EdgeUplinkHandler`).

pub mod registry;

use async_trait::async_trait;
use uuid::Uuid;

/// Each processor handles syncing a specific entity type to/from an edge gateway.
///
/// Downlink = cloud -> edge (push entity state to edge).
/// Uplink   = edge -> cloud (apply edge changes to cloud DB).
#[async_trait]
pub trait EdgeEventProcessor: Send + Sync + 'static {
    /// Entity type this processor handles (e.g., "DEVICE", "ASSET", "ALARM").
    /// Must match `EdgeEvent.edge_event_type` values.
    fn entity_type(&self) -> &'static str;

    /// Process a downlink event: serialize an entity for edge consumption.
    ///
    /// `entity_id` — UUID of the entity to push.
    /// `action` — event action (e.g., "ADDED", "UPDATED", "DELETED").
    ///
    /// Returns the JSON payload to send to the edge, or an error description.
    async fn process_downlink(
        &self,
        entity_id: Uuid,
        action: &str,
    ) -> Result<serde_json::Value, String>;

    /// Process an uplink event: apply edge changes to cloud state.
    ///
    /// `payload` — JSON body received from the edge for this entity type.
    ///
    /// Returns `Ok(())` on success, or an error description.
    async fn process_uplink(
        &self,
        payload: &serde_json::Value,
    ) -> Result<(), String>;
}
