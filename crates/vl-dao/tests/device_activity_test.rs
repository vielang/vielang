/// Integration tests cho DeviceActivityDao (Phase 31).
use sqlx::PgPool;
use uuid::Uuid;

use vl_core::entities::DeviceActivity;
use vl_dao::DeviceActivityDao;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// ── Test 1: save and find ──────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_save_and_find_activity(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);
    let device_id = Uuid::new_v4();

    let a = DeviceActivity {
        device_id,
        last_connect_ts:    1_000,
        last_disconnect_ts: 0,
        last_activity_ts:   1_000,
        last_telemetry_ts:  1_000,
        last_rpc_ts:        0,
        active:             true,
    };
    dao.save(&a).await.unwrap();

    let found = dao.find(device_id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.device_id, device_id);
    assert_eq!(found.last_connect_ts, 1_000);
    assert!(found.active);
}

// ── Test 2: upsert creates if not exists ──────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_upsert_creates_if_not_exists(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);
    let device_id = Uuid::new_v4();

    // Không có record → update_connect tạo mới
    dao.update_connect(device_id, 500).await.unwrap();
    let found = dao.find(device_id).await.unwrap().unwrap();
    assert_eq!(found.last_connect_ts, 500);
    assert!(found.active);
}

// ── Test 3: update_connect sets active=true ────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_update_connect_ts(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);
    let device_id = Uuid::new_v4();
    let ts = now_ms();

    dao.update_connect(device_id, ts).await.unwrap();
    let a = dao.find(device_id).await.unwrap().unwrap();
    assert_eq!(a.last_connect_ts, ts);
    assert!(a.active);
}

// ── Test 4: update_disconnect sets active=false ────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_update_disconnect_ts(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);
    let device_id = Uuid::new_v4();

    dao.update_connect(device_id, 1000).await.unwrap();
    let ts = now_ms();
    dao.update_disconnect(device_id, ts).await.unwrap();

    let a = dao.find(device_id).await.unwrap().unwrap();
    assert_eq!(a.last_disconnect_ts, ts);
    assert!(!a.active, "should be inactive after disconnect");
}

// ── Test 5: update_telemetry sets telemetry + activity ts ─────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_update_telemetry_ts(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);
    let device_id = Uuid::new_v4();
    let ts = now_ms();

    dao.update_telemetry(device_id, ts).await.unwrap();
    let a = dao.find(device_id).await.unwrap().unwrap();
    assert_eq!(a.last_telemetry_ts, ts);
    assert_eq!(a.last_activity_ts, ts);
    assert!(a.active);
}

// ── Test 6: set_active ────────────────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_set_active_true_false(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);
    let device_id = Uuid::new_v4();

    dao.update_connect(device_id, 1000).await.unwrap();
    assert!(dao.find(device_id).await.unwrap().unwrap().active);

    dao.set_active(device_id, false).await.unwrap();
    assert!(!dao.find(device_id).await.unwrap().unwrap().active);

    dao.set_active(device_id, true).await.unwrap();
    assert!(dao.find(device_id).await.unwrap().unwrap().active);
}

// ── Test 7: find_inactive_since ───────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_find_inactive_since_threshold(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);

    let old_device = Uuid::new_v4();
    let new_device = Uuid::new_v4();

    // old_device: last_activity_ts = 1000 (old)
    dao.update_telemetry(old_device, 1_000).await.unwrap();
    // new_device: last_activity_ts = now
    dao.update_telemetry(new_device, now_ms()).await.unwrap();

    // Threshold = 5000: devices with last_activity_ts < 5000 should be returned
    let inactive = dao.find_inactive_since(5_000).await.unwrap();
    assert!(inactive.contains(&old_device), "old_device should be inactive");
    assert!(!inactive.contains(&new_device), "new_device should not be inactive");
}

// ── Test 8: find returns None for unknown device ──────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_find_returns_none_for_unknown(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);
    let result = dao.find(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

// ── Test 9: upsert updates existing ──────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn test_upsert_updates_existing(pool: PgPool) {
    let dao = DeviceActivityDao::new(pool);
    let device_id = Uuid::new_v4();

    dao.update_connect(device_id, 1_000).await.unwrap();
    dao.update_connect(device_id, 2_000).await.unwrap();

    let a = dao.find(device_id).await.unwrap().unwrap();
    assert_eq!(a.last_connect_ts, 2_000, "should have latest connect ts");
}
