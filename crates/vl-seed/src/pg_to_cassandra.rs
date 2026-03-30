//! Migration tool: PostgreSQL ts_kv → Cassandra ts_kv_cf
//!
//! Usage:
//!   cargo run -p vl-seed --bin pg-to-cassandra -- \
//!     --pg-url postgres://vielang:vielang@localhost:5432/vielang \
//!     --cassandra-url 127.0.0.1:9042 \
//!     --keyspace thingsboard \
//!     [--batch-size 1000] \
//!     [--entity-type DEVICE]   # optional filter

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use vl_cassandra::{CassandraCluster, CassandraTs, PartitionGranularity};
use vl_config::CassandraConfig;
use vl_core::entities::TsRecord;
use vl_dao::TimeseriesDao;

// ── CLI args ──────────────────────────────────────────────────────────────────

struct Args {
    pg_url:        String,
    cassandra_url:  String,
    keyspace:      String,
    batch_size:    usize,
    entity_type:   Option<String>,
    latest_only:   bool,
    dry_run:       bool,
}

fn parse_args() -> Args {
    let mut args = std::env::args().skip(1);
    let mut pg_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://vielang:vielang@localhost:5432/vielang".into());
    let mut cassandra_url = "127.0.0.1:9042".to_string();
    let mut keyspace = "thingsboard".to_string();
    let mut batch_size: usize = 500;
    let mut entity_type: Option<String> = None;
    let mut latest_only = false;
    let mut dry_run = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pg-url"        => pg_url        = args.next().expect("--pg-url value"),
            "--cassandra-url" => cassandra_url = args.next().expect("--cassandra-url value"),
            "--keyspace"      => keyspace      = args.next().expect("--keyspace value"),
            "--batch-size"    => batch_size    = args.next().expect("--batch-size value")
                                    .parse().expect("batch-size must be integer"),
            "--entity-type"   => entity_type   = Some(args.next().expect("--entity-type value")),
            "--latest-only"   => latest_only   = true,
            "--dry-run"       => dry_run       = true,
            other => eprintln!("Unknown arg: {other}"),
        }
    }

    Args { pg_url, cassandra_url, keyspace, batch_size, entity_type, latest_only, dry_run }
}

// ── Entity type resolution ─────────────────────────────────────────────────

/// Build a map entity_id → entity_type by querying each entity table.
/// Only includes entity_ids present in ts_kv_latest (fast lookup).
async fn build_entity_type_map(pool: &sqlx::PgPool) -> Result<HashMap<Uuid, String>> {
    let mut map: HashMap<Uuid, String> = HashMap::new();

    // Lấy tất cả entity_ids từ ts_kv_latest
    let ids: Vec<Uuid> = sqlx::query_scalar!("SELECT DISTINCT entity_id FROM ts_kv_latest")
        .fetch_all(pool)
        .await
        .context("query ts_kv_latest entity ids")?;

    if ids.is_empty() {
        return Ok(map);
    }

    // Query từng bảng entity theo thứ tự phổ biến
    let device_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM device WHERE id = ANY($1)",
        &ids as &[Uuid]
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for id in device_ids { map.insert(id, "DEVICE".into()); }

    let asset_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM asset WHERE id = ANY($1)",
        &ids as &[Uuid]
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for id in asset_ids { map.insert(id, "ASSET".into()); }

    let customer_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM customer WHERE id = ANY($1)",
        &ids as &[Uuid]
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for id in customer_ids { map.insert(id, "CUSTOMER".into()); }

    let tenant_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM tenant WHERE id = ANY($1)",
        &ids as &[Uuid]
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for id in tenant_ids { map.insert(id, "TENANT".into()); }

    let user_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM tb_user WHERE id = ANY($1)",
        &ids as &[Uuid]
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for id in user_ids { map.insert(id, "USER".into()); }

    let dashboard_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM dashboard WHERE id = ANY($1)",
        &ids as &[Uuid]
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for id in dashboard_ids { map.insert(id, "DASHBOARD".into()); }

    // Entities không tìm thấy → default DEVICE (most common in IoT)
    let unknown = ids.iter().filter(|id| !map.contains_key(*id)).count();
    if unknown > 0 {
        warn!("{} entity_ids not found in any entity table — defaulting to DEVICE", unknown);
        for id in &ids {
            map.entry(*id).or_insert_with(|| "DEVICE".into());
        }
    }

    Ok(map)
}

