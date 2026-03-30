use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures_util::{Sink, Stream};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::{
        handshake::server::{ErrorResponse, Request, Response},
        Message,
    },
    WebSocketStream,
};
use tracing::{debug, error, info, warn};

use vl_cache::TbCache;
use vl_config::MqttTransportConfig;
use vl_core::entities::{ActivityEvent, TbMsg};
use vl_dao::{DbPool, TimeseriesDao};
use vl_queue::TbProducer;

use super::handler::handle_connection;
use super::session_store::PersistentSessionStore;

// ── WebSocket → AsyncRead + AsyncWrite adapter ────────────────────────────────

/// Wraps a `WebSocketStream<TcpStream>` and exposes it as `AsyncRead + AsyncWrite`.
/// Binary WebSocket frames are transparently forwarded as a raw byte stream,
/// matching the behaviour expected by the MQTT codec.
pub struct WsStreamAdapter {
    inner:    WebSocketStream<TcpStream>,
    read_buf: Vec<u8>,
    read_pos: usize,
}

impl WsStreamAdapter {
    fn new(ws: WebSocketStream<TcpStream>) -> Self {
        Self { inner: ws, read_buf: Vec::new(), read_pos: 0 }
    }
}

impl Unpin for WsStreamAdapter {}

impl AsyncRead for WsStreamAdapter {
    fn poll_read(
        self: Pin<&mut Self>,
        cx:   &mut Context<'_>,
        buf:  &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        // Serve leftover bytes from a previous oversized frame first.
        if this.read_pos < this.read_buf.len() {
            let remaining = &this.read_buf[this.read_pos..];
            let n = remaining.len().min(buf.remaining());
            buf.put_slice(&remaining[..n]);
            this.read_pos += n;
            if this.read_pos >= this.read_buf.len() {
                this.read_buf.clear();
                this.read_pos = 0;
            }
            return Poll::Ready(Ok(()));
        }

        // Poll the next WebSocket frame.
        loop {
            match Pin::new(&mut this.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(Message::Binary(data)))) => {
                    let n = data.len().min(buf.remaining());
                    buf.put_slice(&data[..n]);
                    if n < data.len() {
                        // Buffer the remainder for the next read call.
                        this.read_buf = data.into();
                        this.read_pos = n;
                    }
                    return Poll::Ready(Ok(()));
                }
                // Some clients send MQTT bytes as Text frames (non-standard but seen in practice).
                Poll::Ready(Some(Ok(Message::Text(text)))) => {
                    let data = text.into_bytes();
                    let n = data.len().min(buf.remaining());
                    buf.put_slice(&data[..n]);
                    if n < data.len() {
                        this.read_buf = data;
                        this.read_pos = n;
                    }
                    return Poll::Ready(Ok(()));
                }
                Poll::Ready(Some(Ok(Message::Ping(payload)))) => {
                    // tungstenite auto-responds to pings; just ignore here.
                    let _ = payload;
                    continue;
                }
                Poll::Ready(Some(Ok(Message::Pong(_)))) => continue,
                Poll::Ready(Some(Ok(Message::Close(_)))) => {
                    // EOF
                    return Poll::Ready(Ok(()));
                }
                Poll::Ready(Some(Ok(Message::Frame(_)))) => continue,
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, e)));
                }
                Poll::Ready(None) => return Poll::Ready(Ok(())), // stream closed
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl AsyncWrite for WsStreamAdapter {
    fn poll_write(
        self: Pin<&mut Self>,
        cx:   &mut Context<'_>,
        buf:  &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        // Check Sink readiness before sending.
        match Pin::new(&mut this.inner).poll_ready(cx) {
            Poll::Ready(Ok(())) => {}
            Poll::Ready(Err(e)) => {
                return Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, e)));
            }
            Poll::Pending => return Poll::Pending,
        }
        let msg = Message::Binary(buf.to_vec());
        match Pin::new(&mut this.inner).start_send(msg) {
            Ok(()) => Poll::Ready(Ok(buf.len())),
            Err(e) => Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, e))),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx:   &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner)
            .poll_flush(cx)
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx:   &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner)
            .poll_close(cx)
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
    }
}

// ── WebSocket MQTT server ─────────────────────────────────────────────────────

pub struct MqttWebSocketServer {
    config:          MqttTransportConfig,
    pool:            DbPool,
    ts_dao:          Arc<dyn TimeseriesDao>,
    rule_engine_tx:  Arc<Option<mpsc::Sender<TbMsg>>>,
    queue_producer:  Arc<dyn TbProducer>,
    cache:           Arc<dyn TbCache>,
    ws_tx:           broadcast::Sender<TbMsg>,
    activity_tx:     mpsc::Sender<ActivityEvent>,
    device_registry: Arc<crate::DeviceWriteRegistry>,
    rpc_pending:     Arc<crate::RpcPendingRegistry>,
    session_store:   Arc<PersistentSessionStore>,
    chunk_size_kb:   usize,
}

