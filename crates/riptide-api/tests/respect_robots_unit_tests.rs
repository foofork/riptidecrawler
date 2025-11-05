//! Unit Tests for respect_robots Parameter
//!
//! These tests verify the respect_robots field in CrawlRequest without requiring
//! full API compilation. Tests serialization, deserialization, and type safety.

use riptide_api::models::CrawlRequest;
use serde_json;

// ============================================================================
// Test 1: Field Exists and is Optional<bool>
// ============================================================================

#[test]
fn test_respect_robots_field_exists() {
    let request = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: None,
        max_pages: None,
        concurrency: None,
        respect_robots: Some(true),
        follow_redirects: None,
        user_agent: None,
        timeout: None,
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    assert_eq!(request.respect_robots, Some(true));
}

#[test]
fn test_respect_robots_is_optional() {
    let request = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: None,
        max_pages: None,
        concurrency: None,
        respect_robots: None, // Can be None
        follow_redirects: None,
        user_agent: None,
        timeout: None,
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    assert_eq!(request.respect_robots, None);
}

// ============================================================================
// Test 2: Serialization Tests
// ============================================================================

#[test]
fn test_respect_robots_serializes_when_true() {
    let request = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: None,
        max_pages: None,
        concurrency: None,
        respect_robots: Some(true),
        follow_redirects: None,
        user_agent: None,
        timeout: None,
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("respect_robots"));
}

#[test]
fn test_respect_robots_serializes_when_false() {
    let request = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: None,
        max_pages: None,
        concurrency: None,
        respect_robots: Some(false),
        follow_redirects: None,
        user_agent: None,
        timeout: None,
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("respect_robots"));
    assert!(json.contains("false"));
}

// ============================================================================
// Test 3: Deserialization Tests
// ============================================================================

#[test]
fn test_respect_robots_deserializes_true() {
    let json = r#"{"url": "https://example.com", "respect_robots": true}"#;
    let request: CrawlRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.respect_robots, Some(true));
}

#[test]
fn test_respect_robots_deserializes_false() {
    let json = r#"{"url": "https://example.com", "respect_robots": false}"#;
    let request: CrawlRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.respect_robots, Some(false));
}

#[test]
fn test_respect_robots_deserializes_omitted_as_none() {
    let json = r#"{"url": "https://example.com"}"#;
    let request: CrawlRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.respect_robots, None);
}

#[test]
fn test_respect_robots_rejects_string() {
    let json = r#"{"url": "https://example.com", "respect_robots": "true"}"#;
    let result = serde_json::from_str::<CrawlRequest>(json);

    assert!(
        result.is_err(),
        "Should reject string 'true' instead of boolean"
    );
}

#[test]
fn test_respect_robots_rejects_number() {
    let json = r#"{"url": "https://example.com", "respect_robots": 1}"#;
    let result = serde_json::from_str::<CrawlRequest>(json);

    assert!(result.is_err(), "Should reject number 1 instead of boolean");
}

// ============================================================================
// Test 4: Type Safety
// ============================================================================

#[test]
fn test_respect_robots_type_is_option_bool() {
    fn assert_is_option_bool(_: Option<bool>) {}

    let request = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: None,
        max_pages: None,
        concurrency: None,
        respect_robots: None,
        follow_redirects: None,
        user_agent: None,
        timeout: None,
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    assert_is_option_bool(request.respect_robots);
}

#[test]
fn test_respect_robots_true_false_values() {
    let request_true = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: None,
        max_pages: None,
        concurrency: None,
        respect_robots: Some(true),
        follow_redirects: None,
        user_agent: None,
        timeout: None,
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    let request_false = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: None,
        max_pages: None,
        concurrency: None,
        respect_robots: Some(false),
        follow_redirects: None,
        user_agent: None,
        timeout: None,
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    assert!(request_true.respect_robots.unwrap());
    assert!(!request_false.respect_robots.unwrap());
}

// ============================================================================
// Test 5: Round-Trip Serialization
// ============================================================================

#[test]
fn test_respect_robots_round_trip_true() {
    let original = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: Some(5),
        max_pages: Some(100),
        concurrency: Some(10),
        respect_robots: Some(true),
        follow_redirects: Some(true),
        user_agent: Some("TestBot/1.0".to_string()),
        timeout: Some(30),
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CrawlRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.respect_robots, Some(true));
}

#[test]
fn test_respect_robots_round_trip_false() {
    let original = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: Some(5),
        max_pages: Some(100),
        concurrency: Some(10),
        respect_robots: Some(false),
        follow_redirects: Some(true),
        user_agent: Some("TestBot/1.0".to_string()),
        timeout: Some(30),
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CrawlRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.respect_robots, Some(false));
}

#[test]
fn test_respect_robots_round_trip_none() {
    let original = CrawlRequest {
        url: "https://example.com".to_string(),
        max_depth: Some(5),
        max_pages: Some(100),
        concurrency: Some(10),
        respect_robots: None,
        follow_redirects: Some(true),
        user_agent: Some("TestBot/1.0".to_string()),
        timeout: Some(30),
        headers: None,
        body: None,
        method: None,
        custom_data: None,
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CrawlRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.respect_robots, None);
}
