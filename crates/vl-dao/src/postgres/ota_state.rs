use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use crate::DaoError;

/// Trạng thái OTA update — khớp ThingsBoard OtaPackageUpdateStatus
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtaUpdateStatus {
    Queued,
    Initiated,
    Downloading,
    Downloaded,
    Verified,
    Updating,
    Updated,
    Failed,
}

impl OtaUpdateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OtaUpdateStatus::Queued      => "QUEUED",
            OtaUpdateStatus::Initiated   => "INITIATED",
            OtaUpdateStatus::Downloading => "DOWNLOADING",
            OtaUpdateStatus::Downloaded  => "DOWNLOADED",
            OtaUpdateStatus::Verified    => "VERIFIED",
            OtaUpdateStatus::Updating    => "UPDATING",
            OtaUpdateStatus::Updated     => "UPDATED",
            OtaUpdateStatus::Failed      => "FAILED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "QUEUED"      => OtaUpdateStatus::Queued,
            "INITIATED"   => OtaUpdateStatus::Initiated,
            "DOWNLOADING" => OtaUpdateStatus::Downloading,
            "DOWNLOADED"  => OtaUpdateStatus::Downloaded,
            "VERIFIED"    => OtaUpdateStatus::Verified,
            "UPDATING"    => OtaUpdateStatus::Updating,
            "UPDATED"     => OtaUpdateStatus::Updated,
            "FAILED"      => OtaUpdateStatus::Failed,
            _             => OtaUpdateStatus::Queued,
        }
    }
}

/// OTA device state — theo dõi tiến trình update của từng device
#[derive(Debug, Clone)]
pub struct OtaDeviceState {
    pub id:             Uuid,
    pub device_id:      Uuid,
    pub ota_package_id: Uuid,
    pub status:         OtaUpdateStatus,
    pub error:          Option<String>,
    pub created_time:   i64,
    pub updated_time:   i64,
}

pub struct OtaStateDao {
    pool: PgPool,
}

impl OtaStateDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert hoặc update OTA state cho (device_id, ota_package_id)
    #[instrument(skip(self))]
    pub async fn upsert(&self, state: &OtaDeviceState) -> Result<(), DaoError> {
        sqlx::query!(
            r#"
            INSERT INTO ota_device_state (id, device_id, ota_package_id, status, error, created_time, updated_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (device_id, ota_package_id) DO UPDATE SET
                status       = EXCLUDED.status,
                error        = EXCLUDED.error,
                updated_time = EXCLUDED.updated_time
            "#,
            state.id,
            state.device_id,
            state.ota_package_id,
            state.status.as_str(),
            state.error,
            state.created_time,
            state.updated_time,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Lấy OTA state mới nhất của device (theo updated_time DESC)
    #[instrument(skip(self))]
    pub async fn find_latest_by_device(
        &self,
        device_id: Uuid,
    ) -> Result<Option<OtaDeviceState>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, device_id, ota_package_id, status, error, created_time, updated_time
            FROM ota_device_state
            WHERE device_id = $1
            ORDER BY updated_time DESC
            LIMIT 1
            "#,
            device_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| OtaDeviceState {
            id:             r.id,
            device_id:      r.device_id,
            ota_package_id: r.ota_package_id,
            status:         OtaUpdateStatus::from_str(&r.status),
            error:          r.error,
            created_time:   r.created_time,
            updated_time:   r.updated_time,
        }))
    }

    /// Lấy OTA state cụ thể theo (device_id, package_id)
    #[instrument(skip(self))]
    pub async fn find_by_device_and_package(
        &self,
        device_id:  Uuid,
        package_id: Uuid,
    ) -> Result<Option<OtaDeviceState>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, device_id, ota_package_id, status, error, created_time, updated_time
            FROM ota_device_state
            WHERE device_id = $1 AND ota_package_id = $2
            "#,
            device_id,
            package_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| OtaDeviceState {
            id:             r.id,
            device_id:      r.device_id,
            ota_package_id: r.ota_package_id,
            status:         OtaUpdateStatus::from_str(&r.status),
            error:          r.error,
            created_time:   r.created_time,
            updated_time:   r.updated_time,
        }))
    }

    /// Lấy tất cả QUEUED states cho một package — dùng để retry
    #[instrument(skip(self))]
    pub async fn find_queued_by_package(
        &self,
        pkg_id: Uuid,
    ) -> Result<Vec<OtaDeviceState>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, device_id, ota_package_id, status, error, created_time, updated_time
            FROM ota_device_state
            WHERE ota_package_id = $1 AND status = 'QUEUED'
            "#,
            pkg_id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| OtaDeviceState {
            id:             r.id,
            device_id:      r.device_id,
            ota_package_id: r.ota_package_id,
            status:         OtaUpdateStatus::from_str(&r.status),
            error:          r.error,
            created_time:   r.created_time,
            updated_time:   r.updated_time,
        }).collect())
    }

    /// Tìm các QUEUED states quá hạn retry_delay giây (chưa download sau X phút)
    #[instrument(skip(self))]
    pub async fn find_stale_queued(
        &self,
        older_than_ms: i64,
    ) -> Result<Vec<OtaDeviceState>, DaoError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, device_id, ota_package_id, status, error, created_time, updated_time
            FROM ota_device_state
            WHERE status = 'QUEUED' AND updated_time < $1
            "#,
            older_than_ms,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| OtaDeviceState {
            id:             r.id,
            device_id:      r.device_id,
            ota_package_id: r.ota_package_id,
            status:         OtaUpdateStatus::from_str(&r.status),
            error:          r.error,
            created_time:   r.created_time,
            updated_time:   r.updated_time,
        }).collect())
    }
}
