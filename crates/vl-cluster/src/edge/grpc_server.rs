use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::edge::{
    proto::edge::{
        DownlinkMsg, EntityUpdateMsg, UplinkMsg,
        edge_rpc_service_server::{EdgeRpcService, EdgeRpcServiceServer},
    },
    downlink_sender::EdgeSessionRegistry,
    session::EdgeSession,
    uplink_processor::{EdgeUplinkHandler, process_uplink},
};

const DOWNLINK_CHANNEL_CAPACITY: usize = 512;

pub struct EdgeRpcServiceImpl {
    pub registry: Arc<EdgeSessionRegistry>,
    pub handler:  Arc<dyn EdgeUplinkHandler>,
}

#[tonic::async_trait]
impl EdgeRpcService for EdgeRpcServiceImpl {
    type HandleMsgsStream = ReceiverStream<Result<DownlinkMsg, Status>>;

    async fn handle_msgs(
        &self,
        request: Request<Streaming<UplinkMsg>>,
    ) -> Result<Response<Self::HandleMsgsStream>, Status> {
        let mut stream = request.into_inner();

        // ── Step 1: Receive first message and authenticate ──────────────────────
        let first_msg = stream.message().await
            .map_err(|e| Status::internal(format!("Stream error: {}", e)))?
            .ok_or_else(|| Status::unauthenticated("No initial message received"))?;

        let connect = first_msg.connect_request_msg.into_iter().next()
            .ok_or_else(|| Status::unauthenticated("Missing ConnectRequestMsg in first uplink"))?;

        let (edge_id, tenant_id) = self.handler
            .authenticate_edge(&connect.routing_key, &connect.secret)
            .await
            .map_err(|e| {
                warn!(routing_key = %connect.routing_key, "Edge auth failed: {}", e);
                Status::unauthenticated(format!("Authentication failed: {}", e))
            })?;

        info!(edge_id = %edge_id, tenant_id = %tenant_id, "Edge connected via gRPC");

        // ── Step 2: Create downlink channel and register session ────────────────
        let (json_tx, mut json_rx) = mpsc::channel::<serde_json::Value>(DOWNLINK_CHANNEL_CAPACITY);
        let (proto_tx, proto_rx)   = mpsc::channel::<Result<DownlinkMsg, Status>>(DOWNLINK_CHANNEL_CAPACITY);

        let session = EdgeSession::new(edge_id, tenant_id, json_tx);
        self.registry.register(session);

        // ── Step 3: JSON → proto converter task ─────────────────────────────────
        {
            let proto_tx_clone = proto_tx.clone();
            let mut msg_seq: i32 = 1;
            tokio::spawn(async move {
                while let Some(json_payload) = json_rx.recv().await {
                    let body = match serde_json::to_vec(&json_payload) {
                        Ok(b) => b,
                        Err(e) => {
                            warn!("Failed to serialize downlink payload: {}", e);
                            continue;
                        }
                    };

                    let downlink = DownlinkMsg {
                        downlink_msg_id: msg_seq,
                        entity_data_msg: vec![EntityUpdateMsg {
                            entity_type:   0, // DEVICE = 0
                            entity_id_msc: String::new(),
                            entity_body:   body,
                        }],
                        ..Default::default()
                    };
                    msg_seq += 1;

                    if proto_tx_clone.send(Ok(downlink)).await.is_err() {
                        break;
                    }
                }
            });
        }

        // ── Step 4: Uplink processing task ──────────────────────────────────────
        {
            let registry = self.registry.clone();
            let handler  = self.handler.clone();
            let proto_tx_clone = proto_tx.clone();

            tokio::spawn(async move {
                // Send initial entity sync to Edge
                match handler.send_initial_sync(edge_id, tenant_id).await {
                    Ok(payloads) => {
                        for payload in payloads {
                            registry.push_to_edge_raw(edge_id, payload);
                        }
                    }
                    Err(e) => error!(edge_id = %edge_id, "Initial sync error: {}", e),
                }

                // Process incoming uplink messages
                loop {
                    match stream.message().await {
                        Ok(Some(msg)) => {
                            // Update last_seen
                            if let Some(session) = registry.get_session(edge_id) {
                                *session.last_seen.lock().await = std::time::Instant::now();
                            }

                            match process_uplink(edge_id, tenant_id, msg, handler.as_ref()).await {
                                Ok(ack) => {
                                    if proto_tx_clone.send(Ok(ack)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!(edge_id = %edge_id, "Uplink processing error: {}", e);
                                }
                            }
                        }
                        Ok(None) => break, // stream closed
                        Err(e) => {
                            warn!(edge_id = %edge_id, "Stream error: {}", e);
                            break;
                        }
                    }
                }

                registry.remove(edge_id);
                info!(edge_id = %edge_id, "Edge disconnected");
            });
        }

        Ok(Response::new(ReceiverStream::new(proto_rx)))
    }
}

/// Khởi động Edge gRPC server.
/// Gọi từ `main.rs` nếu `config.edge_grpc.enabled = true`.
pub async fn run_edge_grpc(
    bind: String,
    port: u16,
    registry: Arc<EdgeSessionRegistry>,
    handler:  Arc<dyn EdgeUplinkHandler>,
) -> Result<(), tonic::transport::Error> {
    let addr = format!("{}:{}", bind, port)
        .parse()
        .expect("Invalid edge gRPC bind address");

    let service = EdgeRpcServiceImpl { registry, handler };

    info!("Edge gRPC server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(EdgeRpcServiceServer::new(service))
        .serve(addr)
        .await
}
