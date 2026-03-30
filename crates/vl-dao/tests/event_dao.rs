/// Integration tests for EventDao.
mod helpers;

use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::{Event, EventType, EventFilter};
use vl_dao::{postgres::event::EventDao, PageLink};

fn make_event(tenant_id: Uuid, entity_id: Uuid) -> Event {
    Event {
        id:          Uuid::new_v4(),
        created_time: helpers::now_ms(),
        tenant_id,
        entity_id,
        entity_type: "DEVICE".into(),
        event_type:  EventType::LcEvent,
        event_uid:   format!("evt-{}", Uuid::new_v4()),
        body:        serde_json::json!({"event": "STARTED", "success": true}),
    }
}

fn empty_filter() -> EventFilter {
    EventFilter {
        event_type: None,
        start_ts:   None,
        end_ts:     None,
    }
}

// ── CRUD tests ───────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn save_and_find_by_id(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    let event = make_event(tenant_id, entity_id);
    let saved = dao.save(&event).await.unwrap();
    assert_eq!(saved.id, event.id);
    assert_eq!(saved.tenant_id, tenant_id);

    let found = dao.find_by_id(event.id).await.unwrap().unwrap();
    assert_eq!(found.id, event.id);
    assert_eq!(found.entity_id, entity_id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_id_returns_none_for_unknown(pool: PgPool) {
    let dao = EventDao::new(pool);
    assert!(dao.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

// ── Query by entity tests ────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_entity_pagination(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();
    let other_entity = Uuid::new_v4();

    for _ in 0..5 {
        dao.save(&make_event(tenant_id, entity_id)).await.unwrap();
    }
    // Different entity — should not appear
    dao.save(&make_event(tenant_id, other_entity)).await.unwrap();

    let page0 = dao.find_by_entity(
        tenant_id, entity_id, "DEVICE", &empty_filter(), &PageLink::new(0, 3),
    ).await.unwrap();
    assert_eq!(page0.total_elements, 5);
    assert_eq!(page0.data.len(), 3);
    assert!(page0.has_next);
}

#[sqlx::test(migrations = "../../migrations")]
async fn find_by_entity_with_type_filter(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    // LC event
    dao.save(&make_event(tenant_id, entity_id)).await.unwrap();

    // Error event
    let mut error_event = make_event(tenant_id, entity_id);
    error_event.event_type = EventType::Error;
    dao.save(&error_event).await.unwrap();

    let filter = EventFilter {
        event_type: Some(EventType::LcEvent),
        start_ts: None,
        end_ts: None,
    };
    let page = dao.find_by_entity(
        tenant_id, entity_id, "DEVICE", &filter, &PageLink::new(0, 10),
    ).await.unwrap();
    assert_eq!(page.total_elements, 1);
}

// ── Delete by entity tests ───────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn delete_by_entity(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    for _ in 0..3 {
        dao.save(&make_event(tenant_id, entity_id)).await.unwrap();
    }

    let deleted = dao.delete_by_entity(tenant_id, entity_id, "DEVICE", &empty_filter())
        .await.unwrap();
    assert_eq!(deleted, 3);

    let page = dao.find_by_entity(
        tenant_id, entity_id, "DEVICE", &empty_filter(), &PageLink::new(0, 10),
    ).await.unwrap();
    assert_eq!(page.total_elements, 0);
}

// ── Event types query ────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn get_event_types(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    dao.save(&make_event(tenant_id, entity_id)).await.unwrap(); // LC_EVENT

    let mut error_event = make_event(tenant_id, entity_id);
    error_event.event_type = EventType::Error;
    dao.save(&error_event).await.unwrap();

    let types = dao.get_event_types(tenant_id, entity_id, "DEVICE").await.unwrap();
    assert!(types.len() >= 2);
}

// ── Cleanup tests ────────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn cleanup_old_events(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    // Create events with old timestamps
    let mut old_event = make_event(tenant_id, entity_id);
    old_event.created_time = 1000; // very old
    dao.save(&old_event).await.unwrap();

    let recent_event = make_event(tenant_id, entity_id);
    dao.save(&recent_event).await.unwrap();

    // Cleanup events older than recent
    let retention_ts = helpers::now_ms() - 1000;
    let cleaned = dao.cleanup_old_events(tenant_id, retention_ts).await.unwrap();
    assert!(cleaned >= 1, "should have cleaned at least the old event");
}

// ── Partitioned event save methods ───────────────────────────────────────────
// These write to separate partitioned tables (lc_event, error_event, stats_event),
// NOT the main `event` table. We verify that the insert succeeds without error.

#[sqlx::test(migrations = "../../migrations")]
async fn save_lc_event_succeeds(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    dao.save_lc_event(
        tenant_id, helpers::now_ms(), entity_id, "vielang-1",
        "STARTED", true, None,
    ).await.unwrap();

    // Also test with error
    dao.save_lc_event(
        tenant_id, helpers::now_ms(), entity_id, "vielang-1",
        "STOPPED", false, Some("shutdown signal received"),
    ).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_error_event_succeeds(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    dao.save_error_event(
        tenant_id, helpers::now_ms(), entity_id, "vielang-1",
        "onMsg", Some("NullPointerException"),
    ).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn save_stats_event_succeeds(pool: PgPool) {
    let dao = EventDao::new(pool.clone());
    let tenant_id = helpers::insert_tenant(&pool).await;
    let entity_id = Uuid::new_v4();

    dao.save_stats_event(
        tenant_id, helpers::now_ms(), entity_id, "vielang-1", 100, 5,
    ).await.unwrap();
}
