use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use vl_core::entities::{ClusterLeaderInfo, ClusterNode, ClusterPartition, ClusterTopology};

use crate::{error::ApiError, middleware::auth::SecurityContext, state::{AppState, ClusterState, CoreState}};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cluster/info",              get(get_cluster_info))
        .route("/cluster/nodes",             get(list_nodes))
        .route("/cluster/nodes/{nodeId}",    get(get_node).delete(delete_node))
        .route("/cluster/partitions",        get(get_partitions))
        .route("/cluster/leader",            get(get_leader))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/cluster/info — ClusterTopology for the current node (SYS_ADMIN only)
async fn get_cluster_info(
    State(state): State<ClusterState>,
    State(core): State<CoreState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<ClusterTopology>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    let local_node_id = state.cluster.local_node_id().to_string();
    let all_nodes = state.cluster_node_dao.find_all_nodes().await?;

    // Build the local_node entry; fall back to a synthetic node if not yet registered.
    let local_node = all_nodes
        .iter()
        .find(|n| n.node_id == local_node_id)
        .cloned()
        .unwrap_or_else(|| ClusterNode {
            node_id:        local_node_id.clone(),
            host:           core.config.server.host.clone(),
            port:           core.config.server.port as i32,
            status:         "ACTIVE".into(),
            service_type:   "MONOLITH".into(),
            last_heartbeat: chrono::Utc::now().timestamp_millis(),
            joined_at:      chrono::Utc::now().timestamp_millis(),
            metadata:       serde_json::Value::Object(Default::default()),
            is_leader:      false,
            grpc_port:      core.config.cluster.rpc_port as i32,
            leader_epoch:   0,
        });

    let node_count = all_nodes.len().max(1) as i32;
    let partitions_per_node = 8i32;
    let total_partitions = node_count * partitions_per_node;

    // Determine which partitions belong to local node (simple round-robin assignment).
    let local_index = all_nodes
        .iter()
        .position(|n| n.node_id == local_node_id)
        .unwrap_or(0) as i32;
    let assigned_partitions: Vec<i32> = (0..total_partitions)
        .filter(|p| p % node_count == local_index)
        .collect();

    Ok(Json(ClusterTopology {
        local_node,
        nodes: all_nodes,
        total_partitions,
        assigned_partitions,
    }))
}

/// GET /api/cluster/nodes — list all nodes (SYS_ADMIN only)
async fn list_nodes(
    State(state): State<ClusterState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<Vec<ClusterNode>>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    let nodes = state.cluster_node_dao.find_all_nodes().await?;
    Ok(Json(nodes))
}

/// GET /api/cluster/nodes/{nodeId} — get a specific node (SYS_ADMIN only)
async fn get_node(
    State(state): State<ClusterState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(node_id): Path<String>,
) -> Result<Json<ClusterNode>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    let node = state.cluster_node_dao
        .find_node(&node_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Cluster node '{}' not found", node_id)))?;

    Ok(Json(node))
}

/// DELETE /api/cluster/nodes/{nodeId} — force-down a node (SYS_ADMIN only)
async fn delete_node(
    State(state): State<ClusterState>,
    Extension(ctx): Extension<SecurityContext>,
    Path(node_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    state.cluster_node_dao.mark_down(&node_id).await?;
    Ok(StatusCode::OK)
}

/// GET /api/cluster/partitions — full partition → node assignment map (SYS_ADMIN only)
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PartitionsResponse {
    total_partitions: i32,
    node_count:       i32,
    assignments:      Vec<ClusterPartition>,
}

async fn get_partitions(
    State(state): State<ClusterState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<PartitionsResponse>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    let assignments = state.cluster_partition_dao.find_all().await?;
    let node_count  = state.cluster_node_dao.find_active_nodes().await?.len().max(1) as i32;

    Ok(Json(PartitionsResponse {
        total_partitions: assignments.len().max(1) as i32,
        node_count,
        assignments,
    }))
}

/// GET /api/cluster/leader — current leader info (SYS_ADMIN only)
async fn get_leader(
    State(state): State<ClusterState>,
    Extension(ctx): Extension<SecurityContext>,
) -> Result<Json<ClusterLeaderInfo>, ApiError> {
    if !ctx.is_sys_admin() {
        return Err(ApiError::Forbidden("SYS_ADMIN authority required".into()));
    }

    let local_id = state.cluster.local_node_id().to_string();
    let leader = state.cluster_node_dao.find_leader().await?;

    let info = match leader {
        Some(mut l) => {
            l.is_local = l.leader_node_id.as_deref() == Some(&local_id);
            l
        }
        None => ClusterLeaderInfo {
            leader_node_id: None,
            host:           None,
            grpc_port:      None,
            leader_epoch:   0,
            is_local:       false,
        },
    };

    Ok(Json(info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_creates_without_panic() {
        let _ = router();
    }

    #[test]
    fn partitions_response_serializes_to_camel_case() {
        let resp = PartitionsResponse {
            total_partitions: 16,
            node_count: 2,
            assignments: vec![],
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["totalPartitions"], 16);
        assert_eq!(json["nodeCount"], 2);
        assert!(json["assignments"].is_array());
        // Ensure snake_case fields are NOT present
        assert!(json.get("total_partitions").is_none());
        assert!(json.get("node_count").is_none());
    }

    #[test]
    fn cluster_node_serializes_to_camel_case() {
        let node = ClusterNode {
            node_id:        "node-1".into(),
            host:           "127.0.0.1".into(),
            port:           8080,
            status:         "ACTIVE".into(),
            service_type:   "MONOLITH".into(),
            last_heartbeat: 1000,
            joined_at:      500,
            metadata:       serde_json::json!({}),
            is_leader:      true,
            grpc_port:      9090,
            leader_epoch:   1,
        };
        let json = serde_json::to_value(&node).unwrap();
        assert_eq!(json["nodeId"], "node-1");
        assert_eq!(json["serviceType"], "MONOLITH");
        assert_eq!(json["isLeader"], true);
        assert_eq!(json["grpcPort"], 9090);
        assert!(json.get("node_id").is_none());
    }

    #[test]
    fn cluster_leader_info_serializes_to_camel_case() {
        let info = ClusterLeaderInfo {
            leader_node_id: Some("node-1".into()),
            host:           Some("10.0.0.1".into()),
            grpc_port:      Some(9090),
            leader_epoch:   3,
            is_local:       false,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["leaderNodeId"], "node-1");
        assert_eq!(json["leaderEpoch"], 3);
        assert_eq!(json["isLocal"], false);
    }

    #[test]
    fn cluster_topology_serializes_to_camel_case() {
        let topo = ClusterTopology {
            local_node: ClusterNode {
                node_id: "n1".into(),
                host: "localhost".into(),
                port: 8080,
                status: "ACTIVE".into(),
                service_type: "MONOLITH".into(),
                last_heartbeat: 0,
                joined_at: 0,
                metadata: serde_json::json!({}),
                is_leader: false,
                grpc_port: 9090,
                leader_epoch: 0,
            },
            nodes: vec![],
            total_partitions: 8,
            assigned_partitions: vec![0, 1, 2, 3],
        };
        let json = serde_json::to_value(&topo).unwrap();
        assert!(json.get("localNode").is_some());
        assert_eq!(json["totalPartitions"], 8);
        assert_eq!(json["assignedPartitions"].as_array().unwrap().len(), 4);
    }
}
