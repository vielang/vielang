use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::{AttributeScope, TbMsg};
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Load latitude/longitude from device attributes or message payload and
/// inject into message metadata for use by downstream geofencing nodes.
/// Java: TbGetGeoDataNode
/// Relations: Success, Failure (lat/lon not found and failIfAbsent=true)
/// Config:
/// ```json
/// {
///   "latitudeKeyName": "latitude",
///   "longitudeKeyName": "longitude",
///   "fetchFrom": "MSG_BODY",         // MSG_BODY | METADATA | SS_ATTRIBUTE | CS_ATTRIBUTE | SHARED_ATTRIBUTE
///   "failIfAbsent": false
/// }
/// ```
/// After processing, metadata will contain:
///   - `perimeter_latitude`  → latitude as string
///   - `perimeter_longitude` → longitude as string
pub struct GetGeoDataNode {
    lat_key:       String,
    lon_key:       String,
    fetch_from:    GeoSource,
    fail_if_absent: bool,
}

#[derive(Debug, Clone, Copy)]
enum GeoSource {
    MsgBody,
    Metadata,
    ServerAttribute,
    ClientAttribute,
    SharedAttribute,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "latitudeKeyName", default = "default_lat")]
    latitude_key_name: String,
    #[serde(rename = "longitudeKeyName", default = "default_lon")]
    longitude_key_name: String,
    #[serde(rename = "fetchFrom", default = "default_source")]
    fetch_from: String,
    #[serde(rename = "failIfAbsent", default)]
    fail_if_absent: bool,
}

fn default_lat()    -> String { "latitude".into() }
fn default_lon()    -> String { "longitude".into() }
fn default_source() -> String { "MSG_BODY".into() }

impl GetGeoDataNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GetGeoDataNode: {}", e)))?;
        let fetch_from = match cfg.fetch_from.to_uppercase().as_str() {
            "MSG_BODY"         => GeoSource::MsgBody,
            "METADATA"         => GeoSource::Metadata,
            "SS_ATTRIBUTE"     => GeoSource::ServerAttribute,
            "CS_ATTRIBUTE"     => GeoSource::ClientAttribute,
            "SHARED_ATTRIBUTE" => GeoSource::SharedAttribute,
            other              => return Err(RuleEngineError::Config(
                format!("GetGeoDataNode: unknown fetchFrom '{}'", other))),
        };
        Ok(Self {
            lat_key: cfg.latitude_key_name,
            lon_key: cfg.longitude_key_name,
            fetch_from,
            fail_if_absent: cfg.fail_if_absent,
        })
    }

    async fn fetch_from_attribute(
        ctx: &RuleNodeCtx,
        device_id: uuid::Uuid,
        key: &str,
        scope: AttributeScope,
    ) -> Result<Option<f64>, RuleEngineError> {
        let key_ids = ctx.dao.kv.lookup_key_ids(&[key.to_string()]).await?;
        if let Some(&key_id) = key_ids.get(key) {
            let attrs = ctx.dao.kv.find_attributes(device_id, scope, Some(&[key_id])).await?;
            return Ok(attrs.into_iter().next().and_then(|a| {
                a.dbl_v
                    .or_else(|| a.long_v.map(|v| v as f64))
                    .or_else(|| a.str_v.as_ref().and_then(|s| s.parse::<f64>().ok()))
            }));
        }
        Ok(None)
    }
}

#[async_trait]
impl RuleNode for GetGeoDataNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let mut out = msg;

        let (lat_opt, lon_opt) = match self.fetch_from {
            GeoSource::MsgBody => {
                let data: serde_json::Value = serde_json::from_str(&out.data)
                    .unwrap_or(serde_json::json!({}));
                let lat = data.get(&self.lat_key).and_then(|v| v.as_f64());
                let lon = data.get(&self.lon_key).and_then(|v| v.as_f64());
                (lat, lon)
            }
            GeoSource::Metadata => {
                let lat = out.metadata.get(&self.lat_key)
                    .and_then(|v| v.parse::<f64>().ok());
                let lon = out.metadata.get(&self.lon_key)
                    .and_then(|v| v.parse::<f64>().ok());
                (lat, lon)
            }
            GeoSource::ServerAttribute => {
                let lat = Self::fetch_from_attribute(
                    ctx, out.originator_id, &self.lat_key, AttributeScope::ServerScope).await?;
                let lon = Self::fetch_from_attribute(
                    ctx, out.originator_id, &self.lon_key, AttributeScope::ServerScope).await?;
                (lat, lon)
            }
            GeoSource::ClientAttribute => {
                let lat = Self::fetch_from_attribute(
                    ctx, out.originator_id, &self.lat_key, AttributeScope::ClientScope).await?;
                let lon = Self::fetch_from_attribute(
                    ctx, out.originator_id, &self.lon_key, AttributeScope::ClientScope).await?;
                (lat, lon)
            }
            GeoSource::SharedAttribute => {
                let lat = Self::fetch_from_attribute(
                    ctx, out.originator_id, &self.lat_key, AttributeScope::SharedScope).await?;
                let lon = Self::fetch_from_attribute(
                    ctx, out.originator_id, &self.lon_key, AttributeScope::SharedScope).await?;
                (lat, lon)
            }
        };

        match (lat_opt, lon_opt) {
            (Some(lat), Some(lon)) => {
                out.metadata.insert("perimeter_latitude".into(), lat.to_string());
                out.metadata.insert("perimeter_longitude".into(), lon.to_string());
                Ok(vec![(RelationType::Success, out)])
            }
            _ => {
                if self.fail_if_absent {
                    out.metadata.insert("error".into(),
                        "GetGeoDataNode: lat/lon not found".into());
                    Ok(vec![(RelationType::Failure, out)])
                } else {
                    Ok(vec![(RelationType::Success, out)])
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_config() {
        let node = GetGeoDataNode::new(&json!({})).unwrap();
        assert_eq!(node.lat_key, "latitude");
        assert_eq!(node.lon_key, "longitude");
    }

    #[test]
    fn custom_keys() {
        let node = GetGeoDataNode::new(&json!({
            "latitudeKeyName": "lat",
            "longitudeKeyName": "lng",
            "fetchFrom": "MSG_BODY"
        })).unwrap();
        assert_eq!(node.lat_key, "lat");
        assert_eq!(node.lon_key, "lng");
    }

    #[test]
    fn unknown_source_is_error() {
        assert!(GetGeoDataNode::new(&json!({ "fetchFrom": "REDIS" })).is_err());
    }
}
