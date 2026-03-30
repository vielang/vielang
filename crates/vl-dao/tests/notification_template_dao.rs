/// Integration tests for NotificationTemplateDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{NotificationTemplate, NotificationType};
use vl_dao::{postgres::notification_template::NotificationTemplateDao, DaoError, PageLink};

fn make_template(tenant_id: Uuid) -> NotificationTemplate {
    let id = Uuid::new_v4();
    NotificationTemplate {
        id,
        created_time:      helpers::now_ms(),
        tenant_id,
        name:              format!("Template-{id}"),
        notification_type: NotificationType::Email,
        subject_template:  Some("Alert: {{alarmType}}".into()),
        body_template:     "Device {{deviceName}} triggered alarm".into(),
        additional_config: None,
        enabled:           true,
        version:           1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = NotificationTemplateDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let tmpl = make_template(tenant_id);

    let saved = dao.save(&tmpl).await.unwrap();
    assert_eq!(saved.id, tmpl.id);
    assert_eq!(saved.name, tmpl.name);

    let found = dao.find_by_id(tmpl.id).await.unwrap().unwrap();
    assert_eq!(found.id, tmpl.id);
    assert_eq!(found.body_template, tmpl.body_template);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none(pool: PgPool) {
    let dao = NotificationTemplateDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_tenant_pagination(pool: PgPool) {
    let dao = NotificationTemplateDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;

    for _ in 0..5 {
        dao.save(&make_template(tenant_id)).await.unwrap();
    }

    let page = dao.find_by_tenant(tenant_id, &PageLink::new(0, 3)).await.unwrap();
    assert_eq!(page.total_elements, 5);
    assert_eq!(page.data.len(), 3);
    assert!(page.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_template(pool: PgPool) {
    let dao = NotificationTemplateDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let tmpl = make_template(tenant_id);
    dao.save(&tmpl).await.unwrap();
    dao.delete(tmpl.id).await.unwrap();
    assert!(dao.find_by_id(tmpl.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_template_via_save(pool: PgPool) {
    let dao = NotificationTemplateDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let mut tmpl = make_template(tenant_id);
    dao.save(&tmpl).await.unwrap();

    tmpl.body_template = "Updated body".into();
    tmpl.enabled = false;
    let updated = dao.save(&tmpl).await.unwrap();
    assert_eq!(updated.body_template, "Updated body");
    assert!(!updated.enabled);
}
