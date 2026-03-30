use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, instrument};
use uuid::Uuid;

use vl_core::entities::{
    CommitRequest, VersionCreateRequest, VersionCreateRequestType, VersionRequestStatus,
};
use vl_dao::EntityVersionDao;

/// A job submitted to the background version-creation worker.
struct VersionJob {
    request_id: Uuid,
    tenant_id: Uuid,
    user_id: Uuid,
    request: VersionCreateRequest,
}

/// Async version-control service that processes version create requests
/// sequentially via an mpsc channel, tracking status per request_id.
#[derive(Clone)]
pub struct VersionControlService {
    tx: mpsc::Sender<VersionJob>,
    statuses: Arc<RwLock<HashMap<Uuid, VersionRequestStatus>>>,
}

impl VersionControlService {
    /// Spawn the background worker and return a handle to submit/query jobs.
    #[instrument(skip(version_dao))]
    pub fn start(version_dao: Arc<EntityVersionDao>) -> Self {
        let (tx, rx) = mpsc::channel::<VersionJob>(256);
        let statuses: Arc<RwLock<HashMap<Uuid, VersionRequestStatus>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let svc = Self {
            tx,
            statuses: statuses.clone(),
        };

        tokio::spawn(Self::run_loop(rx, statuses, version_dao));

        info!("VersionControlService started");
        svc
    }

    /// Submit an async version-create request. Returns the request_id to poll.
    #[instrument(skip(self, request))]
    pub async fn submit(
        &self,
        request: VersionCreateRequest,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, String> {
        let request_id = Uuid::new_v4();

        // Pre-populate status as "in progress"
        {
            let mut map = self.statuses.write().await;
            map.insert(
                request_id,
                VersionRequestStatus {
                    request_id,
                    done: false,
                    added: 0,
                    modified: 0,
                    removed: 0,
                    error: None,
                },
            );
        }

        self.tx
            .send(VersionJob {
                request_id,
                tenant_id,
                user_id,
                request,
            })
            .await
            .map_err(|_| "Version control service channel closed".to_string())?;

        Ok(request_id)
    }

    /// Poll the status of an async version request.
    pub async fn get_status(&self, request_id: Uuid) -> Option<VersionRequestStatus> {
        let map = self.statuses.read().await;
        map.get(&request_id).cloned()
    }

    /// Background loop — processes jobs sequentially.
    async fn run_loop(
        mut rx: mpsc::Receiver<VersionJob>,
        statuses: Arc<RwLock<HashMap<Uuid, VersionRequestStatus>>>,
        version_dao: Arc<EntityVersionDao>,
    ) {
        while let Some(job) = rx.recv().await {
            let result = Self::process_job(&version_dao, &job).await;

            let mut map = statuses.write().await;
            match result {
                Ok(status) => {
                    map.insert(job.request_id, status);
                }
                Err(err_msg) => {
                    map.insert(
                        job.request_id,
                        VersionRequestStatus {
                            request_id: job.request_id,
                            done: true,
                            added: 0,
                            modified: 0,
                            removed: 0,
                            error: Some(err_msg),
                        },
                    );
                }
            }
        }
    }

    /// Process a single version-create job.
    async fn process_job(
        version_dao: &EntityVersionDao,
        job: &VersionJob,
    ) -> Result<VersionRequestStatus, String> {
        let req = &job.request;
        let mut added: i64 = 0;

        match req.request_type {
            VersionCreateRequestType::SingleEntity => {
                let entity_id = req
                    .entity_id
                    .ok_or_else(|| "entityId is required for SINGLE_ENTITY request".to_string())?;
                let entity_type = req
                    .entity_type
                    .as_deref()
                    .ok_or_else(|| {
                        "entityType is required for SINGLE_ENTITY request".to_string()
                    })?;

                let commit = CommitRequest {
                    entity_id,
                    entity_type: entity_type.to_string(),
                    snapshot: serde_json::json!({}),
                    commit_msg: Some(req.version_name.clone()),
                };

                version_dao
                    .commit(job.tenant_id, Some(job.user_id), &commit)
                    .await
                    .map_err(|e| format!("Failed to commit version: {e}"))?;

                added = 1;
                info!(
                    request_id = %job.request_id,
                    entity_id = %entity_id,
                    "SingleEntity version created"
                );
            }
            VersionCreateRequestType::Complex => {
                let configs = req.entity_types.as_deref().unwrap_or_default();

                for cfg in configs {
                    // For each entity type config, create a placeholder snapshot.
                    // In a full implementation this would enumerate entities of that type.
                    let commit = CommitRequest {
                        entity_id: Uuid::new_v4(),
                        entity_type: cfg.entity_type.clone(),
                        snapshot: serde_json::json!({
                            "saveRelations": cfg.save_relations,
                            "saveAttributes": cfg.save_attributes,
                            "saveCredentials": cfg.save_credentials,
                        }),
                        commit_msg: Some(req.version_name.clone()),
                    };

                    version_dao
                        .commit(job.tenant_id, Some(job.user_id), &commit)
                        .await
                        .map_err(|e| format!("Failed to commit version: {e}"))?;

                    added += 1;
                }

                info!(
                    request_id = %job.request_id,
                    entity_types = configs.len(),
                    "Complex version created"
                );
            }
        }

        // Also commit to git repo if configured.
        if let Err(e) = GitBackend::commit_to_git(job.tenant_id, &job.request.version_name, added) {
            warn!(tenant = %job.tenant_id, "Git commit failed (non-fatal): {e}");
        }

        Ok(VersionRequestStatus {
            request_id: job.request_id,
            done: true,
            added,
            modified: 0,
            removed: 0,
            error: None,
        })
    }
}

// ── Git Backend ──────────────────────────────────────────────────────────────

/// Git-based version control backend — stores entity snapshots in per-tenant git repos.
///
/// Repo location: `./data/vc/{tenant_id}/`
pub struct GitBackend;

impl GitBackend {
    /// Base directory for version control repos.
    fn base_dir() -> PathBuf {
        PathBuf::from("./data/vc")
    }

