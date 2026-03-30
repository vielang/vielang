//! Main Bevy plugin — wires up all systems, resources, and events.

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use crate::{
    alarm::{update_alarm_registry, AlarmRegistry, AlarmRuleSet, evaluate_alarm_rules},
    analytics::{run_anomaly_detection, AnomalyDetector},
    asset_hierarchy::{AssetNodeData, AssetTree},
    auth::{render_audit_log, AuditLog, UserSession},
    dashboard::{
        render_dashboard, render_dashboard_editor, toggle_dashboard_editor,
        ActiveDashboard, DashboardEditorState,
    },
    scene::{
        handle_load_background, handle_remove_background, spawn_loaded_backgrounds,
        sync_background_transforms, BackgroundLoadState, BackgroundSceneRegistry,
        LoadBackgroundRequest, RemoveBackgroundRequest,
    },
    api::{ApiConfig, DeviceListCache, HistoricalDataCache, RpcResponseQueue},
    assets::ModelRegistry,
    components::{
        AlarmIndicator, DataFreshness, DeviceEntity, DeviceRpcPresets, DeviceStatus,
        SharedAttributes, TelemetryData,
    },
    events::{
        AlarmUpdate, AttributeUpdate, FetchHistoryRequest, RpcResult, SendRpcRequest,
        TelemetryUpdate, WsStatusEvent,
    },
    playback::{PlaybackState, TimeRange},
    systems::{
        animate_by_telemetry, apply_alarm_updates, apply_attribute_updates, apply_lod_visibility,
        apply_telemetry_updates, auto_save_layout, drain_history_results,
        drain_rpc_responses, drain_ws_events, evict_old_cache, handle_device_drag,
        handle_fetch_history, handle_keyboard_shortcuts, handle_rpc_requests,
        handle_screenshot_request, setup_camera, setup_lights, setup_scene, setup_ws_connection,
        spawn_device_labels, spawn_device_models, sync_url_state, update_alarm_visuals,
        update_billboards, update_device_color_by_playback, update_heatmap, update_lod,
        update_playback, update_stale_visuals, update_telemetry_history, AutoSaveTimer,
        CacheEvictionTimer, CurrentLayout, HeatmapConfig, LodLevel,
    },
    telemetry::{observe_telemetry_keys, DeviceKeyRegistry, ProfileRegistry},
    ui::{
        collect_rpc_results, emit_load_from_picker, handle_entity_right_click, handle_login_task,
        open_file_picker_native, push_alarm_notifications, render_alarm_panel, render_alarm_rules,
        render_asset_tree, render_background_settings, render_context_menu, render_device_panel,
        render_file_picker_input, render_layout_manager, render_login_screen, render_map_panel,
        render_notifications, render_rpc_log, render_timeline, toggle_alarm_rules,
        AlarmPanelState, AlarmRulesState, ContextMenuState, DeviceListFilter, DevicePanelState,
        FilePicker, LayoutManager, LayoutMode, LoginFormState, MapViewState, NotificationQueue,
        RpcLogState, SelectedDevice, TelemetryHistory,
    },
    ws::{WsConfig, WsConnectionStatus, WsEventQueue, WsSubscriptions},
    xr::{render_vr_enter_button, render_vr_hud, toggle_fps_mode, update_fps_camera,
         FpsModeState, VrModeState},
};

// ── App state ─────────────────────────────────────────────────────────────────

/// Application state — controls login → running flow.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Show login form (when no token found in config).
    #[default]
    Login,
    /// Digital twin running normally.
    Running,
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct DigitalTwinPlugin;

