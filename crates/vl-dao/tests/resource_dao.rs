/// Integration tests for ResourceDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::TbResource;
use vl_dao::{postgres::resource::ResourceDao, DaoError, PageLink};

fn make_resource(tenant_id: Uuid) -> TbResource {
    let id = Uuid::new_v4();
    TbResource {
        id,
        created_time:      helpers::now_ms(),
        tenant_id:         Some(tenant_id),
        title:             format!("Resource-{id}"),
        resource_type:     "JS_MODULE".into(),
        resource_sub_type: None,
        resource_key:      format!("res_{id}"),
        file_name:         format!("module_{id}.js"),
        is_public:         false,
        public_resource_key: None,
        etag:              None,
        descriptor:        None,
        data:              Some(b"console.log('test')".to_vec()),
        preview:           None,
        external_id:       None,
        version:           1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = ResourceDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let res = make_resource(tenant_id);

    let saved = dao.save(&res).await.unwrap();
    assert_eq!(saved.id, res.id);
    assert_eq!(saved.title, res.title);

    let found = dao.find_by_id(res.id).await.unwrap().unwrap();
    assert_eq!(found.id, res.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = ResourceDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_key(pool: PgPool) {
    let dao = ResourceDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let res = make_resource(tenant_id);
    let key = res.resource_key.clone();
    dao.save(&res).await.unwrap();

    let found = dao.find_by_key(tenant_id, "JS_MODULE", &key).await.unwrap().unwrap();
    assert_eq!(found.id, res.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = ResourceDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for _ in 0..5 {
        dao.save(&make_resource(tenant_id)).await.unwrap();
    }

    let page = dao.find_by_tenant(tenant_id, None, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page.total_elements, 5);
    assert_eq!(page.data.len(), 3);
    assert!(page.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_resource(pool: PgPool) {
    let dao = ResourceDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let res = make_resource(tenant_id);
    dao.save(&res).await.unwrap();
    dao.delete(res.id).await.unwrap();
    assert!(dao.find_by_id(res.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_with_type_filter(pool: PgPool) {
    let dao = ResourceDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    dao.save(&make_resource(tenant_id)).await.unwrap(); // JS_MODULE

    let mut img = make_resource(tenant_id);
    img.resource_type = "IMAGE".into();
    dao.save(&img).await.unwrap();

    let js = dao.find_by_tenant(tenant_id, Some("JS_MODULE"), &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(js.total_elements, 1);

    let all = dao.find_by_tenant(tenant_id, None, &PageLink::new(0, 10)).await.unwrap();
    assert_eq!(all.total_elements, 2);
}
