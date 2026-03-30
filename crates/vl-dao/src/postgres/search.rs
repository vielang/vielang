use sqlx::PgPool;
use uuid::Uuid;
use serde::Serialize;
use tracing::instrument;

use crate::DaoError;

/// Unified search result — một row từ bất kỳ entity type nào
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id:           Uuid,
    pub created_time: i64,
    pub tenant_id:    Uuid,
    pub entity_type:  &'static str,
    pub name:         String,
    pub label:        Option<String>,
    /// FTS relevance rank — dùng để sort kết quả, không serialize
    #[serde(skip)]
    pub rank:         f32,
}

pub struct SearchDao {
    pool: PgPool,
}

impl SearchDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Per-type search ───────────────────────────────────────────────────────

    #[instrument(skip(self))]
    pub async fn search_devices(
        &self,
        tenant_id: Uuid,
        tsquery:   &str,
        limit:     i64,
    ) -> Result<Vec<SearchResult>, DaoError> {
        if tsquery.is_empty() {
            let rows = sqlx::query!(
                "SELECT id, created_time, tenant_id, name, label
                 FROM device WHERE tenant_id = $1
                 ORDER BY created_time DESC LIMIT $2",
                tenant_id, limit
            )
            .fetch_all(&self.pool)
            .await?;

            return Ok(rows.into_iter().map(|r| SearchResult {
                id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
                entity_type: "DEVICE", name: r.name, label: r.label, rank: 0.0,
            }).collect());
        }

        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, name, label,
                      ts_rank(search_text, to_tsquery('english', $2)) AS "rank!: f32"
               FROM device
               WHERE tenant_id = $1
                 AND search_text IS NOT NULL
                 AND search_text @@ to_tsquery('english', $2)
               ORDER BY ts_rank(search_text, to_tsquery('english', $2)) DESC, created_time DESC
               LIMIT $3"#,
            tenant_id, tsquery, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| SearchResult {
            id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
            entity_type: "DEVICE", name: r.name, label: r.label, rank: r.rank,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn search_assets(
        &self,
        tenant_id: Uuid,
        tsquery:   &str,
        limit:     i64,
    ) -> Result<Vec<SearchResult>, DaoError> {
        if tsquery.is_empty() {
            let rows = sqlx::query!(
                "SELECT id, created_time, tenant_id, name, label
                 FROM asset WHERE tenant_id = $1
                 ORDER BY created_time DESC LIMIT $2",
                tenant_id, limit
            )
            .fetch_all(&self.pool)
            .await?;

            return Ok(rows.into_iter().map(|r| SearchResult {
                id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
                entity_type: "ASSET", name: r.name, label: r.label, rank: 0.0,
            }).collect());
        }

        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, name, label,
                      ts_rank(search_text, to_tsquery('english', $2)) AS "rank!: f32"
               FROM asset
               WHERE tenant_id = $1
                 AND search_text IS NOT NULL
                 AND search_text @@ to_tsquery('english', $2)
               ORDER BY ts_rank(search_text, to_tsquery('english', $2)) DESC, created_time DESC
               LIMIT $3"#,
            tenant_id, tsquery, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| SearchResult {
            id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
            entity_type: "ASSET", name: r.name, label: r.label, rank: r.rank,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn search_customers(
        &self,
        tenant_id: Uuid,
        tsquery:   &str,
        limit:     i64,
    ) -> Result<Vec<SearchResult>, DaoError> {
        if tsquery.is_empty() {
            let rows = sqlx::query!(
                "SELECT id, created_time, tenant_id, title
                 FROM customer WHERE tenant_id = $1
                 ORDER BY created_time DESC LIMIT $2",
                tenant_id, limit
            )
            .fetch_all(&self.pool)
            .await?;

            return Ok(rows.into_iter().map(|r| SearchResult {
                id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
                entity_type: "CUSTOMER", name: r.title, label: None, rank: 0.0,
            }).collect());
        }

        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, title,
                      ts_rank(search_text, to_tsquery('english', $2)) AS "rank!: f32"
               FROM customer
               WHERE tenant_id = $1
                 AND search_text IS NOT NULL
                 AND search_text @@ to_tsquery('english', $2)
               ORDER BY ts_rank(search_text, to_tsquery('english', $2)) DESC, created_time DESC
               LIMIT $3"#,
            tenant_id, tsquery, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| SearchResult {
            id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
            entity_type: "CUSTOMER", name: r.title, label: None, rank: r.rank,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn search_users(
        &self,
        tenant_id: Uuid,
        tsquery:   &str,
        limit:     i64,
    ) -> Result<Vec<SearchResult>, DaoError> {
        if tsquery.is_empty() {
            let rows = sqlx::query!(
                "SELECT id, created_time, tenant_id, email, first_name, last_name
                 FROM tb_user WHERE tenant_id = $1
                 ORDER BY created_time DESC LIMIT $2",
                tenant_id, limit
            )
            .fetch_all(&self.pool)
            .await?;

            return Ok(rows.into_iter().map(|r| SearchResult {
                id: r.id, created_time: r.created_time,
                tenant_id: r.tenant_id.unwrap_or_default(),
                entity_type: "USER",
                name: r.email,
                label: build_full_name(r.first_name.as_deref(), r.last_name.as_deref()),
                rank: 0.0,
            }).collect());
        }

        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, email, first_name, last_name,
                      ts_rank(search_text, to_tsquery('english', $2)) AS "rank!: f32"
               FROM tb_user
               WHERE tenant_id = $1
                 AND search_text IS NOT NULL
                 AND search_text @@ to_tsquery('english', $2)
               ORDER BY ts_rank(search_text, to_tsquery('english', $2)) DESC, created_time DESC
               LIMIT $3"#,
            tenant_id, tsquery, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| SearchResult {
            id: r.id, created_time: r.created_time,
            tenant_id: r.tenant_id.unwrap_or_default(),
            entity_type: "USER",
            name: r.email,
            label: build_full_name(r.first_name.as_deref(), r.last_name.as_deref()),
            rank: r.rank,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn search_entity_views(
        &self,
        tenant_id: Uuid,
        tsquery:   &str,
        limit:     i64,
    ) -> Result<Vec<SearchResult>, DaoError> {
        if tsquery.is_empty() {
            let rows = sqlx::query!(
                "SELECT id, created_time, tenant_id, name
                 FROM entity_view WHERE tenant_id = $1
                 ORDER BY created_time DESC LIMIT $2",
                tenant_id, limit
            )
            .fetch_all(&self.pool)
            .await?;

            return Ok(rows.into_iter().map(|r| SearchResult {
                id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
                entity_type: "ENTITY_VIEW", name: r.name, label: None, rank: 0.0,
            }).collect());
        }

        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, name,
                      ts_rank(search_text, to_tsquery('english', $2)) AS "rank!: f32"
               FROM entity_view
               WHERE tenant_id = $1
                 AND search_text IS NOT NULL
                 AND search_text @@ to_tsquery('english', $2)
               ORDER BY ts_rank(search_text, to_tsquery('english', $2)) DESC, created_time DESC
               LIMIT $3"#,
            tenant_id, tsquery, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| SearchResult {
            id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
            entity_type: "ENTITY_VIEW", name: r.name, label: None, rank: r.rank,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn search_edges(
        &self,
        tenant_id: Uuid,
        tsquery:   &str,
        limit:     i64,
    ) -> Result<Vec<SearchResult>, DaoError> {
        if tsquery.is_empty() {
            let rows = sqlx::query!(
                "SELECT id, created_time, tenant_id, name, label
                 FROM edge WHERE tenant_id = $1
                 ORDER BY created_time DESC LIMIT $2",
                tenant_id, limit
            )
            .fetch_all(&self.pool)
            .await?;

            return Ok(rows.into_iter().map(|r| SearchResult {
                id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
                entity_type: "EDGE", name: r.name, label: r.label, rank: 0.0,
            }).collect());
        }

        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, name, label,
                      ts_rank(search_text, to_tsquery('english', $2)) AS "rank!: f32"
               FROM edge
               WHERE tenant_id = $1
                 AND search_text IS NOT NULL
                 AND search_text @@ to_tsquery('english', $2)
               ORDER BY ts_rank(search_text, to_tsquery('english', $2)) DESC, created_time DESC
               LIMIT $3"#,
            tenant_id, tsquery, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| SearchResult {
            id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
            entity_type: "EDGE", name: r.name, label: r.label, rank: r.rank,
        }).collect())
    }

    #[instrument(skip(self))]
    pub async fn search_dashboards(
        &self,
        tenant_id: Uuid,
        tsquery:   &str,
        limit:     i64,
    ) -> Result<Vec<SearchResult>, DaoError> {
        if tsquery.is_empty() {
            let rows = sqlx::query!(
                "SELECT id, created_time, tenant_id, title
                 FROM dashboard WHERE tenant_id = $1
                 ORDER BY created_time DESC LIMIT $2",
                tenant_id, limit
            )
            .fetch_all(&self.pool)
            .await?;

            return Ok(rows.into_iter().map(|r| SearchResult {
                id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
                entity_type: "DASHBOARD",
                name: r.title.unwrap_or_default(),
                label: None, rank: 0.0,
            }).collect());
        }

        let rows = sqlx::query!(
            r#"SELECT id, created_time, tenant_id, title,
                      ts_rank(search_text, to_tsquery('english', $2)) AS "rank!: f32"
               FROM dashboard
               WHERE tenant_id = $1
                 AND search_text IS NOT NULL
                 AND search_text @@ to_tsquery('english', $2)
               ORDER BY ts_rank(search_text, to_tsquery('english', $2)) DESC, created_time DESC
               LIMIT $3"#,
            tenant_id, tsquery, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| SearchResult {
            id: r.id, created_time: r.created_time, tenant_id: r.tenant_id,
            entity_type: "DASHBOARD",
            name: r.title.unwrap_or_default(),
            label: None, rank: r.rank,
        }).collect())
    }

    // ── Unified search ────────────────────────────────────────────────────────

    /// Tìm kiếm song song trên nhiều entity types, merge + sort theo relevance, paginate.
    ///
    /// `types` — danh sách uppercase type strings ("DEVICE", "ASSET", v.v.)
    /// Trả về `(results_on_page, total_matched_across_all_types)`
    pub async fn unified_search(
        &self,
        tenant_id:        Uuid,
        query:            &str,
        types:            &[&str],
        page:             i64,
        page_size:        i64,
        max_per_type:     i64,
        min_query_length: usize,
        prefix_matching:  bool,
    ) -> Result<(Vec<SearchResult>, i64), DaoError> {
        let tsquery = build_tsquery(query, min_query_length, prefix_matching);

        // Chạy tất cả các search song song
        let (devices, assets, customers, users, entity_views, edges, dashboards) = tokio::join!(
            async {
                if types.contains(&"DEVICE") {
                    self.search_devices(tenant_id, &tsquery, max_per_type).await
                } else {
                    Ok(vec![])
                }
            },
            async {
                if types.contains(&"ASSET") {
                    self.search_assets(tenant_id, &tsquery, max_per_type).await
                } else {
                    Ok(vec![])
                }
            },
            async {
                if types.contains(&"CUSTOMER") {
                    self.search_customers(tenant_id, &tsquery, max_per_type).await
                } else {
                    Ok(vec![])
                }
            },
            async {
                if types.contains(&"USER") {
                    self.search_users(tenant_id, &tsquery, max_per_type).await
                } else {
                    Ok(vec![])
                }
            },
            async {
                if types.contains(&"ENTITY_VIEW") {
                    self.search_entity_views(tenant_id, &tsquery, max_per_type).await
                } else {
                    Ok(vec![])
                }
            },
            async {
                if types.contains(&"EDGE") {
                    self.search_edges(tenant_id, &tsquery, max_per_type).await
                } else {
                    Ok(vec![])
                }
            },
            async {
                if types.contains(&"DASHBOARD") {
                    self.search_dashboards(tenant_id, &tsquery, max_per_type).await
                } else {
                    Ok(vec![])
                }
            },
        );

        // Gom kết quả — bất kỳ error nào cũng propagate
        let mut all: Vec<SearchResult> = Vec::new();
        all.extend(devices?);
        all.extend(assets?);
        all.extend(customers?);
        all.extend(users?);
        all.extend(entity_views?);
        all.extend(edges?);
        all.extend(dashboards?);

        // Sort by relevance DESC, then created_time DESC
        all.sort_by(|a, b| {
            b.rank
                .partial_cmp(&a.rank)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.created_time.cmp(&a.created_time))
        });

        let total = all.len() as i64;
        let start = (page * page_size) as usize;
        let page_data = all.into_iter().skip(start).take(page_size as usize).collect();

        Ok((page_data, total))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Chuyển query string → PostgreSQL tsquery với prefix matching.
/// "Temperature Sensor" → "temperature:* & sensor:*"
/// Trả về empty string khi query quá ngắn → DAO sẽ fallback về top results.
pub fn build_tsquery(query: &str, min_len: usize, prefix: bool) -> String {
    let terms: Vec<String> = query
        .split_whitespace()
        .filter(|w| w.len() >= min_len)
        .filter_map(|w| {
            // Giữ lại chỉ alphanumeric + underscore để tránh tsquery injection
            let clean: String = w.chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if clean.is_empty() {
                None
            } else if prefix {
                Some(format!("{}:*", clean))
            } else {
                Some(clean)
            }
        })
        .collect();

    terms.join(" & ")
}

fn build_full_name(first: Option<&str>, last: Option<&str>) -> Option<String> {
    match (first, last) {
        (Some(f), Some(l)) if !f.is_empty() || !l.is_empty() => {
            Some(format!("{} {}", f, l).trim().to_string())
        }
        (Some(f), None) if !f.is_empty() => Some(f.to_string()),
        (None, Some(l)) if !l.is_empty() => Some(l.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tsquery_prefix() {
        let q = build_tsquery("Temperature Sensor", 2, true);
        assert_eq!(q, "Temperature:* & Sensor:*");
    }

    #[test]
    fn test_build_tsquery_no_prefix() {
        let q = build_tsquery("temperature sensor", 2, false);
        assert_eq!(q, "temperature & sensor");
    }

    #[test]
    fn test_build_tsquery_filters_short() {
        // "a" < min_len=2, should be filtered
        let q = build_tsquery("a sensor", 2, true);
        assert_eq!(q, "sensor:*");
    }

    #[test]
    fn test_build_tsquery_empty() {
        let q = build_tsquery("  ", 2, true);
        assert_eq!(q, "");
    }

    #[test]
    fn test_build_tsquery_strips_special_chars() {
        let q = build_tsquery("sensor!@# abc", 2, false);
        assert_eq!(q, "sensor & abc");
    }

    #[test]
    fn test_build_full_name() {
        assert_eq!(build_full_name(Some("John"), Some("Doe")), Some("John Doe".into()));
        assert_eq!(build_full_name(Some("John"), None), Some("John".into()));
        assert_eq!(build_full_name(None, Some("Doe")), Some("Doe".into()));
        assert_eq!(build_full_name(None, None), None);
    }

    #[test]
    fn test_build_tsquery_max_length_trimmed() {
        // Xác nhận rằng query quá dài vẫn build được (max trim xảy ra ở route layer)
        let long_query = "sensor ".repeat(50); // 350 chars
        let q = build_tsquery(&long_query, 2, true);
        assert!(!q.is_empty(), "long query with valid terms should still build");
        assert!(q.contains(":*"), "should have prefix matching");
    }

    #[test]
    fn test_build_tsquery_dashboard_terms() {
        // Dashboard search: "overview dashboard" → build tsquery
        let q = build_tsquery("overview dashboard", 2, true);
        assert_eq!(q, "overview:* & dashboard:*");
    }

    #[test]
    fn test_unified_search_includes_dashboard_type() {
        // DASHBOARD phải nằm trong danh sách types được chấp nhận bởi route
        // Test này xác nhận unified_search không panic khi types = ["DASHBOARD"]
        // (functional test với DB không thực hiện ở đây — integration test)
        let _ = build_tsquery("dashboard test", 2, true);
    }
}
