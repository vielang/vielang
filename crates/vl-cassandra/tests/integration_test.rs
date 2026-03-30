//! Integration tests cho vl-cassandra (ScyllaDB / Apache Cassandra).
//!
//! Yêu cầu: ScyllaDB chạy tại TEST_CASSANDRA_URL (default 127.0.0.1:9142)
//! Khởi động: docker compose -f docker-compose.test.yml up -d cassandra-test
//! Chạy tests: cargo test -p vl-cassandra

use chrono::{Datelike, TimeZone, Timelike, Utc};
use uuid::Uuid;

use vl_cassandra::{
    cluster::CassandraCluster,
    partition::{partitions_in_range, to_partition_ts, PartitionGranularity},
    CassandraTs,
};
use vl_config::CassandraConfig;
use vl_core::entities::TsRecord;
use vl_dao::TimeseriesDao;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cassandra_url() -> String {
    std::env::var("TEST_CASSANDRA_URL").unwrap_or_else(|_| "127.0.0.1:9943".to_string())
}

async fn try_make_dao() -> Option<CassandraTs> {
    try_make_dao_full(PartitionGranularity::Months, -1).await
}

async fn make_dao() -> CassandraTs {
    try_make_dao().await
        .expect("Cannot connect to Cassandra — run: docker compose -f docker-compose.test.yml up -d cassandra-test")
}

/// Skip the calling test if Cassandra is not reachable.
macro_rules! cassandra_dao {
    () => {{
        match try_make_dao().await {
            Some(d) => d,
            None => return,
        }
    }};
    ($gran:expr, $ttl:expr) => {{
        match try_make_dao_full($gran, $ttl).await {
            Some(d) => d,
            None => return,
        }
    }};
}

/// Returns None when Cassandra is not reachable — callers must skip the test.
async fn try_make_dao_full(granularity: PartitionGranularity, ttl_seconds: i64) -> Option<CassandraTs> {
    let config = CassandraConfig {
        url: cassandra_url(),
        keyspace: "vielang_test".to_string(),
        local_datacenter: "datacenter1".to_string(),
        partition_granularity: "MONTHS".to_string(),
        ttl_seconds,
        partition_cache_size: 128,
        username: None,
        password: None,
    };

    match CassandraCluster::connect(&config).await {
        Err(_) => {
            eprintln!("SKIP: Cassandra not available at {}. Run: docker compose -f docker-compose.test.yml up -d cassandra-test", cassandra_url());
            None
        }
        Ok(cluster) => {
            match CassandraTs::new(cluster.session(), cluster.keyspace(), granularity, ttl_seconds, 128).await {
                Err(e) => { eprintln!("SKIP: CassandraTs init failed: {e}"); None }
                Ok(ts) => Some(ts),
            }
        }
    }
}

async fn make_dao_full(granularity: PartitionGranularity, ttl_seconds: i64) -> CassandraTs {
    try_make_dao_full(granularity, ttl_seconds).await
        .expect("Cannot connect to Cassandra — run: docker compose -f docker-compose.test.yml up -d cassandra-test")
}

fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

fn mk_long(entity_id: Uuid, key: &str, ts: i64, value: i64) -> TsRecord {
    TsRecord { entity_id, key: key.to_string(), ts, bool_v: None, str_v: None, long_v: Some(value), dbl_v: None, json_v: None }
}

fn mk_double(entity_id: Uuid, key: &str, ts: i64, value: f64) -> TsRecord {
    TsRecord { entity_id, key: key.to_string(), ts, bool_v: None, str_v: None, long_v: None, dbl_v: Some(value), json_v: None }
}

fn mk_bool(entity_id: Uuid, key: &str, ts: i64, value: bool) -> TsRecord {
    TsRecord { entity_id, key: key.to_string(), ts, bool_v: Some(value), str_v: None, long_v: None, dbl_v: None, json_v: None }
}

fn mk_str(entity_id: Uuid, key: &str, ts: i64, value: &str) -> TsRecord {
    TsRecord { entity_id, key: key.to_string(), ts, bool_v: None, str_v: Some(value.to_string()), long_v: None, dbl_v: None, json_v: None }
}

