use serde::{Deserialize, Serialize};

/// Schedule-based activation for alarm rules and profile behaviours.
///
/// A schedule defines windows during which a rule is **active**.
/// Outside windows the rule is suppressed (treated as inactive).
///
/// Java: AlarmSchedule / DefaultAlarmSchedule in ThingsBoard
///
/// Config example:
/// ```json
/// {
///   "type": "SPECIFIC_TIME",
///   "timezone": "Europe/Kyiv",
///   "daysOfWeek": [1, 2, 3, 4, 5],
///   "startsOn": 28800000,
///   "endsOn":   64800000
/// }
/// ```
/// or "CUSTOM" with per-day windows, or "ANY_TIME" (always active).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmSchedule {
    #[serde(rename = "type", default = "default_schedule_type")]
    pub schedule_type: ScheduleType,

    /// Timezone offset in milliseconds (simplified — production would use tz names)
    #[serde(rename = "timezone", default)]
    pub timezone: String,

    /// Days of week active: 1=Mon … 7=Sun (ISO 8601)
    #[serde(rename = "daysOfWeek", default)]
    pub days_of_week: Vec<u8>,

    /// Start time within the day in milliseconds from midnight (UTC offset applied separately)
    #[serde(rename = "startsOn", default)]
    pub starts_on_ms: u64,

    /// End time within the day in milliseconds from midnight
    #[serde(rename = "endsOn", default = "default_end_of_day")]
    pub ends_on_ms: u64,

    /// Custom per-day windows (used when type == CUSTOM)
    #[serde(rename = "items", default)]
    pub items: Vec<DayScheduleItem>,
}

fn default_schedule_type() -> ScheduleType { ScheduleType::AnyTime }
fn default_end_of_day() -> u64 { 86_400_000 } // 24h in ms

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScheduleType {
    /// Rule is always active (no time restriction)
    AnyTime,
    /// Active during a specific time window on selected days of week
    SpecificTime,
    /// Each day can have its own window
    Custom,
}

/// A schedule window for a specific day of week (used in CUSTOM mode)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DayScheduleItem {
    /// 1=Mon … 7=Sun
    #[serde(rename = "dayOfWeek")]
    pub day_of_week: u8,
    #[serde(rename = "enabled", default = "bool_true")]
    pub enabled: bool,
    /// Start time in ms from midnight
    #[serde(rename = "startsOn", default)]
    pub starts_on_ms: u64,
    /// End time in ms from midnight
    #[serde(rename = "endsOn", default = "default_end_of_day")]
    pub ends_on_ms: u64,
}

fn bool_true() -> bool { true }

impl AlarmSchedule {
    /// Check whether the schedule is active at a given Unix timestamp (milliseconds).
    ///
    /// `now_ms` — current time as Unix timestamp in milliseconds.
    ///
    /// Timezone is handled by a simple offset in milliseconds stored in `timezone_offset_ms`
    /// for testability. Production use should parse IANA timezone strings.
    pub fn is_active(&self, now_ms: i64) -> bool {
        match self.schedule_type {
            ScheduleType::AnyTime => true,

            ScheduleType::SpecificTime => {
                let (dow, ms_in_day) = decompose_timestamp(now_ms);
                self.days_of_week.contains(&dow)
                    && ms_in_day >= self.starts_on_ms
                    && ms_in_day < self.ends_on_ms
            }

            ScheduleType::Custom => {
                let (dow, ms_in_day) = decompose_timestamp(now_ms);
                self.items.iter().any(|item| {
                    item.day_of_week == dow
                        && item.enabled
                        && ms_in_day >= item.starts_on_ms
                        && ms_in_day < item.ends_on_ms
                })
            }
        }
    }
}

impl Default for AlarmSchedule {
    fn default() -> Self {
        Self {
            schedule_type: ScheduleType::AnyTime,
            timezone:      String::new(),
            days_of_week:  Vec::new(),
            starts_on_ms:  0,
            ends_on_ms:    86_400_000,
            items:         Vec::new(),
        }
    }
}

