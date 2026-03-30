use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::simulator::{
    CreateSimulatorRequest, CreateSchematicRequest, CreateDeviceTemplateRequest,
    SaveSchematicNodeRequest, SchematicNodeConfig, SimulationStatusResponse,
    SimulatorConfig, SimulatorSchematic, DeviceTemplate, WokwiComponentInfo,
};
use vl_dao::PageData;

use crate::{
    error::ApiError,
    middleware::auth::SecurityContext,
    state::{AppState, SimulatorState},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/simulator/config", post(create_config))
        .route("/simulator/configs", get(list_configs))
        .route(
            "/simulator/config/{configId}",
            get(get_config).put(update_config).delete(delete_config),
        )
        .route("/simulator/config/{configId}/start", post(start_simulation))
        .route("/simulator/config/{configId}/stop", post(stop_simulation))
        .route("/simulator/status", get(get_all_status))
        .route(
            "/simulator/config/{configId}/status",
            get(get_config_status),
        )
        .route(
            "/simulator/device/{deviceId}/configs",
            get(get_configs_for_device),
        )
        // Phase 3: Script validation + flash
        .route("/simulator/script/validate", post(validate_script))
        .route(
            "/simulator/config/{configId}/flash",
            post(flash_script),
        )
        // Phase 4: Schematics
        .route("/simulator/schematic", post(create_schematic))
        .route("/simulator/schematics", get(list_schematics))
        .route(
            "/simulator/schematic/{schematicId}",
            get(get_schematic).put(update_schematic).delete(delete_schematic),
        )
        .route(
            "/simulator/schematic/{schematicId}/nodes",
            get(get_schematic_nodes).post(save_schematic_node),
        )
        .route(
            "/simulator/schematic/{schematicId}/node/{nodeId}",
            axum::routing::delete(delete_schematic_node),
        )
        .route("/simulator/schematic/{schematicId}/start", post(start_schematic))
        .route("/simulator/schematic/{schematicId}/stop", post(stop_schematic))
        // Device templates
        .route("/simulator/templates", get(list_templates))
        .route("/simulator/template", post(create_template))
        .route(
            "/simulator/template/{templateId}",
            get(get_template).delete(delete_template),
        )
        // Wokwi component registry (static)
        .route("/simulator/wokwi/components", get(list_wokwi_components))
        // Arduino compilation
        .route("/simulator/arduino/compile", post(compile_arduino))
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageParams {
    #[serde(default)]
    pub page: i64,
    #[serde(rename = "pageSize", default = "default_page_size")]
    pub page_size: i64,
}

fn default_page_size() -> i64 {
    10
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/simulator/config — create a simulator config (TENANT_ADMIN+)
async fn create_config(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<CreateSimulatorRequest>,
) -> Result<(StatusCode, Json<SimulatorConfig>), ApiError> {
    validate_request(&req)?;

    // Verify device exists and belongs to tenant
    let device = state
        .device_dao
        .find_by_id(req.device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Device [{}] not found", req.device_id)))?;

    if device.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    let config = state
        .simulator_dao
        .insert(ctx.tenant_id, &req)
        .await?;

    Ok((StatusCode::CREATED, Json(config)))
}

/// GET /api/simulator/configs?page=0&pageSize=10 — list configs for tenant
async fn list_configs(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<SimulatorConfig>>, ApiError> {
    let (data, total) = state
        .simulator_dao
        .find_by_tenant(ctx.tenant_id, params.page, params.page_size)
        .await?;

    let total_pages = if params.page_size > 0 {
        (total + params.page_size - 1) / params.page_size
    } else {
        0
    };
    let has_next = (params.page + 1) * params.page_size < total;

    Ok(Json(PageData {
        data,
        total_pages,
        total_elements: total,
        has_next,
    }))
}

/// GET /api/simulator/config/{configId} — get a config by id
async fn get_config(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(config_id): Path<Uuid>,
) -> Result<Json<SimulatorConfig>, ApiError> {
    let config = state
        .simulator_dao
        .find_by_id(config_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("SimulatorConfig [{}] not found", config_id))
        })?;

    if config.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    Ok(Json(config))
}

/// PUT /api/simulator/config/{configId} — update a config
async fn update_config(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(config_id): Path<Uuid>,
    Json(req): Json<CreateSimulatorRequest>,
) -> Result<Json<SimulatorConfig>, ApiError> {
    let existing = state
        .simulator_dao
        .find_by_id(config_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("SimulatorConfig [{}] not found", config_id))
        })?;

    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    validate_request(&req)?;

    let config = state.simulator_dao.update(config_id, &req).await?;
    Ok(Json(config))
}