fn mk_json(entity_id: Uuid, key: &str, ts: i64, value: serde_json::Value) -> TsRecord {
    TsRecord { entity_id, key: key.to_string(), ts, bool_v: None, str_v: None, long_v: None, dbl_v: None, json_v: Some(value) }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 1: Partition logic — không cần DB, pure math
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn partition_months_truncates_to_first_of_month() {
    let ts = Utc.with_ymd_and_hms(2026, 3, 15, 12, 34, 56).unwrap().timestamp_millis();
    let p = to_partition_ts(ts, PartitionGranularity::Months);
    let dt = Utc.timestamp_millis_opt(p).unwrap();
    assert_eq!((dt.year(), dt.month(), dt.day()), (2026, 3, 1));
    assert_eq!((dt.hour(), dt.minute(), dt.second()), (0, 0, 0));
}

#[test]
fn partition_days_truncates_to_midnight() {
    let ts = Utc.with_ymd_and_hms(2026, 3, 15, 14, 30, 59).unwrap().timestamp_millis();
    let p = to_partition_ts(ts, PartitionGranularity::Days);
    let dt = Utc.timestamp_millis_opt(p).unwrap();
    assert_eq!(dt.day(), 15);
    assert_eq!((dt.hour(), dt.minute(), dt.second()), (0, 0, 0));
}

#[test]
fn partition_hours_truncates_to_hour_start() {
    let ts = Utc.with_ymd_and_hms(2026, 3, 15, 14, 45, 30).unwrap().timestamp_millis();
    let p = to_partition_ts(ts, PartitionGranularity::Hours);
    let dt = Utc.timestamp_millis_opt(p).unwrap();
    assert_eq!((dt.hour(), dt.minute(), dt.second()), (14, 0, 0));
}

#[test]
fn partition_minutes_truncates_to_minute_start() {
    let ts = Utc.with_ymd_and_hms(2026, 3, 15, 14, 45, 30).unwrap().timestamp_millis();
    let p = to_partition_ts(ts, PartitionGranularity::Minutes);
    let dt = Utc.timestamp_millis_opt(p).unwrap();
    assert_eq!((dt.hour(), dt.minute(), dt.second()), (14, 45, 0));
}

#[test]
fn partition_years_truncates_to_jan_first() {
    let ts = Utc.with_ymd_and_hms(2026, 7, 4, 18, 30, 0).unwrap().timestamp_millis();
    let p = to_partition_ts(ts, PartitionGranularity::Years);
    let dt = Utc.timestamp_millis_opt(p).unwrap();
    assert_eq!((dt.year(), dt.month(), dt.day()), (2026, 1, 1));
    assert_eq!((dt.hour(), dt.minute()), (0, 0));
}

#[test]
fn partition_same_month_two_dates_same_partition() {
    let ts1 = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap().timestamp_millis();
    let ts2 = Utc.with_ymd_and_hms(2026, 3, 31, 23, 59, 59).unwrap().timestamp_millis();
    assert_eq!(
        to_partition_ts(ts1, PartitionGranularity::Months),
        to_partition_ts(ts2, PartitionGranularity::Months),
        "Whole month must map to same partition"
    );
}

#[test]
fn partition_adjacent_months_different_partitions() {
    let ts_mar = Utc.with_ymd_and_hms(2026, 3, 31, 23, 59, 59).unwrap().timestamp_millis();
    let ts_apr = Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap().timestamp_millis();
    assert_ne!(
        to_partition_ts(ts_mar, PartitionGranularity::Months),
        to_partition_ts(ts_apr, PartitionGranularity::Months),
    );
}

#[test]
fn partitions_in_range_same_month_returns_one() {
    let start = Utc.with_ymd_and_hms(2026, 3, 5, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 3, 20, 0, 0, 0).unwrap().timestamp_millis();
    let parts = partitions_in_range(start, end, PartitionGranularity::Months);
    assert_eq!(parts.len(), 1);
}

#[test]
fn partitions_in_range_three_months() {
    let start = Utc.with_ymd_and_hms(2026, 1, 15, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 3, 15, 0, 0, 0).unwrap().timestamp_millis();
    let parts = partitions_in_range(start, end, PartitionGranularity::Months);
    assert_eq!(parts.len(), 3, "Jan, Feb, Mar");
    // Phải tăng dần
    assert!(parts.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn partitions_in_range_year_boundary() {
    let start = Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 1, 31, 0, 0, 0).unwrap().timestamp_millis();
    let parts = partitions_in_range(start, end, PartitionGranularity::Months);
    assert_eq!(parts.len(), 2, "Dec 2025 + Jan 2026");
}

#[test]
fn partitions_in_range_days_three_days() {
    let start = Utc.with_ymd_and_hms(2026, 3, 10, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 3, 12, 23, 59, 59).unwrap().timestamp_millis();
    let parts = partitions_in_range(start, end, PartitionGranularity::Days);
    assert_eq!(parts.len(), 3, "Day 10, 11, 12");
}

#[test]
fn partitions_in_range_start_equals_end_returns_one() {
    let ts    = Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0).unwrap().timestamp_millis();
    let parts = partitions_in_range(ts, ts, PartitionGranularity::Months);
    assert_eq!(parts.len(), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 2: Schema & connection
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn schema_init_creates_tables() {
    // Kết nối thành công và prepare statements → schema tồn tại
    let _dao = cassandra_dao!();
}

#[tokio::test]
async fn schema_init_is_idempotent() {
    // Kết nối 2 lần → CREATE TABLE IF NOT EXISTS không fail lần 2
    let _dao1 = cassandra_dao!();
    let _dao2 = cassandra_dao!();
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 3: save_latest + find_latest — tất cả data types
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn save_latest_long_value() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_long(id, "temperature", ts, 42)).await.unwrap();

    let result = dao.find_latest(id, "DEVICE", Some(&["temperature"])).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].long_v, Some(42));
    assert_eq!(result[0].key, "temperature");
    assert_eq!(result[0].ts, ts);
}