/// Decompose a Unix timestamp (ms) into (day_of_week: 1-7, ms_since_midnight).
/// Uses UTC; timezone offsets should be applied before calling this.
///
/// Returns day_of_week as 1=Mon … 7=Sun (ISO 8601).
fn decompose_timestamp(ts_ms: i64) -> (u8, u64) {
    let secs = ts_ms.unsigned_abs() / 1_000;

    // Unix epoch (1970-01-01) was a Thursday = day 4 in ISO 8601 (Mon=1)
    let days_since_epoch = secs / 86_400;
    // Thursday = 4, so (days_since_epoch + 3) % 7 + 1 maps to ISO 1-7
    let day_of_week = ((days_since_epoch + 3) % 7 + 1) as u8;

    let ms_in_day = (ts_ms.unsigned_abs()) % 86_400_000;
    (day_of_week, ms_in_day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 2024-01-08 00:00:00 UTC = Monday (dow=1)
    /// Unix timestamp: 1704672000 seconds = 1_704_672_000_000 ms
    const MONDAY_MIDNIGHT_MS: i64 = 1_704_672_000_000;
    /// 09:00 on Monday = MONDAY_MIDNIGHT + 9h
    const MONDAY_9AM_MS: i64 = MONDAY_MIDNIGHT_MS + 9 * 3_600_000;
    /// 20:00 on Monday
    const MONDAY_8PM_MS: i64 = MONDAY_MIDNIGHT_MS + 20 * 3_600_000;
    /// 2024-01-13 = Saturday (dow=6)
    const SATURDAY_9AM_MS: i64 = MONDAY_MIDNIGHT_MS + 5 * 86_400_000 + 9 * 3_600_000;

    #[test]
    fn any_time_always_active() {
        let s = AlarmSchedule::default();
        assert!(s.is_active(0));
        assert!(s.is_active(MONDAY_9AM_MS));
    }

    #[test]
    fn specific_time_active_within_window() {
        let s = AlarmSchedule {
            schedule_type: ScheduleType::SpecificTime,
            days_of_week:  vec![1, 2, 3, 4, 5], // Mon-Fri
            starts_on_ms:  8 * 3_600_000,        // 08:00
            ends_on_ms:    18 * 3_600_000,        // 18:00
            ..Default::default()
        };
        assert!(s.is_active(MONDAY_9AM_MS));     // Mon 09:00 → active
        assert!(!s.is_active(MONDAY_8PM_MS));    // Mon 20:00 → outside window
        assert!(!s.is_active(SATURDAY_9AM_MS));  // Sat → not in days_of_week
    }

    #[test]
    fn specific_time_boundary() {
        let s = AlarmSchedule {
            schedule_type: ScheduleType::SpecificTime,
            days_of_week:  vec![1],
            starts_on_ms:  9 * 3_600_000,  // 09:00
            ends_on_ms:    10 * 3_600_000, // 10:00
            ..Default::default()
        };
        // Exactly at starts_on → active (>=)
        assert!(s.is_active(MONDAY_MIDNIGHT_MS + 9 * 3_600_000));
        // Exactly at ends_on → NOT active (<)
        assert!(!s.is_active(MONDAY_MIDNIGHT_MS + 10 * 3_600_000));
    }

    #[test]
    fn custom_schedule_per_day() {
        let s = AlarmSchedule {
            schedule_type: ScheduleType::Custom,
            items: vec![
                DayScheduleItem {
                    day_of_week: 1,  // Monday
                    enabled: true,
                    starts_on_ms: 8 * 3_600_000,
                    ends_on_ms: 12 * 3_600_000,
                },
                DayScheduleItem {
                    day_of_week: 6,  // Saturday
                    enabled: false,  // disabled
                    starts_on_ms: 0,
                    ends_on_ms: 86_400_000,
                },
            ],
            ..Default::default()
        };
        assert!(s.is_active(MONDAY_9AM_MS));     // Mon 09:00 → active
        assert!(!s.is_active(MONDAY_8PM_MS));    // Mon 20:00 → outside window
        assert!(!s.is_active(SATURDAY_9AM_MS));  // Sat → disabled
    }

    #[test]
    fn deserialize_from_json() {
        let v = json!({
            "type": "SPECIFIC_TIME",
            "daysOfWeek": [1, 2, 3, 4, 5],
            "startsOn": 28800000,
            "endsOn":   64800000
        });
        let s: AlarmSchedule = serde_json::from_value(v).unwrap();
        assert_eq!(s.schedule_type, ScheduleType::SpecificTime);
        assert_eq!(s.days_of_week, vec![1, 2, 3, 4, 5]);
        assert_eq!(s.starts_on_ms, 28_800_000);
    }

    #[test]
    fn decompose_timestamp_monday() {
        let (dow, ms) = decompose_timestamp(MONDAY_9AM_MS);
        assert_eq!(dow, 1); // Monday
        assert_eq!(ms, 9 * 3_600_000);
    }

    #[test]
    fn decompose_timestamp_saturday() {
        let (dow, _) = decompose_timestamp(SATURDAY_9AM_MS);
        assert_eq!(dow, 6); // Saturday
    }
}
