//! PostGIS-backed geofence DAO.
//!
//! Uses `sqlx::query_as` (non-macro) for PostGIS geometry queries, because
//! compile-time type inference for GEOGRAPHY columns requires the postgis
//! extension to be installed on the compile-time DB.  We represent geometry
//! columns as `String` (WKT text via ST_AsText) on reads and construct them
//! from lat/lng via `ST_MakePoint` on writes.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::DaoError;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Geofence {
    pub id:           Uuid,
    pub tenant_id:    Uuid,
    pub name:         String,
    /// "POLYGON" or "CIRCLE"
    pub r#type:       String,
    /// WKT polygon text (only for POLYGON type)
    pub polygon_wkt:  Option<String>,
    /// Center latitude (only for CIRCLE type)
    pub center_lat:   Option<f64>,
    /// Center longitude (only for CIRCLE type)
    pub center_lng:   Option<f64>,
    /// Radius in metres (only for CIRCLE type)
    pub radius_m:     Option<f64>,
    pub created_time: i64,
    pub extra_info:   Option<serde_json::Value>,
}

/// Upsert/create request
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeofenceUpsert {
    pub name:        String,
    pub r#type:      String,
    /// WKT polygon string for POLYGON type, e.g. "POLYGON((lng lat, ...))"
    pub polygon_wkt: Option<String>,
    pub center_lat:  Option<f64>,
    pub center_lng:  Option<f64>,
    pub radius_m:    Option<f64>,
    pub extra_info:  Option<serde_json::Value>,
}

/// Internal sqlx row for ST_AsText reads
#[derive(sqlx::FromRow)]
struct GeofenceRow {
    id:           Uuid,
    tenant_id:    Uuid,
    name:         String,
    geofence_type: String,
    polygon_wkt:  Option<String>,
    center_lat:   Option<f64>,
    center_lng:   Option<f64>,
    radius_m:     Option<f64>,
    created_time: i64,
    extra_info:   Option<serde_json::Value>,
}

impl From<GeofenceRow> for Geofence {
    fn from(r: GeofenceRow) -> Self {
        Geofence {
            id:           r.id,
            tenant_id:    r.tenant_id,
            name:         r.name,
            r#type:       r.geofence_type,
            polygon_wkt:  r.polygon_wkt,
            center_lat:   r.center_lat,
            center_lng:   r.center_lng,
            radius_m:     r.radius_m,
            created_time: r.created_time,
            extra_info:   r.extra_info,
        }
    }
}

// ── DAO ───────────────────────────────────────────────────────────────────────

pub struct GeofenceDao {
    pool: PgPool,
}

