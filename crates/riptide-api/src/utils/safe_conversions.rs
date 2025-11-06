//! Safe conversion utilities to avoid dangerous `as` casts
//!
//! Provides validated conversion functions that handle edge cases properly.

/// Safely convert a confidence score (0.0-1.0 float) to a quality score (0-100 u8)
///
/// # Arguments
/// * `confidence` - Confidence value, expected to be in range 0.0-1.0
///
/// # Returns
/// * Quality score in range 0-100, or 0 if confidence is invalid (NaN/Inf/negative)
///
/// # Examples
/// ```
/// use riptide_api::utils::safe_conversions::confidence_to_quality_score;
///
/// assert_eq!(confidence_to_quality_score(0.95), 95);
/// assert_eq!(confidence_to_quality_score(1.0), 100);
/// assert_eq!(confidence_to_quality_score(0.0), 0);
/// assert_eq!(confidence_to_quality_score(f64::NAN), 0);
/// assert_eq!(confidence_to_quality_score(-0.5), 0);
/// ```
#[inline]
#[allow(clippy::cast_possible_truncation)]
pub fn confidence_to_quality_score(confidence: f64) -> u8 {
    // Validate input is finite and non-negative
    if !confidence.is_finite() || confidence < 0.0 {
        return 0;
    }

    // Clamp to 0.0-1.0 range, multiply by 100, clamp result to u8 range
    let score = (confidence.clamp(0.0, 1.0) * 100.0).round();

    // Safe: score is guaranteed to be in range [0.0, 100.0] after clamping
    // round() ensures it's an integer value, so conversion to u8 is safe
    score.clamp(0.0, 100.0) as u8
}

/// Safely convert word count (usize) to u32 with saturation
///
/// # Arguments
/// * `count` - Word count as usize
///
/// # Returns
/// * Word count as u32, saturated to u32::MAX if input exceeds u32::MAX
///
/// # Examples
/// ```
/// use riptide_api::utils::safe_conversions::word_count_to_u32;
///
/// assert_eq!(word_count_to_u32(1000), 1000);
/// assert_eq!(word_count_to_u32(usize::MAX), u32::MAX);
/// ```
#[inline]
pub fn word_count_to_u32(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Safely convert element count (usize) to u32 with saturation
///
/// # Arguments
/// * `count` - Element count as usize
///
/// # Returns
/// * Count as u32, saturated to u32::MAX if input exceeds u32::MAX
#[inline]
pub fn count_to_u32(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_to_quality_score_valid_inputs() {
        assert_eq!(confidence_to_quality_score(0.0), 0);
        assert_eq!(confidence_to_quality_score(0.25), 25);
        assert_eq!(confidence_to_quality_score(0.5), 50);
        assert_eq!(confidence_to_quality_score(0.75), 75);
        assert_eq!(confidence_to_quality_score(0.95), 95);
        assert_eq!(confidence_to_quality_score(1.0), 100);
    }

    #[test]
    fn test_confidence_to_quality_score_clamping() {
        // Values > 1.0 should be clamped to 100
        assert_eq!(confidence_to_quality_score(1.5), 100);
        assert_eq!(confidence_to_quality_score(100.0), 100);

        // Negative values should return 0
        assert_eq!(confidence_to_quality_score(-0.1), 0);
        assert_eq!(confidence_to_quality_score(-100.0), 0);
    }

    #[test]
    fn test_confidence_to_quality_score_invalid_inputs() {
        // NaN, Infinity should return 0
        assert_eq!(confidence_to_quality_score(f64::NAN), 0);
        assert_eq!(confidence_to_quality_score(f64::INFINITY), 0);
        assert_eq!(confidence_to_quality_score(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn test_confidence_to_quality_score_rounding() {
        assert_eq!(confidence_to_quality_score(0.954), 95);
        assert_eq!(confidence_to_quality_score(0.955), 96);
        assert_eq!(confidence_to_quality_score(0.999), 100);
    }

    #[test]
    fn test_word_count_to_u32() {
        assert_eq!(word_count_to_u32(0), 0);
        assert_eq!(word_count_to_u32(1000), 1000);
        assert_eq!(word_count_to_u32(u32::MAX as usize), u32::MAX);

        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(word_count_to_u32(usize::MAX), u32::MAX);
            assert_eq!(word_count_to_u32(u32::MAX as usize + 1), u32::MAX);
        }
    }

    #[test]
    fn test_count_to_u32() {
        assert_eq!(count_to_u32(0), 0);
        assert_eq!(count_to_u32(42), 42);
        assert_eq!(count_to_u32(u32::MAX as usize), u32::MAX);
    }
}
