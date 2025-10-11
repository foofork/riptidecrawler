//! Cache Key Consistency Tests
//!
//! TDD tests for cache key generation to ensure:
//! - Deterministic key generation
//! - No collisions across methods
//! - All parameters included in keys
//! - Proper version-based invalidation

#[cfg(test)]
mod cache_key_tests {
    use riptide_core::cache_key::{CacheKeyBuilder, CacheKeyParams};
    use std::collections::{BTreeMap, HashSet};

    #[test]
    fn test_cache_key_deterministic() {
        // Same inputs should always produce same key
        let builder1 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .version("v1.0.0")
            .build();

        let builder2 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .version("v1.0.0")
            .build();

        assert_eq!(builder1, builder2, "Cache keys should be deterministic");
    }

    #[test]
    fn test_cache_key_uniqueness_different_urls() {
        let key1 = CacheKeyBuilder::new()
            .url("https://example.com/page1")
            .method("trek")
            .build();

        let key2 = CacheKeyBuilder::new()
            .url("https://example.com/page2")
            .method("trek")
            .build();

        assert_ne!(key1, key2, "Different URLs must have different keys");
    }

    #[test]
    fn test_cache_key_uniqueness_different_methods() {
        let key1 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .build();

        let key2 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("css_json")
            .build();

        let key3 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("regex")
            .build();

        assert_ne!(key1, key2, "Trek and css_json must have different keys");
        assert_ne!(key2, key3, "css_json and regex must have different keys");
        assert_ne!(key1, key3, "Trek and regex must have different keys");
    }

    #[test]
    fn test_cache_key_includes_all_options() {
        // Without options
        let key_no_opts = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .build();

        // With option1
        let key_opt1 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .option("chunking", "sentence")
            .build();

        // With option2
        let key_opt2 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .option("chunking", "paragraph")
            .build();

        // With both options
        let key_both = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .option("chunking", "sentence")
            .option("language", "en")
            .build();

        assert_ne!(
            key_no_opts, key_opt1,
            "Keys with different options must differ"
        );
        assert_ne!(key_opt1, key_opt2, "Different option values must differ");
        assert_ne!(
            key_opt1, key_both,
            "Different option sets must produce different keys"
        );
    }

    #[test]
    fn test_cache_key_options_order_independence() {
        // Options in different order should produce same key
        let key1 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .option("chunking", "sentence")
            .option("language", "en")
            .option("format", "json")
            .build();

        let key2 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .option("format", "json")
            .option("chunking", "sentence")
            .option("language", "en")
            .build();

        assert_eq!(
            key1, key2,
            "Option order should not affect cache key (BTreeMap ensures sorted)"
        );
    }

    #[test]
    fn test_cache_key_version_invalidation() {
        let key_v1 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .version("v1.0.0")
            .build();

        let key_v2 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .version("v2.0.0")
            .build();

        assert_ne!(
            key_v1, key_v2,
            "Different versions must produce different keys for invalidation"
        );
    }

    #[test]
    fn test_cache_key_no_collision_extensive() {
        // Test 1000 combinations for collisions
        let mut keys = HashSet::new();
        let urls = vec![
            "https://example.com/page1",
            "https://example.com/page2",
            "https://different.com/page",
        ];
        let methods = vec!["trek", "css_json", "regex", "llm", "wasm"];
        let chunking_modes = vec!["sentence", "paragraph", "regex", "topic", "fixed"];
        let cache_modes = vec!["bypass", "write", "read", "refresh"];

        let mut count = 0;
        for url in &urls {
            for method in &methods {
                for chunking in &chunking_modes {
                    for cache_mode in &cache_modes {
                        let key = CacheKeyBuilder::new()
                            .url(url)
                            .method(method)
                            .option("chunking", chunking)
                            .option("cache_mode", cache_mode)
                            .version("v1")
                            .build();

                        assert!(
                            keys.insert(key.clone()),
                            "Collision detected! Key already exists: {}",
                            key
                        );
                        count += 1;
                    }
                }
            }
        }

        println!("Generated {} unique cache keys without collisions", count);
        assert_eq!(keys.len(), count, "All keys must be unique");
    }

