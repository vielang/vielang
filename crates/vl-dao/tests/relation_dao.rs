/// Integration tests for RelationDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{EntityRelation, EntityType, RelationTypeGroup};
use vl_dao::{postgres::relation::RelationDao, DaoError};

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_get_relation(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let ap = helpers::insert_asset_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;
    let asset_id = helpers::insert_asset(&pool, tenant_id, ap).await;

    let rel = helpers::make_relation(asset_id, EntityType::Asset, device_id, EntityType::Device);
    dao.save(&rel).await.unwrap();

    let found = dao.get_relation(
        asset_id, "ASSET", device_id, "DEVICE", "Contains", "COMMON",
    ).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.from_id, asset_id);
    assert_eq!(found.to_id, device_id);
    assert_eq!(found.relation_type, "Contains");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_relation_returns_none_for_unknown(pool: PgPool) {
    let dao = RelationDao::new(pool);
    let result = dao.get_relation(
        Uuid::new_v4(), "DEVICE", Uuid::new_v4(), "ASSET", "Contains", "COMMON",
    ).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_updates_additional_info_on_conflict(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();

    let mut rel = EntityRelation {
        from_id,
        from_type: EntityType::Device,
        to_id,
        to_type: EntityType::Asset,
        relation_type: "Contains".into(),
        relation_type_group: RelationTypeGroup::Common,
        additional_info: None,
    };
    dao.save(&rel).await.unwrap();

    // Update additional_info via ON CONFLICT
    rel.additional_info = Some(serde_json::json!({"description": "updated"}));
    dao.save(&rel).await.unwrap();

    let found = dao.get_relation(
        from_id, "DEVICE", to_id, "ASSET", "Contains", "COMMON",
    ).await.unwrap().unwrap();
    assert!(found.additional_info.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_relation(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let from_id = Uuid::new_v4();
    let to_id = Uuid::new_v4();

    let rel = helpers::make_relation(from_id, EntityType::Device, to_id, EntityType::Asset);
    dao.save(&rel).await.unwrap();

    dao.delete(from_id, "DEVICE", to_id, "ASSET", "Contains", "COMMON")
        .await
        .unwrap();

    let found = dao.get_relation(from_id, "DEVICE", to_id, "ASSET", "Contains", "COMMON")
        .await.unwrap();
    assert!(found.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_nonexistent_returns_not_found(pool: PgPool) {
    let dao = RelationDao::new(pool);
    let result = dao.delete(
        Uuid::new_v4(), "DEVICE", Uuid::new_v4(), "ASSET", "Contains", "COMMON",
    ).await;
    assert!(matches!(result, Err(DaoError::NotFound)));
}

// ── Query tests ──────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_from(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let from_id = Uuid::new_v4();

    // Create 3 relations from the same entity
    for _ in 0..3 {
        let rel = helpers::make_relation(from_id, EntityType::Asset, Uuid::new_v4(), EntityType::Device);
        dao.save(&rel).await.unwrap();
    }
    // Different from entity
    let rel2 = helpers::make_relation(Uuid::new_v4(), EntityType::Asset, Uuid::new_v4(), EntityType::Device);
    dao.save(&rel2).await.unwrap();

    let found = dao.find_by_from(from_id, "ASSET").await.unwrap();
    assert_eq!(found.len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_to(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let to_id = Uuid::new_v4();

    for _ in 0..2 {
        let rel = helpers::make_relation(Uuid::new_v4(), EntityType::Asset, to_id, EntityType::Device);
        dao.save(&rel).await.unwrap();
    }

    let found = dao.find_by_to(to_id, "DEVICE").await.unwrap();
    assert_eq!(found.len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_from_filtered(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let from_id = Uuid::new_v4();

    // Contains relation to Device
    let rel1 = helpers::make_relation(from_id, EntityType::Asset, Uuid::new_v4(), EntityType::Device);
    dao.save(&rel1).await.unwrap();

    // Contains relation to Asset
    let rel2 = helpers::make_relation(from_id, EntityType::Asset, Uuid::new_v4(), EntityType::Asset);
    dao.save(&rel2).await.unwrap();

    // "Manages" relation to Device
    let mut rel3 = helpers::make_relation(from_id, EntityType::Asset, Uuid::new_v4(), EntityType::Device);
    rel3.relation_type = "Manages".into();
    dao.save(&rel3).await.unwrap();

    // Filter by relation_type only
    let by_type = dao.find_by_from_filtered(from_id, "ASSET", Some("Contains"), None)
        .await.unwrap();
    assert_eq!(by_type.len(), 2); // rel1 + rel2

    // Filter by to_type only
    let by_to = dao.find_by_from_filtered(from_id, "ASSET", None, Some("DEVICE"))
        .await.unwrap();
    assert_eq!(by_to.len(), 2); // rel1 + rel3

    // Filter by both
    let both = dao.find_by_from_filtered(from_id, "ASSET", Some("Contains"), Some("DEVICE"))
        .await.unwrap();
    assert_eq!(both.len(), 1); // only rel1

    // No filter — all 3
    let all = dao.find_by_from_filtered(from_id, "ASSET", None, None)
        .await.unwrap();
    assert_eq!(all.len(), 3);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_from_with_group(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let from_id = Uuid::new_v4();

    // COMMON group
    let rel1 = helpers::make_relation(from_id, EntityType::Asset, Uuid::new_v4(), EntityType::Device);
    dao.save(&rel1).await.unwrap();

    // ALARM group
    let mut rel2 = helpers::make_relation(from_id, EntityType::Asset, Uuid::new_v4(), EntityType::Device);
    rel2.relation_type_group = RelationTypeGroup::Alarm;
    dao.save(&rel2).await.unwrap();

    let common = dao.find_by_from_with_group(from_id, "ASSET", "COMMON").await.unwrap();
    assert_eq!(common.len(), 1);

    let alarm = dao.find_by_from_with_group(from_id, "ASSET", "ALARM").await.unwrap();
    assert_eq!(alarm.len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_to_filtered(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let to_id = Uuid::new_v4();

    let rel1 = helpers::make_relation(Uuid::new_v4(), EntityType::Asset, to_id, EntityType::Device);
    dao.save(&rel1).await.unwrap();

    let mut rel2 = helpers::make_relation(Uuid::new_v4(), EntityType::Customer, to_id, EntityType::Device);
    rel2.relation_type = "Manages".into();
    dao.save(&rel2).await.unwrap();

    let by_type = dao.find_by_to_filtered(to_id, "DEVICE", Some("Contains"), None)
        .await.unwrap();
    assert_eq!(by_type.len(), 1);

    let by_from = dao.find_by_to_filtered(to_id, "DEVICE", None, Some("ASSET"))
        .await.unwrap();
    assert_eq!(by_from.len(), 1);
}

// ── Delete all by entity ─────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn delete_all_by_entity(pool: PgPool) {
    let dao = RelationDao::new(pool.clone());
    let entity_id = Uuid::new_v4();

    // From relations (COMMON group)
    let rel1 = helpers::make_relation(entity_id, EntityType::Device, Uuid::new_v4(), EntityType::Asset);
    dao.save(&rel1).await.unwrap();

    // To relations (COMMON group)
    let rel2 = helpers::make_relation(Uuid::new_v4(), EntityType::Asset, entity_id, EntityType::Device);
    dao.save(&rel2).await.unwrap();

    // ALARM group — should NOT be deleted
    let mut rel3 = helpers::make_relation(entity_id, EntityType::Device, Uuid::new_v4(), EntityType::Asset);
    rel3.relation_type_group = RelationTypeGroup::Alarm;
    dao.save(&rel3).await.unwrap();

    dao.delete_all_by_entity(entity_id, "DEVICE").await.unwrap();

    let from = dao.find_by_from(entity_id, "DEVICE").await.unwrap();
    // Only alarm relation should remain
    assert_eq!(from.len(), 1);
    assert!(matches!(from[0].relation_type_group, RelationTypeGroup::Alarm));

    let to = dao.find_by_to(entity_id, "DEVICE").await.unwrap();
    assert_eq!(to.len(), 0); // COMMON to-relation was deleted
}
