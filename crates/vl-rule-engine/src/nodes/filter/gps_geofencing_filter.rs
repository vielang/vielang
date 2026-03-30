use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Filter messages based on GPS coordinates inside/outside a geofence.
/// True if inside geofence, False otherwise.
/// Java: TbGpsGeofencingFilterNode
///
/// Inline mode (config contains perimeter definition):
/// ```json
/// {
///   "latitudeKey": "latitude",
///   "longitudeKey": "longitude",
///   "perimeterType": "CIRCLE",
///   "centerLatitude": 37.4219,
///   "centerLongitude": -122.0840,
///   "range": 1000,
///   "rangeUnit": "METER"
/// }
/// ```
///
/// DB-backed mode (config contains geofence UUIDs stored in tb_geofence):
/// ```json
/// {
///   "latitudeKey": "latitude",
///   "longitudeKey": "longitude",
///   "geofenceIds": ["uuid1", "uuid2"]
/// }
/// ```
/// In DB-backed mode the node returns True if the point falls inside ANY of the
/// listed geofences (PostGIS ST_Contains / ST_DWithin logic in GeofenceDao).
pub struct GpsGeofencingFilterNode {
    lat_key: String,
    lon_key: String,
    mode: FilterMode,
}

enum FilterMode {
    /// Inline geometry — no DB round-trip
    Inline(Perimeter),
    /// PostGIS-backed — query GeofenceDao at eval time
    DbBacked { geofence_ids: Vec<Uuid> },
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

    // DB-backed mode
    #[serde(rename = "geofenceIds", default)]
    geofence_ids: Vec<Uuid>,

    // Inline mode
    #[serde(rename = "perimeterType", default = "default_perimeter_type")]
    perimeter_type: String,
    // Circle
    #[serde(rename = "centerLatitude", default)]
    center_latitude: f64,
    #[serde(rename = "centerLongitude", default)]
    center_longitude: f64,
    #[serde(default)]
    range: f64,
    #[serde(rename = "rangeUnit", default = "default_range_unit")]
    range_unit: String,
    // Polygon
    #[serde(rename = "polygonDefinition", default)]
    polygon_definition: String,
}

fn default_lat_key() -> String { "latitude".into() }
fn default_lon_key() -> String { "longitude".into() }
fn default_perimeter_type() -> String { "CIRCLE".into() }
fn default_range_unit() -> String { "METER".into() }

impl GpsGeofencingFilterNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("GpsGeofencingFilterNode: {}", e)))?;

        let mode = if !cfg.geofence_ids.is_empty() {
            FilterMode::DbBacked { geofence_ids: cfg.geofence_ids }
        } else if cfg.perimeter_type == "POLYGON" {
            let points: Vec<(f64, f64)> = serde_json::from_str(&cfg.polygon_definition)
                .unwrap_or_default();
            FilterMode::Inline(Perimeter::Polygon(points))
        } else {
            let radius_meters = to_meters(cfg.range, &cfg.range_unit);
            FilterMode::Inline(Perimeter::Circle {
                center_lat: cfg.center_latitude,
                center_lon: cfg.center_longitude,
                radius_meters,
            })
        };

        Ok(Self {
            lat_key: cfg.latitude_key,
            lon_key: cfg.longitude_key,
            mode,
        })
    }
}

#[async_trait]
impl RuleNode for GpsGeofencingFilterNode {
    async fn process(
        &self,
        ctx: &RuleNodeCtx,
        msg: TbMsg,
    ) -> Result<Vec<(RelationType, TbMsg)>, RuleEngineError> {
        let lat = parse_coord(&msg, &self.lat_key);
        let lon = parse_coord(&msg, &self.lon_key);

        let (lat, lon) = match (lat, lon) {
            (Some(la), Some(lo)) => (la, lo),
            _ => return Ok(vec![(RelationType::False, msg)]),
        };

        let inside = match &self.mode {
            FilterMode::Inline(Perimeter::Circle { center_lat, center_lon, radius_meters }) => {
                haversine_meters(*center_lat, *center_lon, lat, lon) <= *radius_meters
            }
            FilterMode::Inline(Perimeter::Polygon(points)) => {
                point_in_polygon(lat, lon, points)
            }
            FilterMode::DbBacked { geofence_ids } => {
                ctx.dao
                    .geofence
                    .find_containing_by_ids(geofence_ids, lat, lon)
                    .await
                    .map(|hits| !hits.is_empty())
                    .unwrap_or(false)
            }
        };

        let rel = if inside { RelationType::True } else { RelationType::False };
        Ok(vec![(rel, msg)])
    }
}

fn parse_coord(msg: &TbMsg, key: &str) -> Option<f64> {
    // Try metadata first, then data JSON
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
        _ => range, // METER
    }
}

/// Haversine distance in meters
fn haversine_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().atan2((1.0 - a).sqrt())
}

/// Ray casting algorithm: point-in-polygon
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
