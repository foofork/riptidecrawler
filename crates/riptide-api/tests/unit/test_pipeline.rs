use riptide_api::pipeline::{PipelineResult, PipelineStats, GateDecisionStats};
use riptide_core::types::{ExtractedDoc, CrawlOptions};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[cfg(test)]
mod pipeline_result_tests {
    use super::*;

    #[test]
    fn test_pipeline_result_creation() {
        let doc = ExtractedDoc {
            url: "https://example.com".to_string(),
            title: Some("Test Title".to_string()),
            byline: Some("Test Author".to_string()),
            published_iso: None,
            markdown: "# Test Content".to_string(),
            text: "Test Content".to_string(),
            links: vec!["https://link1.com".to_string()],
            media: vec!["https://image1.jpg".to_string()],
        };

        let result = PipelineResult {
            document: doc.clone(),
            from_cache: false,
            gate_decision: "raw".to_string(),
            quality_score: 0.8,
            processing_time_ms: 150,
            cache_key: "test_key".to_string(),
            http_status: 200,
        };

        assert_eq!(result.document.url, "https://example.com");
        assert!(!result.from_cache);
        assert_eq!(result.gate_decision, "raw");
        assert_eq!(result.quality_score, 0.8);
        assert_eq!(result.processing_time_ms, 150);
        assert_eq!(result.cache_key, "test_key");
        assert_eq!(result.http_status, 200);
    }

    #[test]
    fn test_pipeline_result_debug() {
        let doc = ExtractedDoc {
            url: "https://test.com".to_string(),
            title: None,
            byline: None,
            published_iso: None,
            markdown: "content".to_string(),
            text: "content".to_string(),
            links: vec![],
            media: vec![],
        };

        let result = PipelineResult {
            document: doc,
            from_cache: true,
            gate_decision: "cached".to_string(),
            quality_score: 1.0,
            processing_time_ms: 5,
            cache_key: "cache_123".to_string(),
            http_status: 304,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("PipelineResult"));
        assert!(debug_str.contains("from_cache: true"));
        assert!(debug_str.contains("cached"));
    }

    #[test]
    fn test_pipeline_result_clone() {
        let doc = ExtractedDoc {
            url: "https://clone-test.com".to_string(),
            title: Some("Clone Test".to_string()),
            byline: None,
            published_iso: None,
            markdown: "clone content".to_string(),
            text: "clone content".to_string(),
            links: vec![],
            media: vec![],
        };

        let original = PipelineResult {
            document: doc,
            from_cache: false,
            gate_decision: "probes_first".to_string(),
            quality_score: 0.65,
            processing_time_ms: 250,
            cache_key: "clone_key".to_string(),
            http_status: 200,
        };

        let cloned = original.clone();

        assert_eq!(original.document.url, cloned.document.url);
        assert_eq!(original.from_cache, cloned.from_cache);
        assert_eq!(original.gate_decision, cloned.gate_decision);
        assert_eq!(original.quality_score, cloned.quality_score);
        assert_eq!(original.processing_time_ms, cloned.processing_time_ms);
        assert_eq!(original.cache_key, cloned.cache_key);
        assert_eq!(original.http_status, cloned.http_status);
    }

    #[test]
    fn test_pipeline_result_serialization() {
        use serde_json;

        let doc = ExtractedDoc {
            url: "https://serialize-test.com".to_string(),
            title: Some("Serialize Test".to_string()),
            byline: None,
            published_iso: Some("2024-01-01T00:00:00Z".to_string()),
            markdown: "# Serialize Content".to_string(),
            text: "Serialize Content".to_string(),
            links: vec!["https://link.com".to_string()],
            media: vec![],
        };

        let result = PipelineResult {
            document: doc,
            from_cache: true,
            gate_decision: "headless".to_string(),
            quality_score: 0.92,
            processing_time_ms: 500,
            cache_key: "serialize_key".to_string(),
            http_status: 200,
        };

        // Test serialization
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("serialize-test.com"));
        assert!(json.contains("headless"));
        assert!(json.contains("0.92"));

        // Test deserialization
        let deserialized: PipelineResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.document.url, "https://serialize-test.com");
        assert_eq!(deserialized.gate_decision, "headless");
        assert_eq!(deserialized.quality_score, 0.92);
    }
}

#[cfg(test)]
mod pipeline_stats_tests {
    use super::*;

