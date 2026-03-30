use std::sync::Arc;
use vl_dao::DbPool;
use vl_config::VieLangConfig;
use vl_auth::JwtService;
use vl_core::entities::ActivityEvent;
use vl_dao::ActivationTokenDao;
use vl_dao::{
    AnalyticsDao,
    ApiUsageDao,
    BackupExportDao,
    ClusterPartitionDao,
    DeviceActivityDao,
    GeofenceDao,
    HousekeeperDao,
    CalculatedFieldDao,
    ClusterNodeDao,
    EntityVersionDao,
    MobileSessionDao,
    NotificationChannelSettingsDao,
    NotificationDeliveryDao,
    NotificationInboxDao,
    OtaStateDao,
    QueueStatsDao,
    ScheduledJobDao,
    SearchDao,
    SimulatorDao,
    SchematicDao,
    DeviceTemplateDao,
    SubscriptionPlanDao,
    TenantSubscriptionDao,
    TimeseriesDao,
    RuleNodeDao,
    RuleNodeStateDao,
    EntityAlarmDao,
    AlarmTypesDao,
    UserSettingsDao,
    UserAuthSettingsDao,
    postgres::{
        admin_settings::{AdminSettingsDao, UsageInfoDao},
        alarm::{AlarmCommentDao, AlarmDao},
        mobile_app::{MobileAppBundleDao, MobileAppDao, QrCodeSettingsDao},
        edge::{EdgeDao, EdgeEventDao},
        api_key::ApiKeyDao,
        asset::AssetDao,
        asset_profile::AssetProfileDao,
        audit_log::AuditLogDao,
        customer::CustomerDao,
        dashboard::DashboardDao,
        device::DeviceDao,
        entity_query::EntityQueryDao,
        device_profile::DeviceProfileDao,
        entity_view::EntityViewDao,
        event::EventDao,
        kv::KvDao,
        notification_request::NotificationRequestDao,
        notification_rule::NotificationRuleDao,
        notification_target::NotificationTargetDao,
        notification_template::NotificationTemplateDao,
        oauth2_registration::OAuth2RegistrationDao,
        ota_package::OtaPackageDao,
        relation::RelationDao,
        rpc::RpcDao,
        rule_chain::RuleChainDao,
        tenant::TenantDao,
        tenant_profile::TenantProfileDao,
        resource::ResourceDao,
        ai_model::AiModelDao,
        domain::DomainDao,
        oauth2_template::OAuth2TemplateDao,
        rbac::RbacDao,
        two_factor_auth::TwoFactorAuthDao,
        user::UserDao,
        widget_type::WidgetTypeDao,
        widgets_bundle::WidgetsBundleDao,
        component_descriptor::ComponentDescriptorDao,
    },
};
use vl_rule_engine::{RuleEngine, TenantChainRegistry};
use vl_queue::TbProducer;
use vl_cache::TbCache;
use vl_cluster::{ClusterManager, EdgeSessionRegistry};
use vl_core::entities::TbMsg;
use tokio::sync::broadcast;

use crate::middleware::TenantRateLimiter;
use crate::notification::NotificationService;
use crate::notification::channels::{
    NotificationChannel, SlackChannel, SmsChannel,
    TeamsChannel, TelegramChannel, WebhookChannel,
};
use crate::services::ota_service::OtaService;
use crate::services::fcm_push::FcmPushService;
use crate::services::housekeeper::HousekeeperService;
use crate::services::job_scheduler::{
    CleanupJobHandler, JobSchedulerService, RuleChainTriggerJobHandler,
    ScheduledNotificationJobHandler,
};
use crate::services::notification_delivery::NotificationDeliveryService;
use crate::services::queue_monitor::QueueMonitorService;
use crate::services::cluster_monitor::ClusterMonitorService;
use crate::services::stripe_service::StripeService;
use crate::services::token_blacklist::TokenBlacklist;
use crate::services::usage_tracker::UsageTracker;
use crate::services::activation_service::ActivationService;
use crate::services::backup::{BackupJobHandler, ExportService, ImportService};
use crate::services::cluster_routing::ClusterMessageRouter;
use crate::services::simulator::SimulatorService;
use crate::services::arduino_compiler::ArduinoCompilerService;
use crate::services::version_control::VersionControlService;
use crate::notification::channels::email::EmailChannel;

