/// Integration tests for DeviceProfileDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{DeviceProfile, DeviceProfileType, DeviceTransportType, DeviceProvisionType};
use vl_dao::{postgres::device_profile::DeviceProfileDao, DaoError, PageLink};

fn make_device_profile(tenant_id: Uuid) -> DeviceProfile {
    DeviceProfile {
        id:                        Uuid::new_v4(),
        created_time:              helpers::now_ms(),
        tenant_id,
        name:                      format!("Profile-{}", Uuid::new_v4()),
        description:               None,
        image:                     None,
        is_default:                false,
        device_profile_type:       DeviceProfileType::Default,
        transport_type:            DeviceTransportType::Default,
        provision_type:            DeviceProvisionType::Disabled,
        profile_data:              None,
        default_rule_chain_id:     None,
        default_dashboard_id:      None,
        default_queue_name:        None,
        default_edge_rule_chain_id: None,
        provision_device_key:      None,
        firmware_id:               None,
        software_id:               None,
        external_id:               None,
        version:                   1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = DeviceProfileDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let profile = make_device_profile(tenant_id);

    let saved = dao.save(&profile).await.unwrap();
    assert_eq!(saved.id, profile.id);
    assert_eq!(saved.name, profile.name);

    let found = dao.find_by_id(profile.id).await.unwrap().unwrap();
    assert_eq!(found.id, profile.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = DeviceProfileDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = DeviceProfileDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();

    for i in 0..5u32 {
        let mut p = make_device_profile(tenant_id);
        p.name = format!("DP-{i}");
        dao.save(&p).await.unwrap();
    }

    let page0 = dao.find_by_tenant(tenant_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page0.total_elements, 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_default_profile(pool: PgPool) {
    let dao = DeviceProfileDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();

    let p1 = make_device_profile(tenant_id);
    let p2 = make_device_profile(tenant_id);
    dao.save(&p1).await.unwrap();
    dao.save(&p2).await.unwrap();

    dao.set_default(tenant_id, p1.id).await.unwrap();
    let default = dao.find_default(tenant_id).await.unwrap().unwrap();
    assert_eq!(default.id, p1.id);

    dao.set_default(tenant_id, p2.id).await.unwrap();
    let default2 = dao.find_default(tenant_id).await.unwrap().unwrap();
    assert_eq!(default2.id, p2.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_device_profile(pool: PgPool) {
    let dao = DeviceProfileDao::new(pool.clone());
    let profile = make_device_profile(Uuid::new_v4());
    dao.save(&profile).await.unwrap();
    dao.delete(profile.id).await.unwrap();
    assert!(dao.find_by_id(profile.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_names_by_tenant(pool: PgPool) {
    let dao = DeviceProfileDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();

    for i in 0..3u32 {
        let mut p = make_device_profile(tenant_id);
        p.name = format!("Profile-{i}");
        dao.save(&p).await.unwrap();
    }

    let names = dao.find_names_by_tenant(tenant_id).await.unwrap();
    assert_eq!(names.len(), 3);
}
