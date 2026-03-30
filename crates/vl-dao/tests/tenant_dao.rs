/// Integration tests cho TenantDao.
/// Mỗi test function nhận một PgPool riêng với DB sạch (sqlx::test tự quản lý).
/// Chạy: DATABASE_URL=postgres://vielang:vielang@localhost/vielang cargo test -p vl-dao
use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::Tenant;
use vl_dao::{
    postgres::tenant::TenantDao,
    DaoError, PageLink,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Insert một tenant_profile để thỏa FK constraint của tenant table
async fn insert_profile(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO tenant_profile (id, created_time, name, is_default, isolated_vl_rule_engine)
           VALUES ($1, $2, $3, false, false)"#,
        id,
        now_ms(),
        format!("profile-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

fn make_tenant(profile_id: Uuid) -> Tenant {
    Tenant {
        id:                Uuid::new_v4(),
        created_time:      now_ms(),
        title:             format!("Tenant-{}", Uuid::new_v4()),
        tenant_profile_id: profile_id,
        region:            Some("EU".into()),
        country:           None,
        state:             None,
        city:              None,
        address:           None,
        address2:          None,
        zip:               None,
        phone:             None,
        email:             None,
        additional_info:   None,
        version:           1,
    }
}

// ── CRUD tests ────────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = TenantDao::new(pool.clone());
    let profile_id = insert_profile(&pool).await;
    let tenant = make_tenant(profile_id);

    let saved = dao.save(&tenant).await.unwrap();
    assert_eq!(saved.id, tenant.id);
    assert_eq!(saved.title, tenant.title);
    assert_eq!(saved.tenant_profile_id, profile_id);

    let found = dao.find_by_id(tenant.id).await.unwrap().unwrap();
    assert_eq!(found.id, saved.id);
    assert_eq!(found.title, saved.title);
    assert_eq!(found.region.as_deref(), Some("EU"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = TenantDao::new(pool);
    let result = dao.find_by_id(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_existing_tenant_increments_version(pool: PgPool) {
    let dao = TenantDao::new(pool.clone());
    let profile_id = insert_profile(&pool).await;
    let mut tenant = make_tenant(profile_id);

    dao.save(&tenant).await.unwrap();

    tenant.title = "Updated Title".into();
    let updated = dao.save(&tenant).await.unwrap();

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.id, tenant.id);
    assert_eq!(updated.version, 2); // version phải tăng
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_tenant(pool: PgPool) {
    let dao = TenantDao::new(pool.clone());
    let profile_id = insert_profile(&pool).await;
    let tenant = make_tenant(profile_id);

    dao.save(&tenant).await.unwrap();
    dao.delete(tenant.id).await.unwrap();

    assert!(dao.find_by_id(tenant.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_nonexistent_returns_not_found(pool: PgPool) {
    let dao = TenantDao::new(pool);
    let result = dao.delete(Uuid::new_v4()).await;
    assert!(matches!(result, Err(DaoError::NotFound)));
}

// ── Pagination tests ──────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_all_returns_all_tenants_paginated(pool: PgPool) {
    let dao = TenantDao::new(pool.clone());
    let profile_id = insert_profile(&pool).await;

    for i in 0..5u32 {
        let t = Tenant {
            id:                Uuid::new_v4(),
            created_time:      now_ms() + i as i64,
            title:             format!("Tenant-{i}"),
            tenant_profile_id: profile_id,
            region: None, country: None, state: None, city: None,
            address: None, address2: None, zip: None, phone: None, email: None,
            additional_info: None, version: 1,
        };
        dao.save(&t).await.unwrap();
    }

    let page0 = dao.find_all(&PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page0.total_elements, 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);

    let page1 = dao.find_all(&PageLink::new(1, 3)).await.unwrap();
    assert_eq!(page1.data.len(), 2);
    assert!(!page1.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_all_text_search_is_case_insensitive(pool: PgPool) {
    let dao = TenantDao::new(pool.clone());
    let profile_id = insert_profile(&pool).await;

    for title in ["Acme Corporation", "Globex Industries", "Initech Corp"] {
        let t = Tenant {
            id: Uuid::new_v4(), created_time: now_ms(), title: title.into(),
            tenant_profile_id: profile_id, region: None, country: None,
            state: None, city: None, address: None, address2: None,
            zip: None, phone: None, email: None, additional_info: None, version: 1,
        };
        dao.save(&t).await.unwrap();
    }

    let mut link = PageLink::new(0, 10);
    link.text_search = Some("acme".into());
    let page = dao.find_all(&link).await.unwrap();
    assert_eq!(page.total_elements, 1);
    assert_eq!(page.data[0].title, "Acme Corporation");

    // Uppercase search cũng phải hoạt động
    link.text_search = Some("CORP".into());
    let page2 = dao.find_all(&link).await.unwrap();
    assert_eq!(page2.total_elements, 2); // "Acme Corporation" + "Initech Corp"
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_all_empty_db_returns_zero(pool: PgPool) {
    let dao = TenantDao::new(pool);
    let page = dao.find_all(&PageLink::new(0, 20)).await.unwrap();
    assert_eq!(page.total_elements, 0);
    assert!(page.data.is_empty());
    assert!(!page.has_next);
}