impl MqttWebSocketServer {
    pub fn new(
        config:          MqttTransportConfig,
        pool:            DbPool,
        ts_dao:          Arc<dyn TimeseriesDao>,
        rule_engine_tx:  Arc<Option<mpsc::Sender<TbMsg>>>,
        queue_producer:  Arc<dyn TbProducer>,
        cache:           Arc<dyn TbCache>,
        ws_tx:           broadcast::Sender<TbMsg>,
        activity_tx:     mpsc::Sender<ActivityEvent>,
        device_registry: Arc<crate::DeviceWriteRegistry>,
        rpc_pending:     Arc<crate::RpcPendingRegistry>,
        session_store:   Arc<PersistentSessionStore>,
        chunk_size_kb:   usize,
    ) -> Self {
        Self {
            config,
            pool,
            ts_dao,
            rule_engine_tx,
            queue_producer,
            cache,
            ws_tx,
            activity_tx,
            device_registry,
            rpc_pending,
            session_store,
            chunk_size_kb,
        }
    }

    pub async fn run(self) {
        let addr: SocketAddr =
            match format!("{}:{}", self.config.bind, self.config.ws_port).parse() {
                Ok(a) => a,
                Err(e) => {
                    error!("Invalid MQTT WebSocket bind address: {}", e);
                    return;
                }
            };

        let listener = match TcpListener::bind(addr).await {
            Ok(l) => {
                info!("MQTT WebSocket server listening on ws://{}{}", addr, self.config.ws_path);
                l
            }
            Err(e) => {
                error!("Failed to bind MQTT WebSocket on {}: {}", addr, e);
                return;
            }
        };

        let self_arc = Arc::new(self);

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    debug!("MQTT WS connection from {}", peer);
                    let srv = self_arc.clone();
                    tokio::spawn(async move {
                        srv.handle_ws_connection(stream, peer).await;
                    });
                }
                Err(e) => {
                    error!("MQTT WebSocket accept error: {}", e);
                }
            }
        }
    }

    async fn handle_ws_connection(self: Arc<Self>, stream: TcpStream, peer: SocketAddr) {
        let ws_path = self.config.ws_path.clone();

        // Perform HTTP → WebSocket upgrade.
        let ws_stream = match accept_hdr_async(stream, move |req: &Request, mut resp: Response| {
            // Validate path.
            let path = req.uri().path();
            if path != ws_path {
                warn!("MQTT WS: unexpected path '{}', expected '{}'", path, ws_path);
                // Still accept — the port is dedicated to MQTT WS.
            }
            // Echo back the "mqtt" sub-protocol if the client advertised it.
            if let Some(proto) = req.headers().get("Sec-WebSocket-Protocol") {
                let proto_str = proto.to_str().unwrap_or("");
                if proto_str.split(',').any(|p| p.trim() == "mqtt" || p.trim().starts_with("mqtt")) {
                    if let Ok(val) = "mqtt".parse() {
                        resp.headers_mut().insert("Sec-WebSocket-Protocol", val);
                    }
                }
            }
            Ok::<Response, ErrorResponse>(resp)
        })
        .await
        {
            Ok(ws) => ws,
            Err(e) => {
                debug!("MQTT WS upgrade failed from {}: {}", peer, e);
                return;
            }
        };

        let adapter = WsStreamAdapter::new(ws_stream);

        handle_connection(
            adapter,
            self.pool.clone(),
            self.ts_dao.clone(),
            self.rule_engine_tx.clone(),
            self.queue_producer.clone(),
            self.cache.clone(),
            self.ws_tx.clone(),
            self.activity_tx.clone(),
            self.device_registry.clone(),
            self.rpc_pending.clone(),
            self.session_store.clone(),
            self.chunk_size_kb,
        )
        .await;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    /// Test the buffer-spill logic in isolation (no real WebSocket needed).
    #[test]
    fn test_ws_adapter_read_buf_spill_logic() {
        // Simulate the state: we have leftover bytes from a previous frame.
        let read_buf: Vec<u8> = b"hello world".to_vec();
        let mut read_pos: usize = 0;

        let mut out = [0u8; 5];
        // Simulate the AsyncRead fill logic for the "spill" path.
        let remaining = &read_buf[read_pos..];
        let n = remaining.len().min(out.len());
        out[..n].copy_from_slice(&remaining[..n]);
        read_pos += n;

        assert_eq!(&out[..n], b"hello");
        assert_eq!(read_pos, 5);

        // Simulate second read — gets the rest.
        let mut out2 = [0u8; 10];
        let remaining2 = &read_buf[read_pos..];
        let n2 = remaining2.len().min(out2.len());
        out2[..n2].copy_from_slice(&remaining2[..n2]);
        assert_eq!(&out2[..n2], b" world");
    }

    /// Verify Sec-WebSocket-Protocol header negotiation logic.
    #[test]
    fn test_mqtt_subprotocol_detection() {
        let proto_str = "mqtt, mqttv3.1";
        let found = proto_str.split(',').any(|p| {
            let t = p.trim();
            t == "mqtt" || t.starts_with("mqtt")
        });
        assert!(found);
    }
}
