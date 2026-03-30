use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::{AttributeKvEntry, AttributeScope, TbMsg};
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Read polygon geofence boundary coordinates from the message body and persist
/// them as a server-side attribute on the originator entity.
///
/// Downstream `CheckGeofenceNode` / `GpsGeofencingFilterNode` will read the
/// stored perimeter attribute to evaluate whether a device is inside the fence.
///
/// Java: TbGeofenceUpdateNode (ThingsBoard PE)
/// Relations:
///   - Success — coordinates extracted and saved
///   - Failure — coordinates missing / malformed in message
/// Config:
/// ```json
/// {
///   "perimeterKey":  "perimeter",        // attribute key to write (default: "perimeter")
///   "coordinatesKey": "coordinates",     // message body key with [[lat,lon], ...] array
///   "label": ""                          // optional human-readable label stored alongside
/// }
/// ```
pub struct GeofenceUpdateNode {
    perimeter_key:   String,
    coordinates_key: String,
    label:           String,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "perimeterKey", default = "default_perimeter")]
    perimeter_key: String,
    #[serde(rename = "coordinatesKey", default = "default_coordinates")]
    coordinates_key: String,
    #[serde(rename = "label", default)]
    label: String,
}

fn default_perimeter()   -> String { "perimeter".into() }
fn default_coordinates() -> String { "coordinates".into() }

impl GeofenceUpdateNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GeofenceUpdateNode: {}", e)))?;
        Ok(Self {
            perimeter_key:   cfg.perimeter_key,
            coordinates_key: cfg.coordinates_key,
            label:           cfg.label,
        })
    }
}

#[async_trait]
impl RuleNode for GeofenceUpdateNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        // Parse message body
        let data: serde_json::Value = match serde_json::from_str(&msg.data) {
            Ok(v) => v,
            Err(_) => {
                let mut m = msg;
                m.metadata.insert("error".into(),
                    "GeofenceUpdateNode: message data is not valid JSON".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        // Extract coordinates array
        let coords = match data.get(&self.coordinates_key) {
            Some(v) if v.is_array() => v.clone(),
            _ => {
                let mut m = msg;
                m.metadata.insert("error".into(),
                    format!("GeofenceUpdateNode: '{}' not found or not an array",
                        self.coordinates_key));
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        // Validate: must be array of [lat, lon] with at least 3 points
        let points = coords.as_array().unwrap();
        if points.len() < 3 {
            let mut m = msg;
            m.metadata.insert("error".into(),
                "GeofenceUpdateNode: perimeter must have at least 3 coordinate points".into());
            return Ok(vec![(RelationType::Failure, m)]);
        }

        // Build the perimeter JSON to store as attribute
        let perimeter = if self.label.is_empty() {
            serde_json::json!({ "coordinates": coords })
        } else {
            serde_json::json!({ "label": self.label, "coordinates": coords })
        };

        // Resolve or create the key_id for the perimeter attribute
        let key_id = ctx.dao.kv.get_or_create_key(&self.perimeter_key).await?;

        // Save as server-side attribute on the originator
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        ctx.dao.kv.save_attribute(&AttributeKvEntry {
            entity_id:      msg.originator_id,
            attribute_type: AttributeScope::ServerScope,
            attribute_key:  key_id,
            last_update_ts: now_ms,
            bool_v:         None,
            str_v:          None,
            long_v:         None,
            dbl_v:          None,
            json_v:         Some(perimeter),
            version:        0,
        }).await?;

        Ok(vec![(RelationType::Success, msg)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_config() {
        let node = GeofenceUpdateNode::new(&json!({})).unwrap();
        assert_eq!(node.perimeter_key, "perimeter");
        assert_eq!(node.coordinates_key, "coordinates");
    }

    #[test]
    fn custom_keys() {
        let node = GeofenceUpdateNode::new(&json!({
            "perimeterKey": "fence",
            "coordinatesKey": "polygon",
            "label": "Office Zone"
        })).unwrap();
        assert_eq!(node.perimeter_key, "fence");
        assert_eq!(node.coordinates_key, "polygon");
        assert_eq!(node.label, "Office Zone");
    }
}
