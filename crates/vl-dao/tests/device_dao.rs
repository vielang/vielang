/// Integration tests cho DeviceDao (Phase 1).
use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{Device, DeviceCredentials, DeviceCredentialsType};
use vl_dao::{postgres::device::DeviceDao, DaoError, PageLink};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Tạo device_profile và trả về id của nó
async fn insert_device_profile(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO device_profile
           (id, created_time, tenant_id, name, type, transport_type, provision_type, is_default)
           VALUES ($1, $2, $3, $4, 'DEFAULT', 'DEFAULT', 'DISABLED', false)"#,
        id,
        now_ms(),
        tenant_id,
        format!("profile-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

fn make_device(tenant_id: Uuid, profile_id: Uuid) -> Device {
    Device {
        id:                Uuid::new_v4(),
        created_time:      now_ms(),
        tenant_id,
        customer_id:       None,
        name:              format!("Device-{}", Uuid::new_v4()),
        device_type:       "DEFAULT".into(),
        label:             None,
        device_profile_id: profile_id,
        device_data:       None,
        firmware_id:       None,
        software_id:       None,
        external_id:       None,
        additional_info:   None,
        version:           1,
    }
}

// ── CRUD tests ────────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id  = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;
    let device     = make_device(tenant_id, profile_id);

    let saved = dao.save(&device).await.unwrap();
    assert_eq!(saved.id, device.id);
    assert_eq!(saved.name, device.name);
    assert_eq!(saved.tenant_id, tenant_id);

    let found = dao.find_by_id(device.id).await.unwrap().unwrap();
    assert_eq!(found.id, saved.id);
    assert_eq!(found.name, saved.name);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = DeviceDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_device_increments_version(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id  = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;
    let mut device = make_device(tenant_id, profile_id);

    dao.save(&device).await.unwrap();
    device.label = Some("Updated Label".into());
    let updated = dao.save(&device).await.unwrap();

    assert_eq!(updated.label.as_deref(), Some("Updated Label"));
    assert_eq!(updated.version, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_removes_device(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id  = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;
    let device     = make_device(tenant_id, profile_id);

    dao.save(&device).await.unwrap();
    dao.delete(device.id).await.unwrap();

    assert!(dao.find_by_id(device.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_nonexistent_returns_not_found(pool: PgPool) {
    let dao = DeviceDao::new(pool);
    let result = dao.delete(Uuid::new_v4()).await;
    assert!(matches!(result, Err(DaoError::NotFound)));
}

// ── Pagination tests ──────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id  = Uuid::new_v4();
    let other_tid  = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;
    let other_prof = insert_device_profile(&pool, other_tid).await;

    for i in 0..5u32 {
        let mut d = make_device(tenant_id, profile_id);
        d.name = format!("Device-{i}");
        dao.save(&d).await.unwrap();
    }
    // Device của tenant khác — không được lọt vào kết quả
    dao.save(&make_device(other_tid, other_prof)).await.unwrap();

    let page0 = dao.find_by_tenant(tenant_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page0.total_elements, 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);

    let page1 = dao.find_by_tenant(tenant_id, &PageLink::new(1, 3)).await.unwrap();
    assert_eq!(page1.data.len(), 2);
    assert!(!page1.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_text_search(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id  = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;

    for name in ["Temperature Sensor", "Humidity Sensor", "Pressure Gauge"] {
        let mut d = make_device(tenant_id, profile_id);
        d.name = name.into();
        dao.save(&d).await.unwrap();
    }

    let mut link = PageLink::new(0, 10);
    link.text_search = Some("sensor".into());
    let page = dao.find_by_tenant(tenant_id, &link).await.unwrap();
    assert_eq!(page.total_elements, 2);

    link.text_search = Some("GAUGE".into());
    let page2 = dao.find_by_tenant(tenant_id, &link).await.unwrap();
    assert_eq!(page2.total_elements, 1);
    assert_eq!(page2.data[0].name, "Pressure Gauge");
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_empty_returns_zero(pool: PgPool) {
    let dao = DeviceDao::new(pool);
    let page = dao.find_by_tenant(Uuid::new_v4(), &PageLink::new(0, 20)).await.unwrap();
    assert_eq!(page.total_elements, 0);
    assert!(page.data.is_empty());
}

// ── Device credentials tests ──────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_credentials_id_returns_device_and_creds(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id  = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;
    let device     = make_device(tenant_id, profile_id);
    dao.save(&device).await.unwrap();

    // Insert device_credentials
    let cred_id = format!("access-token-{}", Uuid::new_v4());
    sqlx::query!(
        r#"INSERT INTO device_credentials (id, created_time, device_id, credentials_type, credentials_id)
           VALUES ($1, $2, $3, 'ACCESS_TOKEN', $4)"#,
        Uuid::new_v4(),
        now_ms(),
        device.id,
        cred_id,
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = dao.find_by_credentials_id(&cred_id).await.unwrap().unwrap();
    let (found_device, found_creds) = result;
    assert_eq!(found_device.id, device.id);
    assert_eq!(found_creds.credentials_id, cred_id);
    assert_eq!(found_creds.credentials_type, DeviceCredentialsType::AccessToken);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_credentials_id_returns_none_for_unknown(pool: PgPool) {
    let dao = DeviceDao::new(pool);
    let result = dao.find_by_credentials_id("nonexistent-token-xyz").await.unwrap();
    assert!(result.is_none());
}

// ── Device model tests ────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn device_owner_id_returns_customer_if_set(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id   = Uuid::new_v4();
    let customer_id = Uuid::new_v4();
    let profile_id  = insert_device_profile(&pool, tenant_id).await;

    let mut device = make_device(tenant_id, profile_id);
    device.customer_id = Some(customer_id);
    dao.save(&device).await.unwrap();

    let found = dao.find_by_id(device.id).await.unwrap().unwrap();
    assert_eq!(found.owner_id(), customer_id); // owner = customer khi có
}

#[sqlx::test(migrations = "../../migrations")]
async fn device_owner_id_falls_back_to_tenant(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id  = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;
    let device     = make_device(tenant_id, profile_id); // customer_id = None
    dao.save(&device).await.unwrap();

    let found = dao.find_by_id(device.id).await.unwrap().unwrap();
    assert_eq!(found.owner_id(), tenant_id); // owner = tenant khi không có customer
}
