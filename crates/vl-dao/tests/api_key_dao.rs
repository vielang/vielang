/// Integration tests for ApiKeyDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::ApiKey;
use vl_dao::{postgres::api_key::ApiKeyDao, PageLink};

fn make_api_key(tenant_id: Uuid, user_id: Uuid) -> ApiKey {
    let key_id = Uuid::new_v4();
    ApiKey {
        id:           Uuid::new_v4(),
        created_time: helpers::now_ms(),
        tenant_id,
        user_id,
        name:         format!("Key-{key_id}"),
        key_hash:     format!("hash-{key_id}"),
        key_prefix:   format!("vl_{}", &key_id.to_string()[..8]),
        scopes:       vec!["READ".into(), "WRITE".into()],
        expires_at:   None,
        last_used_at: None,
        enabled:      true,
    }
}

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = ApiKeyDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let user_id = helpers::insert_user(&pool, tenant_id).await;

    let key = make_api_key(tenant_id, user_id);
    dao.save(&key).await.unwrap();

    let found = dao.find_by_id(key.id).await.unwrap().unwrap();
    assert_eq!(found.id, key.id);
    assert_eq!(found.name, key.name);
    assert_eq!(found.tenant_id, tenant_id);
    assert!(found.enabled);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = ApiKeyDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_hash(pool: PgPool) {
    let dao = ApiKeyDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let user_id = helpers::insert_user(&pool, tenant_id).await;

    let key = make_api_key(tenant_id, user_id);
    let hash = key.key_hash.clone();
    dao.save(&key).await.unwrap();

    let found = dao.find_by_hash(&hash).await.unwrap().unwrap();
    assert_eq!(found.id, key.id);

    assert!(dao.find_by_hash("nonexistent-hash").await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_api_key(pool: PgPool) {
    let dao = ApiKeyDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let user_id = helpers::insert_user(&pool, tenant_id).await;

    let key = make_api_key(tenant_id, user_id);
    dao.save(&key).await.unwrap();
    dao.delete(key.id).await.unwrap();

    assert!(dao.find_by_id(key.id).await.unwrap().is_none());
}

// ── Pagination tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_user_pagination(pool: PgPool) {
    let dao = ApiKeyDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let user_id = helpers::insert_user(&pool, tenant_id).await;
    let other_user = helpers::insert_user(&pool, tenant_id).await;

    for _ in 0..5 {
        dao.save(&make_api_key(tenant_id, user_id)).await.unwrap();
    }
    // Another user's key — must not appear
    dao.save(&make_api_key(tenant_id, other_user)).await.unwrap();

    let page0 = dao.find_by_user(user_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page0.total_elements, 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);

    let page1 = dao.find_by_user(user_id, &PageLink::new(1, 3)).await.unwrap();
    assert_eq!(page1.data.len(), 2);
    assert!(!page1.has_next);
}

// ── Enable/disable tests ─────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn set_enabled(pool: PgPool) {
    let dao = ApiKeyDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let user_id = helpers::insert_user(&pool, tenant_id).await;

    let key = make_api_key(tenant_id, user_id);
    dao.save(&key).await.unwrap();

    // Disable
    dao.set_enabled(key.id, false).await.unwrap();
    let found = dao.find_by_id(key.id).await.unwrap().unwrap();
    assert!(!found.enabled);

    // Re-enable
    dao.set_enabled(key.id, true).await.unwrap();
    let found2 = dao.find_by_id(key.id).await.unwrap().unwrap();
    assert!(found2.enabled);
}

// ── Last used tracking ───────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn update_last_used_at(pool: PgPool) {
    let dao = ApiKeyDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let user_id = helpers::insert_user(&pool, tenant_id).await;

    let key = make_api_key(tenant_id, user_id);
    dao.save(&key).await.unwrap();

    assert!(dao.find_by_id(key.id).await.unwrap().unwrap().last_used_at.is_none());

    let ts = helpers::now_ms();
    dao.update_last_used_at(key.id, ts).await.unwrap();

    let found = dao.find_by_id(key.id).await.unwrap().unwrap();
    assert_eq!(found.last_used_at, Some(ts));
}
