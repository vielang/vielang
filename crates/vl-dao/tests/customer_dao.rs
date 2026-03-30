/// Integration tests for CustomerDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_dao::{postgres::customer::CustomerDao, DaoError, PageLink};

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let customer = helpers::make_customer(tenant_id, "Acme Corp");

    let saved = dao.save(&customer).await.unwrap();
    assert_eq!(saved.id, customer.id);
    assert_eq!(saved.title, "Acme Corp");
    assert_eq!(saved.tenant_id, tenant_id);

    let found = dao.find_by_id(customer.id).await.unwrap().unwrap();
    assert_eq!(found.id, saved.id);
    assert_eq!(found.title, "Acme Corp");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = CustomerDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_customer_increments_version(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let mut customer = helpers::make_customer(tenant_id, "Original Title");

    dao.save(&customer).await.unwrap();
    customer.title = "Updated Title".into();
    let updated = dao.save(&customer).await.unwrap();

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.version, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_customer(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let customer = helpers::make_customer(tenant_id, "To Delete");

    dao.save(&customer).await.unwrap();
    dao.delete(customer.id).await.unwrap();

    assert!(dao.find_by_id(customer.id).await.unwrap().is_none());
}

// ── Pagination tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let other_tid = helpers::insert_tenant(&pool).await;

    for i in 0..5u32 {
        dao.save(&helpers::make_customer(tenant_id, &format!("Customer-{i}")))
            .await
            .unwrap();
    }
    // Customer from another tenant — must not appear
    dao.save(&helpers::make_customer(other_tid, "Other"))
        .await
        .unwrap();

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
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for title in ["Acme Corp", "Globex Industries", "Initech Corp"] {
        dao.save(&helpers::make_customer(tenant_id, title)).await.unwrap();
    }

    let mut link = PageLink::new(0, 10);
    link.text_search = Some("corp".into());
    let page = dao.find_by_tenant(tenant_id, &link).await.unwrap();
    assert_eq!(page.total_elements, 2); // "Acme Corp" + "Initech Corp"

    link.text_search = Some("GLOBEX".into());
    let page2 = dao.find_by_tenant(tenant_id, &link).await.unwrap();
    assert_eq!(page2.total_elements, 1);
    assert_eq!(page2.data[0].title, "Globex Industries");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_empty_returns_zero(pool: PgPool) {
    let dao = CustomerDao::new(pool);
    let page = dao.find_by_tenant(Uuid::new_v4(), &PageLink::new(0, 20)).await.unwrap();
    assert_eq!(page.total_elements, 0);
    assert!(page.data.is_empty());
}

// ── Specific method tests ────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_all_by_tenant_returns_all(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for i in 0..3u32 {
        dao.save(&helpers::make_customer(tenant_id, &format!("All-{i}")))
            .await
            .unwrap();
    }

    let all = dao.find_all_by_tenant(tenant_id).await.unwrap();
    assert_eq!(all.len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn count_by_tenant(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    assert_eq!(dao.count_by_tenant(tenant_id).await.unwrap(), 0);

    for i in 0..4u32 {
        dao.save(&helpers::make_customer(tenant_id, &format!("Count-{i}")))
            .await
            .unwrap();
    }
    assert_eq!(dao.count_by_tenant(tenant_id).await.unwrap(), 4);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_title_by_id(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let customer = helpers::make_customer(tenant_id, "Title Lookup");
    dao.save(&customer).await.unwrap();

    let title = dao.find_title_by_id(customer.id).await.unwrap();
    assert_eq!(title.as_deref(), Some("Title Lookup"));

    let none = dao.find_title_by_id(Uuid::new_v4()).await.unwrap();
    assert!(none.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_public_customer(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    // No public customer yet
    assert!(dao.find_public_customer(tenant_id).await.unwrap().is_none());

    // Create a public customer
    let mut public = helpers::make_customer(tenant_id, "Public Customer");
    public.is_public = true;
    dao.save(&public).await.unwrap();

    let found = dao.find_public_customer(tenant_id).await.unwrap().unwrap();
    assert_eq!(found.title, "Public Customer");
    assert!(found.is_public);
}
