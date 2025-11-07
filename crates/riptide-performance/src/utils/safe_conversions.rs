//! Safe conversion utilities to avoid dangerous `as` casts in performance metrics
//!
//! Provides validated conversion functions for performance-critical calculations.

use std::time::Duration;

/// Safely convert bytes (u64) to megabytes (f64)
///
/// # Arguments
/// * `bytes` - Number of bytes
///
/// # Returns
/// * Memory size in megabytes (MB)
///
/// # Examples
/// ```
/// use riptide_performance::utils::bytes_to_mb;
///
/// assert_eq!(bytes_to_mb(1024 * 1024), 1.0);
/// assert_eq!(bytes_to_mb(1024 * 1024 * 512), 512.0);
/// ```
#[inline]
pub fn bytes_to_mb(bytes: u64) -> f64 {
    // Safe: Division by constant, no overflow possible
    bytes as f64 / 1024.0 / 1024.0
}

/// Safely convert kilobytes (u64) to megabytes (f64)
///
/// # Arguments
/// * `kb` - Number of kilobytes
///
/// # Returns
/// * Memory size in megabytes (MB)
#[inline]
pub fn kb_to_mb(kb: u64) -> f64 {
    kb as f64 / 1024.0
}

/// Safely convert usize length to f64 for division operations
///
/// # Arguments
/// * `count` - Count value as usize
///
/// # Returns
/// * Count as f64, returns 1.0 if count is 0 to avoid division by zero
///
/// # Examples
/// ```
/// use riptide_performance::utils::count_to_f64_divisor;
///
/// assert_eq!(count_to_f64_divisor(100), 100.0);
/// assert_eq!(count_to_f64_divisor(0), 1.0); // Prevents division by zero
/// ```
#[inline]
pub fn count_to_f64_divisor(count: usize) -> f64 {
    if count == 0 {
        1.0 // Prevent division by zero
    } else {
        count as f64
    }
}

/// Safely convert usize to u64 with saturation
///
/// # Arguments
/// * `value` - Value as usize
///
/// # Returns
/// * Value as u64, saturated to u64::MAX if input exceeds u64::MAX
#[inline]
pub fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Safely convert u128 nanoseconds to u64 nanoseconds with saturation
///
/// Used for Duration conversions that might overflow.
///
/// # Arguments
/// * `nanos` - Nanoseconds as u128
///
/// # Returns
/// * Nanoseconds as u64, saturated to u64::MAX if overflow
#[inline]
pub fn u128_nanos_to_u64(nanos: u128) -> u64 {
    u64::try_from(nanos).unwrap_or(u64::MAX)
}

/// Safely calculate percentile index from collection length
///
/// # Arguments
/// * `len` - Collection length
/// * `percentile` - Percentile (0.0 to 1.0)
///
/// # Returns
/// * Index for the percentile, clamped to valid range
///
/// # Examples
/// ```
/// use riptide_performance::utils::calculate_percentile_index;
///
/// assert_eq!(calculate_percentile_index(100, 0.95), 95);
/// assert_eq!(calculate_percentile_index(100, 0.99), 99);
/// assert_eq!(calculate_percentile_index(10, 0.95), 9);
/// ```
#[inline]
pub fn calculate_percentile_index(len: usize, percentile: f64) -> usize {
    if len == 0 {
        return 0;
    }

    // Clamp percentile to valid range
    let percentile = percentile.clamp(0.0, 1.0);

    // Calculate index using floor method
    // Use len * percentile for correct percentile calculation
    let index = (len as f64 * percentile).floor() as usize;

    // Ensure index is within bounds
    index.min(len - 1)
}

/// Safely convert i64 seconds to f64 for rate calculations
///
/// # Arguments
/// * `seconds` - Time in seconds as i64 (from chrono)
///
/// # Returns
/// * Time as f64, returns 1.0 if seconds <= 0 to avoid division by zero
#[inline]
pub fn seconds_to_f64_divisor(seconds: i64) -> f64 {
    if seconds <= 0 {
        1.0 // Prevent division by zero or negative time
    } else {
        seconds as f64
    }
}

/// Safely convert f64 to u64 with validation
///
/// # Arguments
/// * `value` - Float value
///
/// # Returns
/// * Value as u64, or 0 if value is negative/NaN, u64::MAX if positive infinity
#[inline]
pub fn f64_to_u64_safe(value: f64) -> u64 {
    if value.is_nan() || value < 0.0 {
        0
    } else if value >= u64::MAX as f64 || value.is_infinite() {
        u64::MAX
    } else {
        value.round() as u64
    }
}

/// Safely convert f32 to f64
///
/// # Arguments
/// * `value` - Float value as f32
///
/// # Returns
/// * Value as f64
#[inline]
pub fn f32_to_f64(value: f32) -> f64 {
    f64::from(value)
}

/// Safely convert Duration to u64 seconds with saturation
///
/// # Arguments
/// * `duration` - Duration value
///
/// # Returns
/// * Duration in seconds as u64, saturated on overflow
#[inline]
pub fn duration_to_u64_seconds(duration: Duration) -> u64 {
    duration.as_secs()
}

/// Safely convert Duration to u64 milliseconds with saturation
///
/// # Arguments
/// * `duration` - Duration value
///
/// # Returns
/// * Duration in milliseconds as u64, saturated to u64::MAX on overflow
#[inline]
pub fn duration_to_u64_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// Safely calculate average from sum and count
///
/// # Arguments
/// * `sum` - Sum of values as f64
/// * `count` - Number of values as usize
///
/// # Returns
/// * Average value, or 0.0 if count is 0
#[inline]
pub fn safe_average(sum: f64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        sum / count as f64
    }
}

