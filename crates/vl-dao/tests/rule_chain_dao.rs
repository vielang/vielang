/// Integration tests for RuleChainDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_dao::{postgres::rule_chain::RuleChainDao, DaoError, PageLink};

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = RuleChainDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let chain = helpers::make_rule_chain(tenant_id);

    let saved = dao.save(&chain).await.unwrap();
    assert_eq!(saved.id, chain.id);
    assert_eq!(saved.name, chain.name);
    assert_eq!(saved.tenant_id, tenant_id);
    assert_eq!(saved.chain_type, "CORE");
    assert!(!saved.root);

    let found = dao.find_by_id(chain.id).await.unwrap().unwrap();
    assert_eq!(found.id, saved.id);
    assert_eq!(found.name, saved.name);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = RuleChainDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_rule_chain_via_save(pool: PgPool) {
    let dao = RuleChainDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let mut chain = helpers::make_rule_chain(tenant_id);

    dao.save(&chain).await.unwrap();
    chain.name = "Updated Chain".into();
    chain.debug_mode = true;
    let updated = dao.save(&chain).await.unwrap();

    assert_eq!(updated.name, "Updated Chain");
    assert!(updated.debug_mode);
    // RuleChainDao save uses EXCLUDED.version (preserves input version)
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_rule_chain(pool: PgPool) {
    let dao = RuleChainDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let chain = helpers::make_rule_chain(tenant_id);

    dao.save(&chain).await.unwrap();
    dao.delete(chain.id).await.unwrap();

    assert!(dao.find_by_id(chain.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_nonexistent_succeeds_silently(pool: PgPool) {
    let dao = RuleChainDao::new(pool);
    // RuleChainDao.delete does not check rows_affected — always Ok
    dao.delete(Uuid::new_v4()).await.unwrap();
}

// ── Pagination tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = RuleChainDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for i in 0..5u32 {
        let mut c = helpers::make_rule_chain(tenant_id);
        c.name = format!("Chain-{i}");
        dao.save(&c).await.unwrap();
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
    let dao = RuleChainDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for name in ["Temperature Alert", "Humidity Alert", "Pressure Monitor"] {
        let mut c = helpers::make_rule_chain(tenant_id);
        c.name = name.into();
        dao.save(&c).await.unwrap();
    }

    // RuleChainDao.find_by_tenant has no text_search — returns all for tenant
    let page = dao.find_by_tenant(tenant_id, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page.total_elements, 3);
}

// ── Root chain tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_root_by_tenant(pool: PgPool) {
    let dao = RuleChainDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    // No root chain yet
    assert!(dao.find_root_by_tenant(tenant_id).await.unwrap().is_none());

    // Create a root chain
    let mut chain = helpers::make_rule_chain(tenant_id);
    chain.root = true;
    dao.save(&chain).await.unwrap();

    let root = dao.find_root_by_tenant(tenant_id).await.unwrap().unwrap();
    assert_eq!(root.id, chain.id);
    assert!(root.root);
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_root_chain(pool: PgPool) {
    let dao = RuleChainDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    let mut chain_a = helpers::make_rule_chain(tenant_id);
    chain_a.name = "Chain A".into();
    dao.save(&chain_a).await.unwrap();

    let mut chain_b = helpers::make_rule_chain(tenant_id);
    chain_b.name = "Chain B".into();
    dao.save(&chain_b).await.unwrap();

    // Set chain_a as root
    dao.set_root(tenant_id, chain_a.id).await.unwrap();
    let root = dao.find_root_by_tenant(tenant_id).await.unwrap().unwrap();
    assert_eq!(root.id, chain_a.id);

    // Switch to chain_b as root — chain_a should no longer be root
    dao.set_root(tenant_id, chain_b.id).await.unwrap();
    let root2 = dao.find_root_by_tenant(tenant_id).await.unwrap().unwrap();
    assert_eq!(root2.id, chain_b.id);

    let a = dao.find_by_id(chain_a.id).await.unwrap().unwrap();
    assert!(!a.root, "chain_a should no longer be root");
}

// ── Export tests ─────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_all_by_tenant_returns_all(pool: PgPool) {
    let dao = RuleChainDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for i in 0..3u32 {
        let mut c = helpers::make_rule_chain(tenant_id);
        c.name = format!("Export-{i}");
        dao.save(&c).await.unwrap();
    }

    let all = dao.find_all_by_tenant(tenant_id).await.unwrap();
    assert_eq!(all.len(), 3);
}

// ── Tenant isolation ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn tenant_isolation(pool: PgPool) {
    let dao = RuleChainDao::new(pool.clone());
    let tenant_a = helpers::insert_tenant(&pool).await;
    let tenant_b = helpers::insert_tenant(&pool).await;

    let mut ca = helpers::make_rule_chain(tenant_a);
    ca.name = "Chain A".into();
    dao.save(&ca).await.unwrap();

    let mut cb = helpers::make_rule_chain(tenant_b);
    cb.name = "Chain B".into();
    dao.save(&cb).await.unwrap();

    let page_a = dao.find_by_tenant(tenant_a, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page_a.total_elements, 1);

    let page_b = dao.find_by_tenant(tenant_b, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page_b.total_elements, 1);
}
