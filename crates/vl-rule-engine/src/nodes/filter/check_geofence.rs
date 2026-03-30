use async_trait::async_trait;
use serde::Deserialize;
use vl_core::entities::TbMsg;
use crate::{error::RuleEngineError, node::{RelationType, RuleNode, RuleNodeCtx}};

/// Check if device location (from metadata) is inside a configured polygon geofence.
/// Expects `perimeter_latitude` and `perimeter_longitude` in message metadata
/// (populated by GetGeoDataNode upstream).
/// Java: TbCheckGeofenceNode
/// Relations:
///   - True  — device is inside the polygon
///   - False — device is outside the polygon
///   - Failure — lat/lon missing from metadata or geofence config invalid
/// Config:
/// ```json
/// {
///   "polygons": [
///     { "label": "HQ Zone", "coordinates": [[lat1,lon1],[lat2,lon2],[lat3,lon3]] }
///   ],
///   "minInsidePolygons": 1
/// }
/// ```
pub struct CheckGeofenceNode {
    polygons:            Vec<Polygon>,
    min_inside_polygons: usize,
}

#[derive(Clone)]
struct Polygon {
    label:       String,
    coordinates: Vec<[f64; 2]>, // [lat, lon]
}

#[derive(Deserialize)]
struct RawPolygon {
    #[serde(default)]
    label: String,
    coordinates: Vec<Vec<f64>>,
}

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    polygons: Vec<RawPolygon>,
    #[serde(rename = "minInsidePolygons", default = "default_min")]
    min_inside_polygons: usize,
}

fn default_min() -> usize { 1 }

impl CheckGeofenceNode {
    pub fn new(config: &serde_json::Value) -> Result<Self, RuleEngineError> {
        let cfg: Config = serde_json::from_value(config.clone())
            .map_err(|e| RuleEngineError::Config(format!("CheckGeofenceNode: {}", e)))?;
        if cfg.polygons.is_empty() {
            return Err(RuleEngineError::Config(
                "CheckGeofenceNode: 'polygons' must not be empty".into()));
        }
        let polygons: Result<Vec<Polygon>, _> = cfg.polygons.into_iter().map(|p| {
            if p.coordinates.len() < 3 {
                return Err(RuleEngineError::Config(
                    "CheckGeofenceNode: polygon needs at least 3 coordinates".into()));
            }
            let coords: Result<Vec<[f64; 2]>, _> = p.coordinates.into_iter().map(|c| {
                if c.len() < 2 {
                    return Err(RuleEngineError::Config(
                        "CheckGeofenceNode: each coordinate needs [lat, lon]".into()));
                }
                Ok([c[0], c[1]])
            }).collect();
            Ok(Polygon { label: p.label, coordinates: coords? })
        }).collect();
        Ok(Self {
            polygons: polygons?,
            min_inside_polygons: cfg.min_inside_polygons,
        })
    }

    /// Ray-casting algorithm: point-in-polygon test.
    /// `point` = [lat, lon], polygon coordinates = [[lat, lon], ...]
    fn point_in_polygon(point: [f64; 2], polygon: &[[f64; 2]]) -> bool {
        let (py, px) = (point[0], point[1]);
        let n = polygon.len();
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let (iy, ix) = (polygon[i][0], polygon[i][1]);
            let (jy, jx) = (polygon[j][0], polygon[j][1]);
            if ((iy > py) != (jy > py)) && (px < (jx - ix) * (py - iy) / (jy - iy) + ix) {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
}

#[async_trait]
impl RuleNode for CheckGeofenceNode {
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
                    "CheckGeofenceNode: 'perimeter_latitude' missing from metadata".into());
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
                    "CheckGeofenceNode: 'perimeter_longitude' missing from metadata".into());
                return Ok(vec![(RelationType::Failure, m)]);
            }
        };

        let point = [lat, lon];
        let inside_count = self.polygons.iter()
            .filter(|p| Self::point_in_polygon(point, &p.coordinates))
            .count();

        let mut out = msg;
        // Store matched polygon labels in metadata
        let matched_labels: Vec<&str> = self.polygons.iter()
            .filter(|p| Self::point_in_polygon(point, &p.coordinates))
            .map(|p| p.label.as_str())
            .collect();
        out.metadata.insert("geofenceLabels".into(), matched_labels.join(","));

        if inside_count >= self.min_inside_polygons {
            Ok(vec![(RelationType::True, out)])
        } else {
            Ok(vec![(RelationType::False, out)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn square_polygon() -> serde_json::Value {
        json!({
            "polygons": [{
                "label": "test_square",
                "coordinates": [
                    [0.0, 0.0],
                    [0.0, 10.0],
                    [10.0, 10.0],
                    [10.0, 0.0]
                ]
            }]
        })
    }

    #[test]
    fn parses_polygon_config() {
        let node = CheckGeofenceNode::new(&square_polygon()).unwrap();
        assert_eq!(node.polygons.len(), 1);
        assert_eq!(node.polygons[0].coordinates.len(), 4);
    }

    #[test]
    fn empty_polygons_is_error() {
        assert!(CheckGeofenceNode::new(&json!({ "polygons": [] })).is_err());
    }

    #[test]
    fn point_inside_square() {
        let coords = vec![[0.0, 0.0], [0.0, 10.0], [10.0, 10.0], [10.0, 0.0]];
        assert!(CheckGeofenceNode::point_in_polygon([5.0, 5.0], &coords));
    }

    #[test]
    fn point_outside_square() {
        let coords = vec![[0.0, 0.0], [0.0, 10.0], [10.0, 10.0], [10.0, 0.0]];
        assert!(!CheckGeofenceNode::point_in_polygon([15.0, 15.0], &coords));
    }
}