// ── Migration helpers ─────────────────────────────────────────────────────

struct Row {
    entity_id: Uuid,
    key_name:  String,
    ts:        i64,
    bool_v:    Option<bool>,
    str_v:     Option<String>,
    long_v:    Option<i64>,
    dbl_v:     Option<f64>,
    json_v:    Option<serde_json::Value>,
}

async fn migrate_latest(
    pool:       &sqlx::PgPool,
    ts_dao:     &CassandraTs,
    entity_map: &HashMap<Uuid, String>,
    filter_et:  Option<&str>,
    dry_run:    bool,
    batch_size: usize,
) -> Result<u64> {
    // Key lookup cache
    let key_names: Vec<(i32, String)> =
        sqlx::query!("SELECT key_id, key FROM key_dictionary")
            .fetch_all(pool)
            .await
            .context("load key_dictionary")?
            .into_iter()
            .map(|r| (r.key_id, r.key))
            .collect();
    let key_map: HashMap<i32, String> = key_names.into_iter().collect();

    let rows = sqlx::query!(
        "SELECT entity_id, key, ts, bool_v, str_v, long_v, dbl_v,
                json_v::text AS json_v
         FROM ts_kv_latest
         ORDER BY entity_id"
    )
    .fetch_all(pool)
    .await
    .context("fetch ts_kv_latest")?;

    let mut written: u64 = 0;
    let mut batch: Vec<(String, TsRecord)> = Vec::with_capacity(batch_size);

    for r in rows {
        let et = entity_map.get(&r.entity_id).map(|s| s.as_str()).unwrap_or("DEVICE");
        if let Some(filter) = filter_et {
            if !et.eq_ignore_ascii_case(filter) { continue; }
        }
        let key_name = match key_map.get(&r.key) {
            Some(k) => k.clone(),
            None    => { warn!("key_id {} not found in key_dictionary", r.key); continue; }
        };

        let record = TsRecord {
            entity_id: r.entity_id,
            key:       key_name,
            ts:        r.ts,
            bool_v:    r.bool_v,
            str_v:     r.str_v,
            long_v:    r.long_v,
            dbl_v:     r.dbl_v,
            json_v:    r.json_v.and_then(|s| serde_json::from_str(&s).ok()),
        };

        batch.push((et.to_string(), record));

        if batch.len() >= batch_size {
            flush_batch(&batch, ts_dao, dry_run, true).await?;
            written += batch.len() as u64;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        let n = batch.len() as u64;
        flush_batch(&batch, ts_dao, dry_run, true).await?;
        written += n;
    }

    Ok(written)
}

async fn migrate_history(
    pool:       &sqlx::PgPool,
    ts_dao:     &CassandraTs,
    entity_map: &HashMap<Uuid, String>,
    filter_et:  Option<&str>,
    dry_run:    bool,
    batch_size: usize,
) -> Result<u64> {
    let key_names: Vec<(i32, String)> =
        sqlx::query!("SELECT key_id, key FROM key_dictionary")
            .fetch_all(pool)
            .await
            .context("load key_dictionary")?
            .into_iter()
            .map(|r| (r.key_id, r.key))
            .collect();
    let key_map: HashMap<i32, String> = key_names.into_iter().collect();

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM ts_kv")
        .fetch_one(pool)
        .await
        .context("count ts_kv")?
        .unwrap_or(0);

    info!("Migrating {} historical timeseries rows in batches of {}", total, batch_size);

    let mut written: u64 = 0;
    let mut offset: i64 = 0;

    loop {
        let rows = sqlx::query!(
            "SELECT entity_id, key, ts, bool_v, str_v, long_v, dbl_v,
                    json_v::text AS json_v
             FROM ts_kv
             ORDER BY entity_id, ts
             LIMIT $1 OFFSET $2",
            batch_size as i64,
            offset,
        )
        .fetch_all(pool)
        .await
        .context("fetch ts_kv batch")?;

        if rows.is_empty() { break; }

        let mut batch: Vec<(String, TsRecord)> = Vec::with_capacity(rows.len());
        for r in &rows {
            let et = entity_map.get(&r.entity_id).map(|s| s.as_str()).unwrap_or("DEVICE");
            if let Some(filter) = filter_et {
                if !et.eq_ignore_ascii_case(filter) { continue; }
            }
            let key_name = match key_map.get(&r.key) {
                Some(k) => k.clone(),
                None    => { warn!("key_id {} not found", r.key); continue; }
            };

            batch.push((et.to_string(), TsRecord {
                entity_id: r.entity_id,
                key:       key_name,
                ts:        r.ts,
                bool_v:    r.bool_v,
                str_v:     r.str_v.clone(),
                long_v:    r.long_v,
                dbl_v:     r.dbl_v,
                json_v:    r.json_v.as_deref().and_then(|s| serde_json::from_str(s).ok()),
            }));
        }

        flush_batch(&batch, ts_dao, dry_run, false).await?;
        written += batch.len() as u64;
        offset  += rows.len() as i64;

        if written % 10_000 == 0 || written == total as u64 {
            info!("Progress: {}/{} rows migrated", written, total);
        }

        if rows.len() < batch_size { break; }
    }

    Ok(written)
}

async fn flush_batch(
    batch:   &[(String, TsRecord)],
    ts_dao:  &CassandraTs,
    dry_run: bool,
    latest:  bool,
) -> Result<()> {
    if dry_run { return Ok(()); }

    // Group by entity_type để gọi save_batch
    let mut by_type: HashMap<&str, Vec<&TsRecord>> = HashMap::new();
    for (et, rec) in batch {
        by_type.entry(et.as_str()).or_default().push(rec);
    }
    for (et, recs) in by_type {
        if latest {
            ts_dao.save_latest_batch(et, &recs.into_iter().cloned().collect::<Vec<_>>())
                .await
                .context("save_latest_batch")?;
        } else {
            ts_dao.save_batch(et, &recs.into_iter().cloned().collect::<Vec<_>>())
                .await
                .context("save_batch")?;
        }
    }
    Ok(())
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = parse_args();

    let start = Utc::now();
    info!("=== PostgreSQL → Cassandra migration ===");
    info!("PG:        {}", args.pg_url);
    info!("Cassandra: {} / keyspace={}", args.cassandra_url, args.keyspace);
    if args.dry_run { info!("DRY RUN — no data will be written to Cassandra"); }

    // ── Connect PostgreSQL ──────────────────────────────────────────────────
    let pool = sqlx::PgPool::connect(&args.pg_url)
        .await
        .context("connect PostgreSQL")?;
    info!("PostgreSQL connected");

    // ── Connect Cassandra ───────────────────────────────────────────────────
    let cass_config = CassandraConfig {
        url:                  args.cassandra_url.clone(),
        keyspace:             args.keyspace.clone(),
        local_datacenter:     "datacenter1".into(),
        partition_granularity: "MONTHS".into(),
        ttl_seconds:          -1,
        partition_cache_size: 200_000,
        username:             None,
        password:             None,
    };

    let cluster = CassandraCluster::connect(&cass_config)
        .await
        .context("connect Cassandra")?;

    let ts_dao = CassandraTs::new(
        cluster.session(),
        cluster.keyspace(),
        PartitionGranularity::Months,
        -1,
        200_000,
    )
    .await
    .context("prepare Cassandra statements")?;

    // ── Build entity_id → entity_type map ──────────────────────────────────
    info!("Resolving entity types...");
    let entity_map = build_entity_type_map(&pool).await?;
    info!("Found {} unique entities in ts_kv_latest", entity_map.len());

    let filter = args.entity_type.as_deref();

    // ── Migrate latest values ───────────────────────────────────────────────
    info!("--- Migrating latest values (ts_kv_latest) ---");
    let n_latest = migrate_latest(&pool, &ts_dao, &entity_map, filter, args.dry_run, args.batch_size)
        .await
        .context("migrate latest")?;
    info!("Latest: {} rows written", n_latest);

    // ── Migrate historical timeseries ───────────────────────────────────────
    if !args.latest_only {
        info!("--- Migrating history (ts_kv) ---");
        let n_hist = migrate_history(&pool, &ts_dao, &entity_map, filter, args.dry_run, args.batch_size)
            .await
            .context("migrate history")?;
        info!("History: {} rows written", n_hist);
    }

    let elapsed = Utc::now().signed_duration_since(start);
    info!(
        "Migration complete in {}s. Latest={}, History={}",
        elapsed.num_seconds(),
        n_latest,
        if args.latest_only { 0 } else { 0 },  // printed above
    );

    Ok(())
}
