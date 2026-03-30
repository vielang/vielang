/// Unit 16 — Unique Constraint Tests (DAO layer).
/// Tests verify that the DB-level unique constraints are correctly surfaced
/// as DaoError::Constraint by the DAO layer.
use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{Authority, Customer, Device, DeviceCredentials, DeviceCredentialsType, User};
use vl_dao::{
    postgres::{customer::CustomerDao, device::DeviceDao, user::UserDao},
    DaoError,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Insert tenant_profile + tenant, return the tenant UUID.
async fn insert_tenant(pool: &PgPool) -> Uuid {
    let profile_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO tenant_profile (id, created_time, name, is_default, isolated_vl_rule_engine)
           VALUES ($1, $2, $3, false, false)"#,
        profile_id,
        now_ms(),
        format!("profile-{}", profile_id),
    )
    .execute(pool)
    .await
    .unwrap();

    let tenant_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO tenant (id, created_time, title, tenant_profile_id, region, version)
           VALUES ($1, $2, $3, $4, 'EU', 1)"#,
        tenant_id,
        now_ms(),
        format!("Tenant-{}", tenant_id),
        profile_id,
    )
    .execute(pool)
    .await
    .unwrap();

    tenant_id
}

/// Insert a device_profile for the given tenant, return its UUID.
async fn insert_device_profile(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO device_profile
           (id, created_time, tenant_id, name, type, transport_type, provision_type, is_default)
           VALUES ($1, $2, $3, $4, 'DEFAULT', 'DEFAULT', 'DISABLED', false)"#,
        id,
        now_ms(),
        tenant_id,
        format!("profile-{}", id),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

fn make_customer(tenant_id: Uuid, title: &str) -> Customer {
    Customer {
        id:              Uuid::new_v4(),
        created_time:    now_ms(),
        tenant_id,
        title:           title.into(),
        country:         None,
        state:           None,
        city:            None,
        address:         None,
        address2:        None,
        zip:             None,
        phone:           None,
        email:           None,
        is_public:       false,
        external_id:     None,
        additional_info: None,
        version:         1,
    }
}

fn make_user(email: &str) -> User {
    User {
        id:              Uuid::new_v4(),
        created_time:    now_ms(),
        tenant_id:       Uuid::new_v4(), // tb_user.tenant_id has no FK
        customer_id:     None,
        email:           email.into(),
        authority:       Authority::TenantAdmin,
        first_name:      Some("Test".into()),
        last_name:       Some("User".into()),
        phone:           None,
        additional_info: None,
        version:         1,
    }
}

fn make_device(tenant_id: Uuid, profile_id: Uuid, name: &str) -> Device {
    Device {
        id:                Uuid::new_v4(),
        created_time:      now_ms(),
        tenant_id,
        customer_id:       None,
        name:              name.into(),
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

// ── Customer unique constraint tests ──────────────────────────────────────────

/// CONSTRAINT customer_title_unq UNIQUE (tenant_id, title)
/// Two customers with the same title in the same tenant must fail.
#[sqlx::test(migrations = "../../migrations")]
async fn customer_unique_title_per_tenant(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_id = insert_tenant(&pool).await;

    dao.save(&make_customer(tenant_id, "Duplicate Title")).await.unwrap();

    let result = dao.save(&make_customer(tenant_id, "Duplicate Title")).await;
    assert!(
        matches!(result, Err(DaoError::Constraint(_))),
        "expected DaoError::Constraint, got: {:?}",
        result
    );
}

/// The same title in different tenants must succeed (constraint is per-tenant).
#[sqlx::test(migrations = "../../migrations")]
async fn customer_title_unique_per_tenant_not_global(pool: PgPool) {
    let dao = CustomerDao::new(pool.clone());
    let tenant_a = insert_tenant(&pool).await;
    let tenant_b = insert_tenant(&pool).await;

    dao.save(&make_customer(tenant_a, "Shared Title")).await.unwrap();
    // Different tenant — must not conflict.
    dao.save(&make_customer(tenant_b, "Shared Title")).await.unwrap();
}

// ── User unique email constraint test ─────────────────────────────────────────

/// CONSTRAINT user_email_unq UNIQUE (email)
/// Two users with the same email (globally) must fail.
#[sqlx::test(migrations = "../../migrations")]
async fn user_unique_email_global(pool: PgPool) {
    let dao = UserDao::new(pool);
    let email = "dup@test.com";

    dao.save(&make_user(email)).await.unwrap();

    let result = dao.save(&make_user(email)).await;
    assert!(
        matches!(result, Err(DaoError::Constraint(_))),
        "expected DaoError::Constraint for duplicate email, got: {:?}",
        result
    );
}

// ── Device unique name constraint test ────────────────────────────────────────

/// CONSTRAINT device_name_unq UNIQUE (tenant_id, name)
/// Two devices with the same name in the same tenant must fail.
#[sqlx::test(migrations = "../../migrations")]
async fn device_unique_name_per_tenant(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;

    dao.save(&make_device(tenant_id, profile_id, "Dup-Device")).await.unwrap();

    let result = dao.save(&make_device(tenant_id, profile_id, "Dup-Device")).await;
    assert!(
        matches!(result, Err(DaoError::Constraint(_))),
        "expected DaoError::Constraint for duplicate device name, got: {:?}",
        result
    );
}

// ── DeviceCredentials unique credentials_id constraint test ───────────────────

/// CONSTRAINT device_credentials_id_unq UNIQUE (credentials_id)
/// Two device_credentials rows with the same credentials_id must fail.
#[sqlx::test(migrations = "../../migrations")]
async fn device_credentials_unique_credentials_id(pool: PgPool) {
    let dao = DeviceDao::new(pool.clone());
    let tenant_id = Uuid::new_v4();
    let profile_id = insert_device_profile(&pool, tenant_id).await;

    // Two distinct devices, but same credentials_id token.
    let device_a = make_device(tenant_id, profile_id, "DeviceA-uniq-cred");
    let device_b = make_device(tenant_id, profile_id, "DeviceB-uniq-cred");
    dao.save(&device_a).await.unwrap();
    dao.save(&device_b).await.unwrap();

    let shared_token = format!("shared-token-{}", Uuid::new_v4());

    let creds_a = DeviceCredentials {
        id:               Uuid::new_v4(),
        created_time:     now_ms(),
        device_id:        device_a.id,
        credentials_type: DeviceCredentialsType::AccessToken,
        credentials_id:   shared_token.clone(),
        credentials_value: None,
    };
    dao.save_credentials(&creds_a).await.unwrap();

    // device_b tries to use the same credentials_id — save_credentials uses
    // ON CONFLICT (device_id), so it will attempt a raw INSERT for device_b
    // (new device_id). The UNIQUE (credentials_id) index must reject it.
    let creds_b = DeviceCredentials {
        id:               Uuid::new_v4(),
        created_time:     now_ms(),
        device_id:        device_b.id,
        credentials_type: DeviceCredentialsType::AccessToken,
        credentials_id:   shared_token.clone(),
        credentials_value: None,
    };
    let result = dao.save_credentials(&creds_b).await;
    assert!(
        matches!(result, Err(DaoError::Constraint(_))),
        "expected DaoError::Constraint for duplicate credentials_id, got: {:?}",
        result
    );
}

// ── Asset unique constraint tests ────────────────────────────────────────────

/// CONSTRAINT asset_name_unq UNIQUE (tenant_id, name)
#[sqlx::test(migrations = "../../migrations")]
async fn asset_unique_name_per_tenant(pool: PgPool) {
    use vl_core::entities::Asset;
    use vl_dao::postgres::asset::AssetDao;

    let dao = AssetDao::new(pool.clone());
    let tenant_id = insert_tenant(&pool).await;
    let profile_id = {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO asset_profile (id, created_time, tenant_id, name, is_default) VALUES ($1, $2, $3, $4, false)",
            id, now_ms(), tenant_id, format!("ap-{id}"),
        ).execute(&pool).await.unwrap();
        id
    };

    let make = |name: &str| Asset {
        id: Uuid::new_v4(), created_time: now_ms(), tenant_id,
        customer_id: None, asset_profile_id: profile_id,
        name: name.into(), asset_type: "DEFAULT".into(), label: None,
        external_id: None, additional_info: None, version: 1,
    };

    dao.save(&make("Dup-Asset")).await.unwrap();
    let result = dao.save(&make("Dup-Asset")).await;
    assert!(
        matches!(result, Err(DaoError::Constraint(_))),
        "expected DaoError::Constraint for duplicate asset name, got: {:?}",
        result
    );
}

/// Same asset name in different tenants must succeed.
#[sqlx::test(migrations = "../../migrations")]
async fn asset_name_unique_per_tenant_not_global(pool: PgPool) {
    use vl_core::entities::Asset;
    use vl_dao::postgres::asset::AssetDao;

    let dao = AssetDao::new(pool.clone());
    let tenant_a = insert_tenant(&pool).await;
    let tenant_b = insert_tenant(&pool).await;
    let prof_a = {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO asset_profile (id, created_time, tenant_id, name, is_default) VALUES ($1, $2, $3, $4, false)",
            id, now_ms(), tenant_a, format!("ap-{id}"),
        ).execute(&pool).await.unwrap();
        id
    };
    let prof_b = {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO asset_profile (id, created_time, tenant_id, name, is_default) VALUES ($1, $2, $3, $4, false)",
            id, now_ms(), tenant_b, format!("ap-{id}"),
        ).execute(&pool).await.unwrap();
        id
    };

    let make = |tid: Uuid, pid: Uuid| Asset {
        id: Uuid::new_v4(), created_time: now_ms(), tenant_id: tid,
        customer_id: None, asset_profile_id: pid,
        name: "Shared Name".into(), asset_type: "DEFAULT".into(), label: None,
        external_id: None, additional_info: None, version: 1,
    };

    dao.save(&make(tenant_a, prof_a)).await.unwrap();
    dao.save(&make(tenant_b, prof_b)).await.unwrap(); // Different tenant — OK
}

// ── Dashboard constraint tests ───────────────────────────────────────────────

/// Dashboards have no unique name constraint (title can be NULL).
/// Verify that two dashboards with the same title in the same tenant are allowed.
#[sqlx::test(migrations = "../../migrations")]
async fn dashboard_allows_duplicate_titles(pool: PgPool) {
    use vl_core::entities::Dashboard;
    use vl_dao::postgres::dashboard::DashboardDao;

    let dao = DashboardDao::new(pool.clone());
    let tenant_id = insert_tenant(&pool).await;

    let make = || Dashboard {
        id: Uuid::new_v4(), created_time: now_ms(), tenant_id,
        title: Some("Same Title".into()), configuration: None,
        external_id: None, mobile_hide: false, mobile_order: None, version: 1,
    };

    dao.save(&make()).await.unwrap();
    dao.save(&make()).await.unwrap(); // Should succeed — no unique constraint on title
}

// NOTE: rule_chain table has NO unique constraint on (tenant_id, name) — duplicates are allowed.
