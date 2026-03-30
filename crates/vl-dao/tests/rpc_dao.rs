/// Integration tests for RpcDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{Rpc, RpcRequest, RpcStatus};
use vl_dao::{postgres::rpc::RpcDao, DaoError, PageLink};

fn make_rpc(tenant_id: Uuid, device_id: Uuid, request_id: i32) -> Rpc {
    Rpc {
        id:              Uuid::new_v4(),
        created_time:    helpers::now_ms(),
        tenant_id,
        device_id,
        request_id,
        expiration_time: helpers::now_ms() + 60_000, // 1 minute
        request:         RpcRequest {
            method: "getData".into(),
            params: serde_json::json!({"pin": 1}),
            oneway: false,
            timeout: 10_000,
            additional_info: None,
        },
        response:        None,
        status:          RpcStatus::Queued,
        additional_info: None,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = RpcDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    let rpc = make_rpc(tenant_id, device_id, 1);
    let saved = dao.save(&rpc).await.unwrap();
    assert_eq!(saved.id, rpc.id);
    assert!(matches!(saved.status, RpcStatus::Queued));

    let found = dao.find_by_id(rpc.id).await.unwrap().unwrap();
    assert_eq!(found.id, rpc.id);
    assert_eq!(found.request.method, "getData");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = RpcDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_device_pagination(pool: PgPool) {
    let dao = RpcDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    for i in 0..5 {
        dao.save(&make_rpc(tenant_id, device_id, i)).await.unwrap();
    }

    let page = dao.find_by_device(tenant_id, device_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page.total_elements, 5);
    assert_eq!(page.data.len(), 3);
    assert!(page.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_status(pool: PgPool) {
    let dao = RpcDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    let rpc = make_rpc(tenant_id, device_id, 1);
    dao.save(&rpc).await.unwrap();

    let response = serde_json::json!({"value": 42});
    dao.update_status(rpc.id, RpcStatus::Delivered, Some(response.clone())).await.unwrap();

    let found = dao.find_by_id(rpc.id).await.unwrap().unwrap();
    assert!(matches!(found.status, RpcStatus::Delivered));
    assert_eq!(found.response, Some(response));
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_pending_by_device(pool: PgPool) {
    let dao = RpcDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    // Queued (pending)
    dao.save(&make_rpc(tenant_id, device_id, 1)).await.unwrap();
    dao.save(&make_rpc(tenant_id, device_id, 2)).await.unwrap();

    // Expired (not pending) — expiration_time in the past
    let mut rpc3 = make_rpc(tenant_id, device_id, 3);
    rpc3.expiration_time = 1000; // far in the past
    dao.save(&rpc3).await.unwrap();

    let pending = dao.find_pending_by_device(device_id).await.unwrap();
    assert_eq!(pending.len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_rpc(pool: PgPool) {
    let dao = RpcDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    let rpc = make_rpc(tenant_id, device_id, 1);
    dao.save(&rpc).await.unwrap();
    dao.delete(rpc.id).await.unwrap();
    assert!(dao.find_by_id(rpc.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_next_request_id(pool: PgPool) {
    let dao = RpcDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let dp = helpers::insert_device_profile(&pool, tenant_id).await;
    let device_id = helpers::insert_device(&pool, tenant_id, dp).await;

    dao.save(&make_rpc(tenant_id, device_id, 5)).await.unwrap();
    dao.save(&make_rpc(tenant_id, device_id, 10)).await.unwrap();

    let next = dao.get_next_request_id(device_id).await.unwrap();
    assert!(next > 10, "next request id should be > max existing (10), got {next}");
}