/// Safely calculate percentage from count and total
///
/// # Arguments
/// * `count` - Count value
/// * `total` - Total value
///
/// # Returns
/// * Percentage (0.0 to 100.0), or 0.0 if total is 0
#[inline]
pub fn safe_percentage(count: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64) * 100.0
    }
}

/// Safely calculate rate (operations per second)
///
/// # Arguments
/// * `operations` - Number of operations
/// * `duration` - Duration in seconds
///
/// # Returns
/// * Operations per second, or 0.0 if duration is 0 or negative
#[inline]
pub fn safe_rate(operations: usize, duration_secs: f64) -> f64 {
    if duration_secs <= 0.0 {
        0.0
    } else {
        operations as f64 / duration_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_mb() {
        assert_eq!(bytes_to_mb(0), 0.0);
        assert_eq!(bytes_to_mb(1024 * 1024), 1.0);
        assert_eq!(bytes_to_mb(1024 * 1024 * 512), 512.0);
        assert_eq!(bytes_to_mb(1536 * 1024), 1.5);
    }

    #[test]
    fn test_kb_to_mb() {
        assert_eq!(kb_to_mb(0), 0.0);
        assert_eq!(kb_to_mb(1024), 1.0);
        assert_eq!(kb_to_mb(2048), 2.0);
    }

    #[test]
    fn test_count_to_f64_divisor() {
        assert_eq!(count_to_f64_divisor(100), 100.0);
        assert_eq!(count_to_f64_divisor(1), 1.0);
        assert_eq!(count_to_f64_divisor(0), 1.0); // Prevents division by zero
    }

    #[test]
    fn test_usize_to_u64() {
        assert_eq!(usize_to_u64(0), 0);
        assert_eq!(usize_to_u64(1000), 1000);
        assert_eq!(usize_to_u64(u32::MAX as usize), u32::MAX as u64);
    }

    #[test]
    fn test_u128_nanos_to_u64() {
        assert_eq!(u128_nanos_to_u64(0), 0);
        assert_eq!(u128_nanos_to_u64(1000), 1000);
        assert_eq!(u128_nanos_to_u64(u64::MAX as u128), u64::MAX);
        assert_eq!(u128_nanos_to_u64(u128::MAX), u64::MAX); // Saturation
    }

    #[test]
    fn test_calculate_percentile_index() {
        assert_eq!(calculate_percentile_index(100, 0.95), 95);
        assert_eq!(calculate_percentile_index(100, 0.99), 99);
        assert_eq!(calculate_percentile_index(10, 0.95), 9);
        assert_eq!(calculate_percentile_index(0, 0.95), 0);
        assert_eq!(calculate_percentile_index(100, 1.5), 99); // Clamped
        assert_eq!(calculate_percentile_index(100, -0.5), 0); // Clamped
    }

    #[test]
    fn test_seconds_to_f64_divisor() {
        assert_eq!(seconds_to_f64_divisor(60), 60.0);
        assert_eq!(seconds_to_f64_divisor(1), 1.0);
        assert_eq!(seconds_to_f64_divisor(0), 1.0); // Prevents division by zero
        assert_eq!(seconds_to_f64_divisor(-10), 1.0); // Prevents negative
    }

    #[test]
    fn test_f64_to_u64_safe() {
        assert_eq!(f64_to_u64_safe(0.0), 0);
        assert_eq!(f64_to_u64_safe(100.5), 101); // Rounds
        assert_eq!(f64_to_u64_safe(100.4), 100);
        assert_eq!(f64_to_u64_safe(-10.0), 0);
        assert_eq!(f64_to_u64_safe(f64::NAN), 0);
        assert_eq!(f64_to_u64_safe(f64::INFINITY), u64::MAX);
        assert_eq!(f64_to_u64_safe(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn test_f32_to_f64() {
        assert_eq!(f32_to_f64(0.0), 0.0);
        assert_eq!(f32_to_f64(1.5), 1.5);
        assert!((f32_to_f64(std::f32::consts::PI) - std::f64::consts::PI).abs() < 0.00001);
    }

    #[test]
    fn test_duration_conversions() {
        let dur = Duration::from_secs(120);
        assert_eq!(duration_to_u64_seconds(dur), 120);

        let dur = Duration::from_millis(5000);
        assert_eq!(duration_to_u64_millis(dur), 5000);
    }

    #[test]
    fn test_safe_average() {
        assert_eq!(safe_average(300.0, 3), 100.0);
        assert_eq!(safe_average(100.0, 0), 0.0); // Division by zero
        assert_eq!(safe_average(0.0, 5), 0.0);
    }

    #[test]
    fn test_safe_percentage() {
        assert_eq!(safe_percentage(50, 100), 50.0);
        assert_eq!(safe_percentage(1, 3), 33.33333333333333);
        assert_eq!(safe_percentage(0, 100), 0.0);
        assert_eq!(safe_percentage(10, 0), 0.0); // Division by zero
    }

    #[test]
    fn test_safe_rate() {
        assert_eq!(safe_rate(100, 10.0), 10.0);
        assert_eq!(safe_rate(50, 2.5), 20.0);
        assert_eq!(safe_rate(100, 0.0), 0.0); // Division by zero
        assert_eq!(safe_rate(100, -5.0), 0.0); // Negative duration
    }
}