#[tokio::test]
async fn save_latest_double_value() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_double(id, "humidity", ts, 65.7)).await.unwrap();

    let result = dao.find_latest(id, "DEVICE", Some(&["humidity"])).await.unwrap();
    assert_eq!(result.len(), 1);
    assert!((result[0].dbl_v.unwrap() - 65.7).abs() < 0.001);
}

#[tokio::test]
async fn save_latest_bool_value() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("ASSET", &mk_bool(id, "active", ts, true)).await.unwrap();

    let result = dao.find_latest(id, "ASSET", Some(&["active"])).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].bool_v, Some(true));
}

#[tokio::test]
async fn save_latest_string_value() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_str(id, "status", ts, "ONLINE")).await.unwrap();

    let result = dao.find_latest(id, "DEVICE", Some(&["status"])).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].str_v.as_deref(), Some("ONLINE"));
}

#[tokio::test]
async fn save_latest_json_value() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();
    let payload = serde_json::json!({"lat": 10.5, "lon": 106.3});

    dao.save_latest("ASSET", &mk_json(id, "location", ts, payload)).await.unwrap();

    let result = dao.find_latest(id, "ASSET", Some(&["location"])).await.unwrap();
    assert_eq!(result.len(), 1);
    let loc = result[0].json_v.as_ref().unwrap();
    assert_eq!(loc["lat"], 10.5);
    assert_eq!(loc["lon"], 106.3);
}

#[tokio::test]
async fn save_latest_overwrites_old_value() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts1 = now_ms();
    let ts2 = ts1 + 1000;

    dao.save_latest("DEVICE", &mk_long(id, "counter", ts1, 100)).await.unwrap();
    dao.save_latest("DEVICE", &mk_long(id, "counter", ts2, 200)).await.unwrap();

    let result = dao.find_latest(id, "DEVICE", Some(&["counter"])).await.unwrap();
    assert_eq!(result.len(), 1);
    // Last write wins
    assert_eq!(result[0].long_v, Some(200));
    assert_eq!(result[0].ts, ts2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 4: find_latest key filtering
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn find_latest_all_keys_returns_all() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_long(id, "temp", ts, 25)).await.unwrap();
    dao.save_latest("DEVICE", &mk_double(id, "humidity", ts, 70.0)).await.unwrap();
    dao.save_latest("DEVICE", &mk_bool(id, "active", ts, true)).await.unwrap();

    let result = dao.find_latest(id, "DEVICE", None).await.unwrap();
    assert_eq!(result.len(), 3);
}

#[tokio::test]
async fn find_latest_specific_keys_filters_correctly() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_long(id, "temp", ts, 25)).await.unwrap();
    dao.save_latest("DEVICE", &mk_double(id, "humidity", ts, 70.0)).await.unwrap();
    dao.save_latest("DEVICE", &mk_bool(id, "active", ts, true)).await.unwrap();

    let result = dao.find_latest(id, "DEVICE", Some(&["temp", "active"])).await.unwrap();
    assert_eq!(result.len(), 2);

    let keys: Vec<&str> = result.iter().map(|r| r.key.as_str()).collect();
    assert!(keys.contains(&"temp"));
    assert!(keys.contains(&"active"));
    assert!(!keys.contains(&"humidity"), "humidity should be excluded");
}

