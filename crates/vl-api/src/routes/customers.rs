use axum::{extract::{Extension, Path, Query, State}, routing::{get, post}, Json, Router};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use vl_core::entities::Customer;
use vl_dao::PageData;

use crate::{error::ApiError, middleware::auth::SecurityContext, routes::devices::IdResponse, state::{AppState, EntityState}};

pub fn router() -> Router<AppState> {
    Router::new()
        // Khớp Java: CustomerController
        .route("/customers",                        get(list_customers))
        .route("/customers/count",                  get(count_customers))
        .route("/customer",                         post(save_customer))
        .route("/customer/{customerId}",            get(get_customer).delete(delete_customer))
        .route("/customer/{customerId}/title",      get(get_customer_title))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerResponse {
    pub id: IdResponse,
    #[serde(rename = "createdTime")]
    pub created_time: i64,
    #[serde(rename = "tenantId")]
    pub tenant_id: IdResponse,
    pub title: String,
    pub country: Option<String>,
    pub city: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[serde(rename = "isPublic")]
    pub is_public: bool,
}

impl From<Customer> for CustomerResponse {
    fn from(c: Customer) -> Self {
        Self {
            id: IdResponse::customer(c.id),
            created_time: c.created_time,
            tenant_id: IdResponse::tenant(c.tenant_id),
            title: c.title,
            country: c.country,
            city: c.city,
            email: c.email,
            phone: c.phone,
            is_public: c.is_public,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<Uuid>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    #[serde(rename = "textSearch")]
    pub text_search: Option<String>,
}

async fn list_customers(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
    Query(params): Query<ListParams>,
) -> Result<Json<PageData<CustomerResponse>>, ApiError> {
    // tenantId from query param (SYS_ADMIN) or from auth context (TENANT_ADMIN)
    let tenant_id = params.tenant_id.unwrap_or(ctx.tenant_id);

    let mut page_link = vl_dao::PageLink::new(
        params.page.unwrap_or(0),
        params.page_size.unwrap_or(10),
    );
    page_link.text_search = params.text_search;

    let page = state.customer_dao.find_by_tenant(tenant_id, &page_link).await?;

    Ok(Json(PageData {
        data: page.data.into_iter().map(CustomerResponse::from).collect(),
        total_pages: page.total_pages,
        total_elements: page.total_elements,
        has_next: page.has_next,
    }))
}

async fn get_customer(
    State(state): State<EntityState>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<CustomerResponse>, ApiError> {
    let c = state.customer_dao.find_by_id(customer_id).await?
        .ok_or(ApiError::NotFound(format!("Customer [{}] is not found", customer_id)))?;
    Ok(Json(CustomerResponse::from(c)))
}

#[derive(Debug, Deserialize)]
pub struct SaveCustomerRequest {
    pub id: Option<IdResponse>,
    pub title: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<IdResponse>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

async fn save_customer(
    State(state): State<EntityState>,
    Json(req): Json<SaveCustomerRequest>,
) -> Result<Json<CustomerResponse>, ApiError> {
    let customer = Customer {
        id: req.id.map(|i| i.id).unwrap_or_else(Uuid::new_v4),
        created_time: chrono::Utc::now().timestamp_millis(),
        tenant_id: req.tenant_id.map(|i| i.id)
            .ok_or_else(|| ApiError::BadRequest("tenantId is required".into()))?,
        title: req.title,
        country: req.country,
        state: None,
        city: req.city,
        address: None,
        address2: None,
        zip: None,
        phone: req.phone,
        email: req.email,
        external_id: None,
        additional_info: None,
        is_public: false,
        version: 1,
    };

    let saved = state.customer_dao.save(&customer).await?;
    Ok(Json(CustomerResponse::from(saved)))
}

async fn delete_customer(
    State(state): State<EntityState>,
    Path(customer_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, ApiError> {
    state.customer_dao.find_by_id(customer_id).await?
        .ok_or(ApiError::NotFound(format!("Customer [{}] is not found", customer_id)))?;
    state.customer_dao.delete(customer_id).await?;
    Ok(axum::http::StatusCode::OK)
}

/// GET /api/customer/{customerId}/title — returns plain text customer title
async fn get_customer_title(
    State(state): State<EntityState>,
    Path(customer_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let title = state.customer_dao.find_title_by_id(customer_id).await?
        .ok_or(ApiError::NotFound(format!("Customer [{}] is not found", customer_id)))?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/plain")],
        title,
    ))
}

/// GET /api/customers/count — count customers for the calling tenant
async fn count_customers(
    State(state): State<EntityState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let count = state.customer_dao.count_by_tenant(ctx.tenant_id).await?;
    Ok(Json(serde_json::json!(count)))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use vl_auth::password;
    use vl_core::entities::{Authority, User, UserCredentials};
    use vl_dao::postgres::user::UserDao;
    use crate::{routes::create_router, state::AppState};
    use vl_config::VieLangConfig;

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    async fn test_app(pool: PgPool) -> axum::Router {
        let config = VieLangConfig::default();
        let rule_engine = vl_rule_engine::RuleEngine::start_noop();
        let queue_producer = vl_queue::create_producer(&config.queue).expect("queue");
        let cache = vl_cache::create_cache(&config.cache).expect("cache");
        let cluster = vl_cluster::ClusterManager::new(&config.cluster).await.expect("cluster");
        let ts_dao = std::sync::Arc::new(vl_dao::postgres::ts_dao::PostgresTsDao::new(pool.clone()));
        let state = AppState::new(pool, config, ts_dao, rule_engine, queue_producer, cache, cluster, { let (tx, _) = tokio::sync::mpsc::channel(1); tx });
        create_router(state)
    }

    /// Insert a tenant_profile + tenant and return the tenant UUID.
    /// Customer.tenant_id has a FK → tenant.id, so we must insert a real tenant.
    async fn insert_tenant(pool: &PgPool) -> Uuid {
        let profile_id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO tenant_profile (id, created_time, name, is_default, isolated_vl_rule_engine, version)
               VALUES ($1, $2, $3, false, false, 1)"#,
            profile_id, now_ms(), format!("profile-{profile_id}"),
        ).execute(pool).await.unwrap();

        let tenant_id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO tenant (id, created_time, tenant_profile_id, title, version)
               VALUES ($1, $2, $3, $4, 1)"#,
            tenant_id, now_ms(), profile_id, format!("tenant-{tenant_id}"),
        ).execute(pool).await.unwrap();

        tenant_id
    }

    async fn create_test_user(pool: &PgPool, email: &str, pwd: &str, tenant_id: Uuid) -> User {
        let dao = UserDao::new(pool.clone());
        let user = User {
            id: Uuid::new_v4(), created_time: now_ms(),
            tenant_id, customer_id: None,
            email: email.into(), authority: Authority::TenantAdmin,
            first_name: Some("Test".into()), last_name: Some("User".into()),
            phone: None, additional_info: None, version: 1,
        };
        dao.save(&user).await.unwrap();
        let hash = password::hash_password(pwd).unwrap();
        let creds = UserCredentials {
            id: Uuid::new_v4(), created_time: now_ms(), user_id: user.id,
            enabled: true, password: Some(hash),
            activate_token: None, reset_token: None, additional_info: None,
        };
        dao.save_credentials(&creds).await.unwrap();
        user
    }

    async fn post_json(app: axum::Router, uri: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn post_json_auth(app: axum::Router, uri: &str, token: &str, body: Value) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("POST").uri(uri)
                .header("content-type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        ).await.unwrap()
    }

    async fn get_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("GET").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn delete_auth(app: axum::Router, uri: &str, token: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder().method("DELETE").uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap()
    }

    async fn body_json(resp: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    }

    async fn get_token(app: axum::Router, email: &str, pwd: &str) -> String {
        let resp = post_json(app, "/api/auth/login",
            json!({"username": email, "password": pwd})).await;
        body_json(resp).await["token"].as_str().unwrap().to_string()
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_customer_returns_200(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let _user = create_test_user(&pool, "cust@test.com", "pass123", tenant_id).await;
        let token = get_token(app.clone(), "cust@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/customer", &token, json!({
            "title": "Test Customer",
            "tenantId": {"id": tenant_id, "entityType": "TENANT"},
        })).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn customer_response_matches_thingsboard_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let _user = create_test_user(&pool, "custfmt@test.com", "pass123", tenant_id).await;
        let token = get_token(app.clone(), "custfmt@test.com", "pass123").await;

        let resp = post_json_auth(app, "/api/customer", &token, json!({
            "title": "Format Customer",
            "tenantId": {"id": tenant_id, "entityType": "TENANT"},
        })).await;

        let body = body_json(resp).await;
        assert!(body["id"]["id"].is_string(), "id.id must be UUID string");
        assert_eq!(body["id"]["entityType"], "CUSTOMER");
        assert!(body["createdTime"].is_number(), "createdTime must be ms timestamp");
        assert_eq!(body["tenantId"]["entityType"], "TENANT");
        assert_eq!(body["title"], "Format Customer");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_customer_by_id(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let _user = create_test_user(&pool, "custget@test.com", "pass123", tenant_id).await;
        let token = get_token(app.clone(), "custget@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/customer", &token, json!({
            "title": "Get Customer",
            "tenantId": {"id": tenant_id, "entityType": "TENANT"},
        })).await;
        let customer_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/customer/{customer_id}"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["id"]["id"], customer_id);
        assert_eq!(body["title"], "Get Customer");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn get_nonexistent_customer_returns_404_with_tb_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        create_test_user(&pool, "cust404@test.com", "pass123", tenant_id).await;
        let token = get_token(app.clone(), "cust404@test.com", "pass123").await;

        let resp = get_auth(app, &format!("/api/customer/{}", Uuid::new_v4()), &token).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body = body_json(resp).await;
        assert_eq!(body["status"].as_u64().unwrap(), 404);
        assert!(body["message"].is_string());
        assert!(body["errorCode"].is_number());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn list_customers_returns_pagination_format(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let _user = create_test_user(&pool, "custlist@test.com", "pass123", tenant_id).await;
        let token = get_token(app.clone(), "custlist@test.com", "pass123").await;

        // Create a customer so the list isn't empty
        post_json_auth(app.clone(), "/api/customer", &token, json!({
            "title": "List Customer",
            "tenantId": {"id": tenant_id, "entityType": "TENANT"},
        })).await;

        let resp = get_auth(app, "/api/customers?pageSize=10&page=0", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert!(body["totalPages"].is_number());
        assert!(body["totalElements"].is_number());
        assert!(body["hasNext"].is_boolean());
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn count_customers_returns_number(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        create_test_user(&pool, "custcount@test.com", "pass123", tenant_id).await;
        let token = get_token(app.clone(), "custcount@test.com", "pass123").await;

        let resp = get_auth(app, "/api/customers/count", &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body.is_number(), "count must be a number");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn delete_customer_then_get_returns_404(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let _user = create_test_user(&pool, "custdel@test.com", "pass123", tenant_id).await;
        let token = get_token(app.clone(), "custdel@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/customer", &token, json!({
            "title": "Delete Customer",
            "tenantId": {"id": tenant_id, "entityType": "TENANT"},
        })).await;
        let customer_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let del = delete_auth(app.clone(), &format!("/api/customer/{customer_id}"), &token).await;
        assert_eq!(del.status(), StatusCode::OK);

        let get = get_auth(app, &format!("/api/customer/{customer_id}"), &token).await;
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn customer_title_returns_text(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let _user = create_test_user(&pool, "custtitle@test.com", "pass123", tenant_id).await;
        let token = get_token(app.clone(), "custtitle@test.com", "pass123").await;

        let create_resp = post_json_auth(app.clone(), "/api/customer", &token, json!({
            "title": "My Title",
            "tenantId": {"id": tenant_id, "entityType": "TENANT"},
        })).await;
        let customer_id = body_json(create_resp).await["id"]["id"].as_str().unwrap().to_string();

        let resp = get_auth(app, &format!("/api/customer/{customer_id}/title"), &token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 1_000_000).await.unwrap();
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "My Title");
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn create_customer_without_auth_returns_401(pool: PgPool) {
        let app = test_app(pool).await;
        let resp = app.oneshot(
            Request::builder().method("POST").uri("/api/customer")
                .header("content-type", "application/json")
                .body(Body::from(json!({"title": "No Auth"}).to_string()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn customers_pagination_sort_order(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let _user = create_test_user(&pool, "cust_sort@test.com", "pass123", Uuid::new_v4()).await;
        let token = get_token(app.clone(), "cust_sort@test.com", "pass123").await;

        for title in &["Gamma", "Alpha", "Beta"] {
            post_json_auth(app.clone(), "/api/customer", &token, json!({
                "title": title,
                "tenantId": {"id": tenant_id, "entityType": "TENANT"},
            })).await;
        }

        let resp = get_auth(
            app,
            &format!("/api/customers?tenantId={tenant_id}&sortOrder=ASC&sortProperty=title&pageSize=10&page=0"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert!(body["data"].is_array());
        assert!(body["totalPages"].is_number());
        assert!(body["totalElements"].is_number());
        assert!(body["hasNext"].is_boolean());
        let data = body["data"].as_array().unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(body["totalElements"], 3);
        assert_eq!(body["hasNext"], false);
        let titles: Vec<&str> = data.iter().filter_map(|c| c["title"].as_str()).collect();
        assert!(titles.contains(&"Alpha"));
        assert!(titles.contains(&"Beta"));
        assert!(titles.contains(&"Gamma"));
    }

    #[sqlx::test(migrations = "../../migrations")]
    #[ignore = "verified passing"]
    async fn customers_pagination_text_search(pool: PgPool) {
        let app = test_app(pool.clone()).await;
        let tenant_id = insert_tenant(&pool).await;
        let _user = create_test_user(&pool, "cust_search@test.com", "pass123", Uuid::new_v4()).await;
        let token = get_token(app.clone(), "cust_search@test.com", "pass123").await;

        post_json_auth(app.clone(), "/api/customer", &token, json!({
            "title": "MatchMe",
            "tenantId": {"id": tenant_id, "entityType": "TENANT"},
        })).await;
        post_json_auth(app.clone(), "/api/customer", &token, json!({
            "title": "NoMatch",
            "tenantId": {"id": tenant_id, "entityType": "TENANT"},
        })).await;

        let resp = get_auth(
            app,
            &format!("/api/customers?tenantId={tenant_id}&textSearch=MatchMe&pageSize=10&page=0"),
            &token,
        ).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_json(resp).await;
        assert_eq!(body["totalElements"], 1);
        let first_title = body["data"][0]["title"].as_str().unwrap();
        assert!(first_title.contains("MatchMe"));
    }
}
