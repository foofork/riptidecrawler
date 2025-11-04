//! Time utilities for timestamp handling and conversions

use chrono::{DateTime, Duration, Utc};

/// Gets the current Unix timestamp in seconds
pub fn now_unix_secs() -> i64 {
    Utc::now().timestamp()
}

/// Gets the current Unix timestamp in milliseconds
pub fn now_unix_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// Gets the current Unix timestamp in microseconds
pub fn now_unix_micros() -> i64 {
    Utc::now().timestamp_micros()
}

/// Converts Unix timestamp (seconds) to DateTime<Utc>
pub fn unix_secs_to_datetime(secs: i64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(secs, 0)
}

/// Converts Unix timestamp (milliseconds) to DateTime<Utc>
pub fn unix_millis_to_datetime(millis: i64) -> Option<DateTime<Utc>> {
    let secs = millis / 1000;
    let nanos = ((millis % 1000) * 1_000_000) as u32;
    DateTime::from_timestamp(secs, nanos)
}

/// Converts DateTime to Unix timestamp in seconds
pub fn datetime_to_unix_secs(dt: &DateTime<Utc>) -> i64 {
    dt.timestamp()
}

/// Converts DateTime to Unix timestamp in milliseconds
pub fn datetime_to_unix_millis(dt: &DateTime<Utc>) -> i64 {
    dt.timestamp_millis()
}

/// Formats DateTime as ISO 8601 string
pub fn format_iso8601(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Parses ISO 8601 string to DateTime
pub fn parse_iso8601(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
}

/// Calculates duration between two timestamps in milliseconds
pub fn duration_millis(start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
    (end - start).num_milliseconds()
}

/// Adds duration in seconds to a DateTime
pub fn add_secs(dt: DateTime<Utc>, secs: i64) -> DateTime<Utc> {
    dt + Duration::seconds(secs)
}

/// Adds duration in milliseconds to a DateTime
pub fn add_millis(dt: DateTime<Utc>, millis: i64) -> DateTime<Utc> {
    dt + Duration::milliseconds(millis)
}

/// Checks if a timestamp is expired (before current time)
pub fn is_expired(dt: DateTime<Utc>) -> bool {
    dt < Utc::now()
}

/// Checks if a timestamp is within a duration from now
pub fn is_within_duration(dt: DateTime<Utc>, duration_secs: i64) -> bool {
    let now = Utc::now();
    let diff = (dt - now).num_seconds().abs();
    diff <= duration_secs
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone};

    #[test]
    fn test_now_unix_secs() {
        let ts = now_unix_secs();
        assert!(ts > 0);
        // Should be reasonable timestamp (after 2020)
        assert!(ts > 1577836800);
    }

    #[test]
    fn test_now_unix_millis() {
        let ts = now_unix_millis();
        assert!(ts > 0);
        // Should be larger than seconds timestamp
        assert!(ts > 1577836800000);
    }

    #[test]
    fn test_now_unix_micros() {
        let ts = now_unix_micros();
        assert!(ts > 0);
        // Should be larger than millis timestamp
        assert!(ts > 1577836800000000);
    }

    #[test]
    fn test_unix_secs_to_datetime() {
        let dt = unix_secs_to_datetime(1609459200).unwrap();
        assert_eq!(dt.year(), 2021);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_unix_millis_to_datetime() {
        let dt = unix_millis_to_datetime(1609459200000).unwrap();
        assert_eq!(dt.year(), 2021);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_datetime_to_unix_secs() {
        let dt = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let ts = datetime_to_unix_secs(&dt);
        assert_eq!(ts, 1609459200);
    }

    #[test]
    fn test_datetime_to_unix_millis() {
        let dt = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let ts = datetime_to_unix_millis(&dt);
        assert_eq!(ts, 1609459200000);
    }

    #[test]
    fn test_format_parse_iso8601() {
        let dt = Utc.with_ymd_and_hms(2021, 1, 1, 12, 30, 45).unwrap();
        let formatted = format_iso8601(&dt);
        let parsed = parse_iso8601(&formatted).unwrap();

        assert_eq!(datetime_to_unix_secs(&dt), datetime_to_unix_secs(&parsed));
    }

    #[test]
    fn test_duration_millis() {
        let start = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 5).unwrap();

        assert_eq!(duration_millis(start, end), 5000);
    }

    #[test]
    fn test_add_secs() {
        let dt = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let new_dt = add_secs(dt, 60);

        assert_eq!(duration_millis(dt, new_dt), 60000);
    }

    #[test]
    fn test_add_millis() {
        let dt = Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap();
        let new_dt = add_millis(dt, 5000);

        assert_eq!(duration_millis(dt, new_dt), 5000);
    }

    #[test]
    fn test_is_expired() {
        let past = Utc::now() - Duration::hours(1);
        let future = Utc::now() + Duration::hours(1);

        assert!(is_expired(past));
        assert!(!is_expired(future));
    }

    #[test]
    fn test_is_within_duration() {
        let near_future = Utc::now() + Duration::seconds(30);
        let far_future = Utc::now() + Duration::seconds(120);

        assert!(is_within_duration(near_future, 60));
        assert!(!is_within_duration(far_future, 60));
    }
}