impl GeofenceDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert or full-replace a geofence.  Returns the saved record.
    #[instrument(skip(self, upsert))]
    pub async fn upsert(
        &self,
        tenant_id: Uuid,
        id: Option<Uuid>,
        upsert: GeofenceUpsert,
    ) -> Result<Geofence, DaoError> {
        let id = id.unwrap_or_else(Uuid::new_v4);
        let now = chrono::Utc::now().timestamp_millis();

        // Build polygon GEOGRAPHY from WKT string
        // Build center GEOGRAPHY from lat/lng pair
        sqlx::query(
            r#"
            INSERT INTO tb_geofence
                (id, tenant_id, name, type, polygon, center, radius_m, created_time, extra_info)
            VALUES (
                $1, $2, $3, $4,
                CASE WHEN $5::text IS NOT NULL THEN ST_GeogFromText($5) ELSE NULL END,
                CASE WHEN $6::float8 IS NOT NULL AND $7::float8 IS NOT NULL
                     THEN ST_SetSRID(ST_MakePoint($7, $6), 4326)::geography
                     ELSE NULL END,
                $8, $9, $10
            )
            ON CONFLICT (id) DO UPDATE SET
                name         = EXCLUDED.name,
                type         = EXCLUDED.type,
                polygon      = EXCLUDED.polygon,
                center       = EXCLUDED.center,
                radius_m     = EXCLUDED.radius_m,
                extra_info   = EXCLUDED.extra_info
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(&upsert.name)
        .bind(&upsert.r#type)
        .bind(upsert.polygon_wkt.as_deref())
        .bind(upsert.center_lat)
        .bind(upsert.center_lng)
        .bind(upsert.radius_m)
        .bind(now)
        .bind(&upsert.extra_info)
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(id)
            .await?
            .ok_or(DaoError::NotFound)
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Geofence>, DaoError> {
        let row = sqlx::query_as::<_, GeofenceRow>(
            r#"
            SELECT
                id, tenant_id, name,
                type AS geofence_type,
                ST_AsText(polygon)  AS polygon_wkt,
                ST_Y(center::geometry) AS center_lat,
                ST_X(center::geometry) AS center_lng,
                radius_m, created_time, extra_info
            FROM tb_geofence
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(row.map(Geofence::from))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Geofence>, DaoError> {
        let rows = sqlx::query_as::<_, GeofenceRow>(
            r#"
            SELECT
                id, tenant_id, name,
                type AS geofence_type,
                ST_AsText(polygon)  AS polygon_wkt,
                ST_Y(center::geometry) AS center_lat,
                ST_X(center::geometry) AS center_lng,
                radius_m, created_time, extra_info
            FROM tb_geofence
            WHERE tenant_id = $1
            ORDER BY created_time DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(rows.into_iter().map(Geofence::from).collect())
    }

    /// Count all geofences for a tenant (for pagination).
    #[instrument(skip(self))]
    pub async fn count_by_tenant(&self, tenant_id: Uuid) -> Result<i64, DaoError> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM tb_geofence WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(row.0)
    }

    /// Returns all geofences whose polygon/circle contains the given point.
    /// For POLYGON type: uses ST_Contains.
    /// For CIRCLE type:  uses ST_DWithin (geodesic distance).
    #[instrument(skip(self))]
    pub async fn find_containing(
        &self,
        tenant_id: Uuid,
        lat: f64,
        lng: f64,
    ) -> Result<Vec<Geofence>, DaoError> {
        let rows = sqlx::query_as::<_, GeofenceRow>(
            r#"
            SELECT
                id, tenant_id, name,
                type AS geofence_type,
                ST_AsText(polygon)     AS polygon_wkt,
                ST_Y(center::geometry) AS center_lat,
                ST_X(center::geometry) AS center_lng,
                radius_m, created_time, extra_info
            FROM tb_geofence
            WHERE tenant_id = $1
              AND (
                  (type = 'POLYGON' AND polygon IS NOT NULL
                   AND ST_Contains(polygon::geometry,
                       ST_SetSRID(ST_MakePoint($3, $2), 4326)))
                  OR
                  (type = 'CIRCLE' AND center IS NOT NULL AND radius_m IS NOT NULL
                   AND ST_DWithin(
                       center,
                       ST_SetSRID(ST_MakePoint($3, $2), 4326)::geography,
                       radius_m))
              )
            "#,
        )
        .bind(tenant_id)
        .bind(lat)
        .bind(lng)
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(rows.into_iter().map(Geofence::from).collect())
    }

    /// Find containing geofences filtered to specific IDs (for rule engine node).
    #[instrument(skip(self, geofence_ids))]
    pub async fn find_containing_by_ids(
        &self,
        geofence_ids: &[Uuid],
        lat: f64,
        lng: f64,
    ) -> Result<Vec<Geofence>, DaoError> {
        if geofence_ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query_as::<_, GeofenceRow>(
            r#"
            SELECT
                id, tenant_id, name,
                type AS geofence_type,
                ST_AsText(polygon)     AS polygon_wkt,
                ST_Y(center::geometry) AS center_lat,
                ST_X(center::geometry) AS center_lng,
                radius_m, created_time, extra_info
            FROM tb_geofence
            WHERE id = ANY($1)
              AND (
                  (type = 'POLYGON' AND polygon IS NOT NULL
                   AND ST_Contains(polygon::geometry,
                       ST_SetSRID(ST_MakePoint($3, $2), 4326)))
                  OR
                  (type = 'CIRCLE' AND center IS NOT NULL AND radius_m IS NOT NULL
                   AND ST_DWithin(
                       center,
                       ST_SetSRID(ST_MakePoint($3, $2), 4326)::geography,
                       radius_m))
              )
            "#,
        )
        .bind(geofence_ids)
        .bind(lat)
        .bind(lng)
        .fetch_all(&self.pool)
        .await
        .map_err(DaoError::Database)?;

        Ok(rows.into_iter().map(Geofence::from).collect())
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid, tenant_id: Uuid) -> Result<bool, DaoError> {
        let res = sqlx::query(
            "DELETE FROM tb_geofence WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await
        .map_err(DaoError::Database)?;
        Ok(res.rows_affected() > 0)
    }
}
