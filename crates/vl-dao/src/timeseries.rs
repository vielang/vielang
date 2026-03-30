use async_trait::async_trait;
use uuid::Uuid;

use vl_core::entities::TsRecord;

use crate::DaoError;

/// Aggregation type — khớp ThingsBoard Java: Aggregation enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggType {
    None,
    Avg,
    Min,
    Max,
    Sum,
    Count,
}

impl AggType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "NONE"  => Some(Self::None),
            "AVG"   => Some(Self::Avg),
            "MIN"   => Some(Self::Min),
            "MAX"   => Some(Self::Max),
            "SUM"   => Some(Self::Sum),
            "COUNT" => Some(Self::Count),
            _ => None,
        }
    }

    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Avg   => "AVG",
            Self::Min   => "MIN",
            Self::Max   => "MAX",
            Self::Sum   => "SUM",
            Self::Count => "COUNT",
            Self::None  => "AVG",
        }
    }
}

/// Trait abstraction cho timeseries storage — PostgreSQL hoặc Cassandra.
///
/// Khớp Java: TimeseriesDao + TimeseriesLatestDao interface
/// (gộp lại thành một trait vì VieLang quản lý cả hai bảng cùng nhau).
///
/// Attribute operations (attribute_kv) KHÔNG thuộc trait này — attributes
/// luôn ở PostgreSQL giống Java default mode.
#[async_trait]
pub trait TimeseriesDao: Send + Sync {
    /// Lưu một datapoint vào bảng timeseries lịch sử.
    /// entity_type: "DEVICE", "ASSET", "CUSTOMER"... (dùng cho Cassandra partition key)
    async fn save(&self, entity_type: &str, record: &TsRecord) -> Result<(), DaoError>;

    /// Upsert latest value — ghi đè nếu key đã tồn tại.
    async fn save_latest(&self, entity_type: &str, record: &TsRecord) -> Result<(), DaoError>;

    /// Bulk save nhiều records — 1 DB call thay vì N. Default delegates to save().
    async fn save_batch(&self, entity_type: &str, records: &[TsRecord]) -> Result<(), DaoError> {
        for r in records {
            self.save(entity_type, r).await?;
        }
        Ok(())
    }

    /// Bulk save latest — cập nhật ts_kv_latest cho nhiều records cùng lúc.
    async fn save_latest_batch(&self, entity_type: &str, records: &[TsRecord]) -> Result<(), DaoError> {
        for r in records {
            self.save_latest(entity_type, r).await?;
        }
        Ok(())
    }

    /// Bulk save with TTL (seconds). For PostgreSQL, TTL is stored as metadata
    /// and cleaned by housekeeper. For Cassandra, TTL is native.
    /// Default impl delegates to save_batch (ignores TTL).
    async fn save_batch_with_ttl(
        &self,
        entity_type: &str,
        records: &[TsRecord],
        _ttl_seconds: i64,
    ) -> Result<(), DaoError> {
        self.save_batch(entity_type, records).await
    }

    /// Lấy latest values.
    /// keys = None → trả về tất cả keys của entity.
    /// keys = Some([...]) → chỉ lấy các keys được chỉ định.
    async fn find_latest(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        keys: Option<&[&str]>,
    ) -> Result<Vec<TsRecord>, DaoError>;

    /// Lấy tất cả telemetry key names cho entity (từ bảng latest).
    async fn get_ts_keys(
        &self,
        entity_id: Uuid,
        entity_type: &str,
    ) -> Result<Vec<String>, DaoError>;

    /// Query timeseries lịch sử cho một key trong khoảng thời gian.
    async fn find_range(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        key: &str,
        start_ts: i64,
        end_ts: i64,
        limit: i64,
    ) -> Result<Vec<TsRecord>, DaoError>;

    /// Aggregated timeseries query — groups by time bucket (interval_ms).
    /// Returns TsRecord per bucket; numeric result is in dbl_v.
    /// interval_ms: bucket width in milliseconds (e.g. 3600000 = 1 hour)
    async fn find_range_agg(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        key: &str,
        start_ts: i64,
        end_ts: i64,
        interval_ms: i64,
        agg: AggType,
        limit: i64,
    ) -> Result<Vec<TsRecord>, DaoError>;

    /// Xóa timeseries lịch sử cho các keys trong khoảng thời gian.
    async fn delete_ts(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        keys: &[&str],
        start_ts: i64,
        end_ts: i64,
    ) -> Result<(), DaoError>;

    /// Xóa latest values cho các keys.
    async fn delete_latest(
        &self,
        entity_id: Uuid,
        entity_type: &str,
        keys: &[&str],
    ) -> Result<(), DaoError>;
}
