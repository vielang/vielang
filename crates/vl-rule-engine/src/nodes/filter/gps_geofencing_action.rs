use async_trait::async_trait;
use std::sync::Arc;
use serde::Deserialize;
use dashmap::DashMap;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Track geofence state transitions and emit events on enter/leave.
/// Outputs: "Entered" when entering geofence, "Left" when leaving, "Inside"/"Outside" otherwise.
/// Java: TbGpsGeofencingActionNode
/// Config: same as GpsGeofencingFilterNode
pub struct GpsGeofencingActionNode {
    lat_key: String,
    lon_key: String,
    perimeter: Perimeter,
    /// Track per-originator: was_inside
    state: Arc<DashMap<uuid::Uuid, bool>>,
}

enum Perimeter {
    Circle { center_lat: f64, center_lon: f64, radius_meters: f64 },
    Polygon(Vec<(f64, f64)>),
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "latitudeKey", default = "default_lat_key")]
    latitude_key: String,
    #[serde(rename = "longitudeKey", default = "default_lon_key")]
    longitude_key: String,
    #[serde(rename = "perimeterType", default = "default_perimeter_type")]
    perimeter_type: String,
    #[serde(rename = "centerLatitude", default)]
    center_latitude: f64,
    #[serde(rename = "centerLongitude", default)]
    center_longitude: f64,
    #[serde(default)]
    range: f64,
    #[serde(rename = "rangeUnit", default = "default_range_unit")]
    range_unit: String,
    #[serde(rename = "polygonDefinition", default)]
    polygon_definition: String,
}

fn default_lat_key() -> String { "latitude".into() }
fn default_lon_key() -> String { "longitude".into() }
fn default_perimeter_type() -> String { "CIRCLE".into() }
fn default_range_unit() -> String { "METER".into() }

impl GpsGeofencingActionNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GpsGeofencingActionNode: {}", e)))?;

        let perimeter = if cfg.perimeter_type == "POLYGON" {
            let points: Vec<(f64, f64)> = serde_json::from_str(&cfg.polygon_definition)
                .unwrap_or_default();
            Perimeter::Polygon(points)
        } else {
            let radius_meters = to_meters(cfg.range, &cfg.range_unit);
            Perimeter::Circle {
                center_lat: cfg.center_latitude,
                center_lon: cfg.center_longitude,
                radius_meters,
            }
        };

        Ok(Self {
            lat_key: cfg.latitude_key,
            lon_key: cfg.longitude_key,
            perimeter,
            state: Arc::new(DashMap::new()),
        })
    }

    fn is_inside(&self, lat: f64, lon: f64) -> bool {
        match &self.perimeter {
            Perimeter::Circle { center_lat, center_lon, radius_meters } => {
                haversine_meters(*center_lat, *center_lon, lat, lon) <= *radius_meters
            }
            Perimeter::Polygon(points) => point_in_polygon(lat, lon, points),
        }
    }
}

#[async_trait]
impl RuleNode for GpsGeofencingActionNode {
    async fn process(
        &self,
        _ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let lat = parse_coord(&msg, &self.lat_key);
        let lon = parse_coord(&msg, &self.lon_key);

        let (lat, lon) = match (lat, lon) {
            (Some(la), Some(lo)) => (la, lo),
            _ => return Ok(vec![(RelationType::Other("Outside".into()), msg)]),
        };

        let now_inside = self.is_inside(lat, lon);
        let was_inside = self.state.get(&msg.originator_id).map(|v| *v);
        self.state.insert(msg.originator_id, now_inside);

        let rel = match (was_inside, now_inside) {
            (None, true) | (Some(false), true)  => RelationType::Other("Entered".into()),
            (None, false) | (Some(false), false) => RelationType::Other("Outside".into()),
            (Some(true), false)                  => RelationType::Other("Left".into()),
            (Some(true), true)                   => RelationType::Other("Inside".into()),
        };

        Ok(vec![(rel, msg)])
    }
}

fn parse_coord(msg: &TbMsg, key: &str) -> Option<f64> {
    if let Some(v) = msg.metadata.get(key) {
        return v.parse().ok();
    }
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.data) {
        if let Some(v) = data.get(key) {
            return v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()));
        }
    }
    None
}

fn to_meters(range: f64, unit: &str) -> f64 {
    match unit {
        "KILOMETER" | "KM" => range * 1000.0,
        "MILE" => range * 1609.344,
        "FOOT" => range * 0.3048,
        _ => range,
    }
}

fn haversine_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().atan2((1.0 - a).sqrt())
}

fn point_in_polygon(lat: f64, lon: f64, polygon: &[(f64, f64)]) -> bool {
    let n = polygon.len();
    if n < 3 { return false; }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > lon) != (yj > lon)) && (lat < (xj - xi) * (lon - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}
