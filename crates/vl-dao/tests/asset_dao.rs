/// Integration tests for AssetDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_dao::{postgres::asset::AssetDao, DaoError, PageLink};

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = AssetDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let profile_id = helpers::insert_asset_profile(&pool, tenant_id).await;
    let asset = helpers::make_asset(tenant_id, profile_id);

    let saved = dao.save(&asset).await.unwrap();
    assert_eq!(saved.id, asset.id);
    assert_eq!(saved.name, asset.name);
    assert_eq!(saved.tenant_id, tenant_id);

    let found = dao.find_by_id(asset.id).await.unwrap().unwrap();
    assert_eq!(found.id, saved.id);
    assert_eq!(found.name, saved.name);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = AssetDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_asset_increments_version(pool: PgPool) {
    let dao = AssetDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let profile_id = helpers::insert_asset_profile(&pool, tenant_id).await;
    let mut asset = helpers::make_asset(tenant_id, profile_id);

    dao.save(&asset).await.unwrap();
    asset.label = Some("Updated Label".into());
    let updated = dao.save(&asset).await.unwrap();

    assert_eq!(updated.label.as_deref(), Some("Updated Label"));
    assert_eq!(updated.version, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_asset(pool: PgPool) {
    let dao = AssetDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let profile_id = helpers::insert_asset_profile(&pool, tenant_id).await;
    let asset = helpers::make_asset(tenant_id, profile_id);

    dao.save(&asset).await.unwrap();
    dao.delete(asset.id).await.unwrap();

    assert!(dao.find_by_id(asset.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_nonexistent_returns_not_found(pool: PgPool) {
    let dao = AssetDao::new(pool);
    let result = dao.delete(Uuid::new_v4()).await;
    assert!(matches!(result, Err(DaoError::NotFound)));
}

// ── Pagination tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = AssetDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let other_tid = Uuid::new_v4();
    let profile_id = helpers::insert_asset_profile(&pool, tenant_id).await;
    let other_prof = helpers::insert_asset_profile(&pool, other_tid).await;

    for i in 0..5u32 {
        let mut a = helpers::make_asset(tenant_id, profile_id);
        a.name = format!("Asset-{i}");
        dao.save(&a).await.unwrap();
    }
    dao.save(&helpers::make_asset(other_tid, other_prof)).await.unwrap();

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
    let dao = AssetDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let profile_id = helpers::insert_asset_profile(&pool, tenant_id).await;

    for name in ["Building Alpha", "Building Beta", "Warehouse"] {
        let mut a = helpers::make_asset(tenant_id, profile_id);
        a.name = name.into();
        dao.save(&a).await.unwrap();
    }

    let mut link = PageLink::new(0, 10);
    link.text_search = Some("building".into());
    let page = dao.find_by_tenant(tenant_id, &link).await.unwrap();
    assert_eq!(page.total_elements, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_empty_returns_zero(pool: PgPool) {
    let dao = AssetDao::new(pool);
    let page = dao.find_by_tenant(Uuid::new_v4(), &PageLink::new(0, 20)).await.unwrap();
    assert_eq!(page.total_elements, 0);
    assert!(page.data.is_empty());
}

// ── Customer assignment tests ────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_customer(pool: PgPool) {
    let dao = AssetDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let customer_id = helpers::insert_customer(&pool, tenant_id).await;
    let profile_id = helpers::insert_asset_profile(&pool, tenant_id).await;

    let mut asset = helpers::make_asset(tenant_id, profile_id);
    asset.customer_id = Some(customer_id);
    dao.save(&asset).await.unwrap();

    // Unassigned asset — should not appear
    dao.save(&helpers::make_asset(tenant_id, profile_id)).await.unwrap();

    let page = dao.find_by_customer(customer_id, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page.total_elements, 1);
    assert_eq!(page.data[0].id, asset.id);
}

// ── Export tests ─────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_all_by_tenant_returns_all(pool: PgPool) {
    let dao = AssetDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let profile_id = helpers::insert_asset_profile(&pool, tenant_id).await;

    for i in 0..3u32 {
        let mut a = helpers::make_asset(tenant_id, profile_id);
        a.name = format!("Export-{i}");
        dao.save(&a).await.unwrap();
    }

    let all = dao.find_all_by_tenant(tenant_id).await.unwrap();
    assert_eq!(all.len(), 3);
}

// ── Tenant isolation ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn tenant_isolation(pool: PgPool) {
    let dao = AssetDao::new(pool.clone());
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let prof_a = helpers::insert_asset_profile(&pool, tenant_a).await;
    let prof_b = helpers::insert_asset_profile(&pool, tenant_b).await;

    dao.save(&helpers::make_asset(tenant_a, prof_a)).await.unwrap();
    dao.save(&helpers::make_asset(tenant_b, prof_b)).await.unwrap();

    let page_a = dao.find_by_tenant(tenant_a, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page_a.total_elements, 1);

    let page_b = dao.find_by_tenant(tenant_b, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page_b.total_elements, 1);
}
