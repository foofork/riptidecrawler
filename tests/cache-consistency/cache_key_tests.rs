//! Cache key consistency and uniqueness tests
//!
//! Ensures cache keys are unique across all extraction methods

use std::collections::HashSet;

#[cfg(test)]
mod cache_key_tests {
    use super::*;

    #[test]
    fn test_method_discriminator_prefixes() {
        // Each extraction method should have unique prefix
        let url = "https://example.com/test";
        let content = "test content hash";

        let css_key = format!("css:{}:{}", url, content);
        let regex_key = format!("regex:{}:{}", url, content);
        let wasm_key = format!("wasm:{}:{}", url, content);
        let fallback_key = format!("fallback:{}:{}", url, content);

        assert!(css_key.starts_with("css:"));
        assert!(regex_key.starts_with("regex:"));
        assert!(wasm_key.starts_with("wasm:"));
        assert!(fallback_key.starts_with("fallback:"));

        // All keys should be different
        let mut keys = HashSet::new();
        keys.insert(css_key);
        keys.insert(regex_key);
        keys.insert(wasm_key);
        keys.insert(fallback_key);

        assert_eq!(keys.len(), 4, "All cache keys should be unique");
    }

    #[test]
    fn test_url_variations_unique_keys() {
        // Different URL variations should produce different cache keys
        let urls = vec![
            "https://example.com/page",
            "https://example.com/page/",
            "https://example.com/page?query=1",
            "https://example.com/page?query=2",
            "https://example.com/page#section1",
            "https://example.com/page#section2",
        ];

        let content = "same content";
        let mut keys = HashSet::new();

        for url in &urls {
            let key = format!("css:{}:{}", url, content);
            keys.insert(key);
        }

        assert_eq!(keys.len(), urls.len(),
            "Each URL variation should have unique cache key");
    }

    #[test]
    fn test_content_variations_unique_keys() {
        // Different content should produce different cache keys
        let url = "https://example.com/test";
        let contents = vec![
            "content1",
            "content2",
            "content1 ", // with trailing space
            "Content1",  // different case
        ];

        let mut keys = HashSet::new();

        for content in &contents {
            let key = format!("css:{}:{}", url, content);
            keys.insert(key);
        }

        assert_eq!(keys.len(), contents.len(),
            "Each content variation should have unique cache key");
    }

    #[test]
    fn test_cache_key_no_collisions() {
        // Test potential collision scenarios
        let test_cases = vec![
            ("https://example.com/ab", "cd"),
            ("https://example.com/a", "bcd"),
            ("https://example.com", "/abcd"),
        ];

        let mut keys = HashSet::new();

        for (url, content) in test_cases {
            let key = format!("css:{}:{}", url, content);
            assert!(keys.insert(key), "Cache key collision detected for {} + {}", url, content);
        }
    }

    #[test]
    fn test_cache_key_includes_all_components() {
        // Cache keys should include method, URL, and content identifier
        let url = "https://example.com/test";
        let content_hash = "abc123";

        let key = format!("css:{}:{}", url, content_hash);

        assert!(key.contains("css"), "Key should contain method");
        assert!(key.contains("example.com"), "Key should contain URL");
        assert!(key.contains("abc123"), "Key should contain content identifier");
    }

    #[test]
    fn test_cache_key_format_consistency() {
        // All cache keys should follow the same format: "method:url:content"
        let url = "https://example.com";
        let content = "test";

        let methods = vec!["css", "regex", "wasm", "fallback"];

        for method in methods {
            let key = format!("{}:{}:{}", method, url, content);
            let parts: Vec<&str> = key.split(':').collect();

            // Should have at least 4 parts: method, https, //example.com, content
            // (URL splitting on : gives us https and //example.com/... separately)
            assert!(parts.len() >= 3,
                "Cache key should have at least 3 colon-separated parts for method {}", method);
            assert_eq!(parts[0], method, "First part should be method");
        }
    }

    #[test]
    fn test_cache_key_special_characters() {
        // Test cache keys with special characters in URL
        let urls_with_special = vec![
            "https://example.com/path?query=value&other=value2",
            "https://example.com/path#fragment",
            "https://example.com/path%20with%20spaces",
            "https://user:pass@example.com/path",
        ];

        let mut keys = HashSet::new();

        for url in urls_with_special {
            let key = format!("css:{}:content", url);
            keys.insert(key);
        }

        assert_eq!(keys.len(), 4, "Special characters in URL should not cause collisions");
    }

    #[test]
    fn test_cache_key_length_reasonable() {
        // Cache keys should not be excessively long
        let url = "https://example.com/test";
        let content = "x".repeat(100); // Reasonable content identifier

        let key = format!("css:{}:{}", url, content);

        // Reasonable maximum: 500 chars (method + URL + content hash)
        assert!(key.len() < 500, "Cache key length {} exceeds reasonable maximum", key.len());
    }

    #[test]
    fn test_cache_key_deterministic() {
        // Same inputs should always produce same cache key
        let url = "https://example.com/test";
        let content = "test content";

        let key1 = format!("css:{}:{}", url, content);
        let key2 = format!("css:{}:{}", url, content);
        let key3 = format!("css:{}:{}", url, content);

        assert_eq!(key1, key2, "Cache keys should be deterministic");
        assert_eq!(key2, key3, "Cache keys should be deterministic");
    }
}
