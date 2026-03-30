//! Phase 40 — Spatial Annotations & Measurement.
//!
//! 3D annotations on twin models: text notes, measurement lines,
//! area markers, maintenance tags. Persisted to TOML.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Annotation types ─────────────────────────────────────────────────────────

/// A 3D position in the scene.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Position3D {
    fn from(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<Position3D> for Vec3 {
    fn from(p: Position3D) -> Self {
        Vec3::new(p.x, p.y, p.z)
    }
}

/// Types of spatial annotations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    /// Text note pinned to a 3D location.
    TextNote {
        text: String,
        font_size: f32,
    },
    /// Distance measurement between two points.
    Measurement {
        start: Position3D,
        end: Position3D,
    },
    /// Area marker (polygon outline).
    AreaMarker {
        points: Vec<Position3D>,
        label: String,
    },
    /// Maintenance tag with status.
    MaintenanceTag {
        status: MaintenanceStatus,
        assigned_to: Option<String>,
        due_date: Option<String>,
        notes: String,
    },
    /// Safety zone marker (hazard area).
    SafetyZone {
        radius: f32,
        hazard_type: HazardType,
        label: String,
    },
    /// Pin/waypoint marker.
    Pin {
        icon: PinIcon,
        label: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaintenanceStatus {
    Scheduled,
    InProgress,
    Completed,
    Overdue,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HazardType {
    HighVoltage,
    HighTemperature,
    MovingParts,
    Chemical,
    Noise,
    Radiation,
    ConfinedSpace,
    FallHazard,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PinIcon {
    Info,
    Warning,
    Error,
    Checkpoint,
    Camera,
    Tool,
}

/// A single annotation in the 3D scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: Uuid,
    /// 3D position where the annotation is anchored.
    pub position: Position3D,
    /// Type-specific data.
    pub annotation_type: AnnotationType,
    /// Annotation color (RGBA).
    pub color: [f32; 4],
    /// Visibility flag.
    pub visible: bool,
    /// Associated device ID (optional).
    pub device_id: Option<Uuid>,
    /// Author.
    pub created_by: Option<String>,
    /// Creation timestamp (ms).
    pub created_at: i64,
    /// Last modified timestamp (ms).
    pub updated_at: i64,
    /// Tags for filtering.
    pub tags: Vec<String>,
}

impl Annotation {
    pub fn text_note(position: Position3D, text: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            position,
            annotation_type: AnnotationType::TextNote {
                text: text.into(),
                font_size: 14.0,
            },
            color: [1.0, 1.0, 1.0, 1.0],
            visible: true,
            device_id: None,
            created_by: None,
            created_at: crate::components::device::current_time_ms(),
            updated_at: crate::components::device::current_time_ms(),
            tags: Vec::new(),
        }
    }

    pub fn measurement(start: Position3D, end: Position3D) -> Self {
        Self {
            id: Uuid::new_v4(),
            position: start,
            annotation_type: AnnotationType::Measurement { start, end },
            color: [0.0, 1.0, 1.0, 1.0], // cyan
            visible: true,
            device_id: None,
            created_by: None,
            created_at: crate::components::device::current_time_ms(),
            updated_at: crate::components::device::current_time_ms(),
            tags: vec!["measurement".into()],
        }
    }

    pub fn maintenance_tag(position: Position3D, status: MaintenanceStatus, notes: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            position,
            annotation_type: AnnotationType::MaintenanceTag {
                status,
                assigned_to: None,
                due_date: None,
                notes: notes.into(),
            },
            color: [1.0, 0.8, 0.0, 1.0], // amber
            visible: true,
            device_id: None,
            created_by: None,
            created_at: crate::components::device::current_time_ms(),
            updated_at: crate::components::device::current_time_ms(),
            tags: vec!["maintenance".into()],
        }
    }

    pub fn safety_zone(position: Position3D, radius: f32, hazard: HazardType, label: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            position,
            annotation_type: AnnotationType::SafetyZone {
                radius,
                hazard_type: hazard,
                label: label.into(),
            },
            color: [1.0, 0.0, 0.0, 0.5], // semi-transparent red
            visible: true,
            device_id: None,
            created_by: None,
            created_at: crate::components::device::current_time_ms(),
            updated_at: crate::components::device::current_time_ms(),
            tags: vec!["safety".into()],
        }
    }

    /// Compute the distance for Measurement annotations.
    pub fn measurement_distance(&self) -> Option<f32> {
        if let AnnotationType::Measurement { start, end } = &self.annotation_type {
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let dz = end.z - start.z;
            Some((dx * dx + dy * dy + dz * dz).sqrt())
        } else {
            None
        }
    }

    /// Compute the area for AreaMarker annotations (using shoelace formula, projected to XZ plane).
    pub fn area_size(&self) -> Option<f32> {
        if let AnnotationType::AreaMarker { points, .. } = &self.annotation_type {
            if points.len() < 3 {
                return None;
            }
            let mut area = 0.0f32;
            let n = points.len();
            for i in 0..n {
                let j = (i + 1) % n;
                area += points[i].x * points[j].z;
                area -= points[j].x * points[i].z;
            }
            Some(area.abs() / 2.0)
        } else {
            None
        }
    }

    pub fn with_device(mut self, device_id: Uuid) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

// ── Annotation registry ──────────────────────────────────────────────────────

/// Central registry for scene annotations — Bevy resource.
#[derive(Resource, Default)]
pub struct AnnotationRegistry {
    annotations: HashMap<Uuid, Annotation>,
}

impl AnnotationRegistry {
    pub fn add(&mut self, annotation: Annotation) -> Uuid {
        let id = annotation.id;
        self.annotations.insert(id, annotation);
        id
    }

    pub fn remove(&mut self, id: Uuid) -> Option<Annotation> {
        self.annotations.remove(&id)
    }

    pub fn get(&self, id: Uuid) -> Option<&Annotation> {
        self.annotations.get(&id)
    }

    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Annotation> {
        self.annotations.get_mut(&id)
    }

    pub fn all(&self) -> impl Iterator<Item = &Annotation> {
        self.annotations.values()
    }

    pub fn count(&self) -> usize {
        self.annotations.len()
    }

    /// Filter annotations by tag.
    pub fn by_tag(&self, tag: &str) -> Vec<&Annotation> {
        self.annotations.values()
            .filter(|a| a.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Filter annotations by associated device.
    pub fn by_device(&self, device_id: Uuid) -> Vec<&Annotation> {
        self.annotations.values()
            .filter(|a| a.device_id == Some(device_id))
            .collect()
    }

    /// Get all visible annotations.
    pub fn visible(&self) -> Vec<&Annotation> {
        self.annotations.values()
            .filter(|a| a.visible)
            .collect()
    }

    /// Get all maintenance annotations with a specific status.
    pub fn maintenance_by_status(&self, status: &MaintenanceStatus) -> Vec<&Annotation> {
        self.annotations.values()
            .filter(|a| {
                if let AnnotationType::MaintenanceTag { status: s, .. } = &a.annotation_type {
                    s == status
                } else {
                    false
                }
            })
            .collect()
    }

    /// Toggle visibility of all annotations matching a tag.
    pub fn toggle_tag_visibility(&mut self, tag: &str) {
        for ann in self.annotations.values_mut() {
            if ann.tags.contains(&tag.to_string()) {
                ann.visible = !ann.visible;
            }
        }
    }

    /// Serialize all annotations to TOML.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        #[derive(Serialize)]
        struct Wrapper {
            annotations: Vec<Annotation>,
        }
        let wrapper = Wrapper {
            annotations: self.annotations.values().cloned().collect(),
        };
        toml::to_string_pretty(&wrapper)
    }

    /// Load annotations from TOML.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        #[derive(Deserialize)]
        struct Wrapper {
            annotations: Vec<Annotation>,
        }
        let wrapper: Wrapper = toml::from_str(toml_str)?;
        let mut registry = Self::default();
        for ann in wrapper.annotations {
            registry.annotations.insert(ann.id, ann);
        }
        Ok(registry)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_note_creation() {
        let note = Annotation::text_note(
            Position3D { x: 1.0, y: 2.0, z: 3.0 },
            "Check valve",
        );
        assert!(matches!(note.annotation_type, AnnotationType::TextNote { .. }));
        assert!(note.visible);
    }

    #[test]
    fn measurement_distance() {
        let m = Annotation::measurement(
            Position3D { x: 0.0, y: 0.0, z: 0.0 },
            Position3D { x: 3.0, y: 4.0, z: 0.0 },
        );
        let dist = m.measurement_distance().expect("should compute");
        assert!((dist - 5.0).abs() < 0.01);
    }

    #[test]
    fn area_computation() {
        // 1x1 square in XZ plane
        let a = Annotation {
            id: Uuid::new_v4(),
            position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
            annotation_type: AnnotationType::AreaMarker {
                points: vec![
                    Position3D { x: 0.0, y: 0.0, z: 0.0 },
                    Position3D { x: 1.0, y: 0.0, z: 0.0 },
                    Position3D { x: 1.0, y: 0.0, z: 1.0 },
                    Position3D { x: 0.0, y: 0.0, z: 1.0 },
                ],
                label: "Zone A".into(),
            },
            color: [0.0, 1.0, 0.0, 0.5],
            visible: true,
            device_id: None,
            created_by: None,
            created_at: 0,
            updated_at: 0,
            tags: Vec::new(),
        };
        let area = a.area_size().expect("should compute");
        assert!((area - 1.0).abs() < 0.01);
    }

    #[test]
    fn registry_crud() {
        let mut reg = AnnotationRegistry::default();
        let note = Annotation::text_note(Position3D { x: 0.0, y: 0.0, z: 0.0 }, "Test");
        let id = reg.add(note);
        assert_eq!(reg.count(), 1);
        assert!(reg.get(id).is_some());
        reg.remove(id);
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn filter_by_tag() {
        let mut reg = AnnotationRegistry::default();
        reg.add(Annotation::text_note(Position3D { x: 0.0, y: 0.0, z: 0.0 }, "A")
            .with_tag("safety"));
        reg.add(Annotation::text_note(Position3D { x: 1.0, y: 0.0, z: 0.0 }, "B")
            .with_tag("maintenance"));
        reg.add(Annotation::text_note(Position3D { x: 2.0, y: 0.0, z: 0.0 }, "C")
            .with_tag("safety"));

        assert_eq!(reg.by_tag("safety").len(), 2);
        assert_eq!(reg.by_tag("maintenance").len(), 1);
    }

    #[test]
    fn filter_by_device() {
        let mut reg = AnnotationRegistry::default();
        let device = Uuid::new_v4();
        reg.add(Annotation::text_note(Position3D { x: 0.0, y: 0.0, z: 0.0 }, "A")
            .with_device(device));
        reg.add(Annotation::text_note(Position3D { x: 1.0, y: 0.0, z: 0.0 }, "B"));

        assert_eq!(reg.by_device(device).len(), 1);
    }

    #[test]
    fn maintenance_filter() {
        let mut reg = AnnotationRegistry::default();
        reg.add(Annotation::maintenance_tag(
            Position3D { x: 0.0, y: 0.0, z: 0.0 },
            MaintenanceStatus::Scheduled,
            "Replace bearing",
        ));
        reg.add(Annotation::maintenance_tag(
            Position3D { x: 1.0, y: 0.0, z: 0.0 },
            MaintenanceStatus::Completed,
            "Oil changed",
        ));

        assert_eq!(reg.maintenance_by_status(&MaintenanceStatus::Scheduled).len(), 1);
        assert_eq!(reg.maintenance_by_status(&MaintenanceStatus::Completed).len(), 1);
    }

    #[test]
    fn toggle_visibility() {
        let mut reg = AnnotationRegistry::default();
        reg.add(Annotation::text_note(Position3D { x: 0.0, y: 0.0, z: 0.0 }, "A")
            .with_tag("safety"));
        reg.add(Annotation::text_note(Position3D { x: 1.0, y: 0.0, z: 0.0 }, "B"));

        reg.toggle_tag_visibility("safety");
        let safety = reg.by_tag("safety");
        assert!(!safety[0].visible);
    }

    #[test]
    fn position3d_vec3_conversion() {
        let p = Position3D { x: 1.0, y: 2.0, z: 3.0 };
        let v: Vec3 = p.into();
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));

        let p2: Position3D = v.into();
        assert_eq!(p2, p);
    }

    #[test]
    fn safety_zone_creation() {
        let zone = Annotation::safety_zone(
            Position3D { x: 5.0, y: 0.0, z: 5.0 },
            3.0,
            HazardType::HighVoltage,
            "Transformer area",
        );
        assert!(matches!(zone.annotation_type, AnnotationType::SafetyZone { .. }));
        assert!(zone.tags.contains(&"safety".to_string()));
    }
}
