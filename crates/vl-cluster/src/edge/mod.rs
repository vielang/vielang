/// Edge protocol module — gRPC session management + uplink/downlink processing.
///
/// Architecture:
///   - `EdgeSessionRegistry` tracks connected Edge instances (DashMap)
///   - `EdgeGrpcServer` (tonic) accepts bidirectional streams from Edge clients
///   - `EdgeUplinkHandler` trait (injected from vl-api) handles telemetry/RPC saves
///   - `EdgeSender` trait (in vl-core) lets rule nodes push data down to Edges

pub mod session;
pub mod downlink_sender;
pub mod uplink_processor;
pub mod grpc_server;
pub mod processors;

/// Proto-generated types (tonic-build output).
/// Requires `protoc` on PATH — see build.rs for instructions.
pub mod proto {
    pub mod edge {
        tonic::include_proto!("edge");
    }
}

pub use downlink_sender::{EdgeSessionRegistry, EdgeSyncable, make_entity_update_payload};
pub use grpc_server::run_edge_grpc;
pub use uplink_processor::{EdgeUplinkHandler, AuthError, TelemetryEntry, TelemetryValue};
pub use processors::{EdgeEventProcessor, registry::EdgeProcessorRegistry};