    #[test]
    fn test_pipeline_stats_creation() {
        let stats = PipelineStats {
            total_processed: 10,
            cache_hits: 3,
            successful_extractions: 8,
            failed_extractions: 2,
            gate_decisions: GateDecisionStats {
                raw: 5,
                probes_first: 2,
                headless: 1,
            },
            avg_processing_time_ms: 125.5,
            total_processing_time_ms: 1255,
        };

        assert_eq!(stats.total_processed, 10);
        assert_eq!(stats.cache_hits, 3);
        assert_eq!(stats.successful_extractions, 8);
        assert_eq!(stats.failed_extractions, 2);
        assert_eq!(stats.gate_decisions.raw, 5);
        assert_eq!(stats.gate_decisions.probes_first, 2);
        assert_eq!(stats.gate_decisions.headless, 1);
        assert_eq!(stats.avg_processing_time_ms, 125.5);
        assert_eq!(stats.total_processing_time_ms, 1255);
    }

    #[test]
    fn test_gate_decision_stats_default() {
        let stats = GateDecisionStats::default();

        assert_eq!(stats.raw, 0);
        assert_eq!(stats.probes_first, 0);
        assert_eq!(stats.headless, 0);
    }

    #[test]
    fn test_pipeline_stats_calculations() {
        // Test realistic pipeline statistics calculations
        let mut stats = PipelineStats {
            total_processed: 100,
            cache_hits: 20,
            successful_extractions: 85,
            failed_extractions: 15,
            gate_decisions: GateDecisionStats {
                raw: 50,
                probes_first: 25,
                headless: 10,
            },
            avg_processing_time_ms: 0.0, // Will calculate
            total_processing_time_ms: 12500,
        };

        // Calculate average processing time
        stats.avg_processing_time_ms = if stats.successful_extractions > 0 {
            stats.total_processing_time_ms as f64 / stats.successful_extractions as f64
        } else {
            0.0
        };

        assert_eq!(stats.avg_processing_time_ms, 147.05882352941177); // 12500 / 85

        // Test cache hit rate calculation
        let cache_hit_rate = stats.cache_hits as f64 / stats.total_processed as f64;
        assert_eq!(cache_hit_rate, 0.2); // 20%

        // Test success rate calculation
        let success_rate = stats.successful_extractions as f64 / stats.total_processed as f64;
        assert_eq!(success_rate, 0.85); // 85%
    }

    #[test]
    fn test_pipeline_stats_edge_cases() {
        // Test with zero successful extractions
        let stats = PipelineStats {
            total_processed: 5,
            cache_hits: 0,
            successful_extractions: 0,
            failed_extractions: 5,
            gate_decisions: GateDecisionStats::default(),
            avg_processing_time_ms: 0.0,
            total_processing_time_ms: 0,
        };

        // Should handle division by zero gracefully
        let avg_time = if stats.successful_extractions > 0 {
            stats.total_processing_time_ms as f64 / stats.successful_extractions as f64
        } else {
            0.0
        };

        assert_eq!(avg_time, 0.0);

        // Test with all cache hits
        let all_cached = PipelineStats {
            total_processed: 10,
            cache_hits: 10,
            successful_extractions: 10,
            failed_extractions: 0,
            gate_decisions: GateDecisionStats::default(),
            avg_processing_time_ms: 5.0, // Fast cache responses
            total_processing_time_ms: 50,
        };

        assert_eq!(all_cached.cache_hits, all_cached.total_processed);
        assert_eq!(all_cached.failed_extractions, 0);
    }

    #[test]
    fn test_pipeline_stats_serialization() {
        use serde_json;

        let stats = PipelineStats {
            total_processed: 25,
            cache_hits: 5,
            successful_extractions: 22,
            failed_extractions: 3,
            gate_decisions: GateDecisionStats {
                raw: 12,
                probes_first: 8,
                headless: 2,
            },
            avg_processing_time_ms: 180.25,
            total_processing_time_ms: 4505,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_processed\":25"));
        assert!(json.contains("\"cache_hits\":5"));
        assert!(json.contains("\"avg_processing_time_ms\":180.25"));

        let deserialized: PipelineStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_processed, 25);
        assert_eq!(deserialized.gate_decisions.raw, 12);
        assert_eq!(deserialized.avg_processing_time_ms, 180.25);
    }
}

#[cfg(test)]
mod cache_key_generation_tests {
    use super::*;

    fn generate_cache_key(url: &str, cache_mode: &str) -> String {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        cache_mode.hash(&mut hasher);
        format!("riptide:v1:{}:{:x}", cache_mode, hasher.finish())
    }

