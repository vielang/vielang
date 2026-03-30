use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Calculate the Haversine distance between device location (from metadata)
/// and a fixed reference point (or another device's location).
/// Java: TbCalculateDistanceNode
/// Relations: Success, Failure (lat/lon missing)
/// Config:
/// ```json
/// {
///   "originLatitude": 51.507,
///   "originLongitude": -0.127,
///   "outputKey": "distanceKm",
///   "unit": "KM",              // KM | MILES | METERS | FEET
///   "round": 3,
///   "addOriginCoordinates": false
/// }
/// ```
/// Reads device lat/lon from metadata keys:
///   `perimeter_latitude`, `perimeter_longitude`
/// (populated by GetGeoDataNode upstream)
pub struct CalculateDistanceNode {
    origin_lat:          f64,
    origin_lon:          f64,
    output_key:          String,
    unit:                DistanceUnit,
    precision:           Option<u32>,
    add_origin_coords:   bool,
}

#[derive(Debug, Clone, Copy)]
enum DistanceUnit { Km, Miles, Meters, Feet }

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "originLatitude")]
    origin_latitude: f64,
    #[serde(rename = "originLongitude")]
    origin_longitude: f64,
    #[serde(rename = "outputKey", default = "default_output")]
    output_key: String,
    #[serde(rename = "unit", default = "default_unit")]
    unit: String,
    #[serde(rename = "round")]
    round: Option<u32>,
    #[serde(rename = "addOriginCoordinates", default)]
    add_origin_coordinates: bool,
}

fn default_output() -> String { "distance".into() }
fn default_unit()   -> String { "KM".into() }

impl CalculateDistanceNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CalculateDistanceNode: {}", e)))?;
        let unit = match cfg.unit.to_uppercase().as_str() {
            "KM" | "KILOMETERS" => DistanceUnit::Km,
            "MILES"             => DistanceUnit::Miles,
            "METERS" | "M"      => DistanceUnit::Meters,
            "FEET" | "FT"       => DistanceUnit::Feet,
            other               => return Err(RuleEngineError::Config(
                format!("CalculateDistanceNode: unknown unit '{}'", other))),
        };
        Ok(Self {
            origin_lat: cfg.origin_latitude,
            origin_lon: cfg.origin_longitude,
            output_key: cfg.output_key,
            unit,
            precision: cfg.round,
            add_origin_coords: cfg.add_origin_coordinates,
        })
    }

    /// Haversine formula — returns distance in kilometers
    fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const R: f64 = 6371.0; // Earth radius in km
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        R * c
    }

    fn convert(km: f64, unit: DistanceUnit) -> f64 {
        match unit {
            DistanceUnit::Km     => km,
            DistanceUnit::Miles  => km * 0.621_371,
            DistanceUnit::Meters => km * 1000.0,
            DistanceUnit::Feet   => km * 3280.84,
        }
    }

    fn round(val: f64, precision: u32) -> f64 {
        let factor = 10f64.powi(precision as i32);
        (val * factor).round() / factor
    }
}

#[async_trait]
impl RuleNode for CalculateDistanceNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let lat = match msg.metadata.get("perimeter_latitude")
            .and_then(|v| v.parse::<f64>().ok())
        {
            Some(v) => v,
            None => {
                let mut m = msg;
                m.metadata.insert("error".into(),
                    "CalculateDistanceNode: 'perimeter_latitude' missing from metadata".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };
        let lon = match msg.metadata.get("perimeter_longitude")
            .and_then(|v| v.parse::<f64>().ok())
        {
            Some(v) => v,
            None => {
                let mut m = msg;
                m.metadata.insert("error".into(),
                    "CalculateDistanceNode: 'perimeter_longitude' missing from metadata".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        let dist_km = Self::haversine_km(lat, lon, self.origin_lat, self.origin_lon);
        let mut dist = Self::convert(dist_km, self.unit);
        if let Some(p) = self.precision {
            dist = Self::round(dist, p);
        }

        let mut out = msg;
        if let Ok(mut obj) = serde_json::from_str::<serde_json::Value>(&out.data) {
            obj[&self.output_key] = serde_json::json!(dist);
            if self.add_origin_coords {
                obj["originLatitude"]  = serde_json::json!(self.origin_lat);
                obj["originLongitude"] = serde_json::json!(self.origin_lon);
            }
            out.data = serde_json::to_string(&obj).unwrap_or(out.data);
        }

        Ok(vec![(RelationType::Success, out)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_config() {
        let node = CalculateDistanceNode::new(&json!({
            "originLatitude": 51.507,
            "originLongitude": -0.127,
            "unit": "KM",
            "round": 3
        })).unwrap();
        assert_eq!(node.origin_lat, 51.507);
        assert_eq!(node.precision, Some(3));
    }

    #[test]
    fn unknown_unit_is_error() {
        assert!(CalculateDistanceNode::new(&json!({
            "originLatitude": 0.0,
            "originLongitude": 0.0,
            "unit": "LIGHTYEARS"
        })).is_err());
    }

    #[test]
    fn london_to_paris_approx_340km() {
        // London: 51.5074, -0.1278 → Paris: 48.8566, 2.3522
        let dist = CalculateDistanceNode::haversine_km(51.5074, -0.1278, 48.8566, 2.3522);
        assert!((dist - 340.0).abs() < 10.0, "Expected ~340km, got {:.1}", dist);
    }
}
