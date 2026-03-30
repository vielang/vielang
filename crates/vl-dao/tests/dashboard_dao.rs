/// Integration tests for DashboardDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::HomeDashboardInfo;
use vl_dao::{postgres::dashboard::DashboardDao, DaoError, PageLink};

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let dashboard = helpers::make_dashboard(tenant_id);

    let saved = dao.save(&dashboard).await.unwrap();
    assert_eq!(saved.id, dashboard.id);
    assert_eq!(saved.title, dashboard.title);
    assert_eq!(saved.tenant_id, tenant_id);

    let found = dao.find_by_id(dashboard.id).await.unwrap().unwrap();
    assert_eq!(found.id, saved.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = DashboardDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_dashboard_increments_version(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let mut dashboard = helpers::make_dashboard(tenant_id);

    dao.save(&dashboard).await.unwrap();
    dashboard.title = Some("Updated Title".into());
    let updated = dao.save(&dashboard).await.unwrap();

    assert_eq!(updated.title.as_deref(), Some("Updated Title"));
    assert_eq!(updated.version, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_dashboard(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let dashboard = helpers::make_dashboard(tenant_id);

    dao.save(&dashboard).await.unwrap();
    dao.delete(dashboard.id).await.unwrap();

    assert!(dao.find_by_id(dashboard.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_nonexistent_returns_not_found(pool: PgPool) {
    let dao = DashboardDao::new(pool);
    let result = dao.delete(Uuid::new_v4()).await;
    assert!(matches!(result, Err(DaoError::NotFound)));
}

// ── Pagination tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for i in 0..5u32 {
        let mut d = helpers::make_dashboard(tenant_id);
        d.title = Some(format!("Dashboard-{i}"));
        dao.save(&d).await.unwrap();
    }

    let page0 = dao.find_by_tenant(tenant_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page0.total_elements, 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);

    let page1 = dao.find_by_tenant(tenant_id, &PageLink::new(1, 3)).await.unwrap();
    assert_eq!(page1.data.len(), 2);
    assert!(!page1.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_text_search(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for title in ["Main Overview", "Sensor Dashboard", "Sensor Alerts"] {
        let mut d = helpers::make_dashboard(tenant_id);
        d.title = Some(title.into());
        dao.save(&d).await.unwrap();
    }

    let mut link = PageLink::new(0, 10);
    link.text_search = Some("sensor".into());
    let page = dao.find_by_tenant(tenant_id, &link).await.unwrap();
    assert_eq!(page.total_elements, 2);
}

// ── Mobile filter tests ──────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_with_mobile_filter(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    // Visible on mobile
    let mut visible = helpers::make_dashboard(tenant_id);
    visible.title = Some("Mobile Visible".into());
    visible.mobile_hide = false;
    dao.save(&visible).await.unwrap();

    // Hidden on mobile
    let mut hidden = helpers::make_dashboard(tenant_id);
    hidden.title = Some("Mobile Hidden".into());
    hidden.mobile_hide = true;
    dao.save(&hidden).await.unwrap();

    // No filter — both visible
    let all = dao.find_by_tenant_with_mobile_filter(tenant_id, None, &PageLink::new(0, 10))
        .await.unwrap();
    assert_eq!(all.total_elements, 2);

    // Mobile only — only visible ones
    let mobile = dao.find_by_tenant_with_mobile_filter(tenant_id, Some(true), &PageLink::new(0, 10))
        .await.unwrap();
    assert_eq!(mobile.total_elements, 1);
    assert_eq!(mobile.data[0].title.as_deref(), Some("Mobile Visible"));
}

// ── Dashboard info tests ─────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_info_by_id(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let dashboard = helpers::make_dashboard(tenant_id);
    dao.save(&dashboard).await.unwrap();

    let info = dao.find_info_by_id(dashboard.id).await.unwrap().unwrap();
    assert_eq!(info.id, dashboard.id);
    assert_eq!(info.tenant_id, tenant_id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_infos_by_tenant(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for i in 0..3u32 {
        let mut d = helpers::make_dashboard(tenant_id);
        d.title = Some(format!("Info-{i}"));
        dao.save(&d).await.unwrap();
    }

    let page = dao.find_infos_by_tenant(tenant_id, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page.total_elements, 3);
}

// ── Home dashboard tests ─────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn get_and_set_home_dashboard(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    // Default: no home dashboard
    let home = dao.get_home_dashboard_info(tenant_id).await.unwrap();
    assert!(home.dashboard_id.is_none());
    assert!(!home.hidden_dashboard_toolbar);

    // Set home dashboard
    let dashboard_id = Uuid::new_v4();
    let info = HomeDashboardInfo {
        dashboard_id: Some(dashboard_id),
        hidden_dashboard_toolbar: true,
    };
    dao.set_home_dashboard(tenant_id, &info).await.unwrap();

    let home2 = dao.get_home_dashboard_info(tenant_id).await.unwrap();
    assert_eq!(home2.dashboard_id, Some(dashboard_id));
    assert!(home2.hidden_dashboard_toolbar);
}

// ── Assigned customers ───────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn update_assigned_customers(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let dashboard = helpers::make_dashboard(tenant_id);
    dao.save(&dashboard).await.unwrap();

    let customers_json = serde_json::json!([{"id": Uuid::new_v4()}]).to_string();
    dao.update_assigned_customers(dashboard.id, Some(customers_json.clone()))
        .await
        .unwrap();

    let info = dao.find_info_by_id(dashboard.id).await.unwrap().unwrap();
    assert!(info.assigned_customers.is_some());
}

// ── Export tests ─────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_all_by_tenant_returns_all(pool: PgPool) {
    let dao = DashboardDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for i in 0..4u32 {
        let mut d = helpers::make_dashboard(tenant_id);
        d.title = Some(format!("Export-{i}"));
        dao.save(&d).await.unwrap();
    }

    let all = dao.find_all_by_tenant(tenant_id).await.unwrap();
    assert_eq!(all.len(), 4);
}
