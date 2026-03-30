//! RPC systems — handle outgoing commands and incoming responses.

use bevy::prelude::*;

use crate::{
    api::{ApiConfig, ApiClient, RpcResponseQueue},
    events::{RpcResult, SendRpcRequest},
};

/// Read SendRpcRequest events and dispatch each as an async background task.
///
/// Uses std::thread::spawn + new Tokio runtime per request — same pattern as login.
/// Each thread pushes its result into RpcResponseQueue for drain_rpc_responses to pick up.
#[cfg(not(target_arch = "wasm32"))]
pub fn handle_rpc_requests(
    mut events:   MessageReader<SendRpcRequest>,
    api_config:   Res<ApiConfig>,
    resp_queue:   Res<RpcResponseQueue>,
) {
    for req in events.read() {
        let config      = (*api_config).clone();
        let queue       = resp_queue.clone();
        let device_id   = req.device_id;
        let device_name = String::new(); // populated by drain system from ECS
        let method      = req.method.clone();
        let params      = req.params.clone();
        let is_twoway   = req.is_twoway;
        let sent_at     = req.sent_at;

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            let method_clone = method.clone();
            let result = rt.block_on(async move {
                let client = ApiClient::new(config);
                if is_twoway {
                    client.send_rpc_twoway(device_id, &method_clone, params).await
                } else {
                    client.send_rpc_oneway(device_id, &method_clone, params)
                        .await
                        .map(|_| serde_json::Value::Null)
                }
            });
            queue.push(RpcResult { device_id, device_name, method, result, sent_at });
        });
    }
}

/// WASM: RPC via gloo-net HTTP POST (spawn_local + push to resp_queue).
#[cfg(target_arch = "wasm32")]
pub fn handle_rpc_requests(
    mut events:  MessageReader<SendRpcRequest>,
    api_config:  Res<ApiConfig>,
    resp_queue:  Res<RpcResponseQueue>,
) {
    for req in events.read() {
        let base_url  = api_config.base_url.clone();
        let jwt_token = api_config.jwt_token.clone();
        let device_id = req.device_id;
        let method    = req.method.clone();
        let params    = req.params.clone();
        let is_twoway = req.is_twoway;
        let sent_at   = req.sent_at;
        let queue     = resp_queue.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let path = if is_twoway {
                format!("{base_url}/api/rpc/twoway/{device_id}")
            } else {
                format!("{base_url}/api/rpc/oneway/{device_id}")
            };

            let body = serde_json::json!({ "method": method, "params": params });

            let result = gloo_net::http::Request::post(&path)
                .header("Authorization", &format!("Bearer {jwt_token}"))
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap_or_else(|_| unreachable!())
                .send()
                .await
                .map_err(|e| format!("RPC send error: {e:?}"))
                .and_then(|resp| {
                    let status = resp.status();
                    if status == 200 || status == 202 {
                        Ok(serde_json::Value::Null)
                    } else {
                        Err(format!("RPC HTTP {status}"))
                    }
                });

            queue.push(RpcResult { device_id, device_name: String::new(), method, result, sent_at });
        });
    }
}

/// Drain RpcResponseQueue into Bevy RpcResult events each frame.
pub fn drain_rpc_responses(
    resp_queue:  Res<RpcResponseQueue>,
    mut writer:  MessageWriter<RpcResult>,
) {
    for resp in resp_queue.drain() {
        tracing::debug!(
            device = %resp.device_id,
            method = %resp.method,
            ok     = resp.result.is_ok(),
            "RPC result received"
        );
        writer.write(resp);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_response_queue_push_drain() {
        let queue = RpcResponseQueue::default();
        queue.push(RpcResult {
            device_id:   uuid::Uuid::nil(),
            device_name: "Test".into(),
            method:      "ping".into(),
            result:      Ok(serde_json::Value::Null),
            sent_at:     0,
        });
        let drained = queue.drain();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].method, "ping");
        assert!(queue.drain().is_empty());
    }
}
