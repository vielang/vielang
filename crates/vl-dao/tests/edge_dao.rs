/// Integration tests for EdgeDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::Edge;
use vl_dao::{postgres::edge::EdgeDao, DaoError, PageLink};

fn make_edge(tenant_id: Uuid) -> Edge {
    let id = Uuid::new_v4();
    Edge {
        id,
        created_time:       helpers::now_ms(),
        tenant_id,
        customer_id:        None,
        root_rule_chain_id: None,
        name:               format!("Edge-{id}"),
        edge_type:          "DEFAULT".into(),
        label:              None,
        routing_key:        format!("rk-{id}"),
        secret:             format!("secret-{id}"),
        additional_info:    None,
        external_id:        None,
        version:            1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = EdgeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let edge = make_edge(tenant_id);

    let saved = dao.save(&edge).await.unwrap();
    assert_eq!(saved.id, edge.id);
    assert_eq!(saved.name, edge.name);

    let found = dao.find_by_id(edge.id).await.unwrap().unwrap();
    assert_eq!(found.id, edge.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = EdgeDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_routing_key(pool: PgPool) {
    let dao = EdgeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let edge = make_edge(tenant_id);
    let rk = edge.routing_key.clone();
    dao.save(&edge).await.unwrap();

    let found = dao.find_by_routing_key(&rk).await.unwrap().unwrap();
    assert_eq!(found.id, edge.id);

    assert!(dao.find_by_routing_key("nonexistent").await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = EdgeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for _ in 0..5 {
        dao.save(&make_edge(tenant_id)).await.unwrap();
    }

    let page0 = dao.find_by_tenant(tenant_id, None, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page0.total_elements, 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn assign_and_unassign_customer(pool: PgPool) {
    let dao = EdgeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let customer_id = helpers::insert_customer(&pool, tenant_id).await;
    let edge = make_edge(tenant_id);
    dao.save(&edge).await.unwrap();

    let assigned = dao.assign_to_customer(edge.id, customer_id).await.unwrap();
    assert_eq!(assigned.customer_id, Some(customer_id));

    let unassigned = dao.unassign_from_customer(edge.id).await.unwrap();
    assert!(unassigned.customer_id.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_edge(pool: PgPool) {
    let dao = EdgeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let edge = make_edge(tenant_id);
    dao.save(&edge).await.unwrap();
    dao.delete(edge.id).await.unwrap();
    assert!(dao.find_by_id(edge.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_types_by_tenant(pool: PgPool) {
    let dao = EdgeDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    let mut e1 = make_edge(tenant_id);
    e1.edge_type = "TYPE_A".into();
    dao.save(&e1).await.unwrap();

    let mut e2 = make_edge(tenant_id);
    e2.edge_type = "TYPE_B".into();
    dao.save(&e2).await.unwrap();

    let types = dao.find_types_by_tenant(tenant_id).await.unwrap();
    assert!(types.len() >= 2);
}