impl Plugin for DigitalTwinPlugin {
    fn build(&self, app: &mut App) {
        app
            // ── Camera + diagnostics plugins ────────────────────────────────
            .add_plugins(PanOrbitCameraPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())

            // ── Resources ───────────────────────────────────────────────────
            .init_resource::<WsConfig>()
            .init_resource::<WsConnectionStatus>()
            .init_resource::<WsEventQueue>()
            .init_resource::<WsSubscriptions>()
            .init_resource::<ApiConfig>()
            .init_resource::<DeviceListCache>()
            .init_resource::<SelectedDevice>()
            .init_resource::<TelemetryHistory>()
            .init_resource::<LoginFormState>()
            .init_resource::<ModelRegistry>()
            .init_resource::<RpcResponseQueue>()
            .init_resource::<ContextMenuState>()
            .init_resource::<RpcLogState>()
            .init_resource::<HistoricalDataCache>()
            .init_resource::<PlaybackState>()
            .init_resource::<TimeRange>()
            // Phase 23 resources
            .init_resource::<CurrentLayout>()
            .init_resource::<AutoSaveTimer>()
            .init_resource::<CacheEvictionTimer>()
            .init_resource::<LayoutMode>()
            .init_resource::<NotificationQueue>()
            .init_resource::<LayoutManager>()
            .init_resource::<DeviceListFilter>()
            // Phase 24-26 resources
            .init_resource::<AlarmRegistry>()
            .init_resource::<AlarmPanelState>()
            .init_resource::<DeviceKeyRegistry>()
            .init_resource::<HeatmapConfig>()
            .init_resource::<DevicePanelState>()
            // Phase 27 resources
            .init_resource::<MapViewState>()
            // Phase 28 resources
            .init_resource::<ActiveDashboard>()
            .init_resource::<DashboardEditorState>()
            // Phase 29 resources
            .init_resource::<BackgroundSceneRegistry>()
            .init_resource::<BackgroundLoadState>()
            .init_resource::<FilePicker>()
            // Phase 30 resources
            .init_resource::<AnomalyDetector>()
            // Phase 31 resources
            .init_resource::<UserSession>()
            .init_resource::<AuditLog>()
            // Phase 32 resources
            .init_resource::<FpsModeState>()
            .init_resource::<VrModeState>()
            // Phase 33 resources
            .init_resource::<AssetTree>()
            // Phase 34 resources
            .insert_resource(ProfileRegistry::default().with_builtins())
            // Phase 35 resources
            .insert_resource(AlarmRuleSet::load())
            .init_resource::<AlarmRulesState>()

            // ── Events ──────────────────────────────────────────────────────
            .add_message::<TelemetryUpdate>()
            .add_message::<AlarmUpdate>()
            .add_message::<WsStatusEvent>()
            .add_message::<AttributeUpdate>()
            .add_message::<SendRpcRequest>()
            .add_message::<RpcResult>()
            .add_message::<FetchHistoryRequest>()
            // Phase 29 messages
            .add_message::<LoadBackgroundRequest>()
            .add_message::<RemoveBackgroundRequest>()

            // ── Startup: camera + scene (always runs, state-independent) ────
            .add_systems(Startup, (setup_scene, setup_camera, setup_lights))

            // ── OnEnter Running: spawn devices + start WS ───────────────────
            .add_systems(
                OnEnter(AppState::Running),
                (
                    setup_demo_scene,
                    setup_ws_connection.after(setup_demo_scene),
                ),
            )

            // ── Login state systems ─────────────────────────────────────────
            .add_systems(
                Update,
                (render_login_screen, handle_login_task)
                    .run_if(in_state(AppState::Login)),
            )

            // ── Running: WS + ECS data pipeline ────────────────────────────
            .add_systems(
                Update,
                (
                    drain_ws_events,
                    apply_telemetry_updates.after(drain_ws_events),
                    apply_alarm_updates.after(drain_ws_events),
                    apply_attribute_updates.after(drain_ws_events),
                    update_telemetry_history.after(drain_ws_events),
                    update_alarm_visuals.after(apply_alarm_updates),
                    update_heatmap.after(apply_telemetry_updates),
                    update_stale_visuals,
                    animate_by_telemetry,
                    handle_entity_click,
                )
                .run_if(in_state(AppState::Running)),
            )

            // ── Running: asset + RPC + playback pipelines ───────────────────
            .add_systems(
                Update,
                (
                    spawn_device_models,
                    spawn_device_labels,
                    update_billboards,
                    update_lod,
                    apply_lod_visibility.after(update_lod),
                    handle_rpc_requests,
                    drain_rpc_responses,
                    update_playback,
                    handle_fetch_history,
                    drain_history_results,
                    update_device_color_by_playback,
                )
                .run_if(in_state(AppState::Running)),
            )

            // ── Running: UI panels ──────────────────────────────────────────
            .add_systems(
                Update,
                (
                    render_asset_tree,    // Phase 33 replaces flat device list
                    render_device_panel,
                    handle_entity_right_click,
                    render_context_menu,
                    collect_rpc_results,
                    render_rpc_log,
                    render_timeline,
                    render_map_panel,   // Phase 27
                )
                .run_if(in_state(AppState::Running)),
            )

            // ── Running: Phase 23 systems ───────────────────────────────────
            .add_systems(
                Update,
                (
                    handle_keyboard_shortcuts,
                    auto_save_layout,
                    handle_device_drag,
                    handle_screenshot_request,
                    push_alarm_notifications,
                    render_notifications,
                    render_layout_manager,
                    sync_url_state,
                    evict_old_cache,
                )
                .run_if(in_state(AppState::Running)),
            )

            // ── Running: Phase 24-26 + 35 systems ───────────────────────────
            .add_systems(
                Update,
                (
                    // Phase 25: Alarm management
                    update_alarm_registry.after(drain_ws_events),
                    render_alarm_panel,
                    // Phase 26: Multi-metric / key registry
                    observe_telemetry_keys.after(drain_ws_events),
                    // Phase 30: Anomaly detection
                    run_anomaly_detection.after(drain_ws_events),
                    // Phase 35: Client-side alarm rules
                    evaluate_alarm_rules.after(drain_ws_events),
                    toggle_alarm_rules,
                    render_alarm_rules,
                )
                .run_if(in_state(AppState::Running)),
            )

            // ── Running: Phase 27-28 systems ────────────────────────────────
            .add_systems(
                Update,
                (
                    // Phase 28: Configurable dashboard
                    toggle_dashboard_editor,
                    render_dashboard,
                    render_dashboard_editor.after(render_dashboard),
                )
                .run_if(in_state(AppState::Running)),
            )

            // ── Running: Phase 29 systems ────────────────────────────────────
            .add_systems(
                Update,
                (
                    open_file_picker_native,
                    emit_load_from_picker,
                    handle_load_background,
                    spawn_loaded_backgrounds,
                    handle_remove_background,
                    sync_background_transforms,
                    render_file_picker_input,
                    render_background_settings,
                )
                .run_if(in_state(AppState::Running)),
            )

            // ── Running: Phase 31-32 systems ─────────────────────────────────
            .add_systems(
                Update,
                (
                    // Phase 31: Audit log
                    render_audit_log,
                    // Phase 32: FPS + VR
                    toggle_fps_mode,
                    update_fps_camera,
                    render_vr_hud,
                    render_vr_enter_button,
                )
                .run_if(in_state(AppState::Running)),
            );
    }
}

