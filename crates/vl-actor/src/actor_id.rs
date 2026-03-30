use std::fmt;
use uuid::Uuid;

/// Unique identifier for an actor within the system.
///
/// Mirrors ThingsBoard's `TbActorId` hierarchy:
/// - `Entity` — wraps an entity ID + type (for tenant, device, rule chain, etc.)
/// - `String` — named system actors (e.g., "APP", "CFM|{tenant}")
/// - `CalculatedField` — separate namespace for CF entity actors (same entity ID
///   as device/asset but different actor)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TbActorId {
    /// Actor identified by entity ID + entity type.
    Entity {
        id: Uuid,
        entity_type: EntityType,
    },
    /// Actor identified by a string name.
    Named(String),
    /// Calculated-field entity actor (distinct from the entity's main actor).
    CalculatedField {
        id: Uuid,
    },
}

impl TbActorId {
    pub fn entity(id: Uuid, entity_type: EntityType) -> Self {
        Self::Entity { id, entity_type }
    }

    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    pub fn calculated_field(id: Uuid) -> Self {
        Self::CalculatedField { id }
    }

    /// Returns the entity type if this is an entity-based actor.
    pub fn entity_type(&self) -> Option<EntityType> {
        match self {
            Self::Entity { entity_type, .. } => Some(*entity_type),
            _ => None,
        }
    }
}

impl fmt::Display for TbActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entity { id, entity_type } => write!(f, "{entity_type:?}[{id}]"),
            Self::Named(name) => write!(f, "{name}"),
            Self::CalculatedField { id } => write!(f, "CF[{id}]"),
        }
    }
}

/// Entity types used as actor identifiers (matches ThingsBoard's EntityType).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    Tenant,
    Device,
    Asset,
    Customer,
    RuleChain,
    RuleNode,
    Edge,
    Dashboard,
    EntityView,
    User,
    WidgetType,
    WidgetsBundle,
    Alarm,
    OtaPackage,
    // Extend as needed
}
