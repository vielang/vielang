use utoipa::OpenApi;

use crate::routes::{
    devices::{DeviceResponse, CredentialsResponse, IdResponse, SaveDeviceRequest, SaveCredentialsRequest},
    alarms::AlarmResponse,
    assets::AssetResponse,
    relations::RelationResponse,
    dashboards::DashboardResponse,
    rule_chains::RuleChainResponse,
};

/// Wrapper cho PageData<DeviceResponse> để utoipa generate schema
#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct DevicePage {
    pub data: Vec<DeviceResponse>,
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
    #[serde(rename = "totalElements")]
    pub total_elements: i64,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
}

#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct AssetPage {
    pub data: Vec<AssetResponse>,
    #[serde(rename = "totalPages")]  pub total_pages: i64,
    #[serde(rename = "totalElements")] pub total_elements: i64,
    #[serde(rename = "hasNext")]     pub has_next: bool,
}

#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct AlarmPage {
    pub data: Vec<AlarmResponse>,
    #[serde(rename = "totalPages")]  pub total_pages: i64,
    #[serde(rename = "totalElements")] pub total_elements: i64,
    #[serde(rename = "hasNext")]     pub has_next: bool,
}

#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct DashboardPage {
    pub data: Vec<DashboardResponse>,
    #[serde(rename = "totalPages")]  pub total_pages: i64,
    #[serde(rename = "totalElements")] pub total_elements: i64,
    #[serde(rename = "hasNext")]     pub has_next: bool,
}

#[derive(utoipa::ToSchema, serde::Serialize)]
pub struct RuleChainPage {
    pub data: Vec<RuleChainResponse>,
    #[serde(rename = "totalPages")]  pub total_pages: i64,
    #[serde(rename = "totalElements")] pub total_elements: i64,
    #[serde(rename = "hasNext")]     pub has_next: bool,
}

#[derive(OpenApi)]
#[openapi(
    info(
        title        = "VieLang API",
        version      = "0.1.0",
        description  = "Rust re-implementation of ThingsBoard IoT Platform REST API.\n\
                        All endpoints are compatible with ThingsBoard v4.4 REST API."
    ),
    components(schemas(
        IdResponse,
        DeviceResponse,
        SaveDeviceRequest,
        CredentialsResponse,
        SaveCredentialsRequest,
        AlarmResponse,
        AssetResponse,
        RelationResponse,
        DashboardResponse,
        RuleChainResponse,
        DevicePage,
        AssetPage,
        AlarmPage,
        DashboardPage,
        RuleChainPage,
    )),
    tags(
        (name = "device",    description = "Device management — POST/GET/DELETE /api/device"),
        (name = "alarm",     description = "Alarm management — POST/GET/DELETE /api/alarm"),
        (name = "asset",     description = "Asset management — POST/GET/DELETE /api/asset"),
        (name = "relation",  description = "Entity relations — POST/GET/DELETE /api/relation"),
        (name = "dashboard",   description = "Dashboard management — POST/GET/DELETE /api/dashboard"),
        (name = "rule_chain", description = "Rule chain management — POST/GET/DELETE /api/ruleChain"),
        (name = "telemetry", description = "Telemetry & attributes — /api/plugins/telemetry/..."),
        (name = "auth",      description = "Authentication — POST /api/auth/login"),
    )
)]
pub struct ApiDoc;
