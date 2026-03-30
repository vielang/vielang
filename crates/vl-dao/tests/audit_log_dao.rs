/// Integration tests for AuditLogDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{AuditLog, AuditActionType, AuditActionStatus};
use vl_dao::{postgres::audit_log::AuditLogDao, PageLink};

fn make_audit_log(tenant_id: Uuid, user_id: Option<Uuid>) -> AuditLog {
    AuditLog {
        id:                     Uuid::new_v4(),
        created_time:           helpers::now_ms(),
        tenant_id,
        user_id,
        user_name:              Some("test@user.com".into()),
        action_type:            AuditActionType::Added,
        action_data:            serde_json::json!({"entity": "device"}),
        action_status:          AuditActionStatus::Success,
        action_failure_details: None,
        entity_type:            Some("DEVICE".into()),
        entity_id:              Some(Uuid::new_v4()),
        entity_name:            Some("Test Device".into()),
    }
}

// ── Save & Query tests ───────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_tenant(pool: PgPool) {
    let dao = AuditLogDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let user_id = helpers::insert_user(&pool, tenant_id).await;

    for _ in 0..3 {
        dao.save(&make_audit_log(tenant_id, Some(user_id))).await.unwrap();
    }

    let page = dao.find_by_tenant(tenant_id, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page.total_elements, 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = AuditLogDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for _ in 0..5 {
        dao.save(&make_audit_log(tenant_id, None)).await.unwrap();
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
async fn find_by_user(pool: PgPool) {
    let dao = AuditLogDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let user_a = helpers::insert_user(&pool, tenant_id).await;
    let user_b = helpers::insert_user(&pool, tenant_id).await;

    for _ in 0..3 {
        dao.save(&make_audit_log(tenant_id, Some(user_a))).await.unwrap();
    }
    dao.save(&make_audit_log(tenant_id, Some(user_b))).await.unwrap();

    let page = dao.find_by_user(tenant_id, user_a, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page.total_elements, 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_entity(pool: PgPool) {
    let dao = AuditLogDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    let mut log = make_audit_log(tenant_id, None);
    log.entity_type = Some("DEVICE".into());
    log.entity_id = Some(entity_id);
    dao.save(&log).await.unwrap();

    // Different entity — should not appear
    dao.save(&make_audit_log(tenant_id, None)).await.unwrap();

    let page = dao.find_by_entity("DEVICE", entity_id, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page.total_elements, 1);
    assert_eq!(page.data[0].entity_id, Some(entity_id));
}

// ── Tenant isolation ─────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn tenant_isolation(pool: PgPool) {
    let dao = AuditLogDao::new(pool.clone());
    let tenant_a = helpers::insert_tenant(&pool).await;
    let tenant_b = helpers::insert_tenant(&pool).await;

    dao.save(&make_audit_log(tenant_a, None)).await.unwrap();
    dao.save(&make_audit_log(tenant_b, None)).await.unwrap();

    let page_a = dao.find_by_tenant(tenant_a, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page_a.total_elements, 1);
}

// ── Action types ─────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_with_failure_status(pool: PgPool) {
    let dao = AuditLogDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    let mut log = make_audit_log(tenant_id, None);
    log.action_type = AuditActionType::LoginFailed;
    log.action_status = AuditActionStatus::Failure;
    log.action_failure_details = Some("Invalid credentials".into());
    dao.save(&log).await.unwrap();

    let page = dao.find_by_tenant(tenant_id, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(page.data.len(), 1);
    assert!(matches!(page.data[0].action_status, AuditActionStatus::Failure));
    assert_eq!(page.data[0].action_failure_details.as_deref(), Some("Invalid credentials"));
}
