/// Integration tests for AdminSettingsDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::AdminSettings;
use vl_dao::postgres::admin_settings::AdminSettingsDao;

fn make_setting(tenant_id: Uuid, key: &str, value: serde_json::Value) -> AdminSettings {
    AdminSettings {
        id:           Uuid::new_v4(),
        created_time: helpers::now_ms(),
        tenant_id,
        key:          key.into(),
        json_value:   value,
    }
}

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_key(pool: PgPool) {
    let dao = AdminSettingsDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    let setting = make_setting(tenant_id, "mail", serde_json::json!({"smtpHost": "smtp.example.com"}));
    let saved = dao.save(&setting).await.unwrap();
    assert_eq!(saved.key, "mail");
    assert_eq!(saved.tenant_id, tenant_id);

    let found = dao.find_by_key(tenant_id, "mail").await.unwrap().unwrap();
    assert_eq!(found.key, "mail");
    assert_eq!(found.json_value["smtpHost"], "smtp.example.com");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_key_returns_none_for_unknown(pool: PgPool) {
    let dao = AdminSettingsDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    assert!(dao.find_by_key(tenant_id, "nonexistent").await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_updates_existing_on_conflict(pool: PgPool) {
    let dao = AdminSettingsDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    let setting = make_setting(tenant_id, "mail", serde_json::json!({"host": "old.com"}));
    dao.save(&setting).await.unwrap();

    // Update same key
    let updated = make_setting(tenant_id, "mail", serde_json::json!({"host": "new.com"}));
    dao.save(&updated).await.unwrap();

    let found = dao.find_by_key(tenant_id, "mail").await.unwrap().unwrap();
    assert_eq!(found.json_value["host"], "new.com");
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_by_key(pool: PgPool) {
    let dao = AdminSettingsDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    let setting = make_setting(tenant_id, "to_delete", serde_json::json!({}));
    dao.save(&setting).await.unwrap();

    let deleted = dao.delete_by_key(tenant_id, "to_delete").await.unwrap();
    assert!(deleted);

    assert!(dao.find_by_key(tenant_id, "to_delete").await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_by_key_nonexistent_returns_false(pool: PgPool) {
    let dao = AdminSettingsDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let deleted = dao.delete_by_key(tenant_id, "nonexistent").await.unwrap();
    assert!(!deleted);
}

// ── Tenant isolation ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn tenant_isolation(pool: PgPool) {
    let dao = AdminSettingsDao::new(pool.clone());
    let tenant_a = helpers::insert_tenant(&pool).await;
    let tenant_b = helpers::insert_tenant(&pool).await;

    dao.save(&make_setting(tenant_a, "mail", serde_json::json!({"tenant": "A"}))).await.unwrap();
    dao.save(&make_setting(tenant_b, "mail", serde_json::json!({"tenant": "B"}))).await.unwrap();

    let a = dao.find_by_key(tenant_a, "mail").await.unwrap().unwrap();
    assert_eq!(a.json_value["tenant"], "A");

    let b = dao.find_by_key(tenant_b, "mail").await.unwrap().unwrap();
    assert_eq!(b.json_value["tenant"], "B");
}

// ── Multiple keys per tenant ─────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn multiple_keys_per_tenant(pool: PgPool) {
    let dao = AdminSettingsDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    dao.save(&make_setting(tenant_id, "mail", serde_json::json!({"type": "mail"}))).await.unwrap();
    dao.save(&make_setting(tenant_id, "general", serde_json::json!({"type": "general"}))).await.unwrap();
    dao.save(&make_setting(tenant_id, "security", serde_json::json!({"type": "security"}))).await.unwrap();

    let mail = dao.find_by_key(tenant_id, "mail").await.unwrap().unwrap();
    assert_eq!(mail.json_value["type"], "mail");

    let general = dao.find_by_key(tenant_id, "general").await.unwrap().unwrap();
    assert_eq!(general.json_value["type"], "general");
}
