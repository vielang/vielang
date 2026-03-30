use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::DeviceActivity;
use crate::DaoError;

pub struct DeviceActivityDao {
    pool: PgPool,
}

impl DeviceActivityDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Full upsert — ghi toàn bộ record
    #[instrument(skip(self))]
    pub async fn save(&self, a: &DeviceActivity) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO device_activity
                (device_id, last_connect_ts, last_disconnect_ts, last_activity_ts,
                 last_telemetry_ts, last_rpc_ts, active)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (device_id) DO UPDATE
            SET last_connect_ts    = EXCLUDED.last_connect_ts,
                last_disconnect_ts = EXCLUDED.last_disconnect_ts,
                last_activity_ts   = EXCLUDED.last_activity_ts,
                last_telemetry_ts  = EXCLUDED.last_telemetry_ts,
                last_rpc_ts        = EXCLUDED.last_rpc_ts,
                active             = EXCLUDED.active
            "#,
            a.device_id,
            a.last_connect_ts,
            a.last_disconnect_ts,
            a.last_activity_ts,
            a.last_telemetry_ts,
            a.last_rpc_ts,
            a.active,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Upsert khi device kết nối — set active=true
    #[instrument(skip(self))]
    pub async fn update_connect(&self, device_id: Uuid, ts: i64) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO device_activity (device_id, last_connect_ts, last_activity_ts, active)
            VALUES ($1, $2, $2, TRUE)
            ON CONFLICT (device_id) DO UPDATE
            SET last_connect_ts  = EXCLUDED.last_connect_ts,
                last_activity_ts = GREATEST(device_activity.last_activity_ts, EXCLUDED.last_activity_ts),
                active           = TRUE
            "#,
            device_id,
            ts,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Upsert khi device ngắt kết nối — set active=false
    #[instrument(skip(self))]
    pub async fn update_disconnect(&self, device_id: Uuid, ts: i64) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO device_activity (device_id, last_disconnect_ts, active)
            VALUES ($1, $2, FALSE)
            ON CONFLICT (device_id) DO UPDATE
            SET last_disconnect_ts = EXCLUDED.last_disconnect_ts,
                active             = FALSE
            "#,
            device_id,
            ts,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Upsert khi nhận telemetry — update last_telemetry_ts + last_activity_ts
    #[instrument(skip(self))]
    pub async fn update_telemetry(&self, device_id: Uuid, ts: i64) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO device_activity (device_id, last_telemetry_ts, last_activity_ts, active)
            VALUES ($1, $2, $2, TRUE)
            ON CONFLICT (device_id) DO UPDATE
            SET last_telemetry_ts = EXCLUDED.last_telemetry_ts,
                last_activity_ts  = GREATEST(device_activity.last_activity_ts, EXCLUDED.last_activity_ts),
                active            = TRUE
            "#,
            device_id,
            ts,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Upsert khi nhận RPC — update last_rpc_ts + last_activity_ts
    #[instrument(skip(self))]
    pub async fn update_rpc(&self, device_id: Uuid, ts: i64) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO device_activity (device_id, last_rpc_ts, last_activity_ts, active)
            VALUES ($1, $2, $2, TRUE)
            ON CONFLICT (device_id) DO UPDATE
            SET last_rpc_ts      = EXCLUDED.last_rpc_ts,
                last_activity_ts = GREATEST(device_activity.last_activity_ts, EXCLUDED.last_activity_ts),
                active           = TRUE
            "#,
            device_id,
            ts,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Trực tiếp set trạng thái active
    #[instrument(skip(self))]
    pub async fn set_active(&self, device_id: Uuid, active: bool) -> Result<(), DaoError> {
        sqlx::query!(
            "UPDATE device_activity SET active = $2 WHERE device_id = $1",
            device_id,
            active,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Lấy activity record của một device
    #[instrument(skip(self))]
    pub async fn find(&self, device_id: Uuid) -> Result<Option<DeviceActivity>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT device_id, last_connect_ts, last_disconnect_ts, last_activity_ts,
                   last_telemetry_ts, last_rpc_ts, active
            FROM device_activity
            WHERE device_id = $1
            "#,
            device_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| DeviceActivity {
            device_id:          r.device_id,
            last_connect_ts:    r.last_connect_ts,
            last_disconnect_ts: r.last_disconnect_ts,
            last_activity_ts:   r.last_activity_ts,
            last_telemetry_ts:  r.last_telemetry_ts,
            last_rpc_ts:        r.last_rpc_ts,
            active:             r.active,
        }))
    }

    /// Tất cả devices đang active của một tenant (JOIN với device table)
    #[instrument(skip(self))]
    pub async fn find_active(&self, tenant_id: Uuid) -> Result<Vec<DeviceActivity>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT da.device_id, da.last_connect_ts, da.last_disconnect_ts, da.last_activity_ts,
                   da.last_telemetry_ts, da.last_rpc_ts, da.active
            FROM device_activity da
            JOIN device d ON d.id = da.device_id
            WHERE da.active = TRUE AND d.tenant_id = $1
            "#,
            tenant_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| DeviceActivity {
            device_id:          r.device_id,
            last_connect_ts:    r.last_connect_ts,
            last_disconnect_ts: r.last_disconnect_ts,
            last_activity_ts:   r.last_activity_ts,
            last_telemetry_ts:  r.last_telemetry_ts,
            last_rpc_ts:        r.last_rpc_ts,
            active:             r.active,
        }).collect())
    }

    /// Lấy device_id của các devices active nhưng không có activity trong threshold_ts ms
    /// Dùng cho Housekeeper (Phase 32) để mark inactive
    #[instrument(skip(self))]
    pub async fn find_inactive_since(&self, threshold_ts: i64) -> Result<Vec<Uuid>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT device_id
            FROM device_activity
            WHERE active = TRUE AND last_activity_ts < $1
            "#,
            threshold_ts,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.device_id).collect())
    }
}
