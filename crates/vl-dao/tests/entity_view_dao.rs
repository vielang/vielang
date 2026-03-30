/// Integration tests for EntityViewDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::EntityView;
use vl_dao::{postgres::entity_view::EntityViewDao, DaoError, PageLink};

fn make_entity_view(tenant_id: Uuid, entity_id: Uuid) -> EntityView {
    EntityView {
        id:                Uuid::new_v4(),
        created_time:      helpers::now_ms(),
        tenant_id,
        customer_id:       None,
        entity_id,
        entity_type:       "DEVICE".into(),
        name:              format!("EV-{}", Uuid::new_v4()),
        entity_view_type:  "DEFAULT".into(),
        keys:              None,
        start_ts:          0,
        end_ts:            0,
        additional_info:   None,
        external_id:       None,
        version:           1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = EntityViewDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;
    let ev = make_entity_view(tenant_id, device_id);

    let saved = dao.save(&ev).await.unwrap();
    assert_eq!(saved.id, ev.id);
    assert_eq!(saved.name, ev.name);

    let found = dao.find_by_id(ev.id).await.unwrap().unwrap();
    assert_eq!(found.id, ev.id);
    assert_eq!(found.entity_id, device_id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = EntityViewDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = EntityViewDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    for i in 0..5u32 {
        let mut ev = make_entity_view(tenant_id, device_id);
        ev.name = format!("EV-{i}");
        dao.save(&ev).await.unwrap();
    }

    let page0 = dao.find_by_tenant(tenant_id, None, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page0.total_elements, 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn assign_and_unassign_customer(pool: PgPool) {
    let dao = EntityViewDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let customer_id = helpers::insert_customer(&pool, tenant_id).await;
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;
    let ev = make_entity_view(tenant_id, device_id);
    dao.save(&ev).await.unwrap();

    let assigned = dao.assign_to_customer(ev.id, customer_id).await.unwrap();
    assert_eq!(assigned.customer_id, Some(customer_id));

    let unassigned = dao.unassign_from_customer(ev.id).await.unwrap();
    assert!(unassigned.customer_id.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_entity_view(pool: PgPool) {
    let dao = EntityViewDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;
    let ev = make_entity_view(tenant_id, device_id);
    dao.save(&ev).await.unwrap();
    dao.delete(ev.id).await.unwrap();
    assert!(dao.find_by_id(ev.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_types_by_tenant(pool: PgPool) {
    let dao = EntityViewDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    let mut ev1 = make_entity_view(tenant_id, device_id);
    ev1.entity_view_type = "SENSOR_VIEW".into();
    dao.save(&ev1).await.unwrap();

    let mut ev2 = make_entity_view(tenant_id, device_id);
    ev2.entity_view_type = "ALARM_VIEW".into();
    dao.save(&ev2).await.unwrap();

    let types = dao.find_types_by_tenant(tenant_id).await.unwrap();
    assert!(types.len() >= 2);
}
