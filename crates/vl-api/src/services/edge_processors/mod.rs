/// Concrete `EdgeEventProcessor` implementations — one per entity type.
///
/// Each processor wraps the relevant DAO(s) and handles cloud<->edge sync
/// for its entity type. Registered into `EdgeProcessorRegistry` during bootstrap.

pub mod device;
pub mod asset;
pub mod alarm;
pub mod rule_chain;
pub mod dashboard;
pub mod relation;
pub mod attribute;
pub mod widget;

pub use device::DeviceProcessor;
pub use asset::AssetProcessor;
pub use alarm::AlarmProcessor;
pub use rule_chain::RuleChainProcessor;
pub use dashboard::DashboardProcessor;
pub use relation::RelationProcessor;
pub use attribute::AttributeProcessor;
pub use widget::WidgetProcessor;

use std::sync::Arc;
use vl_cluster::EdgeProcessorRegistry;
use vl_dao::postgres::{
    alarm::AlarmDao,
    asset::AssetDao,
    dashboard::DashboardDao,
    device::DeviceDao,
    kv::KvDao,
    relation::RelationDao,
    rule_chain::RuleChainDao,
    widget_type::WidgetTypeDao,
    widgets_bundle::WidgetsBundleDao,
};

/// Build a fully-wired `EdgeProcessorRegistry` with all entity-type processors.
pub fn build_edge_processor_registry(
    device_dao:        Arc<DeviceDao>,
    asset_dao:         Arc<AssetDao>,
    alarm_dao:         Arc<AlarmDao>,
    rule_chain_dao:    Arc<RuleChainDao>,
    dashboard_dao:     Arc<DashboardDao>,
    relation_dao:      Arc<RelationDao>,
    kv_dao:            Arc<KvDao>,
    widget_type_dao:   Arc<WidgetTypeDao>,
    widgets_bundle_dao: Arc<WidgetsBundleDao>,
) -> EdgeProcessorRegistry {
    let mut registry = EdgeProcessorRegistry::new();

    registry.register(Arc::new(DeviceProcessor::new(device_dao)));
    registry.register(Arc::new(AssetProcessor::new(asset_dao)));
    registry.register(Arc::new(AlarmProcessor::new(alarm_dao)));
    registry.register(Arc::new(RuleChainProcessor::new(rule_chain_dao)));
    registry.register(Arc::new(DashboardProcessor::new(dashboard_dao)));
    registry.register(Arc::new(RelationProcessor::new(relation_dao)));
    registry.register(Arc::new(AttributeProcessor::new(kv_dao)));
    registry.register(Arc::new(WidgetProcessor::new(widget_type_dao, widgets_bundle_dao)));

    registry
}