    /// Get or create a git repo for a tenant.
    fn get_or_init_repo(tenant_id: Uuid) -> Result<git2::Repository, String> {
        let repo_path = Self::base_dir().join(tenant_id.to_string());

        if repo_path.join(".git").exists() {
            git2::Repository::open(&repo_path).map_err(|e| format!("Git open failed: {e}"))
        } else {
            std::fs::create_dir_all(&repo_path)
                .map_err(|e| format!("Create dir failed: {e}"))?;
            let repo = git2::Repository::init(&repo_path)
                .map_err(|e| format!("Git init failed: {e}"))?;

            // Create initial commit on main branch.
            Self::create_initial_commit(&repo)?;
            info!(tenant = %tenant_id, path = %repo_path.display(), "Git repo initialized");
            Ok(repo)
        }
    }

    fn create_initial_commit(repo: &git2::Repository) -> Result<(), String> {
        let sig = git2::Signature::now("VíeLang VC", "vc@vielang.io")
            .map_err(|e| format!("Signature error: {e}"))?;
        let tree_id = repo
            .index()
            .and_then(|mut idx| idx.write_tree())
            .map_err(|e| format!("Tree error: {e}"))?;
        let tree = repo
            .find_tree(tree_id)
            .map_err(|e| format!("Find tree error: {e}"))?;
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .map_err(|e| format!("Initial commit error: {e}"))?;
        Ok(())
    }

    /// Commit entity snapshot data to the tenant's git repo.
    pub fn commit_to_git(
        tenant_id: Uuid,
        version_name: &str,
        entities_count: i64,
    ) -> Result<(), String> {
        let repo = Self::get_or_init_repo(tenant_id)?;
        let repo_path = Self::base_dir().join(tenant_id.to_string());

        // Write a version manifest file.
        let manifest_path = repo_path.join("version.json");
        let manifest = serde_json::json!({
            "version": version_name,
            "tenant_id": tenant_id,
            "entities": entities_count,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap_or_default())
            .map_err(|e| format!("Write manifest failed: {e}"))?;

        // Stage and commit.
        let mut index = repo.index().map_err(|e| format!("Index error: {e}"))?;
        index
            .add_path(Path::new("version.json"))
            .map_err(|e| format!("Add path error: {e}"))?;
        index.write().map_err(|e| format!("Index write error: {e}"))?;
        let tree_id = index
            .write_tree()
            .map_err(|e| format!("Write tree error: {e}"))?;
        let tree = repo
            .find_tree(tree_id)
            .map_err(|e| format!("Find tree error: {e}"))?;

        let sig = git2::Signature::now("VíeLang VC", "vc@vielang.io")
            .map_err(|e| format!("Signature error: {e}"))?;

        let parent = repo
            .head()
            .and_then(|h| h.peel_to_commit())
            .map_err(|e| format!("Parent commit error: {e}"))?;

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Version: {version_name} ({entities_count} entities)"),
            &tree,
            &[&parent],
        )
        .map_err(|e| format!("Commit error: {e}"))?;

        info!(tenant = %tenant_id, version = %version_name, "Git commit created");
        Ok(())
    }

    /// List git log entries for a tenant repo.
    pub fn list_versions(tenant_id: Uuid, limit: usize) -> Result<Vec<GitVersionEntry>, String> {
        let repo = Self::get_or_init_repo(tenant_id)?;
        let mut revwalk = repo.revwalk().map_err(|e| format!("Revwalk error: {e}"))?;
        revwalk.push_head().map_err(|e| format!("Push head error: {e}"))?;
        revwalk
            .set_sorting(git2::Sort::TIME)
            .map_err(|e| format!("Sort error: {e}"))?;

        let mut entries = Vec::new();
        for oid in revwalk.take(limit).flatten() {
            if let Ok(commit) = repo.find_commit(oid) {
                entries.push(GitVersionEntry {
                    commit_id: oid.to_string(),
                    message: commit.message().unwrap_or("").to_string(),
                    author: commit.author().name().unwrap_or("").to_string(),
                    timestamp: commit.time().seconds(),
                });
            }
        }
        Ok(entries)
    }
}

/// A git version log entry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GitVersionEntry {
    pub commit_id: String,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
}