/// Shared application state — inject vào mọi Axum handler qua State extractor
#[derive(Clone)]
pub struct AppState {
    pub pool:             DbPool,
    pub config:           Arc<VieLangConfig>,
    pub jwt_service:      Arc<JwtService>,
    // ── Timeseries DAO (PostgreSQL hoặc Cassandra) ─────────────────────────────
    pub ts_dao:           Arc<dyn TimeseriesDao>,
    // ── PostgreSQL DAOs ────────────────────────────────────────────────────────
    pub admin_settings_dao:         Arc<AdminSettingsDao>,
    pub usage_info_dao:             Arc<UsageInfoDao>,
    pub mobile_app_dao:             Arc<MobileAppDao>,
    pub mobile_app_bundle_dao:      Arc<MobileAppBundleDao>,
    pub qr_code_settings_dao:       Arc<QrCodeSettingsDao>,
    pub mobile_session_dao:         Arc<MobileSessionDao>,
    pub notification_channel_settings_dao: Arc<NotificationChannelSettingsDao>,
    pub notification_inbox_dao:     Arc<NotificationInboxDao>,
    pub alarm_dao:                  Arc<AlarmDao>,
    pub alarm_comment_dao:          Arc<AlarmCommentDao>,
    pub edge_dao:                   Arc<EdgeDao>,
    pub edge_event_dao:             Arc<EdgeEventDao>,
    pub api_key_dao:                Arc<ApiKeyDao>,
    pub asset_dao:                  Arc<AssetDao>,
    pub asset_profile_dao:          Arc<AssetProfileDao>,
    pub audit_log_dao:              Arc<AuditLogDao>,
    pub device_profile_dao:         Arc<DeviceProfileDao>,
    pub entity_view_dao:            Arc<EntityViewDao>,
    pub customer_dao:               Arc<CustomerDao>,
    pub dashboard_dao:              Arc<DashboardDao>,
    pub device_dao:                 Arc<DeviceDao>,
    pub entity_query_dao:           Arc<EntityQueryDao>,
    pub event_dao:                  Arc<EventDao>,
    /// kv_dao — chỉ dùng cho ATTRIBUTE operations (luôn ở PostgreSQL)
    pub kv_dao:                     Arc<KvDao>,
    pub notification_request_dao:   Arc<NotificationRequestDao>,
    pub notification_rule_dao:      Arc<NotificationRuleDao>,
    pub notification_target_dao:    Arc<NotificationTargetDao>,
    pub notification_template_dao:  Arc<NotificationTemplateDao>,
    pub oauth2_registration_dao:    Arc<OAuth2RegistrationDao>,
    pub ota_package_dao:            Arc<OtaPackageDao>,
    pub ota_state_dao:              Arc<OtaStateDao>,
    pub relation_dao:               Arc<RelationDao>,
    pub rpc_dao:                    Arc<RpcDao>,
    pub rule_chain_dao:             Arc<RuleChainDao>,
    pub tenant_dao:                 Arc<TenantDao>,
    pub tenant_profile_dao:         Arc<TenantProfileDao>,
    pub resource_dao:               Arc<ResourceDao>,
    pub two_factor_auth_dao:        Arc<TwoFactorAuthDao>,
    pub user_dao:                   Arc<UserDao>,
    pub widget_type_dao:            Arc<WidgetTypeDao>,
    pub widgets_bundle_dao:         Arc<WidgetsBundleDao>,
    pub component_descriptor_dao:   Arc<ComponentDescriptorDao>,
    /// Phase 31 — Device Activity
    pub device_activity_dao:    Arc<DeviceActivityDao>,
    pub activity_tx:            tokio::sync::mpsc::Sender<ActivityEvent>,
    // ── Services ──────────────────────────────────────────────────────────────
    pub rule_engine:            Arc<RuleEngine>,
    pub queue_producer:         Arc<dyn TbProducer>,
    pub cache:                  Arc<dyn TbCache>,
    pub cluster:                Arc<ClusterManager>,
    /// Phase 58.1: JWT token revocation blacklist
    pub token_blacklist:        Arc<TokenBlacklist>,
    pub notification_service:   Arc<NotificationService>,
    pub rate_limiter:           Arc<TenantRateLimiter>,
    /// Broadcast channel for real-time WS push — transport handler sends here after saving telemetry
    pub ws_tx:                  broadcast::Sender<TbMsg>,
    // ── Phase 32: Housekeeper ─────────────────────────────────────────────────
    pub housekeeper_dao:        Arc<HousekeeperDao>,
    pub housekeeper_service:    Arc<HousekeeperService>,
    // ── Phase 34: Calculated Fields ───────────────────────────────────────────
    pub calc_field_dao:         Arc<CalculatedFieldDao>,
    // ── Phase 37: Entity Version Control ──────────────────────────────────────
    pub version_dao:            Arc<EntityVersionDao>,
    pub version_control_svc:    Arc<VersionControlService>,
    // ── Phase 35: Job Scheduler ───────────────────────────────────────────────
    pub job_scheduler_dao:      Arc<ScheduledJobDao>,
    pub job_scheduler_service:  Arc<JobSchedulerService>,
    // ── Phase 36: Queue Management ────────────────────────────────────────────
    pub queue_stats_dao:        Arc<QueueStatsDao>,
    pub queue_monitor_service:  Arc<QueueMonitorService>,
    // ── Phase 39: Cluster Mode ────────────────────────────────────────────────
    pub cluster_node_dao:       Arc<ClusterNodeDao>,
    pub cluster_monitor_service: Arc<ClusterMonitorService>,
    // ── MQTT RPC registries ───────────────────────────────────────────────────
    /// MQTT device write registry: device_id → write channel sender
    pub device_rpc_registry: Arc<vl_transport::DeviceWriteRegistry>,
    /// Pending two-way RPC: (device_id, request_id) → oneshot response sender
    pub rpc_pending_registry: Arc<vl_transport::RpcPendingRegistry>,
    // ── Phase 51: WS Session Registry ────────────────────────────────────────
    /// Entity-indexed WebSocket subscriptions — push realtime updates to exact sessions
    pub ws_registry: Arc<crate::ws::WsSessionRegistry>,
    // ── Phase 52: Multi-tenant Rule Engine Registry ───────────────────────────
    /// Per-tenant rule chain cache — used by RuleEngineController for cache invalidation
    pub re_registry: Arc<TenantChainRegistry>,
    // ── Phase 54: New DAOs ────────────────────────────────────────────────────
    pub ai_model_dao:        Arc<AiModelDao>,
    pub domain_dao:          Arc<DomainDao>,
    pub oauth2_template_dao: Arc<OAuth2TemplateDao>,
    // ── Phase 63: Fine-Grained RBAC ───────────────────────────────────────────
    pub rbac_dao:            Arc<RbacDao>,
    // ── Phase 56: Edge gRPC session registry ─────────────────────────────────
    /// Tracks connected Edge instances — used by rule nodes (PushToEdge) and edge events
    pub edge_session_registry: Arc<EdgeSessionRegistry>,
    // ── Phase 67: FCM Push Notifications ─────────────────────────────────────
    pub notification_delivery_svc: Arc<NotificationDeliveryService>,
    // ── Phase 70: Subscription Billing ────────────────────────────────────────
    pub plan_dao:         Arc<SubscriptionPlanDao>,
    pub subscription_dao: Arc<TenantSubscriptionDao>,
    pub stripe_service:   Option<Arc<StripeService>>,
    // ── Phase 71: API Usage Tracking ──────────────────────────────────────────
    pub api_usage_dao:    Arc<ApiUsageDao>,
    pub usage_tracker:    Arc<UsageTracker>,
    // ── Phase 72: SaaS Analytics ──────────────────────────────────────────────
    pub analytics_dao:    Arc<AnalyticsDao>,
    // ── P2: OTA Service ───────────────────────────────────────────────────────
    pub ota_service:      Arc<OtaService>,
    // ── P5: Full-Text Search ──────────────────────────────────────────────────
    pub search_dao:       Arc<SearchDao>,
    // ── P8: Geofencing ────────────────────────────────────────────────────────
    pub geofence_dao:     Arc<GeofenceDao>,
    // ── P11: Queue Message Persistence ────────────────────────────────────────
    pub queue_message_dao: Arc<vl_dao::QueueMessageDao>,
    // ── P14: Backup / Restore ──────────────────────────────────────────────────
    pub backup_export_dao: Arc<BackupExportDao>,
    pub backup_export_svc: Arc<ExportService>,
    pub backup_import_svc: Arc<ImportService>,
    // ── P15: Cluster Raft — Partition Service + Message Router ────────────────
    pub cluster_partition_dao: Arc<ClusterPartitionDao>,
    pub partition_svc:         Arc<vl_cluster::PartitionService>,
    pub cluster_router:        Arc<ClusterMessageRouter>,
    // ── P3: Notification Delivery Tracking ────────────────────────────────────
    pub notification_delivery_dao: Arc<NotificationDeliveryDao>,
    // ── P4: Activation Service ────────────────────────────────────────────────
    pub activation_service: Arc<ActivationService>,
    // ── P6: Dead-Letter Queue ─────────────────────────────────────────────────
    pub dlq_dao: Arc<vl_dao::DlqDao>,
    // ── IoT Simulator ─────────────────────────────────────────────────────────
    pub simulator_dao:     Arc<SimulatorDao>,
    pub simulator_service: Arc<SimulatorService>,
    pub schematic_dao:     Arc<SchematicDao>,
    pub device_template_dao: Arc<DeviceTemplateDao>,
    pub arduino_compiler:   Arc<ArduinoCompilerService>,
    // ── Phase 61: Rule Node + State DAOs ──────────────────────────────────────
    pub rule_node_dao:          Arc<RuleNodeDao>,
    pub rule_node_state_dao:    Arc<RuleNodeStateDao>,
    // ── Phase 63: Entity Alarm + Alarm Types + User Settings ──────────────────
    pub entity_alarm_dao:       Arc<EntityAlarmDao>,
    pub alarm_types_dao:        Arc<AlarmTypesDao>,
    pub user_settings_dao:      Arc<UserSettingsDao>,
    pub user_auth_settings_dao: Arc<UserAuthSettingsDao>,
}