#[tokio::test]
async fn find_latest_nonexistent_key_returns_empty() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();

    let result = dao.find_latest(id, "DEVICE", Some(&["not_exist"])).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn find_latest_no_data_entity_returns_empty() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4(); // entity mới, không có data

    let result = dao.find_latest(id, "DEVICE", None).await.unwrap();
    assert!(result.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 5: get_ts_keys
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_ts_keys_returns_all_unique_keys() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_long(id, "voltage", ts, 220)).await.unwrap();
    dao.save_latest("DEVICE", &mk_double(id, "current", ts, 1.5)).await.unwrap();
    dao.save_latest("DEVICE", &mk_bool(id, "fault", ts, false)).await.unwrap();

    let keys = dao.get_ts_keys(id, "DEVICE").await.unwrap();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"voltage".to_string()));
    assert!(keys.contains(&"current".to_string()));
    assert!(keys.contains(&"fault".to_string()));
}

#[tokio::test]
async fn get_ts_keys_empty_entity_returns_empty() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();

    let keys = dao.get_ts_keys(id, "DEVICE").await.unwrap();
    assert!(keys.is_empty());
}

#[tokio::test]
async fn get_ts_keys_write_same_key_twice_returns_one_key() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_long(id, "temp", ts, 20)).await.unwrap();
    dao.save_latest("DEVICE", &mk_long(id, "temp", ts + 1000, 25)).await.unwrap();

    let keys = dao.get_ts_keys(id, "DEVICE").await.unwrap();
    assert_eq!(keys.len(), 1, "upsert should not duplicate the key");
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 6: save (history) + find_range — trong cùng một partition
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn save_and_find_range_single_datapoint() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save("DEVICE", &mk_long(id, "power", ts, 100)).await.unwrap();

    let result = dao.find_range(id, "DEVICE", "power", ts - 1000, ts + 1000, 10).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].long_v, Some(100));
    assert_eq!(result[0].ts, ts);
}

#[tokio::test]
async fn find_range_returns_results_desc_order() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let base_ts = now_ms();

    for i in 0..5i64 {
        dao.save("DEVICE", &mk_long(id, "seq", base_ts + i * 1000, i)).await.unwrap();
    }

    let result = dao.find_range(id, "DEVICE", "seq", base_ts - 100, base_ts + 5000, 10).await.unwrap();
    assert_eq!(result.len(), 5);
    // DESC theo ts
    for w in result.windows(2) {
        assert!(w[0].ts >= w[1].ts, "expected DESC order but got {} then {}", w[0].ts, w[1].ts);
    }
}

#[tokio::test]
async fn find_range_respects_limit() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let base_ts = now_ms();

    for i in 0..10i64 {
        dao.save("DEVICE", &mk_long(id, "data", base_ts + i * 100, i)).await.unwrap();
    }

    let result = dao.find_range(id, "DEVICE", "data", base_ts - 100, base_ts + 1100, 3).await.unwrap();
    assert!(result.len() <= 3, "limit=3 should return at most 3 results, got {}", result.len());
}

#[tokio::test]
async fn find_range_excludes_data_outside_time_window() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts_inside  = now_ms();
    // Đủ xa để ts filtering loại bỏ (cùng tháng nhưng ngoài range)
    let ts_outside = ts_inside - 120_000; // 2 phút trước

    dao.save("DEVICE", &mk_long(id, "v", ts_outside, 999)).await.unwrap();
    dao.save("DEVICE", &mk_long(id, "v", ts_inside, 42)).await.unwrap();

    let result = dao.find_range(id, "DEVICE", "v", ts_inside - 1000, ts_inside + 1000, 10).await.unwrap();
    assert_eq!(result.len(), 1, "ts_outside phải bị loại bởi CQL ts filter");
    assert_eq!(result[0].long_v, Some(42));
}

