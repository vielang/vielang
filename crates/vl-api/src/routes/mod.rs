pub mod admin;
pub mod alarms;
pub mod connectivity;
pub mod entity_query;
pub mod api_keys;
pub mod asset_profiles;
pub mod assets;
pub mod audit;
pub mod device_profiles;
pub mod edges;
pub mod entity_views;
pub mod tenant_profiles;
pub mod auth;
pub mod customers;
pub mod dashboards;
pub mod devices;
pub mod events;
pub mod health;
pub mod notifications;
pub mod oauth2;
pub mod ota;
pub mod relations;
pub mod rpc;
pub mod rule_chains;
pub mod telemetry;
pub mod tenants;
pub mod twofa;
pub mod lwm2m;
pub mod mobile;
pub mod resources;
pub mod system_info;
pub mod users;
pub mod widgets;
pub mod ws;
pub mod component_descriptor;
pub mod housekeeper;
pub mod calculated_fields;
pub mod entity_versions;
pub mod scheduled_jobs;
pub mod queues;
pub mod cluster;
pub mod ai_model;
pub mod domain;
pub mod mail_config_template;
pub mod rule_engine;
pub mod trendz;
pub mod ui_settings;
pub mod usage_info;
pub mod oauth2_templates;
pub mod whitelabel;
pub mod partner;
pub mod rbac;
pub mod billing;
pub mod admin_analytics;
pub mod ldap;
pub mod saml;
pub mod search;
pub mod geofence;
pub mod backup;
pub mod simulator;

use axum::{Router, middleware};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
};

use crate::middleware::{audit_log_middleware, auth_middleware, correlation_id_middleware, quota_middleware, rate_limit_middleware, security_headers_middleware, track_metrics};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = build_cors_layer(&state.config.server.allowed_origins);
    // ── Public routes (no auth) ───────────────────────────────────────────────
    let public_routes = Router::new()
        .merge(health::router())
        .merge(auth::router())
        // P2: device-token firmware download (no JWT)
        .merge(ota::device_token_router())
        .merge(ws::router())
        // Phase 16: OAuth2 public + 2FA verify
        .merge(oauth2::public_router())
        .merge(twofa::public_router())
        // Phase 30: Mobile app public endpoints
        .merge(mobile::public_router())
        // White-label login theme (no auth required)
        .merge(whitelabel::public_router())
        // Public image access (no auth)
        .merge(resources::public_router())
        // Partner API (X-Partner-Key auth, not JWT)
        .merge(partner::router())
        // Server discovery for self-hosted deployments (Flutter/Angular URL entry)
        .merge(system_info::public_router())
        // Billing — plan list + Stripe webhook (no JWT)
        .merge(billing::public_router())
        .merge(billing::webhook_router())
        // P4: SAML SSO public endpoints (IdP-initiated or SP-initiated ACS)
        .merge(saml::public_router());

    // ── Protected routes (JWT or API-Key required) ────────────────────────────
    let protected_routes = Router::new()
        .merge(auth::protected_router())
        .merge(devices::router())
        .merge(device_profiles::router())
        .merge(asset_profiles::router())
        .merge(entity_views::router())
        .merge(tenant_profiles::router())
        .merge(tenants::router())
        .merge(users::router())
        .merge(customers::router())
        .merge(alarms::router())
        .merge(telemetry::router())
        .merge(assets::router())
        .merge(relations::router())
        .merge(dashboards::router())
        .merge(rule_chains::router())
        .merge(admin::router())
        // Phase 13: OTA, Events, RPC
        .merge(ota::router())
        .merge(events::router())
        .merge(rpc::router())
        // Phase 15: Notifications
        .merge(notifications::router())
        // Phase 16: OAuth2, 2FA, API Keys, Audit
        .merge(oauth2::router())
        .merge(twofa::router())
        .merge(api_keys::router())
        .merge(audit::router())
        // Phase 21: Widget Management
        .merge(widgets::router())
        // Phase 22: Resource & Image
        .merge(resources::router())
        // Phase 23: Edge Gateway
        .merge(edges::router())
        // Phase 27: Entity Query Language
        .merge(entity_query::router())
        // Phase 28: Device Connectivity
        .merge(connectivity::router())
        // Phase 29: System Admin & Monitoring
        .merge(system_info::router())
        // Phase 30: Mobile App Framework + LwM2M stub
        .merge(mobile::router())
        .merge(lwm2m::router())
        // ComponentDescriptor
        .merge(component_descriptor::router())
        // Phase 32: Housekeeper
        .merge(housekeeper::router())
        // Phase 34: Calculated Fields
        .merge(calculated_fields::router())
        // Phase 37: Entity Version Control
        .merge(entity_versions::router())
        // Phase 35: Job Scheduler
        .merge(scheduled_jobs::router())
        // Phase 36: Queue Management
        .merge(queues::router())
        // Phase 39: Cluster Mode
        .merge(cluster::router())
        // Phase 41: New stubs (AI Model, Domain, Mail Config Template, Trendz, UI Settings)
        .merge(ai_model::router())
        .merge(domain::router())
        .merge(mail_config_template::router())
        .merge(trendz::router())
        .merge(ui_settings::router())
        // Phase 52: Rule Engine Controller
        .merge(rule_engine::router())
        // Phase 54: New controllers
        .merge(usage_info::router())
        .merge(oauth2_templates::router())
        // White-label branding (JWT protected)
        .merge(whitelabel::router())
        // Phase 63: Fine-Grained RBAC
        .merge(rbac::router())
        // Phase 70: Subscription billing (protected endpoints)
        .merge(billing::router())
        // Phase 72: SaaS admin analytics
        .merge(admin_analytics::router())
        // P4: LDAP + SAML admin settings
        .merge(ldap::router())
        .merge(saml::router())
        // P5: Full-Text Search
        .merge(search::router())
        // P8: Advanced Geofencing
        .merge(geofence::router())
        // P14: Backup / Restore
        .merge(backup::router())
        // IoT Simulator
        .merge(simulator::router())
        // Middleware layers (inner → outer execution order):
        //   auth → rate_limit → quota
        .route_layer(middleware::from_fn_with_state(state.clone(), quota_middleware))
        .route_layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .nest("/api", public_routes.merge(protected_routes))
        // P9: Well-known files served at root (no /api prefix)
        .merge(mobile::well_known_router())
        .layer(middleware::from_fn_with_state(state.clone(), audit_log_middleware))
        .layer(middleware::from_fn(track_metrics))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(cors)
        // correlation_id records X-Request-ID into the active tracing span
        .layer(middleware::from_fn(correlation_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state)
}

fn build_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    use axum::http::{HeaderName, HeaderValue, Method, header};

    let methods = [
        Method::GET, Method::POST, Method::PUT, Method::DELETE,
        Method::PATCH, Method::OPTIONS, Method::HEAD,
    ];
    let headers = [
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        header::ACCEPT,
        HeaderName::from_static("x-authorization"),
    ];

    if allowed_origins.is_empty() {
        // Production default: no cross-origin (same-origin only)
        return CorsLayer::new()
            .allow_methods(methods)
            .allow_headers(headers);
    }

    // Wildcard "*" → allow any origin (useful for self-hosted + Flutter/mobile)
    // NOTE: wildcard cannot be combined with allow_credentials(true) per CORS spec
    if allowed_origins.iter().any(|o| o == "*") {
        return CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(methods)
            .allow_headers(headers);
    }

    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(methods)
        .allow_headers(headers)
        .allow_credentials(true)
}