/// DELETE /api/simulator/config/{configId} — delete a config
async fn delete_config(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(config_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let existing = state
        .simulator_dao
        .find_by_id(config_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("SimulatorConfig [{}] not found", config_id))
        })?;

    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    // Stop if running
    state.simulator_service.stop_one(config_id).await;

    state.simulator_dao.delete(config_id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/simulator/config/{configId}/start — start a simulation
async fn start_simulation(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(config_id): Path<Uuid>,
) -> Result<Json<SimulationStatusResponse>, ApiError> {
    let config = state
        .simulator_dao
        .find_by_id(config_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("SimulatorConfig [{}] not found", config_id))
        })?;

    if config.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    // Enable in DB
    state.simulator_dao.set_enabled(config_id, true).await?;

    // Start the runtime task
    state.simulator_service.start_one(config).await;

    let status = state.simulator_service.get_status(config_id).await;
    Ok(Json(status))
}

/// POST /api/simulator/config/{configId}/stop — stop a simulation
async fn stop_simulation(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(config_id): Path<Uuid>,
) -> Result<Json<SimulationStatusResponse>, ApiError> {
    let config = state
        .simulator_dao
        .find_by_id(config_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("SimulatorConfig [{}] not found", config_id))
        })?;

    if config.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    // Disable in DB
    state.simulator_dao.set_enabled(config_id, false).await?;

    // Stop the runtime task
    state.simulator_service.stop_one(config_id).await;

    let status = state.simulator_service.get_status(config_id).await;
    Ok(Json(status))
}

/// GET /api/simulator/status — all running simulations for tenant
async fn get_all_status(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<SimulationStatusResponse>>, ApiError> {
    let statuses = state
        .simulator_service
        .get_status_for_tenant(ctx.tenant_id)
        .await;
    Ok(Json(statuses))
}

/// GET /api/simulator/config/{configId}/status — status for a single config
async fn get_config_status(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(config_id): Path<Uuid>,
) -> Result<Json<SimulationStatusResponse>, ApiError> {
    // Verify ownership
    let config = state
        .simulator_dao
        .find_by_id(config_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("SimulatorConfig [{}] not found", config_id))
        })?;

    if config.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    let status = state.simulator_service.get_status(config_id).await;
    Ok(Json(status))
}

/// GET /api/simulator/device/{deviceId}/configs — configs for a device
async fn get_configs_for_device(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<Vec<SimulatorConfig>>, ApiError> {
    // Verify device belongs to tenant
    let device = state
        .device_dao
        .find_by_id(device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Device [{}] not found", device_id)))?;

    if device.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    let configs = state.simulator_dao.find_by_device(device_id).await?;
    Ok(Json(configs))
}

// ── Phase 3: Script validation + flash ────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScriptValidateRequest {
    script: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScriptValidateResponse {
    valid: bool,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlashScriptRequest {
    script: String,
}

/// POST /api/simulator/script/validate — validate Rhai script (compile-only)
async fn validate_script(
    Extension(_ctx): Extension<SecurityContext>,
    Json(req): Json<ScriptValidateRequest>,
) -> Result<Json<ScriptValidateResponse>, ApiError> {
    let engine = rhai::Engine::new();
    match engine.compile(&req.script) {
        Ok(_) => Ok(Json(ScriptValidateResponse {
            valid: true,
            error: None,
        })),
        Err(e) => Ok(Json(ScriptValidateResponse {
            valid: false,
            error: Some(e.to_string()),
        })),
    }
}

/// POST /api/simulator/config/{configId}/flash — apply script and restart simulation
async fn flash_script(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(config_id): Path<Uuid>,
    Json(req): Json<FlashScriptRequest>,
) -> Result<Json<SimulationStatusResponse>, ApiError> {
    let config = state
        .simulator_dao
        .find_by_id(config_id)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!("SimulatorConfig [{}] not found", config_id))
        })?;

    if config.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    // Validate script first
    let engine = rhai::Engine::new();
    if let Err(e) = engine.compile(&req.script) {
        return Err(ApiError::BadRequest(format!("Invalid script: {}", e)));
    }

    // Update config with new script
    let update_req = CreateSimulatorRequest {
        device_id: config.device_id,
        name: config.name.clone(),
        interval_ms: config.interval_ms,
        telemetry_schema: config.telemetry_schema.clone(),
        script: Some(req.script),
        enabled: true,
        transport_mode: config.transport_mode.clone(),
    };
    let updated = state.simulator_dao.update(config_id, &update_req).await?;

    // Restart: stop then start
    state.simulator_service.stop_one(config_id).await;
    state.simulator_service.start_one(updated).await;

    let status = state.simulator_service.get_status(config_id).await;
    Ok(Json(status))
}

