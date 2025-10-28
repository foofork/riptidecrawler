/// UTF-8 utilities for safe string conversion in WASM environment
/// Provides fallback mechanisms for handling invalid UTF-8 sequences
use std::borrow::Cow;

/// Safely convert bytes to UTF-8 string with lossy conversion fallback
///
/// This is critical for WASM environments where Unicode normalization
/// or validation might fail. Uses lossy conversion to replace invalid
/// UTF-8 sequences with U+FFFD (�) replacement character.
///
/// # Arguments
/// * `bytes` - Byte slice to convert
///
/// # Returns
/// * `Cow<str>` - Borrowed string if valid UTF-8, owned with replacements if invalid
pub fn safe_utf8_conversion(bytes: &[u8]) -> Cow<'_, str> {
    String::from_utf8_lossy(bytes)
}

/// Convert attribute bytes to string with safe error handling
///
/// Designed for use with `tl` parser attribute values that come as Option<Bytes>
///
/// # Arguments
/// * `attr_bytes` - Optional attribute bytes from tl::HTMLTag::attributes()
///
/// # Returns
/// * `Option<String>` - Some(string) if valid, None if attribute missing or conversion fails
#[allow(dead_code)]
pub fn attr_bytes_to_string(attr_bytes: Option<impl AsRef<[u8]>>) -> Option<String> {
    attr_bytes.map(|bytes| safe_utf8_conversion(bytes.as_ref()).into_owned())
}

/// Extract string from tl attribute with fallback
///
/// Convenience function that handles the common pattern of:
/// tag.attributes().get("name").and_then(|bytes| convert to string)
///
/// # Arguments
/// * `attributes` - tl::Attributes reference
/// * `name` - Attribute name to extract
///
/// # Returns
/// * `Option<String>` - Attribute value as string, or None if not present
pub fn get_attr_string(attributes: &tl::Attributes, name: &str) -> Option<String> {
    attributes.get(name).and_then(|opt_bytes| {
        opt_bytes.map(|bytes| safe_utf8_conversion(bytes.as_bytes()).into_owned())
    })
}

/// Validate that a string contains only valid Unicode
///
/// Checks for:
/// - Invalid UTF-8 sequences
/// - Surrogate pairs (U+D800 to U+DFFF)
/// - Non-characters (U+FDD0 to U+FDEF, and ending in FFFE/FFFF)
///
/// # Arguments
/// * `s` - String to validate
///
/// # Returns
/// * `bool` - true if string is valid Unicode, false otherwise
#[allow(dead_code)]
pub fn is_valid_unicode(s: &str) -> bool {
    // Check for invalid characters
    !s.chars().any(|c| {
        let code = c as u32;
        // Surrogate pairs
        (0xD800..=0xDFFF).contains(&code) ||
        // Non-characters
        (0xFDD0..=0xFDEF).contains(&code) ||
        (code & 0xFFFE) == 0xFFFE
    })
}

/// Sanitize string by removing invalid Unicode characters
///
/// # Arguments
/// * `s` - String to sanitize
///
/// # Returns
/// * `String` - Sanitized string with invalid characters removed
#[allow(dead_code)]
pub fn sanitize_unicode(s: &str) -> String {
    s.chars()
        .filter(|c| {
            let code = *c as u32;
            !((0xD800..=0xDFFF).contains(&code)
                || (0xFDD0..=0xFDEF).contains(&code)
                || (code & 0xFFFE) == 0xFFFE)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_utf8_conversion_valid() {
        let bytes = b"Hello, World!";
        let result = safe_utf8_conversion(bytes);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_safe_utf8_conversion_emoji() {
        let bytes = "👋 Hello 🌍".as_bytes();
        let result = safe_utf8_conversion(bytes);
        assert_eq!(result, "👋 Hello 🌍");
    }

    #[test]
    fn test_safe_utf8_conversion_cjk() {
        let bytes = "你好世界".as_bytes();
        let result = safe_utf8_conversion(bytes);
        assert_eq!(result, "你好世界");
    }

    #[test]
    fn test_safe_utf8_conversion_rtl() {
        let bytes = "مرحبا بالعالم".as_bytes();
        let result = safe_utf8_conversion(bytes);
        assert_eq!(result, "مرحبا بالعالم");
    }

    #[test]
    fn test_safe_utf8_conversion_mixed() {
        let bytes = "Hello 世界 🌍 مرحبا".as_bytes();
        let result = safe_utf8_conversion(bytes);
        assert_eq!(result, "Hello 世界 🌍 مرحبا");
    }

    #[test]
    fn test_safe_utf8_conversion_invalid() {
        // Invalid UTF-8 sequence
        let bytes = &[0xFF, 0xFE, 0xFD];
        let result = safe_utf8_conversion(bytes);
        // Should contain replacement characters
        assert!(result.contains('�'));
    }

    #[test]
    fn test_is_valid_unicode() {
        assert!(is_valid_unicode("Hello, World!"));
        assert!(is_valid_unicode("👋 Hello 🌍"));
        assert!(is_valid_unicode("你好世界"));
        assert!(is_valid_unicode("مرحبا بالعالم"));
    }

    #[test]
    fn test_sanitize_unicode() {
        let input = "Hello, World!";
        assert_eq!(sanitize_unicode(input), input);

        let input_with_emoji = "👋 Hello 🌍";
        assert_eq!(sanitize_unicode(input_with_emoji), input_with_emoji);
    }
}
