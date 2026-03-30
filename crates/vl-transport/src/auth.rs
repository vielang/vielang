use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use vl_cache::{TbCache, get_cached, keys, put_cached};
use vl_core::entities::DeviceCredentialsType;
use vl_dao::postgres::device::DeviceDao;

/// Thông tin device đã xác thực — dùng trong suốt vòng đời kết nối
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthDevice {
    pub device_id: Uuid,
    pub tenant_id: Uuid,
}

/// TTL cho credentials cache (15 phút)
pub const CREDENTIALS_TTL: Duration = Duration::from_secs(900);

/// Xác thực device bằng Access Token (dùng cho HTTP và CoAP transport).
///
/// 1. Cache lookup (hot path)
/// 2. Cache miss → DB lookup
/// 3. Cache kết quả 15 phút
pub async fn authenticate_by_token(
    token: &str,
    device_dao: &DeviceDao,
    cache: &Arc<dyn TbCache>,
) -> Option<AuthDevice> {
    if token.is_empty() {
        return None;
    }

    // ── 1. Cache lookup ───────────────────────────────────────────────────────
    let cache_key = keys::credentials_key(token);
    if let Ok(Some(auth)) = get_cached::<AuthDevice>(cache.as_ref(), &cache_key).await {
        return Some(auth);
    }

    // ── 2. DB lookup ──────────────────────────────────────────────────────────
    match device_dao.find_by_credentials_id(token).await {
        Ok(Some((device, creds))) => {
            if creds.credentials_type != DeviceCredentialsType::AccessToken {
                return None;
            }
            let auth = AuthDevice {
                device_id: device.id,
                tenant_id: device.tenant_id,
            };
            // ── 3. Cache for next requests ────────────────────────────────────
            let _ = put_cached(cache.as_ref(), &cache_key, &auth, Some(CREDENTIALS_TTL)).await;
            Some(auth)
        }
        Ok(None) => None,
        Err(e) => {
            error!("DB error during token auth: {}", e);
            None
        }
    }
}