    #[test]
    fn test_cache_key_sha256_collision_resistance() {
        // SHA256 should be collision resistant even for similar inputs
        let key1 = CacheKeyBuilder::new()
            .url("https://example.com/a")
            .method("trek")
            .build();

        let key2 = CacheKeyBuilder::new()
            .url("https://example.com/b")
            .method("trek")
            .build();

        // Keys should be sufficiently different despite similar URLs
        assert_ne!(key1, key2);

        // Extract hash portions (assuming format: prefix:version:hash)
        let hash1 = key1.split(':').last().unwrap();
        let hash2 = key2.split(':').last().unwrap();

        // SHA256 hashes should be completely different
        let differing_chars = hash1.chars()
            .zip(hash2.chars())
            .filter(|(c1, c2)| c1 != c2)
            .count();

        // At least 50% of characters should differ (avalanche effect)
        assert!(
            differing_chars > hash1.len() / 2,
            "SHA256 avalanche effect should cause significant differences"
        );
    }

    #[test]
    fn test_cache_key_special_characters_handling() {
        // URLs with special characters should be handled correctly
        let key1 = CacheKeyBuilder::new()
            .url("https://example.com/page?query=value&other=123")
            .method("trek")
            .build();

        let key2 = CacheKeyBuilder::new()
            .url("https://example.com/page?other=123&query=value")
            .method("trek")
            .build();

        // Query parameter order in URL should matter (different URLs)
        assert_ne!(
            key1, key2,
            "Different URL strings should produce different keys"
        );
    }

    #[test]
    fn test_cache_key_empty_options() {
        let key1 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .build();

        let mut empty_options = BTreeMap::new();
        let key2 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .options(empty_options)
            .build();

        assert_eq!(
            key1, key2,
            "Empty options should be equivalent to no options"
        );
    }

    #[test]
    fn test_cache_key_namespace_support() {
        let key1 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .namespace("strategies")
            .build();

        let key2 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .namespace("fetch")
            .build();

        let key3 = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .build();

        assert_ne!(key1, key2, "Different namespaces must differ");
        assert_ne!(key1, key3, "With and without namespace must differ");
    }

    #[test]
    fn test_cache_key_from_params() {
        // Test builder from params struct
        let params = CacheKeyParams {
            url: "https://example.com/page".to_string(),
            method: "trek".to_string(),
            version: "v1.0.0".to_string(),
            options: {
                let mut map = BTreeMap::new();
                map.insert("chunking".to_string(), "sentence".to_string());
                map
            },
            namespace: Some("strategies".to_string()),
        };

        let key_from_params = CacheKeyBuilder::from_params(&params).build();
        let key_manual = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .version("v1.0.0")
            .option("chunking", "sentence")
            .namespace("strategies")
            .build();

        assert_eq!(
            key_from_params, key_manual,
            "Keys from params and manual builder should match"
        );
    }

    #[test]
    fn test_cache_key_length_consistency() {
        // All cache keys should have consistent length (based on SHA256)
        let key1 = CacheKeyBuilder::new()
            .url("https://a.com")
            .method("trek")
            .build();

        let key2 = CacheKeyBuilder::new()
            .url("https://very-long-domain-name-example.com/very/long/path/with/many/segments")
            .method("css_json")
            .option("chunking", "sentence")
            .option("language", "en")
            .option("format", "json")
            .build();

        // Keys should be similar length despite input differences
        let len_diff = (key1.len() as i32 - key2.len() as i32).abs();
        assert!(
            len_diff < 20,
            "Cache key lengths should be consistent regardless of input size"
        );
    }

    #[test]
    fn test_cache_key_render_mode_distinction() {
        // Different render modes should produce different keys
        let key_raw = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .option("render_mode", "raw")
            .build();

        let key_headless = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .option("render_mode", "headless")
            .build();

        let key_pdf = CacheKeyBuilder::new()
            .url("https://example.com/page")
            .method("trek")
            .option("render_mode", "pdf")
            .build();

        assert_ne!(key_raw, key_headless);
        assert_ne!(key_headless, key_pdf);
        assert_ne!(key_raw, key_pdf);
    }
}
