/// Integration tests for AlarmDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::AlarmSeverity;
use vl_dao::{postgres::alarm::AlarmDao, DaoError, PageLink};

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let device_id = {
        let dp = helpers::insert_device_profile(&pool, tenant_id).await;
        helpers::insert_device(&pool, tenant_id, dp).await
    };

    let alarm = helpers::make_alarm(tenant_id, device_id);
    let saved = dao.save(&alarm).await.unwrap();

    assert_eq!(saved.id, alarm.id);
    assert_eq!(saved.tenant_id, tenant_id);
    assert_eq!(saved.originator_id, device_id);
    assert!(!saved.acknowledged);
    assert!(!saved.cleared);

    let found = dao.find_by_id(alarm.id).await.unwrap().unwrap();
    assert_eq!(found.id, alarm.id);
    assert_eq!(found.alarm_type, alarm.alarm_type);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = AlarmDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_alarm(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    let alarm = helpers::make_alarm(tenant_id, device_id);
    dao.save(&alarm).await.unwrap();
    dao.delete(alarm.id).await.unwrap();

    assert!(dao.find_by_id(alarm.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_nonexistent_returns_not_found(pool: PgPool) {
    let dao = AlarmDao::new(pool);
    let result = dao.delete(Uuid::new_v4()).await;
    assert!(matches!(result, Err(DaoError::NotFound)));
}

// ── Update operations ────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn update_alarm_via_save(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    let mut alarm = helpers::make_alarm(tenant_id, device_id);
    dao.save(&alarm).await.unwrap();

    // Update severity via save (ON CONFLICT DO UPDATE)
    alarm.severity = AlarmSeverity::Critical;
    alarm.details = Some(serde_json::json!({"reason": "threshold exceeded"}));
    let updated = dao.save(&alarm).await.unwrap();

    assert!(matches!(updated.severity, AlarmSeverity::Critical));
    assert!(updated.details.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn acknowledge_alarm(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    let alarm = helpers::make_alarm(tenant_id, device_id);
    dao.save(&alarm).await.unwrap();

    let ack_ts = helpers::now_ms();
    dao.acknowledge(alarm.id, ack_ts).await.unwrap();

    let found = dao.find_by_id(alarm.id).await.unwrap().unwrap();
    assert!(found.acknowledged);
    assert_eq!(found.ack_ts, Some(ack_ts));
}

#[sqlx::test(migrations = "../../migrations")]
async fn clear_alarm(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    let alarm = helpers::make_alarm(tenant_id, device_id);
    dao.save(&alarm).await.unwrap();

    let clear_ts = helpers::now_ms();
    dao.clear(alarm.id, clear_ts).await.unwrap();

    let found = dao.find_by_id(alarm.id).await.unwrap().unwrap();
    assert!(found.cleared);
    assert_eq!(found.clear_ts, Some(clear_ts));
}

#[sqlx::test(migrations = "../../migrations")]
async fn assign_and_unassign_alarm(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;
    let user_id = helpers::insert_user(&pool, tenant_id).await;

    let alarm = helpers::make_alarm(tenant_id, device_id);
    dao.save(&alarm).await.unwrap();

    // Assign
    let ts = helpers::now_ms();
    dao.assign_to_user(alarm.id, user_id, ts).await.unwrap();
    let found = dao.find_by_id(alarm.id).await.unwrap().unwrap();
    assert_eq!(found.assignee_id, Some(user_id));
    assert_eq!(found.assign_ts, ts);

    // Unassign
    dao.unassign(alarm.id).await.unwrap();
    let found2 = dao.find_by_id(alarm.id).await.unwrap().unwrap();
    assert!(found2.assignee_id.is_none());
    assert_eq!(found2.assign_ts, 0);
}

// ── Pagination tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    for _ in 0..5 {
        dao.save(&helpers::make_alarm(tenant_id, device_id)).await.unwrap();
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
async fn find_by_tenant_empty_returns_zero(pool: PgPool) {
    let dao = AlarmDao::new(pool);
    let page = dao.find_by_tenant(Uuid::new_v4(), &PageLink::new(0, 20)).await.unwrap();
    assert_eq!(page.total_elements, 0);
    assert!(page.data.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_originator_pagination(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_a = helpers::insert_device(&pool, tenant_id, dp).await;
    let device_b = helpers::insert_device(&pool, tenant_id, dp).await;

    for _ in 0..3 {
        dao.save(&helpers::make_alarm(tenant_id, device_a)).await.unwrap();
    }
    dao.save(&helpers::make_alarm(tenant_id, device_b)).await.unwrap();

    let page = dao.find_by_originator(tenant_id, device_a, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page.total_elements, 3);

    let page_b = dao.find_by_originator(tenant_id, device_b, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page_b.total_elements, 1);
}

// ── Tenant isolation ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn tenant_isolation(pool: PgPool) {
    let dao = AlarmDao::new(pool.clone());
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let dp_a = helpers::insert_device_profile(&pool, tenant_a).await;
    let dp_b = helpers::insert_device_profile(&pool, tenant_b).await;
    let dev_a = helpers::insert_device(&pool, tenant_a, dp_a).await;
    let dev_b = helpers::insert_device(&pool, tenant_b, dp_b).await;

    dao.save(&helpers::make_alarm(tenant_a, dev_a)).await.unwrap();
    dao.save(&helpers::make_alarm(tenant_b, dev_b)).await.unwrap();

    let page_a = dao.find_by_tenant(tenant_a, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page_a.total_elements, 1);

    let page_b = dao.find_by_tenant(tenant_b, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page_b.total_elements, 1);
}