// ── Phase 4: Schematic handlers ───────────────────────────────────────────────

/// POST /api/simulator/schematic
async fn create_schematic(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<CreateSchematicRequest>,
) -> Result<(StatusCode, Json<SimulatorSchematic>), ApiError> {
    if req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let schematic = state.schematic_dao.insert(ctx.tenant_id, &req).await?;
    Ok((StatusCode::CREATED, Json(schematic)))
}

/// GET /api/simulator/schematics
async fn list_schematics(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<PageParams>,
) -> Result<Json<PageData<SimulatorSchematic>>, ApiError> {
    let (data, total) = state
        .schematic_dao
        .find_by_tenant(ctx.tenant_id, params.page, params.page_size)
        .await?;

    let total_pages = if params.page_size > 0 {
        (total + params.page_size - 1) / params.page_size
    } else {
        0
    };
    let has_next = (params.page + 1) * params.page_size < total;

    Ok(Json(PageData { data, total_pages, total_elements: total, has_next }))
}

/// GET /api/simulator/schematic/{schematicId}
async fn get_schematic(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(schematic_id): Path<Uuid>,
) -> Result<Json<SimulatorSchematic>, ApiError> {
    let s = state.schematic_dao.find_by_id(schematic_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Schematic [{}] not found", schematic_id)))?;
    if s.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }
    Ok(Json(s))
}

/// PUT /api/simulator/schematic/{schematicId}
async fn update_schematic(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(schematic_id): Path<Uuid>,
    Json(req): Json<CreateSchematicRequest>,
) -> Result<Json<SimulatorSchematic>, ApiError> {
    let existing = state.schematic_dao.find_by_id(schematic_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Schematic [{}] not found", schematic_id)))?;
    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }
    let updated = state.schematic_dao.update(schematic_id, &req).await?;
    Ok(Json(updated))
}