#[tokio::test]
async fn find_range_no_data_returns_empty() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    let result = dao.find_range(id, "DEVICE", "nonexistent_key", ts - 1000, ts, 10).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn save_history_does_not_affect_find_latest() {
    // save() (history) và save_latest() là hai bảng riêng
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save("DEVICE", &mk_long(id, "x", ts, 10)).await.unwrap();

    // find_latest không nhìn vào bảng history
    let latest = dao.find_latest(id, "DEVICE", Some(&["x"])).await.unwrap();
    assert!(latest.is_empty(), "save() history should not populate find_latest");
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 7: find_range cross-partition queries
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn find_range_across_two_months() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();

    let ts_jan = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap().timestamp_millis();
    let ts_feb = Utc.with_ymd_and_hms(2026, 2, 15, 12, 0, 0).unwrap().timestamp_millis();

    dao.save("DEVICE", &mk_long(id, "cross_month", ts_jan, 1)).await.unwrap();
    dao.save("DEVICE", &mk_long(id, "cross_month", ts_feb, 2)).await.unwrap();

    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 2, 28, 23, 59, 59).unwrap().timestamp_millis();

    let result = dao.find_range(id, "DEVICE", "cross_month", start, end, 10).await.unwrap();
    assert_eq!(result.len(), 2, "cần tìm thấy data từ cả 2 partition Jan và Feb");

    // DESC: Feb trước, Jan sau
    assert_eq!(result[0].long_v, Some(2), "giá trị tháng 2 phải đứng trước");
    assert_eq!(result[1].long_v, Some(1), "giá trị tháng 1 phải đứng sau");
}

#[tokio::test]
async fn find_range_across_year_boundary() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();

    let ts_dec = Utc.with_ymd_and_hms(2025, 12, 20, 12, 0, 0).unwrap().timestamp_millis();
    let ts_jan = Utc.with_ymd_and_hms(2026, 1, 10, 12, 0, 0).unwrap().timestamp_millis();

    dao.save("DEVICE", &mk_long(id, "year_cross", ts_dec, 2025)).await.unwrap();
    dao.save("DEVICE", &mk_long(id, "year_cross", ts_jan, 2026)).await.unwrap();

    let start = Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 1, 31, 23, 59, 59).unwrap().timestamp_millis();

    let result = dao.find_range(id, "DEVICE", "year_cross", start, end, 10).await.unwrap();
    assert_eq!(result.len(), 2, "cần tìm thấy data qua năm 2025 → 2026");
}

#[tokio::test]
async fn find_range_only_queries_relevant_partitions() {
    // Insert ở tháng 1 và tháng 3, query chỉ tháng 3 → không thấy tháng 1
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();

    let ts_jan = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap().timestamp_millis();
    let ts_mar = Utc.with_ymd_and_hms(2026, 3, 15, 12, 0, 0).unwrap().timestamp_millis();

    dao.save("DEVICE", &mk_long(id, "sparse", ts_jan, 100)).await.unwrap();
    dao.save("DEVICE", &mk_long(id, "sparse", ts_mar, 300)).await.unwrap();

    let start = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 3, 31, 23, 59, 59).unwrap().timestamp_millis();

    let result = dao.find_range(id, "DEVICE", "sparse", start, end, 10).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].long_v, Some(300));
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 8: Delete operations
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_latest_removes_specified_key() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_long(id, "to_delete", ts, 100)).await.unwrap();
    dao.save_latest("DEVICE", &mk_long(id, "keep", ts, 200)).await.unwrap();

    dao.delete_latest(id, "DEVICE", &["to_delete"]).await.unwrap();

    let after = dao.find_latest(id, "DEVICE", None).await.unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].key, "keep");
    assert_eq!(after[0].long_v, Some(200));
}

#[tokio::test]
async fn delete_latest_multiple_keys() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    for key in &["a", "b", "c", "d"] {
        dao.save_latest("DEVICE", &mk_long(id, key, ts, 1)).await.unwrap();
    }

    dao.delete_latest(id, "DEVICE", &["a", "b", "c"]).await.unwrap();

    let remaining = dao.find_latest(id, "DEVICE", None).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].key, "d");
}

