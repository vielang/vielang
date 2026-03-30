/// Integration tests cho UserDao (bao gồm Phase 2: activate_user, reset_password_token).
use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{Authority, User, UserCredentials};
use vl_dao::{postgres::user::UserDao, DaoError, PageLink};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn make_user(tenant_id: Uuid) -> User {
    User {
        id:              Uuid::new_v4(),
        created_time:    now_ms(),
        tenant_id,
        customer_id:     None,
        email:           format!("test-{}@example.com", Uuid::new_v4()),
        authority:       Authority::TenantAdmin,
        first_name:      Some("John".into()),
        last_name:       Some("Doe".into()),
        phone:           None,
        additional_info: None,
        version:         1,
    }
}

fn make_credentials(user_id: Uuid, enabled: bool, password: Option<&str>) -> UserCredentials {
    UserCredentials {
        id:              Uuid::new_v4(),
        created_time:    now_ms(),
        user_id,
        enabled,
        password:        password.map(|s| s.to_string()),
        activate_token:  None,
        reset_token:     None,
        additional_info: None,
    }
}

// ── User CRUD ─────────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = UserDao::new(pool);
    let user = make_user(Uuid::new_v4());

    let saved = dao.save(&user).await.unwrap();
    assert_eq!(saved.id, user.id);
    assert_eq!(saved.email, user.email);
    assert_eq!(saved.authority, Authority::TenantAdmin);
    assert_eq!(saved.first_name.as_deref(), Some("John"));

    let found = dao.find_by_id(user.id).await.unwrap().unwrap();
    assert_eq!(found.id, user.id);
    assert_eq!(found.email, user.email);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = UserDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_email_exact_match(pool: PgPool) {
    let dao = UserDao::new(pool);
    let user = make_user(Uuid::new_v4());
    dao.save(&user).await.unwrap();

    let found = dao.find_by_email(&user.email).await.unwrap().unwrap();
    assert_eq!(found.id, user.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_email_case_insensitive(pool: PgPool) {
    let dao = UserDao::new(pool);
    let mut user = make_user(Uuid::new_v4());
    user.email = "Test.User@Example.COM".into();
    dao.save(&user).await.unwrap();

    let found1 = dao.find_by_email("test.user@example.com").await.unwrap().unwrap();
    let found2 = dao.find_by_email("TEST.USER@EXAMPLE.COM").await.unwrap().unwrap();
    assert_eq!(found1.id, user.id);
    assert_eq!(found2.id, user.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_email_returns_none_for_unknown(pool: PgPool) {
    let dao = UserDao::new(pool);
    assert!(dao.find_by_email("nobody@nowhere.com").await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_user_increments_version(pool: PgPool) {
    let dao = UserDao::new(pool);
    let mut user = make_user(Uuid::new_v4());
    dao.save(&user).await.unwrap();

    user.first_name = Some("Jane".into());
    let updated = dao.save(&user).await.unwrap();
    assert_eq!(updated.first_name.as_deref(), Some("Jane"));
    assert_eq!(updated.version, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_sys_admin_user(pool: PgPool) {
    let dao = UserDao::new(pool);
    let mut user = make_user(Uuid::nil()); // SYS_ADMIN không cần tenant cụ thể
    user.authority = Authority::SysAdmin;
    dao.save(&user).await.unwrap();

    let found = dao.find_by_id(user.id).await.unwrap().unwrap();
    assert_eq!(found.authority, Authority::SysAdmin);
}

// ── find_by_tenant + pagination ───────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = UserDao::new(pool);
    let tenant_id = Uuid::new_v4();
    let other_tenant = Uuid::new_v4();

    for i in 0..5u32 {
        let mut u = make_user(tenant_id);
        u.email = format!("tenant-user-{i}@example.com");
        dao.save(&u).await.unwrap();
    }
    // User của tenant khác — không được lọt vào kết quả
    dao.save(&make_user(other_tenant)).await.unwrap();

    let page = dao.find_by_tenant(tenant_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page.total_elements, 5);
    assert_eq!(page.data.len(), 3);
    assert!(page.has_next);

    let page2 = dao.find_by_tenant(tenant_id, &PageLink::new(1, 3)).await.unwrap();
    assert_eq!(page2.data.len(), 2);
    assert!(!page2.has_next);
}

// ── UserCredentials ───────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_credentials(pool: PgPool) {
    let dao = UserDao::new(pool);
    let user = make_user(Uuid::new_v4());
    dao.save(&user).await.unwrap();

    let creds = make_credentials(user.id, true, Some("$argon2id$v=19$m=19456,t=2,p=1$salt$hash"));
    dao.save_credentials(&creds).await.unwrap();

    let found = dao.find_credentials(user.id).await.unwrap().unwrap();
    assert_eq!(found.user_id, user.id);
    assert!(found.enabled);
    assert!(found.password.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_credentials_returns_none_for_unknown_user(pool: PgPool) {
    let dao = UserDao::new(pool);
    assert!(dao.find_credentials(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_credentials_upserts(pool: PgPool) {
    let dao = UserDao::new(pool);
    let user = make_user(Uuid::new_v4());
    dao.save(&user).await.unwrap();

    // Lưu lần đầu
    let creds = make_credentials(user.id, false, None);
    dao.save_credentials(&creds).await.unwrap();

    // Lưu lại với enabled = true và password (ON CONFLICT DO UPDATE)
    let mut updated = creds.clone();
    updated.enabled = true;
    updated.password = Some("new-hash".into());
    dao.save_credentials(&updated).await.unwrap();

    let found = dao.find_credentials(user.id).await.unwrap().unwrap();
    assert!(found.enabled);
    assert_eq!(found.password.as_deref(), Some("new-hash"));
}

// ── Phase 2: activate_user ────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn activate_user_sets_password_and_enables(pool: PgPool) {
    let dao = UserDao::new(pool);
    let user = make_user(Uuid::new_v4());
    dao.save(&user).await.unwrap();

    // Credentials với activate_token và disabled
    let creds = UserCredentials {
        id:             Uuid::new_v4(),
        created_time:   now_ms(),
        user_id:        user.id,
        enabled:        false,
        password:       None,
        activate_token: Some("valid-activate-token-abc".into()),
        reset_token:    None,
        additional_info: None,
    };
    dao.save_credentials(&creds).await.unwrap();

    let new_hash = "$argon2id$v=19$m=19456,t=2,p=1$saltsalt$hashvalue";
    let activated = dao.activate_user("valid-activate-token-abc", new_hash).await.unwrap();
    assert_eq!(activated.id, user.id);

    let updated_creds = dao.find_credentials(user.id).await.unwrap().unwrap();
    assert!(updated_creds.enabled, "User phải được enable sau khi activate");
    assert!(updated_creds.activate_token.is_none(), "activate_token phải bị xóa");
    assert_eq!(updated_creds.password.as_deref(), Some(new_hash));
}

#[sqlx::test(migrations = "../../migrations")]
async fn activate_user_with_invalid_token_returns_not_found(pool: PgPool) {
    let dao = UserDao::new(pool);
    let result = dao.activate_user("nonexistent-token-xyz", "some-hash").await;
    assert!(matches!(result, Err(DaoError::NotFound)));
}

// ── Phase 2: reset_password_token ─────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn reset_password_token_stores_token(pool: PgPool) {
    let dao = UserDao::new(pool);
    let user = make_user(Uuid::new_v4());
    dao.save(&user).await.unwrap();

    let creds = make_credentials(user.id, true, Some("current-hash"));
    dao.save_credentials(&creds).await.unwrap();

    dao.reset_password_token(&user.email, "reset-token-123456").await.unwrap();

    let updated = dao.find_credentials(user.id).await.unwrap().unwrap();
    assert_eq!(updated.reset_token.as_deref(), Some("reset-token-123456"));
    // Password vẫn không đổi
    assert_eq!(updated.password.as_deref(), Some("current-hash"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn reset_password_token_unknown_email_returns_not_found(pool: PgPool) {
    let dao = UserDao::new(pool);
    let result = dao.reset_password_token("nobody@example.com", "token").await;
    assert!(matches!(result, Err(DaoError::NotFound)));
}
