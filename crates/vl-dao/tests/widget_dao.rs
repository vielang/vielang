/// Integration tests for WidgetTypeDao and WidgetsBundleDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{WidgetType, WidgetsBundle};
use vl_dao::postgres::{
    widget_type::WidgetTypeDao,
    widgets_bundle::WidgetsBundleDao,
};
use vl_dao::{DaoError, PageLink};

fn make_widget_type(tenant_id: Uuid) -> WidgetType {
    let id = Uuid::new_v4();
    WidgetType {
        id,
        created_time: helpers::now_ms(),
        tenant_id:    Some(tenant_id),
        fqn:          format!("com.test.widget_{id}"),
        name:         format!("Widget-{id}"),
        descriptor:   serde_json::json!({"type": "timeseries"}),
        deprecated:   false,
        scada:        false,
        image:        None,
        description:  None,
        tags:         None,
        external_id:  None,
        version:      1,
    }
}

fn make_widgets_bundle(tenant_id: Uuid) -> WidgetsBundle {
    let id = Uuid::new_v4();
    WidgetsBundle {
        id,
        created_time: helpers::now_ms(),
        tenant_id:    Some(tenant_id),
        alias:        format!("bundle_{id}"),
        title:        format!("Bundle-{id}"),
        image:        None,
        scada:        false,
        description:  None,
        order_index:  None,
        external_id:  None,
        version:      1,
    }
}

// ── WidgetType tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn wt_save_and_find_by_id(pool: PgPool) {
    let dao = WidgetTypeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let wt = make_widget_type(tenant_id);

    let saved = dao.save(&wt).await.unwrap();
    assert_eq!(saved.id, wt.id);

    let found = dao.find_by_id(wt.id).await.unwrap().unwrap();
    assert_eq!(found.fqn, wt.fqn);
}

#[sqlx::test(migrations = "../../migrations")]
async fn wt_find_by_fqn(pool: PgPool) {
    let dao = WidgetTypeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let wt = make_widget_type(tenant_id);
    let fqn = wt.fqn.clone();
    dao.save(&wt).await.unwrap();

    let found = dao.find_by_fqn(&fqn).await.unwrap().unwrap();
    assert_eq!(found.id, wt.id);

    assert!(dao.find_by_fqn("nonexistent.fqn").await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn wt_find_by_tenant_pagination(pool: PgPool) {
    let dao = WidgetTypeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for _ in 0..5 {
        dao.save(&make_widget_type(tenant_id)).await.unwrap();
    }

    let page = dao.find_by_tenant(tenant_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page.total_elements, 5);
    assert_eq!(page.data.len(), 3);
    assert!(page.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn wt_delete(pool: PgPool) {
    let dao = WidgetTypeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let wt = make_widget_type(tenant_id);
    dao.save(&wt).await.unwrap();
    dao.delete(wt.id).await.unwrap();
    assert!(dao.find_by_id(wt.id).await.unwrap().is_none());
}

// ── WidgetsBundle tests ──────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn wb_save_and_find_by_id(pool: PgPool) {
    let dao = WidgetsBundleDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let wb = make_widgets_bundle(tenant_id);

    let saved = dao.save(&wb).await.unwrap();
    assert_eq!(saved.id, wb.id);

    let found = dao.find_by_id(wb.id).await.unwrap().unwrap();
    assert_eq!(found.title, wb.title);
}

#[sqlx::test(migrations = "../../migrations")]
async fn wb_find_by_tenant_pagination(pool: PgPool) {
    let dao = WidgetsBundleDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for _ in 0..4 {
        dao.save(&make_widgets_bundle(tenant_id)).await.unwrap();
    }

    let page = dao.find_by_tenant(tenant_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page.total_elements, 4);
    assert_eq!(page.data.len(), 3);
    assert!(page.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn wb_delete(pool: PgPool) {
    let dao = WidgetsBundleDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let wb = make_widgets_bundle(tenant_id);
    dao.save(&wb).await.unwrap();
    dao.delete(wb.id).await.unwrap();
    assert!(dao.find_by_id(wb.id).await.unwrap().is_none());
}
