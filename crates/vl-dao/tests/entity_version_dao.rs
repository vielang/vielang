/// Integration tests for EntityVersionDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::CommitRequest;
use vl_dao::postgres::entity_version::EntityVersionDao;

fn make_commit(entity_id: Uuid) -> CommitRequest {
    CommitRequest {
        entity_id,
        entity_type: "DEVICE".into(),
        snapshot:    serde_json::json!({"name": "Device-1", "type": "DEFAULT"}),
        commit_msg:  Some("initial version".into()),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn commit_and_get_version(pool: PgPool) {
    let dao = EntityVersionDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    let v1 = dao.commit(tenant_id, None, &make_commit(entity_id)).await.unwrap();
    assert_eq!(v1.version_number, 1);
    assert_eq!(v1.entity_id, entity_id);
    assert_eq!(v1.tenant_id, tenant_id);

    let found = dao.get_version(v1.id).await.unwrap().unwrap();
    assert_eq!(found.id, v1.id);
    assert_eq!(found.commit_msg.as_deref(), Some("initial version"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_version_returns_none(pool: PgPool) {
    let dao = EntityVersionDao::new(pool);
    assert!(dao.get_version(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn multiple_commits_increment_version(pool: PgPool) {
    let dao = EntityVersionDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    let v1 = dao.commit(tenant_id, None, &make_commit(entity_id)).await.unwrap();
    assert_eq!(v1.version_number, 1);

    let mut req2 = make_commit(entity_id);
    req2.commit_msg = Some("second version".into());
    req2.snapshot = serde_json::json!({"name": "Device-1-updated"});
    let v2 = dao.commit(tenant_id, None, &req2).await.unwrap();
    assert_eq!(v2.version_number, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_versions_pagination(pool: PgPool) {
    let dao = EntityVersionDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    for i in 0..5 {
        let mut req = make_commit(entity_id);
        req.commit_msg = Some(format!("version {}", i + 1));
        dao.commit(tenant_id, None, &req).await.unwrap();
    }

    let (versions, total) = dao.list_versions(entity_id, 0, 3).await.unwrap();
    assert_eq!(total, 5);
    assert_eq!(versions.len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_by_number(pool: PgPool) {
    let dao = EntityVersionDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    dao.commit(tenant_id, None, &make_commit(entity_id)).await.unwrap();
    let mut req2 = make_commit(entity_id);
    req2.commit_msg = Some("v2".into());
    dao.commit(tenant_id, None, &req2).await.unwrap();

    let v1 = dao.get_by_number(entity_id, 1).await.unwrap().unwrap();
    assert_eq!(v1.version_number, 1);

    let v2 = dao.get_by_number(entity_id, 2).await.unwrap().unwrap();
    assert_eq!(v2.version_number, 2);

    assert!(dao.get_by_number(entity_id, 99).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn cleanup_old_versions(pool: PgPool) {
    let dao = EntityVersionDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    for _ in 0..5 {
        dao.commit(tenant_id, None, &make_commit(entity_id)).await.unwrap();
    }

    let cleaned = dao.cleanup_old_versions(entity_id, 2).await.unwrap();
    assert_eq!(cleaned, 3); // keep 2, remove 3

    let (_, total) = dao.list_versions(entity_id, 0, 100).await.unwrap();
    assert_eq!(total, 2);
}
