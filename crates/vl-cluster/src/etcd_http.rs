/// Minimal etcd v3 client dùng HTTP/JSON gateway.
///
/// etcd v3 expose gRPC-gateway trên port 2379 — mọi RPC đều có thể gọi
/// bằng POST JSON, key/value là base64-encoded bytes.
///
/// Tài liệu: https://etcd.io/docs/v3.5/dev-guide/api_grpc_gateway/
use std::time::Duration;

use serde_json::json;

use crate::error::ClusterError;

#[derive(Clone)]
pub struct EtcdClient {
    base_url: String,
    http:     reqwest::Client,
}

impl EtcdClient {
    pub fn new(etcd_url: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build reqwest client");
        Self { base_url: etcd_url.trim_end_matches('/').to_string(), http }
    }

    // ── KV ────────────────────────────────────────────────────────────────────

    /// PUT key/value with optional lease.
    pub async fn put(&self, key: &str, value: &[u8], lease_id: Option<i64>) -> Result<(), ClusterError> {
        let mut body = json!({
            "key":   b64(key.as_bytes()),
            "value": b64(value),
        });
        if let Some(lid) = lease_id {
            body["lease"] = json!(lid.to_string()); // etcd gateway uses string for i64
        }
        let resp = self.http
            .post(format!("{}/v3/kv/put", self.base_url))
            .json(&body)
            .send()
            .await?;
        check_status(resp, "kv/put").await
    }

    /// DELETE key.
    pub async fn delete(&self, key: &str) -> Result<(), ClusterError> {
        let body = json!({ "key": b64(key.as_bytes()) });
        let resp = self.http
            .post(format!("{}/v3/kv/deleterange", self.base_url))
            .json(&body)
            .send()
            .await?;
        check_status(resp, "kv/deleterange").await
    }

    /// GET all keys with prefix, returns Vec<(key_str, value_bytes)>.
    pub async fn get_prefix(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>, ClusterError> {
        // Range end for prefix = prefix with last byte incremented
        let range_end = prefix_range_end(prefix.as_bytes());
        let body = json!({
            "key":       b64(prefix.as_bytes()),
            "range_end": b64(&range_end),
        });
        let resp = self.http
            .post(format!("{}/v3/kv/range", self.base_url))
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        let parsed: serde_json::Value = serde_json::from_str(&text)?;
        let mut result = Vec::new();
        if let Some(kvs) = parsed["kvs"].as_array() {
            for kv in kvs {
                let k = decode_b64_str(kv["key"].as_str().unwrap_or(""))?;
                let v = decode_b64_bytes(kv["value"].as_str().unwrap_or(""))?;
                result.push((k, v));
            }
        }
        Ok(result)
    }

    // ── Lease ─────────────────────────────────────────────────────────────────

    /// Create a lease with TTL, returns lease_id.
    pub async fn lease_grant(&self, ttl_secs: i64) -> Result<i64, ClusterError> {
        let body = json!({ "TTL": ttl_secs.to_string(), "ID": "0" });
        let resp = self.http
            .post(format!("{}/v3/lease/grant", self.base_url))
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        let parsed: serde_json::Value = serde_json::from_str(&text)?;
        let id_str = parsed["ID"].as_str().unwrap_or("0");
        id_str.parse::<i64>().map_err(|e| ClusterError::Etcd(e.to_string()))
    }

    /// Keep-alive lease — resets TTL.
    pub async fn lease_keepalive(&self, lease_id: i64) -> Result<(), ClusterError> {
        let body = json!({ "ID": lease_id.to_string() });
        let resp = self.http
            .post(format!("{}/v3/lease/keepalive", self.base_url))
            .json(&body)
            .send()
            .await?;
        check_status(resp, "lease/keepalive").await
    }

    /// Revoke a lease (remove all keys attached to it).
    pub async fn lease_revoke(&self, lease_id: i64) -> Result<(), ClusterError> {
        let body = json!({ "ID": lease_id.to_string() });
        let resp = self.http
            .post(format!("{}/v3/lease/revoke", self.base_url))
            .json(&body)
            .send()
            .await?;
        check_status(resp, "lease/revoke").await
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// base64-encode bytes (standard)
fn b64(bytes: &[u8]) -> String {
    base64_encode(bytes)
}

fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(CHARS[(b0 >> 2) & 0x3f]);
        out.push(CHARS[((b0 << 4) | (b1 >> 4)) & 0x3f]);
        out.push(if chunk.len() > 1 { CHARS[((b1 << 2) | (b2 >> 6)) & 0x3f] } else { b'=' });
        out.push(if chunk.len() > 2 { CHARS[b2 & 0x3f] } else { b'=' });
    }
    String::from_utf8(out).unwrap()
}

fn base64_decode(input: &str) -> Result<Vec<u8>, ClusterError> {
    let input = input.trim_end_matches('=');
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let decode_char = |c: u8| -> Result<u8, ClusterError> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err(ClusterError::Etcd(format!("Invalid base64 char: {}", c))),
        }
    };
    let bytes: Vec<u8> = input.bytes().collect();
    for chunk in bytes.chunks(4) {
        let b0 = decode_char(chunk[0])?;
        let b1 = decode_char(chunk[1])?;
        out.push((b0 << 2) | (b1 >> 4));
        if chunk.len() > 2 {
            let b2 = decode_char(chunk[2])?;
            out.push((b1 << 4) | (b2 >> 2));
            if chunk.len() > 3 {
                let b3 = decode_char(chunk[3])?;
                out.push((b2 << 6) | b3);
            }
        }
    }
    Ok(out)
}

fn decode_b64_str(s: &str) -> Result<String, ClusterError> {
    let bytes = base64_decode(s)?;
    String::from_utf8(bytes).map_err(|e| ClusterError::Etcd(e.to_string()))
}

fn decode_b64_bytes(s: &str) -> Result<Vec<u8>, ClusterError> {
    base64_decode(s)
}

/// etcd prefix range end: last byte + 1
fn prefix_range_end(prefix: &[u8]) -> Vec<u8> {
    let mut end = prefix.to_vec();
    for i in (0..end.len()).rev() {
        end[i] = end[i].wrapping_add(1);
        if end[i] != 0 {
            return end;
        }
    }
    // Overflow — use "\x00" sentinel meaning "all keys"
    vec![0]
}

async fn check_status(resp: reqwest::Response, op: &str) -> Result<(), ClusterError> {
    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Err(ClusterError::Etcd(format!("{} failed: {} — {}", op, status, body)))
    }
}
