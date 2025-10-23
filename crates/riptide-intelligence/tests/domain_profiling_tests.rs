//! Comprehensive tests for domain profiling module

use riptide_intelligence::domain_profiling::*;
use std::collections::HashMap;

/// Test domain profile creation with defaults
#[test]
fn test_domain_profile_creation() {
    let profile = DomainProfile::new("example.com".to_string());

    assert_eq!(profile.name, "example.com");
    assert_eq!(profile.domain, "example.com");
    assert_eq!(profile.version, "1.0.0");
    assert!(profile.baseline.is_none());
    assert_eq!(profile.metadata.total_requests, 0);
    assert_eq!(profile.metadata.success_rate, 0.0);
}

/// Test domain config defaults
#[test]
fn test_domain_config_defaults() {
    let config = DomainConfig::default();

    assert_eq!(config.stealth_level, "medium");
    assert_eq!(config.rate_limit, 1.0);
    assert!(config.respect_robots_txt);
    assert_eq!(config.ua_strategy, "random");
    assert!(config.schema.is_none());
    assert_eq!(config.confidence_threshold, 0.7);
    assert!(!config.enable_javascript);
    assert_eq!(config.request_timeout_secs, 30);
    assert!(config.custom_headers.is_empty());
    assert!(config.proxy.is_none());
}

/// Test domain config customization
#[test]
fn test_domain_config_custom() {
    let mut custom_headers = HashMap::new();
    custom_headers.insert("Authorization".to_string(), "Bearer token".to_string());

    let config = DomainConfig {
        stealth_level: "high".to_string(),
        rate_limit: 2.0,
        respect_robots_txt: false,
        ua_strategy: "fixed".to_string(),
        schema: Some("news_schema".to_string()),
        confidence_threshold: 0.85,
        enable_javascript: true,
        request_timeout_secs: 60,
        custom_headers,
        proxy: Some("http://proxy:8080".to_string()),
    };

    assert_eq!(config.stealth_level, "high");
    assert_eq!(config.rate_limit, 2.0);
    assert!(!config.respect_robots_txt);
    assert_eq!(config.ua_strategy, "fixed");
    assert_eq!(config.schema.as_ref().unwrap(), "news_schema");
    assert_eq!(config.confidence_threshold, 0.85);
    assert!(config.enable_javascript);
    assert_eq!(config.request_timeout_secs, 60);
    assert!(!config.custom_headers.is_empty());
    assert!(config.proxy.is_some());
}

/// Test domain metadata
#[test]
fn test_domain_metadata() {
    let metadata = DomainMetadata {
        description: Some("Example domain".to_string()),
        tags: vec!["news".to_string(), "media".to_string()],
        author: Some("test_user".to_string()),
        total_requests: 1000,
        success_rate: 0.95,
        avg_response_time_ms: 250,
        last_accessed: None,
    };

    assert_eq!(metadata.total_requests, 1000);
    assert_eq!(metadata.success_rate, 0.95);
    assert_eq!(metadata.avg_response_time_ms, 250);
    assert_eq!(metadata.tags.len(), 2);
}

/// Test domain patterns
#[test]
fn test_domain_patterns() {
    let patterns = DomainPatterns {
        subdomain_regex: vec![r"^www\.".to_string(), r"^blog\.".to_string()],
        path_patterns: vec!["/articles/*".to_string(), "/news/*".to_string()],
        exclude_patterns: vec!["/ads/*".to_string(), "/tracking/*".to_string()],
    };

    assert_eq!(patterns.subdomain_regex.len(), 2);
    assert_eq!(patterns.path_patterns.len(), 2);
    assert_eq!(patterns.exclude_patterns.len(), 2);
}

/// Test URL pattern structure
#[test]
fn test_url_pattern() {
    let pattern = UrlPattern {
        pattern: "/articles/{id}".to_string(),
        regex: "^/articles/\\d+$".to_string(),
        content_type: "article".to_string(),
        examples: vec!["/articles/123".to_string()],
    };

    assert_eq!(pattern.pattern, "/articles/{id}");
    assert_eq!(pattern.content_type, "article");
}

/// Test content pattern
#[test]
fn test_content_pattern() {
    let pattern = ContentPattern {
        pattern_type: "article".to_string(),
        selector: "article.post".to_string(),
        frequency: 0.95,
        confidence: 0.9,
    };

    assert_eq!(pattern.selector, "article.post");
    assert_eq!(pattern.frequency, 0.95);
    assert_eq!(pattern.confidence, 0.9);
}

