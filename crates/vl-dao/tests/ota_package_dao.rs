/// Integration tests for OtaPackageDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{OtaPackage, OtaPackageType, ChecksumAlgorithm};
use vl_dao::{postgres::ota_package::OtaPackageDao, DaoError, PageLink};

fn make_ota(tenant_id: Uuid) -> OtaPackage {
    let id = Uuid::new_v4();
    OtaPackage {
        id,
        created_time:       helpers::now_ms(),
        tenant_id,
        device_profile_id:  None,
        ota_package_type:   OtaPackageType::Firmware,
        title:              format!("FW-{id}"),
        version:            "1.0.0".into(),
        tag:                None,
        url:                None,
        file_name:          Some("firmware.bin".into()),
        content_type:       Some("application/octet-stream".into()),
        data_size:          None,
        checksum_algorithm: Some(ChecksumAlgorithm::Sha256),
        checksum:           Some(format!("sha256-{id}")),
        has_data:           false,
        additional_info:    None,
        version_int:        1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = OtaPackageDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let pkg = make_ota(tenant_id);

    let saved = dao.save(&pkg).await.unwrap();
    assert_eq!(saved.id, pkg.id);
    assert_eq!(saved.title, pkg.title);

    let found = dao.find_by_id(pkg.id).await.unwrap().unwrap();
    assert_eq!(found.id, pkg.id);
    assert_eq!(found.version, "1.0.0");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = OtaPackageDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_get_data(pool: PgPool) {
    let dao = OtaPackageDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let pkg = make_ota(tenant_id);
    dao.save(&pkg).await.unwrap();

    let data = b"fake firmware binary data here";
    dao.save_data(pkg.id, data).await.unwrap();

    let retrieved = dao.get_data(pkg.id).await.unwrap().unwrap();
    assert_eq!(retrieved, data);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = OtaPackageDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for i in 0..5u32 {
        let mut pkg = make_ota(tenant_id);
        pkg.title = format!("FW-{i}");
        pkg.version = format!("{i}.0.0");
        dao.save(&pkg).await.unwrap();
    }

    let page = dao.find_by_tenant(tenant_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page.total_elements, 5);
    assert_eq!(page.data.len(), 3);
    assert!(page.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_checksum(pool: PgPool) {
    let dao = OtaPackageDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let pkg = make_ota(tenant_id);
    let checksum = pkg.checksum.clone().unwrap();
    dao.save(&pkg).await.unwrap();

    let found = dao.find_by_checksum(&checksum).await.unwrap().unwrap();
    assert_eq!(found.id, pkg.id);

    assert!(dao.find_by_checksum("nonexistent-checksum").await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn exists_by_title_version(pool: PgPool) {
    let dao = OtaPackageDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let pkg = make_ota(tenant_id);
    let title = pkg.title.clone();
    dao.save(&pkg).await.unwrap();

    assert!(dao.exists_by_title_version(tenant_id, OtaPackageType::Firmware, &title, "1.0.0").await.unwrap());
    assert!(!dao.exists_by_title_version(tenant_id, OtaPackageType::Firmware, &title, "2.0.0").await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_ota_package(pool: PgPool) {
    let dao = OtaPackageDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let pkg = make_ota(tenant_id);
    dao.save(&pkg).await.unwrap();
    dao.delete(pkg.id).await.unwrap();
    assert!(dao.find_by_id(pkg.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_chunk(pool: PgPool) {
    let dao = OtaPackageDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let pkg = make_ota(tenant_id);
    dao.save(&pkg).await.unwrap();

    let data = vec![0u8; 1024]; // 1KB
    dao.save_data(pkg.id, &data).await.unwrap();

    let chunk = dao.get_chunk(pkg.id, 0, 512).await.unwrap().unwrap();
    assert_eq!(chunk.len(), 512);
}
