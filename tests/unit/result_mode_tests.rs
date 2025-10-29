//! Unit tests for Phase 2 ResultMode enum and serialization
//!
//! Tests:
//! - ResultMode enum variants (Stats, Urls)
//! - Serde serialization/deserialization
//! - Default behavior (backward compatibility)
//! - Case-insensitive parsing

#[cfg(test)]
mod result_mode_tests {
    use riptide_api::dto::{ResultMode, SpiderResultStats, SpiderResultUrls};
    use serde_json;

    #[test]
    fn test_result_mode_default_is_stats() {
        let default_mode = ResultMode::default();
        assert_eq!(default_mode, ResultMode::Stats);
    }

    #[test]
    fn test_result_mode_serialize_stats() {
        let mode = ResultMode::Stats;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""stats""#);
    }

    #[test]
    fn test_result_mode_serialize_urls() {
        let mode = ResultMode::Urls;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""urls""#);
    }

    #[test]
    fn test_result_mode_deserialize_stats() {
        let json = r#""stats""#;
        let mode: ResultMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, ResultMode::Stats);
    }

    #[test]
    fn test_result_mode_deserialize_urls() {
        let json = r#""urls""#;
        let mode: ResultMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, ResultMode::Urls);
    }

    #[test]
    fn test_result_mode_deserialize_case_insensitive() {
        // Lowercase (should work)
        let json = r#""stats""#;
        let mode: ResultMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, ResultMode::Stats);

        let json = r#""urls""#;
        let mode: ResultMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, ResultMode::Urls);
    }

    #[test]
    fn test_result_mode_deserialize_invalid() {
        let json = r#""invalid_mode""#;
        let result: Result<ResultMode, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Should fail to deserialize invalid mode");
    }

    #[test]
    fn test_result_mode_equality() {
        assert_eq!(ResultMode::Stats, ResultMode::Stats);
        assert_eq!(ResultMode::Urls, ResultMode::Urls);
        assert_ne!(ResultMode::Stats, ResultMode::Urls);
    }

    #[test]
    fn test_result_mode_clone() {
        let mode = ResultMode::Urls;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_result_mode_debug() {
        let mode = ResultMode::Stats;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Stats"));

        let mode = ResultMode::Urls;
        let debug_str = format!("{:?}", mode);
        assert!(debug_str.contains("Urls"));
    }

    #[test]
    fn test_spider_result_stats_serialization() {
        let stats = SpiderResultStats {
            pages_crawled: 42,
            pages_failed: 3,
            duration_seconds: 12.5,
            stop_reason: "max_pages_reached".to_string(),
            domains: vec!["example.com".to_string(), "test.com".to_string()],
        };

        let json = serde_json::to_value(&stats).unwrap();

        assert_eq!(json["pages_crawled"], 42);
        assert_eq!(json["pages_failed"], 3);
        assert_eq!(json["duration_seconds"], 12.5);
        assert_eq!(json["stop_reason"], "max_pages_reached");
        assert_eq!(json["domains"].as_array().unwrap().len(), 2);

        // Should NOT have discovered_urls field
        assert!(json.get("discovered_urls").is_none());
    }

    #[test]
    fn test_spider_result_urls_serialization() {
        let result = SpiderResultUrls {
            pages_crawled: 10,
            pages_failed: 1,
            duration_seconds: 25.3,
            stop_reason: "completed".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: vec![
                "https://example.com".to_string(),
                "https://example.com/page1".to_string(),
                "https://example.com/page2".to_string(),
            ],
        };

        let json = serde_json::to_value(&result).unwrap();

        assert_eq!(json["pages_crawled"], 10);
        assert_eq!(json["pages_failed"], 1);
        assert_eq!(json["duration_seconds"], 25.3);
        assert_eq!(json["stop_reason"], "completed");
        assert_eq!(json["domains"].as_array().unwrap().len(), 1);

        // MUST have discovered_urls field
        assert!(json.get("discovered_urls").is_some());
        let urls = json["discovered_urls"].as_array().unwrap();
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "https://example.com");
        assert_eq!(urls[1], "https://example.com/page1");
        assert_eq!(urls[2], "https://example.com/page2");
    }

    #[test]
    fn test_spider_result_urls_empty_discovered_urls() {
        let result = SpiderResultUrls {
            pages_crawled: 1,
            pages_failed: 0,
            duration_seconds: 2.0,
            stop_reason: "no_links".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: vec![], // Empty array
        };

        let json = serde_json::to_value(&result).unwrap();

        // discovered_urls should be empty array, not null
        assert!(json["discovered_urls"].is_array());
        assert_eq!(json["discovered_urls"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_spider_result_urls_default_serde() {
        // Test that missing discovered_urls defaults to empty array
        let json = r#"{
            "pages_crawled": 5,
            "pages_failed": 0,
            "duration_seconds": 10.0,
            "stop_reason": "completed",
            "domains": ["example.com"]
        }"#;

        let result: SpiderResultUrls = serde_json::from_str(json).unwrap();
        assert_eq!(result.discovered_urls.len(), 0);
    }

    #[test]
    fn test_spider_result_urls_large_collection() {
        let large_urls: Vec<String> = (0..1000)
            .map(|i| format!("https://example.com/page{}", i))
            .collect();

        let result = SpiderResultUrls {
            pages_crawled: 1000,
            pages_failed: 0,
            duration_seconds: 120.0,
            stop_reason: "max_pages_reached".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: large_urls.clone(),
        };

        let json = serde_json::to_value(&result).unwrap();
        let urls = json["discovered_urls"].as_array().unwrap();
        assert_eq!(urls.len(), 1000);

        // Verify serialization/deserialization round-trip
        let json_str = serde_json::to_string(&result).unwrap();
        let deserialized: SpiderResultUrls = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.discovered_urls.len(), 1000);
        assert_eq!(deserialized.discovered_urls[0], "https://example.com/page0");
        assert_eq!(deserialized.discovered_urls[999], "https://example.com/page999");
    }

    #[test]
    fn test_spider_result_urls_special_characters() {
        let result = SpiderResultUrls {
            pages_crawled: 3,
            pages_failed: 0,
            duration_seconds: 5.0,
            stop_reason: "completed".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: vec![
                "https://example.com/page?param=value&other=123".to_string(),
                "https://example.com/path/to/resource#section".to_string(),
                "https://example.com/encoded%20space".to_string(),
            ],
        };

        let json_str = serde_json::to_string(&result).unwrap();
        let deserialized: SpiderResultUrls = serde_json::from_str(&json_str).unwrap();

        assert_eq!(deserialized.discovered_urls.len(), 3);
        assert!(deserialized.discovered_urls[0].contains("param=value"));
        assert!(deserialized.discovered_urls[1].contains("#section"));
        assert!(deserialized.discovered_urls[2].contains("%20"));
    }

    #[test]
    fn test_spider_result_stats_debug() {
        let stats = SpiderResultStats {
            pages_crawled: 10,
            pages_failed: 1,
            duration_seconds: 5.5,
            stop_reason: "timeout".to_string(),
            domains: vec!["test.com".to_string()],
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("pages_crawled"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_spider_result_urls_debug() {
        let result = SpiderResultUrls {
            pages_crawled: 5,
            pages_failed: 0,
            duration_seconds: 3.0,
            stop_reason: "completed".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: vec!["https://example.com".to_string()],
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("discovered_urls"));
        assert!(debug_str.contains("https://example.com"));
    }
}