// ── Startup: demo scene (devices + asset hierarchy) ───────────────────────────

/// Spawn demo device entities and build the asset hierarchy when no live backend
/// is available.  In production, assets + devices come from the ThingsBoard REST API.
fn setup_demo_scene(
    mut commands:     Commands,
    mut meshes:       ResMut<Assets<Mesh>>,
    mut materials:    ResMut<Assets<StandardMaterial>>,
    mut device_cache: ResMut<DeviceListCache>,
    mut tree:         ResMut<AssetTree>,
) {
    // ── Phase 33: build asset hierarchy ─────────────────────────────────────
    let site_id       = uuid::Uuid::new_v4();
    let wind_farm_id  = uuid::Uuid::new_v4();
    let sensor_hub_id = uuid::Uuid::new_v4();

    tree.nodes = vec![
        AssetNodeData {
            asset_id:   site_id,
            name:       "Hà Nội Site".into(),
            asset_type: "Site".into(),
            parent_id:  None,
        },
        AssetNodeData {
            asset_id:   wind_farm_id,
            name:       "Nhà máy điện gió".into(),
            asset_type: "Building".into(),
            parent_id:  Some(site_id),
        },
        AssetNodeData {
            asset_id:   sensor_hub_id,
            name:       "Trạm đo lường".into(),
            asset_type: "Building".into(),
            parent_id:  Some(site_id),
        },
    ];
    tree.loaded = true;

    // ── Demo devices with hierarchy: (name, type, pos, lat, lon, parent_asset_id)
    let demo: &[(&str, &str, [f32; 3], f64, f64, uuid::Uuid)] = &[
        ("Sensor A",  "temperature_sensor", [-4.0, 0.5,  0.0], 21.0285, 105.8342, sensor_hub_id),
        ("Sensor B",  "temperature_sensor", [ 0.0, 0.5,  0.0], 21.0290, 105.8350, sensor_hub_id),
        ("Sensor C",  "temperature_sensor", [ 4.0, 0.5,  0.0], 21.0280, 105.8358, sensor_hub_id),
        ("Turbine 1", "wind_turbine",        [-4.0, 0.5,  4.0], 21.0270, 105.8340, wind_farm_id),
        ("Turbine 2", "wind_turbine",        [ 4.0, 0.5,  4.0], 21.0275, 105.8360, wind_farm_id),
    ];

    // Small cube (0.2³) acts as alarm-status LED; GLTF model spawned on top by asset_system.
    let device_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.2));

    for (name, dtype, pos, lat, lon, parent_asset_id) in demo {
        let device_id = uuid::Uuid::new_v4();
        let material  = materials.add(StandardMaterial {
            base_color: Color::from(crate::components::AlarmSeverity::None.to_linear_color()),
            ..default()
        });

        commands.spawn((
            Mesh3d(device_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(pos[0], pos[1], pos[2]),
            DeviceEntity {
                device_id,
                name:            name.to_string(),
                device_type:     dtype.to_string(),
                tenant_id:       uuid::Uuid::nil(),
                latitude:        Some(*lat),
                longitude:       Some(*lon),
                parent_asset_id: Some(*parent_asset_id),
            },
            DeviceStatus::default(),
            TelemetryData::default(),
            AlarmIndicator::default(),
            DataFreshness::default(),
            LodLevel::default(),
            SharedAttributes::default(),
            DeviceRpcPresets::for_device_type(dtype),
        ));

        // Record device → asset mapping in the tree resource
        tree.device_parent.insert(device_id, *parent_asset_id);
        tracing::debug!(name = %name, id = %device_id, "Spawned demo device");
    }

    device_cache.loaded = true;
    tracing::info!(
        "Demo scene ready: {} asset nodes, {} devices",
        tree.nodes.len(),
        demo.len()
    );
}

// ── Click handler ─────────────────────────────────────────────────────────────

fn handle_entity_click(
    mut click_events: MessageReader<
        bevy::picking::events::Pointer<bevy::picking::events::Click>,
    >,
    query:        Query<&DeviceEntity>,
    mut selected: ResMut<SelectedDevice>,
) {
    for event in click_events.read() {
        // Left-click only — right-click handled by handle_entity_right_click
        if event.button != bevy::picking::pointer::PointerButton::Primary {
            continue;
        }
        if let Ok(device) = query.get(event.entity) {
            selected.entity    = Some(event.entity);
            selected.device_id = Some(device.device_id);
            selected.name      = device.name.clone();
            tracing::info!(device = %device.name, "Device selected via click");
        }
    }
}
