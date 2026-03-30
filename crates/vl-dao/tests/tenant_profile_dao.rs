/// Integration tests for TenantProfileDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::TenantProfile;
use vl_dao::{postgres::tenant_profile::TenantProfileDao, DaoError, PageLink};

fn make_tenant_profile() -> TenantProfile {
    TenantProfile {
        id:                        Uuid::new_v4(),
        created_time:              helpers::now_ms(),
        name:                      format!("TP-{}", Uuid::new_v4()),
        description:               None,
        is_default:                false,
        isolated_vl_rule_engine:   false,
        profile_data:              None,
        version:                   1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = TenantProfileDao::new(pool.clone());
    let profile = make_tenant_profile();

    let saved = dao.save(&profile).await.unwrap();
    assert_eq!(saved.id, profile.id);
    assert_eq!(saved.name, profile.name);

    let found = dao.find_by_id(profile.id).await.unwrap().unwrap();
    assert_eq!(found.id, profile.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = TenantProfileDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_page_pagination(pool: PgPool) {
    let dao = TenantProfileDao::new(pool.clone());

    for i in 0..5u32 {
        let mut p = make_tenant_profile();
        p.name = format!("TP-{i}");
        dao.save(&p).await.unwrap();
    }

    let page0 = dao.find_by_page(&PageLink::new(0, 3)).await.unwrap();
    assert!(page0.total_elements >= 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_default_profile(pool: PgPool) {
    let dao = TenantProfileDao::new(pool.clone());
    let p1 = make_tenant_profile();
    let p2 = make_tenant_profile();
    dao.save(&p1).await.unwrap();
    dao.save(&p2).await.unwrap();

    dao.set_default(p1.id).await.unwrap();
    let default = dao.find_default().await.unwrap().unwrap();
    assert_eq!(default.id, p1.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_tenant_profile(pool: PgPool) {
    let dao = TenantProfileDao::new(pool.clone());
    let profile = make_tenant_profile();
    dao.save(&profile).await.unwrap();
    dao.delete(profile.id).await.unwrap();
    assert!(dao.find_by_id(profile.id).await.unwrap().is_none());
}