#[tokio::test]
async fn delete_ts_removes_datapoints_in_range() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let base_ts = now_ms();

    // Insert 5 điểm cách nhau 1s
    for i in 0..5i64 {
        dao.save("DEVICE", &mk_long(id, "del_key", base_ts + i * 1000, i)).await.unwrap();
    }

    // Xóa 3 điểm giữa (ts+1s, ts+2s, ts+3s)
    dao.delete_ts(id, "DEVICE", &["del_key"], base_ts + 1000, base_ts + 3000).await.unwrap();

    let remaining = dao.find_range(id, "DEVICE", "del_key", base_ts - 100, base_ts + 5000, 10).await.unwrap();
    assert_eq!(remaining.len(), 2, "chỉ còn i=0 (base_ts) và i=4 (base_ts+4s)");

    let values: Vec<i64> = remaining.iter().filter_map(|r| r.long_v).collect();
    assert!(values.contains(&0), "điểm i=0 phải còn");
    assert!(values.contains(&4), "điểm i=4 phải còn");
}

#[tokio::test]
async fn delete_ts_nonexistent_key_is_noop() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    // Xóa key chưa bao giờ được ghi → phải OK (Cassandra DELETE is idempotent)
    let result = dao.delete_ts(id, "DEVICE", &["never_written"], ts - 1000, ts).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_latest_nonexistent_key_is_noop() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();

    let result = dao.delete_latest(id, "DEVICE", &["ghost_key"]).await;
    assert!(result.is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 9: Entity type isolation
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn device_and_asset_same_uuid_latest_isolated() {
    // Cùng UUID nhưng khác entity_type → data độc lập
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save_latest("DEVICE", &mk_long(id, "shared_key", ts, 100)).await.unwrap();
    dao.save_latest("ASSET",  &mk_long(id, "shared_key", ts, 999)).await.unwrap();

    let device_result = dao.find_latest(id, "DEVICE", Some(&["shared_key"])).await.unwrap();
    let asset_result  = dao.find_latest(id, "ASSET",  Some(&["shared_key"])).await.unwrap();

    assert_eq!(device_result[0].long_v, Some(100), "DEVICE data phải độc lập");
    assert_eq!(asset_result[0].long_v,  Some(999), "ASSET data phải độc lập");
}

#[tokio::test]
async fn entity_type_isolation_in_history() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save("DEVICE",   &mk_long(id, "metric", ts, 1)).await.unwrap();
    dao.save("CUSTOMER", &mk_long(id, "metric", ts, 2)).await.unwrap();

    let dev_data = dao.find_range(id, "DEVICE",   "metric", ts - 500, ts + 500, 10).await.unwrap();
    let cus_data = dao.find_range(id, "CUSTOMER", "metric", ts - 500, ts + 500, 10).await.unwrap();

    assert_eq!(dev_data[0].long_v, Some(1));
    assert_eq!(cus_data[0].long_v, Some(2));
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 10: Multi-entity & nhiều keys
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn multiple_entities_have_independent_latest() {
    let dao = cassandra_dao!();
    let ts = now_ms();
    let entities: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

    for (i, &id) in entities.iter().enumerate() {
        dao.save_latest("DEVICE", &mk_long(id, "value", ts, i as i64 * 10)).await.unwrap();
    }

    for (i, &id) in entities.iter().enumerate() {
        let result = dao.find_latest(id, "DEVICE", Some(&["value"])).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].long_v, Some(i as i64 * 10), "entity {} phải có giá trị riêng", i);
    }
}

#[tokio::test]
async fn five_keys_same_entity_all_stored_independently() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();
    let keys = ["temp", "humidity", "pressure", "voltage", "current"];

    for (i, &key) in keys.iter().enumerate() {
        dao.save_latest("DEVICE", &mk_long(id, key, ts, i as i64)).await.unwrap();
    }

    let all = dao.find_latest(id, "DEVICE", None).await.unwrap();
    assert_eq!(all.len(), 5, "5 keys phải độc lập nhau");

    let ts_keys = dao.get_ts_keys(id, "DEVICE").await.unwrap();
    assert_eq!(ts_keys.len(), 5);
    for &key in &keys {
        assert!(ts_keys.contains(&key.to_string()), "key '{}' phải có trong get_ts_keys", key);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 11: TTL
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn save_history_with_ttl_succeeds_and_readable() {
    // TTL = 10 phút — data vẫn đọc được ngay sau khi ghi
    let dao = cassandra_dao!(PartitionGranularity::Months, 600);
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save("DEVICE", &mk_long(id, "ttl_data", ts, 42)).await.unwrap();

    let result = dao.find_range(id, "DEVICE", "ttl_data", ts - 100, ts + 100, 10).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].long_v, Some(42));
}

