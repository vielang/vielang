// Shared test helpers for vl-dao integration tests.
// Usage: `mod helpers;` at the top of any test file, then call `helpers::insert_tenant(&pool).await`.

#![allow(dead_code)]

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{
    Alarm, AlarmComment, AlarmSeverity, Asset, Customer, Dashboard,
    Device, EntityRelation, EntityType,
    RelationTypeGroup, RuleChain, Tenant, User, Authority,
};

// ── Timestamp ────────────────────────────────────────────────────────────────

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// ── FK Insert Helpers (raw SQL for parent entities) ──────────────────────────

/// Insert a tenant_profile row and return its UUID.
pub async fn insert_tenant_profile(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO tenant_profile (id, created_time, name, is_default, isolated_vl_rule_engine)
           VALUES ($1, $2, $3, false, false)"#,
        id,
        now_ms(),
        format!("profile-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Insert tenant_profile + tenant, return the tenant UUID.
pub async fn insert_tenant(pool: &PgPool) -> Uuid {
    let profile_id = insert_tenant_profile(pool).await;
    let tenant_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO tenant (id, created_time, title, tenant_profile_id, region, version)
           VALUES ($1, $2, $3, $4, 'EU', 1)"#,
        tenant_id,
        now_ms(),
        format!("Tenant-{tenant_id}"),
        profile_id,
    )
    .execute(pool)
    .await
    .unwrap();
    tenant_id
}

/// Insert a customer row for the given tenant, return its UUID.
pub async fn insert_customer(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO customer (id, created_time, tenant_id, title, is_public, version)
           VALUES ($1, $2, $3, $4, false, 1)"#,
        id,
        now_ms(),
        tenant_id,
        format!("Customer-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Insert a device_profile for the given tenant, return its UUID.
pub async fn insert_device_profile(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO device_profile
           (id, created_time, tenant_id, name, type, transport_type, provision_type, is_default)
           VALUES ($1, $2, $3, $4, 'DEFAULT', 'DEFAULT', 'DISABLED', false)"#,
        id,
        now_ms(),
        tenant_id,
        format!("dp-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Insert an asset_profile for the given tenant, return its UUID.
pub async fn insert_asset_profile(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO asset_profile
           (id, created_time, tenant_id, name, is_default)
           VALUES ($1, $2, $3, $4, false)"#,
        id,
        now_ms(),
        tenant_id,
        format!("ap-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Insert a device row, return its UUID.
pub async fn insert_device(pool: &PgPool, tenant_id: Uuid, profile_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO device
           (id, created_time, tenant_id, device_profile_id, name, type, version)
           VALUES ($1, $2, $3, $4, $5, 'DEFAULT', 1)"#,
        id,
        now_ms(),
        tenant_id,
        profile_id,
        format!("Device-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Insert an asset row, return its UUID.
pub async fn insert_asset(pool: &PgPool, tenant_id: Uuid, profile_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO asset
           (id, created_time, tenant_id, asset_profile_id, name, type, version)
           VALUES ($1, $2, $3, $4, $5, 'DEFAULT', 1)"#,
        id,
        now_ms(),
        tenant_id,
        profile_id,
        format!("Asset-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Insert a user row (tb_user table), return its UUID.
pub async fn insert_user(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO tb_user
           (id, created_time, tenant_id, email, authority, version)
           VALUES ($1, $2, $3, $4, 'TENANT_ADMIN', 1)"#,
        id,
        now_ms(),
        tenant_id,
        format!("user-{id}@test.com"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Insert a rule_chain row, return its UUID.
pub async fn insert_rule_chain(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO rule_chain
           (id, created_time, tenant_id, name, type, root, debug_mode, version)
           VALUES ($1, $2, $3, $4, 'CORE', false, false, 1)"#,
        id,
        now_ms(),
        tenant_id,
        format!("RuleChain-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Insert a dashboard row, return its UUID.
pub async fn insert_dashboard(pool: &PgPool, tenant_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO dashboard
           (id, created_time, tenant_id, title, mobile_hide, version)
           VALUES ($1, $2, $3, $4, false, 1)"#,
        id,
        now_ms(),
        tenant_id,
        format!("Dashboard-{id}"),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

// ── Entity Builder Helpers ───────────────────────────────────────────────────

pub fn make_tenant(profile_id: Uuid) -> Tenant {
    Tenant {
        id:                Uuid::new_v4(),
        created_time:      now_ms(),
        title:             format!("Tenant-{}", Uuid::new_v4()),
        tenant_profile_id: profile_id,
        region:            Some("EU".into()),
        country:           None,
        state:             None,
        city:              None,
        address:           None,
        address2:          None,
        zip:               None,
        phone:             None,
        email:             None,
        additional_info:   None,
        version:           1,
    }
}

pub fn make_customer(tenant_id: Uuid, title: &str) -> Customer {
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

pub fn make_device(tenant_id: Uuid, profile_id: Uuid) -> Device {
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

pub fn make_asset(tenant_id: Uuid, profile_id: Uuid) -> Asset {
    Asset {
        id:               Uuid::new_v4(),
        created_time:     now_ms(),
        tenant_id,
        customer_id:      None,
        asset_profile_id: profile_id,
        name:             format!("Asset-{}", Uuid::new_v4()),
        asset_type:       "DEFAULT".into(),
        label:            None,
        external_id:      None,
        additional_info:  None,
        version:          1,
    }
}

pub fn make_dashboard(tenant_id: Uuid) -> Dashboard {
    Dashboard {
        id:            Uuid::new_v4(),
        created_time:  now_ms(),
        tenant_id,
        title:         Some(format!("Dashboard-{}", Uuid::new_v4())),
        configuration: None,
        external_id:   None,
        mobile_hide:   false,
        mobile_order:  None,
        version:       1,
    }
}

pub fn make_alarm(tenant_id: Uuid, originator_id: Uuid) -> Alarm {
    let ts = now_ms();
    Alarm {
        id:                       Uuid::new_v4(),
        created_time:             ts,
        tenant_id,
        customer_id:              None,
        alarm_type:               format!("ALARM-{}", Uuid::new_v4()),
        originator_id,
        originator_type:          EntityType::Device,
        severity:                 AlarmSeverity::Warning,
        acknowledged:             false,
        cleared:                  false,
        assignee_id:              None,
        start_ts:                 ts,
        end_ts:                   ts,
        ack_ts:                   None,
        clear_ts:                 None,
        assign_ts:                0,
        propagate:                false,
        propagate_to_owner:       false,
        propagate_to_tenant:      false,
        propagate_relation_types: None,
        details:                  None,
    }
}

pub fn make_alarm_comment(alarm_id: Uuid) -> AlarmComment {
    AlarmComment {
        id:           Uuid::new_v4(),
        created_time: now_ms(),
        alarm_id,
        user_id:      None,
        comment_type: "OTHER".into(),
        comment:      serde_json::json!({"text": "test comment"}),
    }
}

pub fn make_relation(
    from_id: Uuid,
    from_type: EntityType,
    to_id: Uuid,
    to_type: EntityType,
) -> EntityRelation {
    EntityRelation {
        from_id,
        from_type,
        to_id,
        to_type,
        relation_type:       "Contains".into(),
        relation_type_group: RelationTypeGroup::Common,
        additional_info:     None,
    }
}

pub fn make_rule_chain(tenant_id: Uuid) -> RuleChain {
    RuleChain {
        id:                 Uuid::new_v4(),
        created_time:       now_ms(),
        tenant_id,
        name:               format!("RuleChain-{}", Uuid::new_v4()),
        chain_type:         "CORE".into(),
        first_rule_node_id: None,
        root:               false,
        debug_mode:         false,
        configuration:      None,
        additional_info:    None,
        external_id:        None,
        version:            1,
    }
}

pub fn make_user(tenant_id: Uuid, email: &str) -> User {
    User {
        id:              Uuid::new_v4(),
        created_time:    now_ms(),
        tenant_id,
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