impl AppState {
    pub fn new(
        pool:           DbPool,
        config:         VieLangConfig,
        ts_dao:         Arc<dyn TimeseriesDao>,
        rule_engine:    RuleEngine,
        queue_producer: Arc<dyn TbProducer>,
        cache:          Arc<dyn TbCache>,
        cluster:        ClusterManager,
        activity_tx:    tokio::sync::mpsc::Sender<ActivityEvent>,
    ) -> Self {
        // ── Phase 1: Create ALL DAOs once, reuse via Arc::clone() ─────────────
        let admin_settings_dao         = Arc::new(AdminSettingsDao::new(pool.clone()));
        let usage_info_dao             = Arc::new(UsageInfoDao::new(pool.clone()));
        let mobile_app_dao             = Arc::new(MobileAppDao::new(pool.clone()));
        let mobile_app_bundle_dao      = Arc::new(MobileAppBundleDao::new(pool.clone()));
        let qr_code_settings_dao       = Arc::new(QrCodeSettingsDao::new(pool.clone()));
        let mobile_session_dao         = Arc::new(MobileSessionDao::new(pool.clone()));
        let notification_channel_settings_dao = Arc::new(NotificationChannelSettingsDao::new(pool.clone()));
        let notification_inbox_dao     = Arc::new(NotificationInboxDao::new(pool.clone()));
        let alarm_dao                  = Arc::new(AlarmDao::new(pool.clone()));
        let alarm_comment_dao          = Arc::new(AlarmCommentDao::new(pool.clone()));
        let edge_dao                   = Arc::new(EdgeDao::new(pool.clone()));
        let edge_event_dao             = Arc::new(EdgeEventDao::new(pool.clone()));
        let api_key_dao                = Arc::new(ApiKeyDao::new(pool.clone()));
        let asset_dao                  = Arc::new(AssetDao::new(pool.clone()));
        let asset_profile_dao          = Arc::new(AssetProfileDao::new(pool.clone()));
        let audit_log_dao              = Arc::new(AuditLogDao::new(pool.clone()));
        let device_profile_dao         = Arc::new(DeviceProfileDao::new(pool.clone()));
        let entity_view_dao            = Arc::new(EntityViewDao::new(pool.clone()));
        let customer_dao               = Arc::new(CustomerDao::new(pool.clone()));
        let dashboard_dao              = Arc::new(DashboardDao::new(pool.clone()));
        let device_dao                 = Arc::new(DeviceDao::new(pool.clone()));
        let entity_query_dao           = Arc::new(EntityQueryDao::new(pool.clone()));
        let event_dao                  = Arc::new(EventDao::new(pool.clone()));
        let kv_dao                     = Arc::new(KvDao::new(pool.clone()));
        let notification_request_dao   = Arc::new(NotificationRequestDao::new(pool.clone()));
        let notification_rule_dao      = Arc::new(NotificationRuleDao::new(pool.clone()));
        let notification_target_dao    = Arc::new(NotificationTargetDao::new(pool.clone()));
        let notification_template_dao  = Arc::new(NotificationTemplateDao::new(pool.clone()));
        let notification_delivery_dao  = Arc::new(NotificationDeliveryDao::new(pool.clone()));
        let oauth2_registration_dao    = Arc::new(OAuth2RegistrationDao::new(pool.clone()));
        let ota_package_dao            = Arc::new(OtaPackageDao::new(pool.clone()));
        let ota_state_dao              = Arc::new(OtaStateDao::new(pool.clone()));
        let relation_dao               = Arc::new(RelationDao::new(pool.clone()));
        let rpc_dao                    = Arc::new(RpcDao::new(pool.clone()));
        let rule_chain_dao             = Arc::new(RuleChainDao::new(pool.clone()));
        let tenant_dao                 = Arc::new(TenantDao::new(pool.clone()));
        let tenant_profile_dao         = Arc::new(TenantProfileDao::new(pool.clone()));
        let resource_dao               = Arc::new(ResourceDao::new(pool.clone()));
        let two_factor_auth_dao        = Arc::new(TwoFactorAuthDao::new(pool.clone()));
        let user_dao                   = Arc::new(UserDao::new(pool.clone()));
        let widget_type_dao            = Arc::new(WidgetTypeDao::new(pool.clone()));
        let widgets_bundle_dao         = Arc::new(WidgetsBundleDao::new(pool.clone()));
        let component_descriptor_dao   = Arc::new(ComponentDescriptorDao::new(pool.clone()));
        let device_activity_dao        = Arc::new(DeviceActivityDao::new(pool.clone()));
        let housekeeper_dao            = Arc::new(HousekeeperDao::new(pool.clone()));
        let job_scheduler_dao          = Arc::new(ScheduledJobDao::new(pool.clone()));
        let queue_stats_dao            = Arc::new(QueueStatsDao::new(pool.clone()));
        let cluster_node_dao           = Arc::new(ClusterNodeDao::new(pool.clone()));
        let calc_field_dao             = Arc::new(CalculatedFieldDao::new(pool.clone()));
        let version_dao                = Arc::new(EntityVersionDao::new(pool.clone()));
        let ai_model_dao               = Arc::new(AiModelDao::new(pool.clone()));
        let domain_dao                 = Arc::new(DomainDao::new(pool.clone()));
        let oauth2_template_dao        = Arc::new(OAuth2TemplateDao::new(pool.clone()));
        let rbac_dao                   = Arc::new(RbacDao::new(pool.clone()));
        let plan_dao                   = Arc::new(SubscriptionPlanDao::new(pool.clone()));
        let subscription_dao           = Arc::new(TenantSubscriptionDao::new(pool.clone()));
        let api_usage_dao              = Arc::new(ApiUsageDao::new(pool.clone()));
        let analytics_dao              = Arc::new(AnalyticsDao::new(pool.clone()));
        let search_dao                 = Arc::new(SearchDao::new(pool.clone()));
        let geofence_dao               = Arc::new(GeofenceDao::new(pool.clone()));
        let queue_message_dao          = Arc::new(vl_dao::QueueMessageDao::new(pool.clone()));
        let backup_export_dao          = Arc::new(BackupExportDao::new(pool.clone()));
        let cluster_partition_dao      = Arc::new(ClusterPartitionDao::new(pool.clone()));
        let dlq_dao                    = Arc::new(vl_dao::DlqDao::new(pool.clone()));
        let simulator_dao              = Arc::new(SimulatorDao::new(pool.clone()));
        let schematic_dao              = Arc::new(SchematicDao::new(pool.clone()));
        let device_template_dao        = Arc::new(DeviceTemplateDao::new(pool.clone()));
        let rule_node_dao              = Arc::new(RuleNodeDao::new(pool.clone()));
        let rule_node_state_dao        = Arc::new(RuleNodeStateDao::new(pool.clone()));
        let entity_alarm_dao           = Arc::new(EntityAlarmDao::new(pool.clone()));
        let alarm_types_dao            = Arc::new(AlarmTypesDao::new(pool.clone()));
        let user_settings_dao          = Arc::new(UserSettingsDao::new(pool.clone()));
        let user_auth_settings_dao     = Arc::new(UserAuthSettingsDao::new(pool.clone()));
        let arduino_compiler           = Arc::new(ArduinoCompilerService::new(
            &config.simulator.arduino_cli_path,
            config.simulator.arduino_compile_timeout_secs,
            &config.simulator.arduino_sketch_dir,
        ));
        let activation_token_dao       = Arc::new(ActivationTokenDao::new(pool.clone()));

        // ── Phase 2: Build services reusing shared DAO instances ──────────────
        let jwt_service = Arc::new(JwtService::new_with_rotation(
            &config.security.jwt.secret,
            config.security.jwt.previous_signing_key.as_deref(),
            config.security.jwt.expiration_secs,
            config.security.jwt.refresh_expiration_secs,
        ));
        let (ws_tx, _) = broadcast::channel::<TbMsg>(1024);

        let firebase_config = config.firebase.clone();
        let fcm_svc_opt: Option<Arc<FcmPushService>> =
            if firebase_config.enabled && !firebase_config.server_key.is_empty() {
                Some(Arc::new(FcmPushService::new(firebase_config.server_key.clone())))
            } else {
                None
            };

        let notification_service = {
            let svc = NotificationService::new(
                config.notification.smtp.clone(),
                &config.notification.sms,
                notification_template_dao.clone(),
                notification_target_dao.clone(),
                notification_request_dao.clone(),
                notification_delivery_dao.clone(),
            );
            let svc = if let Some(fcm) = fcm_svc_opt.clone() {
                svc.with_fcm(fcm)
            } else {
                svc
            };
            Arc::new(svc)
        };

        let rate_limiter = Arc::new(TenantRateLimiter::new(1000));
        let housekeeper_config = config.housekeeper.clone();
        let stripe_config = config.stripe.clone();
        let local_node_id = cluster.local_node_id().to_string();
        let device_rpc_registry = Arc::new(vl_transport::DeviceWriteRegistry::new());

        let re_registry = rule_engine.registry().unwrap_or_else(|| {
            Arc::new(TenantChainRegistry::new(rule_chain_dao.clone()))
        });

        let rule_engine = Arc::new(rule_engine);
        let cluster = Arc::new(cluster);

        let num_partitions = config.cluster.num_partitions.max(1);
        let partition_svc = Arc::new(vl_cluster::PartitionService::new(num_partitions));

        // Build notification delivery channels
        let http_client = reqwest::Client::new();
        let delivery_channels: Vec<Arc<dyn NotificationChannel>> = vec![
            Arc::new(EmailChannel::new(config.notification.smtp.clone())),
            Arc::new(SlackChannel::new(http_client.clone())),
            Arc::new(SmsChannel::new(&config.notification.sms, http_client.clone())),
            Arc::new(TeamsChannel::new(http_client.clone())),
            Arc::new(TelegramChannel::new(http_client.clone())),
            Arc::new(WebhookChannel::new(http_client)),
        ];

        // Notification delivery service — shared by job scheduler and AppState
        let notification_delivery_svc = Arc::new(NotificationDeliveryService::new(
            notification_inbox_dao.clone(),
            mobile_session_dao.clone(),
            fcm_svc_opt.clone(),
            delivery_channels,
        ));

        let queue_type = config.queue.queue_type.clone();
        let activation_smtp_config = config.notification.smtp.clone();
        let activation_token_ttl_hours = config.auth.activation_token_ttl_hours;
        let activation_base_url = config.server.base_url();

        // Housekeeper service
        let housekeeper_service = Arc::new(HousekeeperService::new(
            housekeeper_dao.clone(),
            housekeeper_config.clone(),
        ));

        // Export service — shared by BackupJobHandler and backup_export_svc
        let export_svc = Arc::new(ExportService::new(
            tenant_dao.clone(),
            device_dao.clone(),
            asset_dao.clone(),
            customer_dao.clone(),
            dashboard_dao.clone(),
            rule_chain_dao.clone(),
            user_dao.clone(),
            backup_export_dao.clone(),
        ));

        // Import service
        let import_svc = Arc::new(ImportService::new(
            device_dao.clone(),
            asset_dao.clone(),
            customer_dao.clone(),
            dashboard_dao.clone(),
            rule_chain_dao.clone(),
            user_dao.clone(),
        ));

        // OTA service — reuses shared DAOs + device_rpc_registry
        let ota_service = Arc::new(OtaService::new(
            ota_state_dao.clone(),
            ota_package_dao.clone(),
            device_dao.clone(),
            device_rpc_registry.clone(),
        ));

        // Activation service — reuses shared DAOs
        let activation_service = Arc::new(ActivationService::new(
            activation_token_dao.clone(),
            user_dao.clone(),
            EmailChannel::new(activation_smtp_config),
            activation_token_ttl_hours,
            activation_base_url,
        ));

        // Queue monitor service
        let queue_monitor_service = {
            let svc = QueueMonitorService::new(queue_stats_dao.clone());
            let svc = if queue_type == vl_config::QueueType::Persistent {
                svc.with_queue_message_dao(queue_message_dao.clone())
            } else {
                svc
            };
            Arc::new(svc)
        };

        // Cluster monitor service
        let cluster_monitor_service = Arc::new(ClusterMonitorService::with_partition_dao(
            cluster_node_dao.clone(),
            cluster_partition_dao.clone(),
            partition_svc.clone(),
            local_node_id,
        ));

        // Cluster message router
        let cluster_router = Arc::new(ClusterMessageRouter::new(
            cluster.clone(),
            partition_svc.clone(),
            rule_engine.clone(),
        ));

        // Job scheduler with real handlers
        let scheduler_config = config.scheduler.clone();
        let job_scheduler_service = Arc::new(
            JobSchedulerService::with_config(
                job_scheduler_dao.clone(),
                scheduler_config.check_interval_s,
                scheduler_config.max_concurrent_jobs,
            )
            .register(Arc::new(CleanupJobHandler::new(
                housekeeper_dao.clone(),
                housekeeper_config,
            )))
            .register(Arc::new(ScheduledNotificationJobHandler::new(
                notification_delivery_svc.clone(),
            )))
            .register(Arc::new(RuleChainTriggerJobHandler::new(
                rule_engine.clone(),
            )))
            .register(Arc::new(BackupJobHandler::new(export_svc.clone()))),
        );

        // Usage tracker — reuses shared api_usage_dao
        let usage_tracker = Arc::new(UsageTracker::new(api_usage_dao.clone()));

        // Token blacklist
        let token_blacklist = Arc::new(TokenBlacklist::new(cache.clone()));

        // Stripe service (optional)
        let stripe_service = if stripe_config.enabled && !stripe_config.secret_key.is_empty() {
            Some(Arc::new(StripeService::new(&stripe_config)))
        } else {
            None
        };

        // Simulator service
        let simulator_service = Arc::new(SimulatorService::new(
            config.simulator.clone(),
            simulator_dao.clone(),
            ts_dao.clone(),
            ws_tx.clone(),
            queue_producer.clone(),
        ));

        // Version control service — async version creation worker
        let version_control_svc = Arc::new(VersionControlService::start(version_dao.clone()));

        // ── Phase 3: Assemble AppState using shared instances ─────────────────
        Self {
            pool:            pool.clone(),
            config:          Arc::new(config),
            jwt_service,
            ts_dao,
            // DAOs — all created once above, assigned via clone()
            admin_settings_dao,
            usage_info_dao,
            mobile_app_dao,
            mobile_app_bundle_dao,
            qr_code_settings_dao,
            mobile_session_dao,
            notification_channel_settings_dao,
            notification_inbox_dao,
            alarm_dao,
            alarm_comment_dao,
            edge_dao,
            edge_event_dao,
            api_key_dao,
            asset_dao,
            asset_profile_dao,
            audit_log_dao,
            device_profile_dao,
            entity_view_dao,
            customer_dao,
            dashboard_dao,
            device_dao,
            entity_query_dao,
            event_dao,
            kv_dao,
            notification_request_dao,
            notification_rule_dao,
            notification_target_dao,
            notification_template_dao,
            oauth2_registration_dao,
            ota_package_dao,
            ota_state_dao,
            relation_dao,
            rpc_dao,
            rule_chain_dao,
            tenant_dao,
            tenant_profile_dao,
            resource_dao,
            two_factor_auth_dao,
            user_dao,
            widget_type_dao,
            widgets_bundle_dao,
            component_descriptor_dao,
            device_activity_dao,
            activity_tx,
            // Services
            housekeeper_dao,
            housekeeper_service,
            calc_field_dao,
            version_dao,
            version_control_svc,
            job_scheduler_dao,
            job_scheduler_service,
            queue_stats_dao,
            queue_monitor_service,
            cluster_node_dao,
            cluster_monitor_service,
            rule_engine,
            queue_producer,
            token_blacklist,
            cache,
            cluster,
            notification_service,
            rate_limiter,
            ws_tx,
            device_rpc_registry,
            rpc_pending_registry: Arc::new(vl_transport::RpcPendingRegistry::new()),
            ws_registry: Arc::new(crate::ws::WsSessionRegistry::new()),
            re_registry,
            ai_model_dao,
            domain_dao,
            oauth2_template_dao,
            rbac_dao,
            edge_session_registry: Arc::new(EdgeSessionRegistry::new()),
            notification_delivery_svc,
            notification_delivery_dao,
            plan_dao,
            subscription_dao,
            stripe_service,
            api_usage_dao,
            usage_tracker,
            analytics_dao,
            ota_service,
            search_dao,
            geofence_dao,
            queue_message_dao,
            backup_export_dao,
            backup_export_svc: export_svc,
            backup_import_svc: import_svc,
            cluster_partition_dao,
            partition_svc,
            cluster_router,
            dlq_dao,
            activation_service,
            simulator_dao,
            simulator_service,
            schematic_dao,
            device_template_dao,
            arduino_compiler,
            rule_node_dao,
            rule_node_state_dao,
            entity_alarm_dao,
            alarm_types_dao,
            user_settings_dao,
            user_auth_settings_dao,
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Domain Sub-States — handlers can extract State<SubState> via FromRef
// ══════════════════════════════════════════════════════════════════════════════

/// Core infrastructure — used by middleware (auth, rate_limit, audit_log).
#[derive(Clone)]
pub struct CoreState {
    pub pool:            DbPool,
    pub config:          Arc<VieLangConfig>,
    pub jwt_service:     Arc<JwtService>,
    pub cache:           Arc<dyn TbCache>,
    pub queue_producer:  Arc<dyn TbProducer>,
    pub token_blacklist: Arc<TokenBlacklist>,
    pub rate_limiter:    Arc<TenantRateLimiter>,
    pub audit_log_dao:   Arc<AuditLogDao>,
    pub api_key_dao:     Arc<ApiKeyDao>,
    pub tenant_profile_dao: Arc<TenantProfileDao>,
}

/// Authentication and user management.
#[derive(Clone)]
pub struct AuthState {
    pub user_dao:                Arc<UserDao>,
    pub tenant_dao:              Arc<TenantDao>,
    pub tenant_profile_dao:      Arc<TenantProfileDao>,
    pub two_factor_auth_dao:     Arc<TwoFactorAuthDao>,
    pub oauth2_registration_dao: Arc<OAuth2RegistrationDao>,
    pub oauth2_template_dao:     Arc<OAuth2TemplateDao>,
    pub rbac_dao:                Arc<RbacDao>,
    pub activation_service:      Arc<ActivationService>,
    pub domain_dao:              Arc<DomainDao>,
}

/// Device management and IoT transport.
#[derive(Clone)]
pub struct DeviceState {
    pub device_dao:          Arc<DeviceDao>,
    pub device_profile_dao:  Arc<DeviceProfileDao>,
    pub device_activity_dao: Arc<DeviceActivityDao>,
    pub activity_tx:         tokio::sync::mpsc::Sender<ActivityEvent>,
    pub device_rpc_registry: Arc<vl_transport::DeviceWriteRegistry>,
    pub rpc_pending_registry: Arc<vl_transport::RpcPendingRegistry>,
    pub rpc_dao:             Arc<RpcDao>,
}

/// Timeseries, attributes, and real-time streaming.
#[derive(Clone)]
pub struct TelemetryState {
    pub ts_dao:        Arc<dyn TimeseriesDao>,
    pub kv_dao:        Arc<KvDao>,
    pub ws_tx:         broadcast::Sender<TbMsg>,
    pub ws_registry:   Arc<crate::ws::WsSessionRegistry>,
    pub calc_field_dao: Arc<CalculatedFieldDao>,
}

/// Alarm management.
#[derive(Clone)]
pub struct AlarmState {
    pub alarm_dao:         Arc<AlarmDao>,
    pub alarm_comment_dao: Arc<AlarmCommentDao>,
}

/// Assets, dashboards, relations, entity views, entity queries, search, geofence.
#[derive(Clone)]
pub struct EntityState {
    pub asset_dao:        Arc<AssetDao>,
    pub asset_profile_dao: Arc<AssetProfileDao>,
    pub customer_dao:     Arc<CustomerDao>,
    pub dashboard_dao:    Arc<DashboardDao>,
    pub relation_dao:     Arc<RelationDao>,
    pub entity_view_dao:  Arc<EntityViewDao>,
    pub entity_query_dao: Arc<EntityQueryDao>,
    pub version_dao:      Arc<EntityVersionDao>,
    pub version_control_svc: Arc<VersionControlService>,
    pub search_dao:       Arc<SearchDao>,
    pub geofence_dao:     Arc<GeofenceDao>,
}

/// Notification system.
#[derive(Clone)]
pub struct NotificationState {
    pub notification_template_dao:         Arc<NotificationTemplateDao>,
    pub notification_target_dao:           Arc<NotificationTargetDao>,
    pub notification_request_dao:          Arc<NotificationRequestDao>,
    pub notification_rule_dao:             Arc<NotificationRuleDao>,
    pub notification_inbox_dao:            Arc<NotificationInboxDao>,
    pub notification_channel_settings_dao: Arc<NotificationChannelSettingsDao>,
    pub notification_delivery_dao:         Arc<NotificationDeliveryDao>,
    pub notification_service:              Arc<NotificationService>,
    pub notification_delivery_svc:         Arc<NotificationDeliveryService>,
}

/// Rule engine and rule chains.
#[derive(Clone)]
pub struct RuleEngineState {
    pub rule_engine:             Arc<RuleEngine>,
    pub rule_chain_dao:          Arc<RuleChainDao>,
    pub re_registry:             Arc<TenantChainRegistry>,
    pub component_descriptor_dao: Arc<ComponentDescriptorDao>,
}

/// Edge computing.
#[derive(Clone)]
pub struct EdgeState {
    pub edge_dao:              Arc<EdgeDao>,
    pub edge_event_dao:        Arc<EdgeEventDao>,
    pub edge_session_registry: Arc<EdgeSessionRegistry>,
}

/// System admin, housekeeping, scheduling, events.
#[derive(Clone)]
pub struct AdminState {
    pub admin_settings_dao:   Arc<AdminSettingsDao>,
    pub usage_info_dao:       Arc<UsageInfoDao>,
    pub event_dao:            Arc<EventDao>,
    pub housekeeper_dao:      Arc<HousekeeperDao>,
    pub housekeeper_service:  Arc<HousekeeperService>,
    pub job_scheduler_dao:    Arc<ScheduledJobDao>,
    pub job_scheduler_service: Arc<JobSchedulerService>,
    pub queue_stats_dao:      Arc<QueueStatsDao>,
    pub queue_monitor_service: Arc<QueueMonitorService>,
    pub queue_message_dao:    Arc<vl_dao::QueueMessageDao>,
    pub dlq_dao:              Arc<vl_dao::DlqDao>,
}

/// Cluster coordination.
#[derive(Clone)]
pub struct ClusterState {
    pub cluster:                 Arc<ClusterManager>,
    pub cluster_node_dao:        Arc<ClusterNodeDao>,
    pub cluster_monitor_service: Arc<ClusterMonitorService>,
    pub cluster_partition_dao:   Arc<ClusterPartitionDao>,
    pub partition_svc:           Arc<vl_cluster::PartitionService>,
    pub cluster_router:          Arc<ClusterMessageRouter>,
}

/// OTA firmware updates.
#[derive(Clone)]
pub struct OtaState {
    pub ota_package_dao: Arc<OtaPackageDao>,
    pub ota_state_dao:   Arc<OtaStateDao>,
    pub ota_service:     Arc<OtaService>,
}

/// Subscription billing and API usage.
#[derive(Clone)]
pub struct BillingState {
    pub plan_dao:         Arc<SubscriptionPlanDao>,
    pub subscription_dao: Arc<TenantSubscriptionDao>,
    pub stripe_service:   Option<Arc<StripeService>>,
    pub api_usage_dao:    Arc<ApiUsageDao>,
    pub usage_tracker:    Arc<UsageTracker>,
    pub analytics_dao:    Arc<AnalyticsDao>,
}

/// Mobile app management.
#[derive(Clone)]
pub struct MobileState {
    pub mobile_app_dao:        Arc<MobileAppDao>,
    pub mobile_app_bundle_dao: Arc<MobileAppBundleDao>,
    pub qr_code_settings_dao:  Arc<QrCodeSettingsDao>,
    pub mobile_session_dao:    Arc<MobileSessionDao>,
}

/// Backup and restore.
#[derive(Clone)]
pub struct BackupState {
    pub backup_export_dao: Arc<BackupExportDao>,
    pub backup_export_svc: Arc<ExportService>,
    pub backup_import_svc: Arc<ImportService>,
}

/// Widgets, resources, AI model.
#[derive(Clone)]
pub struct UiState {
    pub widget_type_dao:    Arc<WidgetTypeDao>,
    pub widgets_bundle_dao: Arc<WidgetsBundleDao>,
    pub resource_dao:       Arc<ResourceDao>,
    pub ai_model_dao:       Arc<AiModelDao>,
}

// ── FromRef implementations ──────────────────────────────────────────────────
// Each impl constructs a sub-state from AppState's flat fields via Arc::clone().

impl axum::extract::FromRef<AppState> for CoreState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            pool: s.pool.clone(), config: s.config.clone(), jwt_service: s.jwt_service.clone(),
            cache: s.cache.clone(), queue_producer: s.queue_producer.clone(),
            token_blacklist: s.token_blacklist.clone(), rate_limiter: s.rate_limiter.clone(),
            audit_log_dao: s.audit_log_dao.clone(), api_key_dao: s.api_key_dao.clone(),
            tenant_profile_dao: s.tenant_profile_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for AuthState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            user_dao: s.user_dao.clone(), tenant_dao: s.tenant_dao.clone(),
            tenant_profile_dao: s.tenant_profile_dao.clone(),
            two_factor_auth_dao: s.two_factor_auth_dao.clone(),
            oauth2_registration_dao: s.oauth2_registration_dao.clone(),
            oauth2_template_dao: s.oauth2_template_dao.clone(),
            rbac_dao: s.rbac_dao.clone(), activation_service: s.activation_service.clone(),
            domain_dao: s.domain_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for DeviceState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            device_dao: s.device_dao.clone(), device_profile_dao: s.device_profile_dao.clone(),
            device_activity_dao: s.device_activity_dao.clone(), activity_tx: s.activity_tx.clone(),
            device_rpc_registry: s.device_rpc_registry.clone(),
            rpc_pending_registry: s.rpc_pending_registry.clone(), rpc_dao: s.rpc_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for TelemetryState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            ts_dao: s.ts_dao.clone(), kv_dao: s.kv_dao.clone(), ws_tx: s.ws_tx.clone(),
            ws_registry: s.ws_registry.clone(), calc_field_dao: s.calc_field_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for AlarmState {
    fn from_ref(s: &AppState) -> Self {
        Self { alarm_dao: s.alarm_dao.clone(), alarm_comment_dao: s.alarm_comment_dao.clone() }
    }
}

impl axum::extract::FromRef<AppState> for EntityState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            asset_dao: s.asset_dao.clone(), asset_profile_dao: s.asset_profile_dao.clone(),
            customer_dao: s.customer_dao.clone(), dashboard_dao: s.dashboard_dao.clone(),
            relation_dao: s.relation_dao.clone(), entity_view_dao: s.entity_view_dao.clone(),
            entity_query_dao: s.entity_query_dao.clone(), version_dao: s.version_dao.clone(),
            version_control_svc: s.version_control_svc.clone(),
            search_dao: s.search_dao.clone(), geofence_dao: s.geofence_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for NotificationState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            notification_template_dao: s.notification_template_dao.clone(),
            notification_target_dao: s.notification_target_dao.clone(),
            notification_request_dao: s.notification_request_dao.clone(),
            notification_rule_dao: s.notification_rule_dao.clone(),
            notification_inbox_dao: s.notification_inbox_dao.clone(),
            notification_channel_settings_dao: s.notification_channel_settings_dao.clone(),
            notification_delivery_dao: s.notification_delivery_dao.clone(),
            notification_service: s.notification_service.clone(),
            notification_delivery_svc: s.notification_delivery_svc.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for RuleEngineState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            rule_engine: s.rule_engine.clone(), rule_chain_dao: s.rule_chain_dao.clone(),
            re_registry: s.re_registry.clone(),
            component_descriptor_dao: s.component_descriptor_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for EdgeState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            edge_dao: s.edge_dao.clone(), edge_event_dao: s.edge_event_dao.clone(),
            edge_session_registry: s.edge_session_registry.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for AdminState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            admin_settings_dao: s.admin_settings_dao.clone(),
            usage_info_dao: s.usage_info_dao.clone(), event_dao: s.event_dao.clone(),
            housekeeper_dao: s.housekeeper_dao.clone(),
            housekeeper_service: s.housekeeper_service.clone(),
            job_scheduler_dao: s.job_scheduler_dao.clone(),
            job_scheduler_service: s.job_scheduler_service.clone(),
            queue_stats_dao: s.queue_stats_dao.clone(),
            queue_monitor_service: s.queue_monitor_service.clone(),
            queue_message_dao: s.queue_message_dao.clone(), dlq_dao: s.dlq_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for ClusterState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            cluster: s.cluster.clone(), cluster_node_dao: s.cluster_node_dao.clone(),
            cluster_monitor_service: s.cluster_monitor_service.clone(),
            cluster_partition_dao: s.cluster_partition_dao.clone(),
            partition_svc: s.partition_svc.clone(), cluster_router: s.cluster_router.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for OtaState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            ota_package_dao: s.ota_package_dao.clone(), ota_state_dao: s.ota_state_dao.clone(),
            ota_service: s.ota_service.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for BillingState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            plan_dao: s.plan_dao.clone(), subscription_dao: s.subscription_dao.clone(),
            stripe_service: s.stripe_service.clone(), api_usage_dao: s.api_usage_dao.clone(),
            usage_tracker: s.usage_tracker.clone(), analytics_dao: s.analytics_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for MobileState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            mobile_app_dao: s.mobile_app_dao.clone(),
            mobile_app_bundle_dao: s.mobile_app_bundle_dao.clone(),
            qr_code_settings_dao: s.qr_code_settings_dao.clone(),
            mobile_session_dao: s.mobile_session_dao.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for BackupState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            backup_export_dao: s.backup_export_dao.clone(),
            backup_export_svc: s.backup_export_svc.clone(),
            backup_import_svc: s.backup_import_svc.clone(),
        }
    }
}

impl axum::extract::FromRef<AppState> for UiState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            widget_type_dao: s.widget_type_dao.clone(),
            widgets_bundle_dao: s.widgets_bundle_dao.clone(),
            resource_dao: s.resource_dao.clone(), ai_model_dao: s.ai_model_dao.clone(),
        }
    }
}

/// IoT Simulator.
#[derive(Clone)]
pub struct SimulatorState {
    pub simulator_dao:     Arc<SimulatorDao>,
    pub simulator_service: Arc<SimulatorService>,
    pub device_dao:        Arc<DeviceDao>,
    pub schematic_dao:     Arc<SchematicDao>,
    pub device_template_dao: Arc<DeviceTemplateDao>,
    pub arduino_compiler:   Arc<ArduinoCompilerService>,
}

impl axum::extract::FromRef<AppState> for SimulatorState {
    fn from_ref(s: &AppState) -> Self {
        Self {
            simulator_dao: s.simulator_dao.clone(),
            simulator_service: s.simulator_service.clone(),
            device_dao: s.device_dao.clone(),
            schematic_dao: s.schematic_dao.clone(),
            device_template_dao: s.device_template_dao.clone(),
            arduino_compiler: s.arduino_compiler.clone(),
        }
    }
}
