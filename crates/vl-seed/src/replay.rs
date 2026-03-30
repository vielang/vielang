/// Replay tool — re-publish TbMsg messages captured in DEBUG_RULE_NODE events.
///
/// Usage:
///   DATABASE_URL=... REPLAY_NODE_ID=<uuid> cargo run -p vl-seed -- replay
///   DATABASE_URL=... REPLAY_NODE_ID=<uuid> REPLAY_LIMIT=20 cargo run -p vl-seed -- replay
///   DATABASE_URL=... REPLAY_NODE_ID=<uuid> RULE_ENGINE_URL=http://localhost:9090 cargo run -p vl-seed -- replay
///
/// Behaviour:
///   - Reads the last REPLAY_LIMIT (default: 10) DEBUG_RULE_NODE events for the node.
///   - Reconstructs TbMsg from the event body.
///   - If RULE_ENGINE_URL is set, POSTs each message to POST /api/v1/{deviceToken}/telemetry
///     (for telemetry messages) or just prints a dry-run summary otherwise.
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
struct ReplayMsg {
    event_id:      Uuid,
    created_time:  i64,
    msg_type:      String,
    data:          String,
    metadata:      String,
    relation_type: String,
}

pub async fn run(pool: &PgPool) -> anyhow::Result<()> {
    let node_id_str = std::env::var("REPLAY_NODE_ID")
        .unwrap_or_else(|_| Uuid::nil().to_string());
    let node_id = Uuid::parse_str(&node_id_str)?;

    let limit: i64 = std::env::var("REPLAY_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    println!("🔁 Replaying last {} debug events for node {}", limit, node_id);

    let rows = sqlx::query!(
        r#"
        SELECT id, created_time, body
        FROM event
        WHERE entity_id = $1
          AND entity_type = 'RULE_NODE'
          AND event_type  = 'DEBUG_RULE_NODE'
        ORDER BY created_time DESC
        LIMIT $2
        "#,
        node_id,
        limit,
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        println!("  ℹ No debug events found for this node.");
        return Ok(());
    }

    let mut msgs: Vec<ReplayMsg> = Vec::new();
    for row in rows {
        let body = &row.body;
        msgs.push(ReplayMsg {
            event_id:      row.id,
            created_time:  row.created_time,
            msg_type:      body["msg_type"].as_str().unwrap_or("").to_string(),
            data:          body["data"].as_str().unwrap_or("{}").to_string(),
            metadata:      body["metadata"].as_str().unwrap_or("{}").to_string(),
            relation_type: body["relation_type"].as_str().unwrap_or("Success").to_string(),
        });
    }

    let engine_url = std::env::var("RULE_ENGINE_URL").ok();

    for (i, msg) in msgs.iter().enumerate() {
        println!(
            "  [{}/{}] event={} ts={} type={} relation={} data={}",
            i + 1, msgs.len(),
            msg.event_id, msg.created_time,
            msg.msg_type, msg.relation_type,
            &msg.data[..msg.data.len().min(80)],
        );

        if let Some(ref base_url) = engine_url {
            // POST to the rule engine telemetry ingestion endpoint
            let url = format!("{}/api/v1/replay/telemetry", base_url.trim_end_matches('/'));
            let payload = serde_json::json!({
                "msgType":  msg.msg_type,
                "data":     msg.data,
                "metadata": msg.metadata,
                "nodeId":   node_id,
            });
            match reqwest::Client::new().post(&url).json(&payload).send().await {
                Ok(resp) => println!("    → POST {} {}", url, resp.status()),
                Err(e)   => println!("    → POST failed: {}", e),
            }
        }
    }

    if engine_url.is_none() {
        println!("\n  ℹ Dry-run mode. Set RULE_ENGINE_URL to replay messages to the engine.");
    }

    Ok(())
}