    #[test]
    fn test_cache_key_generation() {
        let url = "https://example.com";
        let cache_mode = "enabled";

        let key1 = generate_cache_key(url, cache_mode);
        let key2 = generate_cache_key(url, cache_mode);

        // Same input should produce same key
        assert_eq!(key1, key2);

        // Key should contain expected components
        assert!(key1.starts_with("riptide:v1:"));
        assert!(key1.contains("enabled"));
    }

    #[test]
    fn test_cache_key_uniqueness() {
        let url = "https://example.com";

        let key1 = generate_cache_key(url, "enabled");
        let key2 = generate_cache_key(url, "bypass");
        let key3 = generate_cache_key("https://different.com", "enabled");

        // Different inputs should produce different keys
        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
    }

    #[test]
    fn test_cache_key_format() {
        let key = generate_cache_key("https://test.com", "read_through");

        // Should follow expected format: riptide:v1:{mode}:{hash}
        let parts: Vec<&str> = key.split(':').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "riptide");
        assert_eq!(parts[1], "v1");
        assert_eq!(parts[2], "read_through");

        // Hash part should be hexadecimal
        let hash_part = parts[3];
        assert!(hash_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cache_key_with_special_characters() {
        let urls_with_special_chars = vec![
            "https://example.com/path?query=value&other=123",
            "https://example.com/path#fragment",
            "https://example.com:8080/secure/path",
            "https://sub.domain.example.com/very/long/path/with/many/segments",
        ];

        for url in urls_with_special_chars {
            let key = generate_cache_key(url, "enabled");

            // Should always produce valid cache keys
            assert!(key.starts_with("riptide:v1:"));
            assert!(!key.contains("?"));
            assert!(!key.contains("#"));
            assert!(!key.contains("/"));
        }
    }

    #[test]
    fn test_cache_key_collision_resistance() {
        use std::collections::HashSet;

        let mut keys = HashSet::new();
        let base_urls = vec![
            "https://example.com",
            "https://test.org",
            "https://demo.net",
            "https://sample.io",
        ];

        let cache_modes = vec!["enabled", "bypass", "read_through"];

        // Generate keys for all combinations
        for url in &base_urls {
            for mode in &cache_modes {
                let key = generate_cache_key(url, mode);
                assert!(keys.insert(key), "Cache key collision detected");
            }
        }

        // Should have unique keys for all combinations
        assert_eq!(keys.len(), base_urls.len() * cache_modes.len());
    }
}

#[cfg(test)]
mod pipeline_types_tests {
    use super::*;

    #[test]
    fn test_crawl_options_default() {
        let options = CrawlOptions::default();

        assert_eq!(options.concurrency, 16);
        assert_eq!(options.cache_mode, "read_through");
        assert_eq!(options.dynamic_wait_for, None);
        assert_eq!(options.scroll_steps, 8);
        assert_eq!(options.token_chunk_max, 1200);
        assert_eq!(options.token_overlap, 120);
    }

    #[test]
    fn test_crawl_options_clone() {
        let options = CrawlOptions {
            concurrency: 32,
            cache_mode: "bypass".to_string(),
            dynamic_wait_for: Some("networkidle0".to_string()),
            scroll_steps: 5,
            token_chunk_max: 2000,
            token_overlap: 200,
        };

        let cloned = options.clone();

        assert_eq!(cloned.concurrency, 32);
        assert_eq!(cloned.cache_mode, "bypass");
        assert_eq!(cloned.dynamic_wait_for, Some("networkidle0".to_string()));
        assert_eq!(cloned.scroll_steps, 5);
        assert_eq!(cloned.token_chunk_max, 2000);
        assert_eq!(cloned.token_overlap, 200);
    }

    #[test]
    fn test_extracted_doc_creation() {
        let doc = ExtractedDoc {
            url: "https://example.com/article".to_string(),
            title: Some("Test Article".to_string()),
            byline: Some("John Doe".to_string()),
            published_iso: Some("2024-01-15T10:30:00Z".to_string()),
            markdown: "# Test Article\n\nThis is a test article.".to_string(),
            text: "Test Article\n\nThis is a test article.".to_string(),
            links: vec![
                "https://example.com/related".to_string(),
                "https://external.com/resource".to_string(),
            ],
            media: vec![
                "https://example.com/image.jpg".to_string(),
                "https://example.com/video.mp4".to_string(),
            ],
        };

        assert_eq!(doc.url, "https://example.com/article");
        assert_eq!(doc.title, Some("Test Article".to_string()));
        assert_eq!(doc.byline, Some("John Doe".to_string()));
        assert_eq!(doc.published_iso, Some("2024-01-15T10:30:00Z".to_string()));
        assert!(doc.markdown.contains("# Test Article"));
        assert!(doc.text.contains("This is a test article"));
        assert_eq!(doc.links.len(), 2);
        assert_eq!(doc.media.len(), 2);
    }