/// Test site structure
#[test]
fn test_site_structure() {
    let structure = SiteStructure {
        common_elements: vec!["article".to_string(), "div.content".to_string()],
        navigation_patterns: vec!["nav".to_string()],
        content_patterns: vec!["article".to_string()],
        metadata_patterns: vec!["meta".to_string()],
        url_patterns: vec![UrlPattern {
            pattern: "/posts/*".to_string(),
            regex: "^/posts/.*$".to_string(),
            content_type: "post".to_string(),
            examples: vec!["/posts/123".to_string()],
        }],
    };

    assert_eq!(structure.common_elements.len(), 2);
    assert_eq!(structure.navigation_patterns.len(), 1);
    assert_eq!(structure.url_patterns.len(), 1);
}

/// Test drift change types
#[test]
fn test_drift_change() {
    let change = DriftChange {
        change_type: "selector_missing".to_string(),
        location: "div.old-content".to_string(),
        severity: "high".to_string(),
        description: "Content selector no longer found".to_string(),
        before: Some("present".to_string()),
        after: Some("missing".to_string()),
        impact: 0.8,
    };

    assert_eq!(change.change_type, "selector_missing");
    assert_eq!(change.severity, "high");
    assert!(change.before.is_some());
    assert!(change.after.is_some());
}

/// Test drift summary
#[test]
fn test_drift_summary() {
    let summary = DriftSummary {
        total_changes: 5,
        critical: 2,
        major: 2,
        minor: 1,
        structural_changes: 3,
        selector_changes: 1,
        metadata_changes: 1,
    };

    assert_eq!(summary.total_changes, 5);
    assert_eq!(summary.critical, 2);
    assert_eq!(summary.major, 2);
    assert_eq!(summary.minor, 1);
    assert_eq!(summary.structural_changes, 3);
}

/// Test domain patterns empty case
#[test]
fn test_domain_patterns_empty() {
    let patterns = DomainPatterns {
        subdomain_regex: vec![],
        path_patterns: vec![],
        exclude_patterns: vec![],
    };

    assert!(patterns.subdomain_regex.is_empty());
    assert!(patterns.path_patterns.is_empty());
    assert!(patterns.exclude_patterns.is_empty());
}

/// Test domain config serialization
#[test]
fn test_domain_config_serialization() {
    let config = DomainConfig::default();

    // Should serialize to JSON
    let json = serde_json::to_string(&config);
    assert!(json.is_ok());

    // Should deserialize back
    let deserialized = serde_json::from_str::<DomainConfig>(&json.unwrap());
    assert!(deserialized.is_ok());
}

/// Test domain profile serialization
#[test]
fn test_domain_profile_serialization() {
    let profile = DomainProfile::new("example.com".to_string());

    // Should serialize to JSON
    let json = serde_json::to_string(&profile);
    assert!(json.is_ok());

    // Should deserialize back
    let deserialized = serde_json::from_str::<DomainProfile>(&json.unwrap());
    assert!(deserialized.is_ok());
}

/// Test domain metadata update
#[test]
fn test_domain_metadata_update() {
    let mut metadata = DomainMetadata {
        description: None,
        tags: vec![],
        author: None,
        total_requests: 0,
        success_rate: 0.0,
        avg_response_time_ms: 0,
        last_accessed: None,
    };

    // Simulate updates
    metadata.total_requests = 100;
    metadata.success_rate = 0.95;
    metadata.avg_response_time_ms = 200;

    assert_eq!(metadata.total_requests, 100);
    assert_eq!(metadata.success_rate, 0.95);
    assert_eq!(metadata.avg_response_time_ms, 200);
}

/// Test stealth level values
#[test]
fn test_stealth_levels() {
    let levels = vec!["low", "medium", "high", "paranoid"];

    for level in levels {
        let config = DomainConfig {
            stealth_level: level.to_string(),
            ..Default::default()
        };
        assert_eq!(config.stealth_level, level);
    }
}

/// Test rate limit bounds
#[test]
fn test_rate_limit_bounds() {
    let rates = vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0];

    for rate in rates {
        let config = DomainConfig {
            rate_limit: rate,
            ..Default::default()
        };
        assert_eq!(config.rate_limit, rate);
        assert!(config.rate_limit > 0.0);
    }
}
