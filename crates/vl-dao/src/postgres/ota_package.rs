use sqlx::PgPool;
use uuid::Uuid;
use tracing::instrument;

use vl_core::entities::{OtaPackage, OtaPackageInfo, OtaPackageType, ChecksumAlgorithm};
use crate::{DaoError, PageData, PageLink};

pub struct OtaPackageDao {
    pool: PgPool,
}

impl OtaPackageDao {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OtaPackage>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, device_profile_id,
                   type as ota_type, title, version, tag, url,
                   file_name, content_type, data_size,
                   checksum_algorithm, checksum,
                   data IS NOT NULL as has_data,
                   additional_info, ver
            FROM ota_package WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| OtaPackage {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_profile_id: r.device_profile_id,
            ota_package_type: OtaPackageType::from_str(&r.ota_type),
            title: r.title,
            version: r.version,
            tag: r.tag,
            url: r.url,
            file_name: r.file_name,
            content_type: r.content_type,
            data_size: r.data_size,
            checksum_algorithm: r.checksum_algorithm.and_then(|s| ChecksumAlgorithm::from_str(&s)),
            checksum: r.checksum,
            has_data: r.has_data.unwrap_or(false),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version_int: r.ver,
        }))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant(
        &self,
        tenant_id: Uuid,
        page_link: &PageLink,
    ) -> Result<PageData<OtaPackageInfo>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM ota_package
               WHERE tenant_id = $1
               AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))"#,
            tenant_id,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, device_profile_id,
                   type as ota_type, title, version, tag, url,
                   file_name, content_type, data_size,
                   checksum_algorithm, checksum,
                   data IS NOT NULL as has_data
            FROM ota_package
            WHERE tenant_id = $1
            AND ($2::text IS NULL OR LOWER(title) LIKE LOWER($2))
            ORDER BY created_time DESC
            LIMIT $3 OFFSET $4
            "#,
            tenant_id,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| OtaPackageInfo {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_profile_id: r.device_profile_id,
            ota_package_type: OtaPackageType::from_str(&r.ota_type),
            title: r.title,
            version: r.version,
            tag: r.tag,
            url: r.url,
            has_data: r.has_data.unwrap_or(false),
            file_name: r.file_name,
            content_type: r.content_type,
            data_size: r.data_size,
            checksum_algorithm: r.checksum_algorithm.and_then(|s| ChecksumAlgorithm::from_str(&s)),
            checksum: r.checksum,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    #[instrument(skip(self))]
    pub async fn find_by_tenant_and_type(
        &self,
        tenant_id: Uuid,
        ota_type: OtaPackageType,
        page_link: &PageLink,
    ) -> Result<PageData<OtaPackageInfo>, DaoError> {
        let text_search = page_link.text_search.as_deref().map(|s| format!("%{}%", s));
        let ota_type_str = ota_type.as_str();

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM ota_package
               WHERE tenant_id = $1 AND type = $2
               AND ($3::text IS NULL OR LOWER(title) LIKE LOWER($3))"#,
            tenant_id,
            ota_type_str,
            text_search,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, device_profile_id,
                   type as ota_type, title, version, tag, url,
                   file_name, content_type, data_size,
                   checksum_algorithm, checksum,
                   data IS NOT NULL as has_data
            FROM ota_package
            WHERE tenant_id = $1 AND type = $2
            AND ($3::text IS NULL OR LOWER(title) LIKE LOWER($3))
            ORDER BY created_time DESC
            LIMIT $4 OFFSET $5
            "#,
            tenant_id,
            ota_type_str,
            text_search,
            page_link.page_size,
            page_link.offset(),
        )
        .fetch_all(&self.pool)
        .await?;

        let data = rows.into_iter().map(|r| OtaPackageInfo {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_profile_id: r.device_profile_id,
            ota_package_type: OtaPackageType::from_str(&r.ota_type),
            title: r.title,
            version: r.version,
            tag: r.tag,
            url: r.url,
            has_data: r.has_data.unwrap_or(false),
            file_name: r.file_name,
            content_type: r.content_type,
            data_size: r.data_size,
            checksum_algorithm: r.checksum_algorithm.and_then(|s| ChecksumAlgorithm::from_str(&s)),
            checksum: r.checksum,
        }).collect();

        Ok(PageData::new(data, total, page_link))
    }

    /// Find OTA packages by device profile
    #[instrument(skip(self))]
    pub async fn find_by_device_profile(
        &self,
        device_profile_id: Uuid,
        ota_type: OtaPackageType,
    ) -> Result<Option<OtaPackageInfo>, DaoError> {
        let ota_type_str = ota_type.as_str();

        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, device_profile_id,
                   type as ota_type, title, version, tag, url,
                   file_name, content_type, data_size,
                   checksum_algorithm, checksum,
                   data IS NOT NULL as has_data
            FROM ota_package
            WHERE device_profile_id = $1 AND type = $2
            ORDER BY created_time DESC
            LIMIT 1
            "#,
            device_profile_id,
            ota_type_str,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| OtaPackageInfo {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_profile_id: r.device_profile_id,
            ota_package_type: OtaPackageType::from_str(&r.ota_type),
            title: r.title,
            version: r.version,
            tag: r.tag,
            url: r.url,
            has_data: r.has_data.unwrap_or(false),
            file_name: r.file_name,
            content_type: r.content_type,
            data_size: r.data_size,
            checksum_algorithm: r.checksum_algorithm.and_then(|s| ChecksumAlgorithm::from_str(&s)),
            checksum: r.checksum,
        }))
    }

    #[instrument(skip(self))]
    pub async fn save(&self, pkg: &OtaPackage) -> Result<OtaPackage, DaoError> {
        let additional_info = pkg.additional_info.as_ref()
            .map(|v| v.to_string());
        let checksum_alg = pkg.checksum_algorithm.map(|a| a.as_str().to_string());

        sqlx::query!(
            r#"
            INSERT INTO ota_package (
                id, created_time, tenant_id, device_profile_id,
                type, title, version, tag, url,
                file_name, content_type, checksum_algorithm, checksum,
                data_size, additional_info, ver
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            ON CONFLICT (id) DO UPDATE SET
                device_profile_id = EXCLUDED.device_profile_id,
                type              = EXCLUDED.type,
                title             = EXCLUDED.title,
                version           = EXCLUDED.version,
                tag               = EXCLUDED.tag,
                url               = EXCLUDED.url,
                file_name         = EXCLUDED.file_name,
                content_type      = EXCLUDED.content_type,
                checksum_algorithm = EXCLUDED.checksum_algorithm,
                checksum          = EXCLUDED.checksum,
                data_size         = EXCLUDED.data_size,
                additional_info   = EXCLUDED.additional_info,
                ver               = ota_package.ver + 1
            "#,
            pkg.id,
            pkg.created_time,
            pkg.tenant_id,
            pkg.device_profile_id,
            pkg.ota_package_type.as_str(),
            pkg.title,
            pkg.version,
            pkg.tag,
            pkg.url,
            pkg.file_name,
            pkg.content_type,
            checksum_alg,
            pkg.checksum,
            pkg.data_size,
            additional_info,
            pkg.version_int,
        )
        .execute(&self.pool)
        .await
        .map_err(DaoError::from_sqlx)?;

        self.find_by_id(pkg.id).await?.ok_or(DaoError::NotFound)
    }

    /// Save binary data for OTA package
    #[instrument(skip(self, data))]
    pub async fn save_data(&self, id: Uuid, data: &[u8]) -> Result<(), DaoError> {
        let data_size = data.len() as i64;

        sqlx::query!(
            r#"
            UPDATE ota_package
            SET data = $2, data_size = $3
            WHERE id = $1
            "#,
            id,
            data,
            data_size,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get binary data for OTA package
    #[instrument(skip(self))]
    pub async fn get_data(&self, id: Uuid) -> Result<Option<Vec<u8>>, DaoError> {
        let row = sqlx::query!(
            r#"SELECT data FROM ota_package WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.data))
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), DaoError> {
        let result = sqlx::query!("DELETE FROM ota_package WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DaoError::NotFound);
        }
        Ok(())
    }

    /// Lấy OTA package đang pending cho device (firmware_id từ device table)
    #[instrument(skip(self))]
    pub async fn find_pending_for_device(
        &self,
        device_id: Uuid,
    ) -> Result<Option<OtaPackage>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT op.id, op.created_time, op.tenant_id, op.device_profile_id,
                   op.type as ota_type, op.title, op.version, op.tag, op.url,
                   op.file_name, op.content_type, op.data_size,
                   op.checksum_algorithm, op.checksum,
                   op.data IS NOT NULL as has_data,
                   op.additional_info, op.ver
            FROM device d
            JOIN ota_package op ON op.id = d.firmware_id
            WHERE d.id = $1
              AND d.firmware_id IS NOT NULL
            "#,
            device_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| OtaPackage {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_profile_id: r.device_profile_id,
            ota_package_type: OtaPackageType::from_str(&r.ota_type),
            title: r.title,
            version: r.version,
            tag: r.tag,
            url: r.url,
            file_name: r.file_name,
            content_type: r.content_type,
            data_size: r.data_size,
            checksum_algorithm: r.checksum_algorithm.and_then(|s| ChecksumAlgorithm::from_str(&s)),
            checksum: r.checksum,
            has_data: r.has_data.unwrap_or(false),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version_int: r.ver,
        }))
    }

    /// Lấy một chunk bytes từ binary data của package
    /// chunk_index: 0-based, chunk_size_bytes: kích thước mỗi chunk
    #[instrument(skip(self))]
    pub async fn get_chunk(
        &self,
        id:               Uuid,
        chunk_index:      u32,
        chunk_size_bytes: usize,
    ) -> Result<Option<Vec<u8>>, DaoError> {
        let offset = (chunk_index as usize * chunk_size_bytes) as i64;
        let length = chunk_size_bytes as i64;

        // PostgreSQL substr is 1-based; data column is bytea
        let row = sqlx::query!(
            r#"
            SELECT substring(data FROM $2::int FOR $3::int) as chunk
            FROM ota_package
            WHERE id = $1 AND data IS NOT NULL
            "#,
            id,
            (offset + 1) as i32,
            length as i32,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.chunk).filter(|b| !b.is_empty()))
    }

    /// Tìm OTA package theo checksum — dùng để verify integrity
    #[instrument(skip(self))]
    pub async fn find_by_checksum(
        &self,
        checksum: &str,
    ) -> Result<Option<OtaPackage>, DaoError> {
        let row = sqlx::query!(
            r#"
            SELECT id, created_time, tenant_id, device_profile_id,
                   type as ota_type, title, version, tag, url,
                   file_name, content_type, data_size,
                   checksum_algorithm, checksum,
                   data IS NOT NULL as has_data,
                   additional_info, ver
            FROM ota_package WHERE checksum = $1
            LIMIT 1
            "#,
            checksum,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| OtaPackage {
            id: r.id,
            created_time: r.created_time,
            tenant_id: r.tenant_id,
            device_profile_id: r.device_profile_id,
            ota_package_type: OtaPackageType::from_str(&r.ota_type),
            title: r.title,
            version: r.version,
            tag: r.tag,
            url: r.url,
            file_name: r.file_name,
            content_type: r.content_type,
            data_size: r.data_size,
            checksum_algorithm: r.checksum_algorithm.and_then(|s| ChecksumAlgorithm::from_str(&s)),
            checksum: r.checksum,
            has_data: r.has_data.unwrap_or(false),
            additional_info: r.additional_info
                .and_then(|s| serde_json::from_str(&s).ok()),
            version_int: r.ver,
        }))
    }

    /// Check if OTA package with title+version exists for tenant
    #[instrument(skip(self))]
    pub async fn exists_by_title_version(
        &self,
        tenant_id: Uuid,
        ota_type: OtaPackageType,
        title: &str,
        version: &str,
    ) -> Result<bool, DaoError> {
        let count: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM ota_package
               WHERE tenant_id = $1 AND type = $2 AND title = $3 AND version = $4"#,
            tenant_id,
            ota_type.as_str(),
            title,
            version,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(count > 0)
    }
}