    #[test]
    fn test_extracted_doc_minimal() {
        let doc = ExtractedDoc {
            url: "https://minimal.com".to_string(),
            title: None,
            byline: None,
            published_iso: None,
            markdown: "".to_string(),
            text: "".to_string(),
            links: vec![],
            media: vec![],
        };

        assert_eq!(doc.url, "https://minimal.com");
        assert!(doc.title.is_none());
        assert!(doc.byline.is_none());
        assert!(doc.published_iso.is_none());
        assert!(doc.markdown.is_empty());
        assert!(doc.text.is_empty());
        assert!(doc.links.is_empty());
        assert!(doc.media.is_empty());
    }
}

// Property-based tests for pipeline data structures
#[cfg(test)]
mod pipeline_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_pipeline_stats_invariants(
            total in 0usize..1000,
            successful in 0usize..1000,
            failed in 0usize..1000,
            cache_hits in 0usize..1000,
            raw in 0usize..500,
            probes in 0usize..500,
            headless in 0usize..500
        ) {
            let stats = PipelineStats {
                total_processed: total,
                cache_hits: cache_hits.min(total), // Cache hits can't exceed total
                successful_extractions: successful.min(total),
                failed_extractions: failed.min(total),
                gate_decisions: GateDecisionStats {
                    raw,
                    probes_first: probes,
                    headless,
                },
                avg_processing_time_ms: 100.0,
                total_processing_time_ms: 10000,
            };

            // Cache hits should never exceed total processed
            prop_assert!(stats.cache_hits <= stats.total_processed);

            // Successful extractions should never exceed total processed
            prop_assert!(stats.successful_extractions <= stats.total_processed);

            // Failed extractions should never exceed total processed
            prop_assert!(stats.failed_extractions <= stats.total_processed);

            // All counts should be non-negative
            prop_assert!(stats.total_processed >= 0);
            prop_assert!(stats.cache_hits >= 0);
            prop_assert!(stats.successful_extractions >= 0);
            prop_assert!(stats.failed_extractions >= 0);
            prop_assert!(stats.gate_decisions.raw >= 0);
            prop_assert!(stats.gate_decisions.probes_first >= 0);
            prop_assert!(stats.gate_decisions.headless >= 0);
        }

        #[test]
        fn test_cache_key_properties(
            url in "https://[a-zA-Z0-9.-]+\\.com(/[a-zA-Z0-9.-]*)*",
            mode in "(enabled|bypass|read_through)"
        ) {
            let key = generate_cache_key(&url, &mode);

            // Key should always start with prefix
            prop_assert!(key.starts_with("riptide:v1:"));

            // Key should contain the cache mode
            prop_assert!(key.contains(&mode));

            // Key should be deterministic
            let key2 = generate_cache_key(&url, &mode);
            prop_assert_eq!(key, key2);

            // Key should be reasonably short
            prop_assert!(key.len() < 200);
        }

        #[test]
        fn test_quality_score_bounds(score in 0.0f32..1.0f32) {
            let doc = ExtractedDoc {
                url: "https://test.com".to_string(),
                title: Some("Test".to_string()),
                byline: None,
                published_iso: None,
                markdown: "content".to_string(),
                text: "content".to_string(),
                links: vec![],
                media: vec![],
            };

            let result = PipelineResult {
                document: doc,
                from_cache: false,
                gate_decision: "raw".to_string(),
                quality_score: score,
                processing_time_ms: 100,
                cache_key: "test".to_string(),
                http_status: 200,
            };

            // Quality score should be within valid bounds
            prop_assert!(result.quality_score >= 0.0);
            prop_assert!(result.quality_score <= 1.0);
        }
    }

    fn generate_cache_key(url: &str, cache_mode: &str) -> String {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        cache_mode.hash(&mut hasher);
        format!("riptide:v1:{}:{:x}", cache_mode, hasher.finish())
    }
}