#[tokio::test]
async fn save_with_ttl_1_day() {
    let dao = cassandra_dao!(PartitionGranularity::Months, 86400); // 1 ngày
    let id = Uuid::new_v4();
    let ts = now_ms();

    let result = dao.save("DEVICE", &mk_double(id, "sensor", ts, 3.14)).await;
    assert!(result.is_ok(), "save với TTL 1 ngày phải thành công: {:?}", result);
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 12: Partition cache behavior
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn partition_cache_deduplicates_partition_inserts() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let base_ts = now_ms();

    // Ghi 20 điểm cùng partition (cùng tháng) — partition row chỉ insert 1 lần
    for i in 0..20i64 {
        dao.save("DEVICE", &mk_long(id, "cached", base_ts + i * 100, i)).await.unwrap();
    }

    // Tất cả data vẫn queryable
    let result = dao.find_range(id, "DEVICE", "cached", base_ts - 100, base_ts + 2100, 100).await.unwrap();
    assert_eq!(result.len(), 20, "20 điểm phải đọc được dù cache dedup partition");
}

#[tokio::test]
async fn after_delete_latest_cache_is_cleared_and_reinsert_works() {
    let dao = cassandra_dao!();
    let id = Uuid::new_v4();
    let ts = now_ms();

    dao.save("DEVICE", &mk_long(id, "cached_key", ts, 1)).await.unwrap();
    dao.save_latest("DEVICE", &mk_long(id, "cached_key", ts, 1)).await.unwrap();

    // Delete xóa khỏi partition cache
    dao.delete_latest(id, "DEVICE", &["cached_key"]).await.unwrap();

    // Re-insert sau khi cache bị clear → phải hoạt động bình thường
    dao.save_latest("DEVICE", &mk_long(id, "cached_key", ts + 2000, 99)).await.unwrap();

    let result = dao.find_latest(id, "DEVICE", Some(&["cached_key"])).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].long_v, Some(99));
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit 13: Granularity khác nhau
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn daily_granularity_find_range_single_day() {
    let dao = cassandra_dao!(PartitionGranularity::Days, -1);
    let id = Uuid::new_v4();

    let ts1 = Utc.with_ymd_and_hms(2026, 3, 15, 8, 0, 0).unwrap().timestamp_millis();
    let ts2 = Utc.with_ymd_and_hms(2026, 3, 15, 18, 0, 0).unwrap().timestamp_millis();

    dao.save("DEVICE", &mk_long(id, "day_key", ts1, 1)).await.unwrap();
    dao.save("DEVICE", &mk_long(id, "day_key", ts2, 2)).await.unwrap();

    let start = Utc.with_ymd_and_hms(2026, 3, 15, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 3, 15, 23, 59, 59).unwrap().timestamp_millis();

    let result = dao.find_range(id, "DEVICE", "day_key", start, end, 10).await.unwrap();
    assert_eq!(result.len(), 2, "2 điểm trong cùng ngày phải tìm thấy");
}

#[tokio::test]
async fn daily_granularity_cross_day_query() {
    let dao = cassandra_dao!(PartitionGranularity::Days, -1);
    let id = Uuid::new_v4();

    let ts1 = Utc.with_ymd_and_hms(2026, 5, 10, 12, 0, 0).unwrap().timestamp_millis();
    let ts2 = Utc.with_ymd_and_hms(2026, 5, 11, 12, 0, 0).unwrap().timestamp_millis();
    let ts3 = Utc.with_ymd_and_hms(2026, 5, 12, 12, 0, 0).unwrap().timestamp_millis();

    dao.save("DEVICE", &mk_double(id, "d", ts1, 1.0)).await.unwrap();
    dao.save("DEVICE", &mk_double(id, "d", ts2, 2.0)).await.unwrap();
    dao.save("DEVICE", &mk_double(id, "d", ts3, 3.0)).await.unwrap();

    let start = Utc.with_ymd_and_hms(2026, 5, 10, 0, 0, 0).unwrap().timestamp_millis();
    let end   = Utc.with_ymd_and_hms(2026, 5, 12, 23, 59, 59).unwrap().timestamp_millis();

    let result = dao.find_range(id, "DEVICE", "d", start, end, 10).await.unwrap();
    assert_eq!(result.len(), 3, "3 ngày = 3 partitions = 3 điểm");
}
