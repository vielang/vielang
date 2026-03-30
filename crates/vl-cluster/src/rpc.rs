/// Node-to-node RPC layer.
///
/// Dùng tokio TCP + length-prefixed JSON framing thay vì gRPC để tránh
/// dependency vào `protoc`. Có thể migrate sang tonic khi cần binary efficiency.
///
/// Protocol:
///   client → server: [4 bytes BE length][JSON payload]
///   server → client: [4 bytes BE length][JSON payload]
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::ClusterError;
use crate::node::NodeInfo;

// ── Message types ─────────────────────────────────────────────────────────────

/// Tin nhắn gửi giữa các node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMsg {
    /// Node gửi
    pub origin_node_id: Uuid,
    /// Loại message (vd: "FORWARD_MSG", "SESSION_PING")
    pub msg_type: String,
    /// Payload dưới dạng JSON bytes (base64-encoded)
    pub payload: Vec<u8>,
}

/// Response từ node nhận.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResponse {
    pub success: bool,
    pub payload: Vec<u8>,
}

// ── Handler trait ─────────────────────────────────────────────────────────────

/// Implement trait này để xử lý message nhận được từ node khác.
#[async_trait::async_trait]
pub trait ClusterMsgHandler: Send + Sync + 'static {
    async fn handle(&self, msg: ClusterMsg) -> ClusterResponse;
}

/// No-op handler — dùng khi cluster disabled hoặc test.
pub struct NoopHandler;

#[async_trait::async_trait]
impl ClusterMsgHandler for NoopHandler {
    async fn handle(&self, _msg: ClusterMsg) -> ClusterResponse {
        ClusterResponse { success: true, payload: vec![] }
    }
}

// ── RPC Server ────────────────────────────────────────────────────────────────

/// Khởi động TCP RPC server, trả về JoinHandle.
pub fn start_rpc_server(
    bind_addr: String,
    handler: Arc<dyn ClusterMsgHandler>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = match TcpListener::bind(&bind_addr).await {
            Ok(l) => {
                info!("Cluster RPC server listening on {}", bind_addr);
                l
            }
            Err(e) => {
                error!("Failed to bind cluster RPC server on {}: {}", bind_addr, e);
                return;
            }
        };

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    debug!("Cluster RPC connection from {}", peer);
                    let handler = handler.clone();
                    tokio::spawn(handle_connection(stream, handler));
                }
                Err(e) => {
                    error!("Cluster RPC accept error: {}", e);
                }
            }
        }
    })
}

async fn handle_connection(mut stream: TcpStream, handler: Arc<dyn ClusterMsgHandler>) {
    loop {
        // Read message
        let msg = match read_framed(&mut stream).await {
            Ok(Some(m)) => m,
            Ok(None) => break, // connection closed
            Err(e) => {
                warn!("Cluster RPC read error: {}", e);
                break;
            }
        };

        let response = handler.handle(msg).await;

        if let Err(e) = write_framed(&mut stream, &response).await {
            warn!("Cluster RPC write error: {}", e);
            break;
        }
    }
}

// ── RPC Client ────────────────────────────────────────────────────────────────

/// Client gửi ClusterMsg tới một node khác.
pub struct ClusterRpcClient {
    local_node_id: Uuid,
}

impl ClusterRpcClient {
    pub fn new(local_node_id: Uuid) -> Self {
        Self { local_node_id }
    }

    pub async fn send_msg(
        &self,
        target: &NodeInfo,
        msg_type: impl Into<String>,
        payload: Vec<u8>,
    ) -> Result<ClusterResponse, ClusterError> {
        let addr = target.rpc_addr();
        let mut stream = TcpStream::connect(&addr).await?;

        let msg = ClusterMsg {
            origin_node_id: self.local_node_id,
            msg_type: msg_type.into(),
            payload,
        };

        write_framed(&mut stream, &msg).await?;
        let resp: ClusterResponse = read_framed(&mut stream)
            .await?
            .ok_or_else(|| ClusterError::Rpc("Connection closed before response".into()))?;

        Ok(resp)
    }
}

// ── Framing helpers ───────────────────────────────────────────────────────────

async fn write_framed<T: Serialize>(
    stream: &mut TcpStream,
    value: &T,
) -> Result<(), ClusterError> {
    let payload = serde_json::to_vec(value)?;
    let len = payload.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&payload).await?;
    Ok(())
}

async fn read_framed<T: for<'de> Deserialize<'de>>(
    stream: &mut TcpStream,
) -> Result<Option<T>, ClusterError> {
    let mut len_buf = [0u8; 4];
    match stream.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(ClusterError::Io(e)),
    }
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    let value = serde_json::from_slice(&buf)?;
    Ok(Some(value))
}

// ── Channel bridge ────────────────────────────────────────────────────────────

/// Gửi ClusterMsg qua mpsc channel để ClusterManager xử lý bất đồng bộ.
pub fn msg_channel() -> (mpsc::Sender<ClusterMsg>, mpsc::Receiver<ClusterMsg>) {
    mpsc::channel(256)
}