/// DELETE /api/simulator/schematic/{schematicId}
async fn delete_schematic(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(schematic_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let existing = state.schematic_dao.find_by_id(schematic_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Schematic [{}] not found", schematic_id)))?;
    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }
    state.schematic_dao.delete(schematic_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/simulator/schematic/{schematicId}/nodes
async fn get_schematic_nodes(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(schematic_id): Path<Uuid>,
) -> Result<Json<Vec<SchematicNodeConfig>>, ApiError> {
    let existing = state.schematic_dao.find_by_id(schematic_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Schematic [{}] not found", schematic_id)))?;
    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }
    let nodes = state.schematic_dao.find_nodes(schematic_id).await?;
    Ok(Json(nodes))
}

/// POST /api/simulator/schematic/{schematicId}/nodes
async fn save_schematic_node(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(schematic_id): Path<Uuid>,
    Json(req): Json<SaveSchematicNodeRequest>,
) -> Result<Json<SchematicNodeConfig>, ApiError> {
    let existing = state.schematic_dao.find_by_id(schematic_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Schematic [{}] not found", schematic_id)))?;
    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }
    let node = state.schematic_dao.save_node(schematic_id, &req).await?;
    Ok(Json(node))
}

/// DELETE /api/simulator/schematic/{schematicId}/node/{nodeId}
async fn delete_schematic_node(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path((schematic_id, node_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let existing = state.schematic_dao.find_by_id(schematic_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Schematic [{}] not found", schematic_id)))?;
    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }
    state.schematic_dao.delete_node(node_id).await?;
    Ok(StatusCode::OK)
}

/// POST /api/simulator/schematic/{schematicId}/start — start all linked simulations
async fn start_schematic(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(schematic_id): Path<Uuid>,
) -> Result<Json<Vec<SimulationStatusResponse>>, ApiError> {
    let existing = state.schematic_dao.find_by_id(schematic_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Schematic [{}] not found", schematic_id)))?;
    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    let nodes = state.schematic_dao.find_nodes(schematic_id).await?;
    let mut statuses = Vec::new();
    for node in &nodes {
        if let Some(config_id) = node.simulator_config_id {
            if let Some(cfg) = state.simulator_dao.find_by_id(config_id).await? {
                state.simulator_dao.set_enabled(config_id, true).await?;
                state.simulator_service.start_one(cfg).await;
                statuses.push(state.simulator_service.get_status(config_id).await);
            }
        }
    }
    Ok(Json(statuses))
}

/// POST /api/simulator/schematic/{schematicId}/stop — stop all linked simulations
async fn stop_schematic(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(schematic_id): Path<Uuid>,
) -> Result<Json<Vec<SimulationStatusResponse>>, ApiError> {
    let existing = state.schematic_dao.find_by_id(schematic_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Schematic [{}] not found", schematic_id)))?;
    if existing.tenant_id != ctx.tenant_id && !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("Access denied".into()));
    }

    let nodes = state.schematic_dao.find_nodes(schematic_id).await?;
    let mut statuses = Vec::new();
    for node in &nodes {
        if let Some(config_id) = node.simulator_config_id {
            state.simulator_dao.set_enabled(config_id, false).await?;
            state.simulator_service.stop_one(config_id).await;
            statuses.push(state.simulator_service.get_status(config_id).await);
        }
    }
    Ok(Json(statuses))
}

// ── Device Templates ──────────────────────────────────────────────────────────

/// GET /api/simulator/templates — builtin + tenant custom templates
async fn list_templates(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<DeviceTemplate>>, ApiError> {
    let templates = state.device_template_dao.find_all_for_tenant(ctx.tenant_id).await?;
    Ok(Json(templates))
}

/// POST /api/simulator/template — create a custom template
async fn create_template(
    State(state): State<SimulatorState>,
    Extension(ctx): Extension<SecurityContext>,
    Json(req): Json<CreateDeviceTemplateRequest>,
) -> Result<(StatusCode, Json<DeviceTemplate>), ApiError> {
    let template = state.device_template_dao.insert(ctx.tenant_id, &req).await?;
    Ok((StatusCode::CREATED, Json(template)))
}

/// GET /api/simulator/template/{templateId}
async fn get_template(
    State(state): State<SimulatorState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<DeviceTemplate>, ApiError> {
    let t = state.device_template_dao.find_by_id(template_id).await?
        .ok_or_else(|| ApiError::NotFound(format!("Template [{}] not found", template_id)))?;
    Ok(Json(t))
}

/// DELETE /api/simulator/template/{templateId}
async fn delete_template(
    State(state): State<SimulatorState>,
    Extension(_ctx): Extension<SecurityContext>,
    Path(template_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.device_template_dao.delete(template_id).await?;
    Ok(StatusCode::OK)
}

// ── Wokwi Component Registry (static) ────────────────────────────────────────

/// GET /api/simulator/wokwi/components — static component registry
async fn list_wokwi_components(
    Extension(_ctx): Extension<SecurityContext>,
) -> Json<Vec<WokwiComponentInfo>> {
    let components = vec![
        WokwiComponentInfo { component_type: "wokwi-dht22".into(), category: "sensor".into(), label: "DHT22 (Temp & Humidity)".into(), telemetry_keys: vec!["temperature".into(), "humidity".into()] },
        WokwiComponentInfo { component_type: "wokwi-hc-sr04".into(), category: "sensor".into(), label: "HC-SR04 (Ultrasonic)".into(), telemetry_keys: vec!["distance".into()] },
        WokwiComponentInfo { component_type: "wokwi-pir-motion-sensor".into(), category: "sensor".into(), label: "PIR Motion Sensor".into(), telemetry_keys: vec!["motion".into()] },
        WokwiComponentInfo { component_type: "wokwi-led".into(), category: "display".into(), label: "LED".into(), telemetry_keys: vec!["brightness".into()] },
        WokwiComponentInfo { component_type: "wokwi-lcd1602".into(), category: "display".into(), label: "LCD 16x2".into(), telemetry_keys: vec!["display_text".into()] },
        WokwiComponentInfo { component_type: "wokwi-neopixel".into(), category: "display".into(), label: "NeoPixel RGB LED".into(), telemetry_keys: vec!["color_r".into(), "color_g".into(), "color_b".into()] },
        WokwiComponentInfo { component_type: "wokwi-servo".into(), category: "actuator".into(), label: "Servo Motor".into(), telemetry_keys: vec!["angle".into()] },
        WokwiComponentInfo { component_type: "wokwi-buzzer".into(), category: "actuator".into(), label: "Buzzer".into(), telemetry_keys: vec!["alarm".into()] },
        WokwiComponentInfo { component_type: "wokwi-pushbutton".into(), category: "input".into(), label: "Push Button".into(), telemetry_keys: vec!["pressed".into()] },
        WokwiComponentInfo { component_type: "wokwi-potentiometer".into(), category: "input".into(), label: "Potentiometer".into(), telemetry_keys: vec!["value".into()] },
        WokwiComponentInfo { component_type: "wokwi-slide-switch".into(), category: "input".into(), label: "Slide Switch".into(), telemetry_keys: vec!["switch_state".into()] },
        WokwiComponentInfo { component_type: "wokwi-esp32-devkit-v1".into(), category: "microcontroller".into(), label: "ESP32 DevKit V1".into(), telemetry_keys: vec![] },
        WokwiComponentInfo { component_type: "wokwi-arduino-uno".into(), category: "microcontroller".into(), label: "Arduino Uno".into(), telemetry_keys: vec![] },
        WokwiComponentInfo { component_type: "wokwi-resistor".into(), category: "passive".into(), label: "Resistor".into(), telemetry_keys: vec![] },
    ];
    Json(components)
}

// ── Arduino Compilation ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArduinoCompileRequest {
    code: String,
    board: String,
}

/// POST /api/simulator/arduino/compile — compile Arduino sketch
async fn compile_arduino(
    State(state): State<SimulatorState>,
    Extension(_ctx): Extension<SecurityContext>,
    Json(req): Json<ArduinoCompileRequest>,
) -> Result<Json<crate::services::arduino_compiler::CompileResult>, ApiError> {
    if req.code.trim().is_empty() {
        return Err(ApiError::BadRequest("code is required".into()));
    }
    let allowed_boards = ["arduino:avr:uno", "arduino:avr:mega", "arduino:avr:nano"];
    if !allowed_boards.contains(&req.board.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "Unsupported board: {}. Supported: {:?}",
            req.board, allowed_boards
        )));
    }
    let result = state.arduino_compiler.compile(&req.code, &req.board).await;
    Ok(Json(result))
}

// ── Validation ───────────────────────────────────────────────────────────────

fn validate_request(req: &CreateSimulatorRequest) -> Result<(), ApiError> {
    if req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    if req.interval_ms < 100 {
        return Err(ApiError::BadRequest(
            "intervalMs must be at least 100ms".into(),
        ));
    }
    if req.telemetry_schema.is_empty() && req.script.is_none() {
        return Err(ApiError::BadRequest(
            "telemetrySchema or script is required".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vl_core::entities::simulator::TransportMode;

    #[test]
    fn router_creation_does_not_panic() {
        let _r: Router<AppState> = router();
    }

    #[test]
    fn page_params_deserializes_defaults() {
        let params: PageParams = serde_json::from_str("{}").unwrap();
        assert_eq!(params.page, 0);
        assert_eq!(params.page_size, 10);
    }

    #[test]
    fn page_params_deserializes_camel_case() {
        let params: PageParams =
            serde_json::from_str(r#"{"page": 2, "pageSize": 25}"#).unwrap();
        assert_eq!(params.page, 2);
        assert_eq!(params.page_size, 25);
    }

    #[test]
    fn script_validate_response_serializes_camel_case() {
        let resp = ScriptValidateResponse {
            valid: true,
            error: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["valid"], true);
        assert!(json["error"].is_null());
    }

    #[test]
    fn script_validate_response_with_error() {
        let resp = ScriptValidateResponse {
            valid: false,
            error: Some("syntax error".into()),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["valid"], false);
        assert_eq!(json["error"], "syntax error");
    }

    #[test]
    fn validate_request_rejects_empty_name() {
        let req = CreateSimulatorRequest {
            device_id: Uuid::new_v4(),
            name: "  ".into(),
            interval_ms: 1000,
            telemetry_schema: vec![],
            script: Some("let x = 1;".into()),
            enabled: true,
            transport_mode: TransportMode::default(),
        };
        assert!(validate_request(&req).is_err());
    }

    #[test]
    fn validate_request_rejects_low_interval() {
        let req = CreateSimulatorRequest {
            device_id: Uuid::new_v4(),
            name: "test".into(),
            interval_ms: 50,
            telemetry_schema: vec![],
            script: Some("let x = 1;".into()),
            enabled: true,
            transport_mode: TransportMode::default(),
        };
        assert!(validate_request(&req).is_err());
    }

    #[test]
    fn validate_request_rejects_no_schema_no_script() {
        let req = CreateSimulatorRequest {
            device_id: Uuid::new_v4(),
            name: "test".into(),
            interval_ms: 1000,
            telemetry_schema: vec![],
            script: None,
            enabled: true,
            transport_mode: TransportMode::default(),
        };
        assert!(validate_request(&req).is_err());
    }

    #[test]
    fn validate_request_accepts_valid_with_script() {
        let req = CreateSimulatorRequest {
            device_id: Uuid::new_v4(),
            name: "test".into(),
            interval_ms: 500,
            telemetry_schema: vec![],
            script: Some("let x = 42;".into()),
            enabled: true,
            transport_mode: TransportMode::default(),
        };
        assert!(validate_request(&req).is_ok());
    }
}
