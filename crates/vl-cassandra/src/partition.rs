use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};

/// Granularity phân vùng thời gian — khớp Java: NoSqlTsPartitionDate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionGranularity {
    Minutes,
    Hours,
    Days,
    Months,
    Years,
}

impl PartitionGranularity {
    pub fn from_str_val(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "MINUTES" => Self::Minutes,
            "HOURS"   => Self::Hours,
            "DAYS"    => Self::Days,
            "YEARS"   => Self::Years,
            _         => Self::Months, // mặc định MONTHS
        }
    }
}

impl std::str::FromStr for PartitionGranularity {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_str_val(s))
    }
}

/// Tính giá trị partition từ timestamp (milliseconds).
/// Trả về timestamp của đầu kỳ phân vùng (tính bằng ms).
///
/// Ví dụ MONTHS: ts=2026-03-15 12:34:56 → 2026-03-01 00:00:00 UTC (ms)
pub fn to_partition_ts(ts_ms: i64, granularity: PartitionGranularity) -> i64 {
    let dt: DateTime<Utc> = match Utc.timestamp_millis_opt(ts_ms) {
        chrono::LocalResult::Single(d) => d,
        _ => return ts_ms, // fallback nếu timestamp không hợp lệ
    };

    let truncated = match granularity {
        PartitionGranularity::Minutes => {
            Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), 0)
                .single()
        }
        PartitionGranularity::Hours => {
            Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), dt.hour(), 0, 0)
                .single()
        }
        PartitionGranularity::Days => {
            Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
                .single()
        }
        PartitionGranularity::Months => {
            Utc.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0)
                .single()
        }
        PartitionGranularity::Years => {
            Utc.with_ymd_and_hms(dt.year(), 1, 1, 0, 0, 0)
                .single()
        }
    };

    truncated
        .map(|d| d.timestamp_millis())
        .unwrap_or(ts_ms)
}

/// Tính tất cả partitions cần query trong khoảng [start_ts, end_ts].
/// Dùng để build WHERE clause với nhiều partition values.
pub fn partitions_in_range(
    start_ts: i64,
    end_ts: i64,
    granularity: PartitionGranularity,
) -> Vec<i64> {
    let mut partitions = Vec::new();
    let mut current = to_partition_ts(start_ts, granularity);
    let end_partition = to_partition_ts(end_ts, granularity);

    while current <= end_partition {
        partitions.push(current);
        current = next_partition(current, granularity);
    }

    partitions
}

fn next_partition(partition_ts: i64, granularity: PartitionGranularity) -> i64 {
    let dt: DateTime<Utc> = match Utc.timestamp_millis_opt(partition_ts) {
        chrono::LocalResult::Single(d) => d,
        _ => return partition_ts + 1,
    };

    let next = match granularity {
        PartitionGranularity::Minutes => dt + chrono::Duration::minutes(1),
        PartitionGranularity::Hours   => dt + chrono::Duration::hours(1),
        PartitionGranularity::Days    => dt + chrono::Duration::days(1),
        PartitionGranularity::Months  => {
            let (year, month) = if dt.month() == 12 {
                (dt.year() + 1, 1)
            } else {
                (dt.year(), dt.month() + 1)
            };
            match Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).single() {
                Some(d) => d,
                None => dt + chrono::Duration::days(32),
            }
        }
        PartitionGranularity::Years => {
            match Utc.with_ymd_and_hms(dt.year() + 1, 1, 1, 0, 0, 0).single() {
                Some(d) => d,
                None => dt + chrono::Duration::days(366),
            }
        }
    };

    next.timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_partition_truncates_to_first_of_month() {
        // 2026-03-15 12:34:56 UTC
        let ts = 1773820496000i64;
        let p = to_partition_ts(ts, PartitionGranularity::Months);
        let dt = Utc.timestamp_millis_opt(p).unwrap();
        assert_eq!(dt.day(), 1);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn days_partition_truncates_to_midnight() {
        let ts = 1773820496000i64;
        let p = to_partition_ts(ts, PartitionGranularity::Days);
        let dt = Utc.timestamp_millis_opt(p).unwrap();
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn partitions_in_range_single_month() {
        // Start and end in same month → 1 partition
        let start = to_partition_ts(1773820496000, PartitionGranularity::Months);
        let end = start + 86400_000; // +1 day, same month
        let parts = partitions_in_range(start, end, PartitionGranularity::Months);
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn partitions_in_range_two_months() {
        // 2026-01-01 to 2026-02-01 → 2 partitions
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap().timestamp_millis();
        let end = Utc.with_ymd_and_hms(2026, 2, 15, 0, 0, 0).unwrap().timestamp_millis();
        let parts = partitions_in_range(start, end, PartitionGranularity::Months);
        assert_eq!(parts.len(), 2);
    }
}
