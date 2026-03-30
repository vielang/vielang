use std::sync::Arc;

use vl_cache::TbCache;
use vl_dao::postgres::device::DeviceDao;

pub use crate::auth::AuthDevice;
use crate::auth::authenticate_by_token;

/// Xác thực MQTT device từ CONNECT packet.
///
/// ThingsBoard hỗ trợ 3 loại auth:
/// - Access Token: username = access_token, password = ignored → cache result
/// - MQTT Basic:   credentials_id = username, validate password từ credentials_value JSON
/// - X.509 Cert:   DER-encoded certificate → extract CN → lookup device credentials
pub async fn authenticate(
    device_dao: &DeviceDao,
    cache: &Arc<dyn TbCache>,
    username: Option<&str>,
    password: Option<&str>,
) -> Option<AuthDevice> {
    let username = username?;
    if username.is_empty() {
        return None;
    }

    // Try Access Token path via shared auth (cache-aware)
    if let Some(auth) = authenticate_by_token(username, device_dao, cache).await {
        return Some(auth);
    }

    // Fallback: MQTT Basic auth (password validation required, not cached)
    use vl_core::entities::DeviceCredentialsType;
    use tracing::error;
    match device_dao.find_by_credentials_id(username).await {
        Ok(Some((device, creds))) => {
            if creds.credentials_type == DeviceCredentialsType::MqttBasic {
                let pw = password.unwrap_or("");
                if validate_mqtt_basic_password(&creds.credentials_value, pw) {
                    return Some(AuthDevice {
                        device_id: device.id,
                        tenant_id: device.tenant_id,
                    });
                }
            }
            None
        }
        Ok(None) => None,
        Err(e) => {
            error!("DB error during MQTT Basic auth: {}", e);
            None
        }
    }
}

/// Authenticate an MQTT device using a DER-encoded X.509 client certificate.
///
/// The certificate's Subject Common Name (CN) is used as the `credentials_id`
/// to look up the device — matching ThingsBoard Java's x.509 auth flow.
pub async fn authenticate_by_certificate(
    device_dao: &DeviceDao,
    cert_der: &[u8],
) -> Option<AuthDevice> {
    use x509_cert::Certificate;
    use x509_cert::der::Decode;
    use tracing::{debug, error};

    let cert = match Certificate::from_der(cert_der) {
        Ok(c) => c,
        Err(e) => {
            debug!("Failed to parse DER certificate: {}", e);
            return None;
        }
    };

    // Extract CN from Subject
    let cn = extract_common_name(&cert)?;
    debug!("MQTT x.509 auth: CN = {}", cn);

    match device_dao.find_by_credentials_id(&cn).await {
        Ok(Some((device, creds))) => {
            use vl_core::entities::DeviceCredentialsType;
            if creds.credentials_type == DeviceCredentialsType::X509Certificate {
                Some(AuthDevice {
                    device_id: device.id,
                    tenant_id: device.tenant_id,
                })
            } else {
                None
            }
        }
        Ok(None) => None,
        Err(e) => {
            error!("DB error during MQTT x.509 auth: {}", e);
            None
        }
    }
}

/// Extract the Common Name (CN) from an X.509 certificate's Subject.
/// Tries UTF-8 string, then PrintableString, then IA5String encoding.
fn extract_common_name(cert: &x509_cert::Certificate) -> Option<String> {
    use x509_cert::der::asn1::{Ia5StringRef, PrintableStringRef, Utf8StringRef};

    let subject = &cert.tbs_certificate.subject;
    for rdn in subject.0.iter() {
        for atv in rdn.0.iter() {
            // OID 2.5.4.3 = id-at-commonName
            if atv.oid.to_string() == "2.5.4.3" {
                if let Ok(cn) = atv.value.decode_as::<Utf8StringRef<'_>>() {
                    return Some(cn.as_str().to_owned());
                }
                if let Ok(cn) = atv.value.decode_as::<PrintableStringRef<'_>>() {
                    return Some(cn.as_str().to_owned());
                }
                if let Ok(cn) = atv.value.decode_as::<Ia5StringRef<'_>>() {
                    return Some(cn.as_str().to_owned());
                }
            }
        }
    }
    None
}

/// Validate MQTT Basic password khớp với JSON credentials_value.
/// credentials_value format: {"clientId":"...", "userName":"...", "password":"..."}
fn validate_mqtt_basic_password(credentials_value: &Option<String>, password: &str) -> bool {
    let Some(json_str) = credentials_value else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return false;
    };
    value
        .get("password")
        .and_then(|p| p.as_str())
        .map(|p| p == password)
        .unwrap_or(false)
}
