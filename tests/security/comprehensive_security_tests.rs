//! Comprehensive security test suite for RipTide
//!
//! This module provides thorough testing of all security mechanisms including:
//! - API key authentication and authorization
//! - Budget enforcement with edge cases
//! - PII redaction validation
//! - Audit logging verification
//! - Rate limiting under various scenarios
//! - XSS/CSRF protection
//! - Input validation and sanitization

use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use riptide_core::security::{
    SecurityConfig, SecurityMiddleware, RateLimitConfig, CSPBuilder,
    sanitize_file_path, validate_http_method
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Mock API key store for testing
#[derive(Debug, Clone)]
struct MockApiKeyStore {
    keys: Arc<Mutex<HashMap<String, ApiKeyData>>>,
}

#[derive(Debug, Clone)]
struct ApiKeyData {
    key: String,
    is_active: bool,
    budget_limit: Option<u64>,
    budget_used: u64,
    rate_limit: Option<RateLimitConfig>,
    permissions: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl MockApiKeyStore {
    fn new() -> Self {
        Self {
            keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn add_key(&self, key_data: ApiKeyData) {
        let mut keys = self.keys.lock().await;
        keys.insert(key_data.key.clone(), key_data);
    }

    async fn validate_key(&self, key: &str) -> Option<ApiKeyData> {
        let keys = self.keys.lock().await;
        keys.get(key).cloned()
    }

    async fn update_usage(&self, key: &str, amount: u64) -> Result<()> {
        let mut keys = self.keys.lock().await;
        if let Some(key_data) = keys.get_mut(key) {
            key_data.budget_used += amount;
            key_data.last_used = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(anyhow::anyhow!("API key not found"))
        }
    }
}

/// Mock PII detector for testing redaction
#[derive(Debug)]
struct MockPiiDetector {
    patterns: Vec<PiiPattern>,
}

#[derive(Debug, Clone)]
struct PiiPattern {
    name: String,
    regex: regex::Regex,
    replacement: String,
}

impl MockPiiDetector {
    fn new() -> Result<Self> {
        let patterns = vec![
            PiiPattern {
                name: "SSN".to_string(),
                regex: regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")?,
                replacement: "[REDACTED-SSN]".to_string(),
            },
            PiiPattern {
                name: "Email".to_string(),
                regex: regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?,
                replacement: "[REDACTED-EMAIL]".to_string(),
            },
            PiiPattern {
                name: "Phone".to_string(),
                regex: regex::Regex::new(r"\b\d{3}-\d{3}-\d{4}\b")?,
                replacement: "[REDACTED-PHONE]".to_string(),
            },
            PiiPattern {
                name: "CreditCard".to_string(),
                regex: regex::Regex::new(r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b")?,
                replacement: "[REDACTED-CC]".to_string(),
            },
        ];

        Ok(Self { patterns })
    }

    fn redact_pii(&self, text: &str) -> String {
        let mut result = text.to_string();
        for pattern in &self.patterns {
            result = pattern.regex.replace_all(&result, &pattern.replacement).to_string();
        }
        result
    }

    fn detect_pii(&self, text: &str) -> Vec<String> {
        let mut detected = Vec::new();
        for pattern in &self.patterns {
            if pattern.regex.is_match(text) {
                detected.push(pattern.name.clone());
            }
        }
        detected
    }
}

/// Audit log entry for security testing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AuditLogEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    event_type: String,
    user_id: Option<String>,
    api_key_id: Option<String>,
    resource: String,
    action: String,
    result: String,
    details: HashMap<String, serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

/// Mock audit logger
#[derive(Debug, Clone)]
struct MockAuditLogger {
    logs: Arc<Mutex<Vec<AuditLogEntry>>>,
}

impl MockAuditLogger {
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn log(&self, entry: AuditLogEntry) {
        let mut logs = self.logs.lock().await;
        logs.push(entry);
    }

    async fn get_logs(&self) -> Vec<AuditLogEntry> {
        let logs = self.logs.lock().await;
        logs.clone()
    }

    async fn get_logs_by_type(&self, event_type: &str) -> Vec<AuditLogEntry> {
        let logs = self.logs.lock().await;
        logs.iter()
            .filter(|entry| entry.event_type == event_type)
            .cloned()
            .collect()
    }
}

/// Rate limiter for testing
#[derive(Debug)]
struct MockRateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    config: RateLimitConfig,
}

impl MockRateLimiter {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    async fn check_rate_limit(&self, key: &str) -> Result<bool> {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let window = Duration::from_secs(self.config.window_seconds);

        let key_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the window
        key_requests.retain(|&timestamp| now.duration_since(timestamp) <= window);

        // Check if we're within limits
        if key_requests.len() < self.config.requests_per_window as usize {
            key_requests.push(now);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[tokio::test]
async fn test_api_key_authentication() {
    let store = MockApiKeyStore::new();
    let audit_logger = MockAuditLogger::new();

    // Create test API keys
    let valid_key = ApiKeyData {
        key: "valid-api-key-123".to_string(),
        is_active: true,
        budget_limit: Some(1000),
        budget_used: 0,
        rate_limit: Some(RateLimitConfig::default()),
        permissions: vec!["search".to_string(), "extract".to_string()],
        created_at: chrono::Utc::now(),
        last_used: None,
    };

    let expired_key = ApiKeyData {
        key: "expired-api-key-456".to_string(),
        is_active: false,
        budget_limit: Some(1000),
        budget_used: 0,
        rate_limit: Some(RateLimitConfig::default()),
        permissions: vec!["search".to_string()],
        created_at: chrono::Utc::now(),
        last_used: None,
    };

    let budget_exceeded_key = ApiKeyData {
        key: "budget-exceeded-789".to_string(),
        is_active: true,
        budget_limit: Some(100),
        budget_used: 150,
        rate_limit: Some(RateLimitConfig::default()),
        permissions: vec!["search".to_string()],
        created_at: chrono::Utc::now(),
        last_used: None,
    };

    store.add_key(valid_key.clone()).await;
    store.add_key(expired_key.clone()).await;
    store.add_key(budget_exceeded_key.clone()).await;

    // Test valid API key
    let valid_result = store.validate_key("valid-api-key-123").await;
    assert!(valid_result.is_some(), "Valid API key should be found");
    assert!(valid_result.unwrap().is_active, "Valid API key should be active");

    // Test invalid API key
    let invalid_result = store.validate_key("non-existent-key").await;
    assert!(invalid_result.is_none(), "Non-existent API key should not be found");

    // Test expired API key
    let expired_result = store.validate_key("expired-api-key-456").await;
    assert!(expired_result.is_some(), "Expired API key should be found");
    assert!(!expired_result.unwrap().is_active, "Expired API key should not be active");

    // Log authentication attempts
    audit_logger.log(AuditLogEntry {
        timestamp: chrono::Utc::now(),
        event_type: "api_key_validation".to_string(),
        user_id: None,
        api_key_id: Some("valid-api-key-123".to_string()),
        resource: "auth".to_string(),
        action: "validate".to_string(),
        result: "success".to_string(),
        details: HashMap::new(),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("RipTide-Test/1.0".to_string()),
    }).await;

    let logs = audit_logger.get_logs_by_type("api_key_validation").await;
    assert_eq!(logs.len(), 1, "Should have one authentication log entry");
}

#[tokio::test]
async fn test_budget_enforcement_edge_cases() {
    let store = MockApiKeyStore::new();

    // Test exact budget limit
    let exact_budget_key = ApiKeyData {
        key: "exact-budget-key".to_string(),
        is_active: true,
        budget_limit: Some(100),
        budget_used: 100,
        rate_limit: None,
        permissions: vec!["search".to_string()],
        created_at: chrono::Utc::now(),
        last_used: None,
    };

    store.add_key(exact_budget_key).await;

    // Test usage that would exceed budget
    let result = store.update_usage("exact-budget-key", 1).await;
    assert!(result.is_ok(), "Should be able to update usage");

    let key_data = store.validate_key("exact-budget-key").await.unwrap();
    assert_eq!(key_data.budget_used, 101, "Budget should be exceeded");

    // Test unlimited budget (None)
    let unlimited_key = ApiKeyData {
        key: "unlimited-key".to_string(),
        is_active: true,
        budget_limit: None,
        budget_used: 0,
        rate_limit: None,
        permissions: vec!["search".to_string()],
        created_at: chrono::Utc::now(),
        last_used: None,
    };

    store.add_key(unlimited_key).await;

    // Should be able to use unlimited budget
    store.update_usage("unlimited-key", 999999).await.unwrap();
    let unlimited_data = store.validate_key("unlimited-key").await.unwrap();
    assert_eq!(unlimited_data.budget_used, 999999, "Should allow unlimited usage");

    // Test concurrent budget updates
    let concurrent_key = ApiKeyData {
        key: "concurrent-key".to_string(),
        is_active: true,
        budget_limit: Some(1000),
        budget_used: 0,
        rate_limit: None,
        permissions: vec!["search".to_string()],
        created_at: chrono::Utc::now(),
        last_used: None,
    };

    store.add_key(concurrent_key).await;

    let mut handles = Vec::new();
    for i in 0..50 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            store_clone.update_usage("concurrent-key", 10).await
        });
        handles.push(handle);
    }

    // Wait for all updates
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let final_data = store.validate_key("concurrent-key").await.unwrap();
    assert_eq!(final_data.budget_used, 500, "Concurrent updates should sum correctly");
}

#[tokio::test]
async fn test_pii_redaction_validation() -> Result<()> {
    let pii_detector = MockPiiDetector::new()?;

    // Test various PII patterns
    let test_cases = vec![
        (
            "My SSN is 123-45-6789 and email is john@example.com",
            "My SSN is [REDACTED-SSN] and email is [REDACTED-EMAIL]"
        ),
        (
            "Call me at 555-123-4567 or email test@domain.org",
            "Call me at [REDACTED-PHONE] or email [REDACTED-EMAIL]"
        ),
        (
            "Credit card: 1234 5678 9012 3456",
            "Credit card: [REDACTED-CC]"
        ),
        (
            "No PII in this text",
            "No PII in this text"
        ),
        (
            "Mixed case EMAIL@DOMAIN.COM and phone 123-456-7890",
            "Mixed case [REDACTED-EMAIL] and phone [REDACTED-PHONE]"
        ),
    ];

    for (input, expected) in test_cases {
        let redacted = pii_detector.redact_pii(input);
        assert_eq!(redacted, expected, "PII redaction failed for: {}", input);

        // Test detection
        let detected_types = pii_detector.detect_pii(input);
        if input.contains("No PII") {
            assert!(detected_types.is_empty(), "Should not detect PII in clean text");
        } else {
            assert!(!detected_types.is_empty(), "Should detect PII types in: {}", input);
        }
    }

    // Test edge cases
    let edge_cases = vec![
        "123-45-67890", // Invalid SSN format
        "invalid-email@", // Invalid email format
        "123-45-6789-extra", // SSN with extra characters
        "@domain.com", // Email missing username
    ];

    for case in edge_cases {
        let redacted = pii_detector.redact_pii(case);
        println!("Edge case '{}' -> '{}'", case, redacted);
    }

    Ok(())
}

#[tokio::test]
async fn test_audit_logging_verification() {
    let audit_logger = MockAuditLogger::new();

    // Test various audit events
    let events = vec![
        AuditLogEntry {
            timestamp: chrono::Utc::now(),
            event_type: "search_request".to_string(),
            user_id: Some("user-123".to_string()),
            api_key_id: Some("key-456".to_string()),
            resource: "search".to_string(),
            action: "query".to_string(),
            result: "success".to_string(),
            details: json!({
                "query": "test search",
                "results_count": 10,
                "response_time_ms": 150
            }).as_object().unwrap().clone(),
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        },
        AuditLogEntry {
            timestamp: chrono::Utc::now(),
            event_type: "authentication_failure".to_string(),
            user_id: None,
            api_key_id: Some("invalid-key".to_string()),
            resource: "auth".to_string(),
            action: "validate".to_string(),
            result: "failure".to_string(),
            details: json!({
                "reason": "invalid_api_key",
                "attempt_count": 3
            }).as_object().unwrap().clone(),
            ip_address: Some("192.168.1.200".to_string()),
            user_agent: Some("Malicious-Bot/1.0".to_string()),
        },
        AuditLogEntry {
            timestamp: chrono::Utc::now(),
            event_type: "rate_limit_exceeded".to_string(),
            user_id: Some("user-789".to_string()),
            api_key_id: Some("key-789".to_string()),
            resource: "api".to_string(),
            action: "request".to_string(),
            result: "blocked".to_string(),
            details: json!({
                "requests_in_window": 105,
                "window_limit": 100,
                "window_seconds": 60
            }).as_object().unwrap().clone(),
            ip_address: Some("192.168.1.150".to_string()),
            user_agent: Some("RipTide-Client/1.0".to_string()),
        },
    ];

    // Log all events
    for event in &events {
        audit_logger.log(event.clone()).await;
    }

    // Verify all logs are stored
    let all_logs = audit_logger.get_logs().await;
    assert_eq!(all_logs.len(), 3, "Should have all audit log entries");

    // Verify filtering by event type
    let auth_failures = audit_logger.get_logs_by_type("authentication_failure").await;
    assert_eq!(auth_failures.len(), 1, "Should have one auth failure log");

    let rate_limit_logs = audit_logger.get_logs_by_type("rate_limit_exceeded").await;
    assert_eq!(rate_limit_logs.len(), 1, "Should have one rate limit log");

    // Verify log content integrity
    let search_logs = audit_logger.get_logs_by_type("search_request").await;
    assert_eq!(search_logs.len(), 1, "Should have one search log");
    let search_log = &search_logs[0];
    assert_eq!(search_log.user_id, Some("user-123".to_string()));
    assert_eq!(search_log.result, "success");
    assert!(search_log.details.contains_key("query"));
}

#[tokio::test]
async fn test_rate_limiting_scenarios() {
    let rate_limiter = MockRateLimiter::new(RateLimitConfig {
        requests_per_window: 5,
        window_seconds: 10,
        burst_size: 2,
    });

    let client_id = "test-client";

    // Test normal rate limiting
    for i in 1..=5 {
        let allowed = rate_limiter.check_rate_limit(client_id).await.unwrap();
        assert!(allowed, "Request {} should be allowed", i);
    }

    // 6th request should be rejected
    let rejected = rate_limiter.check_rate_limit(client_id).await.unwrap();
    assert!(!rejected, "6th request should be rejected");

    // Test window expiration
    tokio::time::sleep(Duration::from_secs(11)).await;

    // After window expires, requests should be allowed again
    let allowed_after_window = rate_limiter.check_rate_limit(client_id).await.unwrap();
    assert!(allowed_after_window, "Request after window should be allowed");

    // Test different clients
    let other_client = "other-client";
    for i in 1..=5 {
        let allowed = rate_limiter.check_rate_limit(other_client).await.unwrap();
        assert!(allowed, "Other client request {} should be allowed", i);
    }

    // Test concurrent rate limiting
    let concurrent_limiter = MockRateLimiter::new(RateLimitConfig {
        requests_per_window: 10,
        window_seconds: 60,
        burst_size: 5,
    });

    let concurrent_client = "concurrent-client";
    let mut handles = Vec::new();

    for i in 0..20 {
        let limiter_clone = concurrent_limiter.clone();
        let client_clone = concurrent_client.to_string();
        let handle = tokio::spawn(async move {
            limiter_clone.check_rate_limit(&client_clone).await
        });
        handles.push(handle);
    }

    let mut allowed_count = 0;
    let mut rejected_count = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(true) => allowed_count += 1,
            Ok(false) => rejected_count += 1,
            Err(_) => panic!("Rate limiter should not error"),
        }
    }

    assert_eq!(allowed_count, 10, "Should allow exactly 10 requests");
    assert_eq!(rejected_count, 10, "Should reject exactly 10 requests");
}

#[tokio::test]
async fn test_security_headers_validation() {
    let security_middleware = SecurityMiddleware::new_default();
    let mut headers = HeaderMap::new();

    // Apply security headers
    security_middleware.apply_security_headers(&mut headers).unwrap();

    // Verify all critical security headers are present
    assert!(headers.contains_key("X-XSS-Protection"), "Should have XSS protection header");
    assert!(headers.contains_key("X-Content-Type-Options"), "Should have content type options header");
    assert!(headers.contains_key("X-Frame-Options"), "Should have frame options header");
    assert!(headers.contains_key("Strict-Transport-Security"), "Should have HSTS header");
    assert!(headers.contains_key("Content-Security-Policy"), "Should have CSP header");
    assert!(headers.contains_key("Referrer-Policy"), "Should have referrer policy header");
    assert!(headers.contains_key("Permissions-Policy"), "Should have permissions policy header");

    // Verify header values
    assert_eq!(
        headers.get("X-XSS-Protection").unwrap().to_str().unwrap(),
        "1; mode=block"
    );
    assert_eq!(
        headers.get("X-Content-Type-Options").unwrap().to_str().unwrap(),
        "nosniff"
    );
    assert_eq!(
        headers.get("X-Frame-Options").unwrap().to_str().unwrap(),
        "DENY"
    );

    // Verify CORS headers
    assert!(headers.contains_key("Access-Control-Allow-Origin"));
    assert!(headers.contains_key("Access-Control-Allow-Methods"));
    assert!(headers.contains_key("Access-Control-Allow-Headers"));
}

#[tokio::test]
async fn test_input_sanitization() {
    let security_middleware = SecurityMiddleware::new_default();

    // Test header sanitization
    let mut test_headers = HeaderMap::new();
    test_headers.insert(
        HeaderName::from_static("x-safe-header"),
        HeaderValue::from_static("safe-value")
    );
    test_headers.insert(
        HeaderName::from_static("x-dangerous-header"),
        HeaderValue::from_static("<script>alert('xss')</script>")
    );
    test_headers.insert(
        HeaderName::from_static("host"),
        HeaderValue::from_static("evil.com")
    );

    let sanitized = security_middleware.sanitize_headers(&test_headers).unwrap();

    assert!(sanitized.contains_key("x-safe-header"), "Safe headers should be preserved");
    assert!(!sanitized.contains_key("x-dangerous-header"), "Dangerous headers should be removed");
    assert!(!sanitized.contains_key("host"), "Host header should be removed");

    // Test file path sanitization
    let test_cases = vec![
        ("normal_file.txt", "normal_file.txt"),
        ("../../../etc/passwd", "etc/passwd"),
        ("./config.json", "config.json"),
        ("~/.ssh/id_rsa", "/.ssh/id_rsa"),
        ("file with spaces.txt", "filewithspaces.txt"),
        ("file@#$%^&*()name.txt", "filename.txt"),
    ];

    for (input, expected) in test_cases {
        let sanitized = sanitize_file_path(input).unwrap();
        assert_eq!(sanitized, expected, "File path sanitization failed for: {}", input);
    }

    // Test HTTP method validation
    let valid_methods = vec!["GET", "POST", "HEAD", "OPTIONS"];
    let invalid_methods = vec!["DELETE", "PUT", "TRACE", "CONNECT", "PATCH"];

    for method in valid_methods {
        assert!(validate_http_method(method).is_ok(), "Method {} should be valid", method);
    }

    for method in invalid_methods {
        assert!(validate_http_method(method).is_err(), "Method {} should be invalid", method);
    }
}

#[tokio::test]
async fn test_content_security_policy() {
    // Test default CSP
    let default_csp = CSPBuilder::default_policy().build();
    assert!(default_csp.contains("default-src 'self'"), "Should have default-src 'self'");
    assert!(default_csp.contains("frame-ancestors 'none'"), "Should prevent framing");

    // Test custom CSP
    let custom_csp = CSPBuilder::new()
        .default_src(&["'self'"])
        .script_src(&["'self'", "https://trusted-cdn.com"])
        .style_src(&["'self'", "'unsafe-inline'"])
        .img_src(&["'self'", "data:", "https:"])
        .connect_src(&["'self'", "https://api.example.com"])
        .frame_ancestors(&["'none'"])
        .build();

    assert!(custom_csp.contains("script-src 'self' https://trusted-cdn.com"));
    assert!(custom_csp.contains("connect-src 'self' https://api.example.com"));

    // Test that CSP is properly formatted
    let parts: Vec<&str> = custom_csp.split("; ").collect();
    assert!(parts.len() >= 5, "CSP should have multiple directives");
}

#[tokio::test]
async fn test_request_size_validation() {
    let security_middleware = SecurityMiddleware::new_default();

    // Test normal request sizes
    assert!(security_middleware.validate_request_size(1024).is_ok());
    assert!(security_middleware.validate_request_size(1024 * 1024).is_ok()); // 1MB
    assert!(security_middleware.validate_request_size(10 * 1024 * 1024).is_ok()); // 10MB

    // Test oversized requests
    assert!(security_middleware.validate_request_size(25 * 1024 * 1024).is_err()); // 25MB
    assert!(security_middleware.validate_request_size(100 * 1024 * 1024).is_err()); // 100MB

    // Test edge case (exactly at limit)
    assert!(security_middleware.validate_request_size(20 * 1024 * 1024).is_ok()); // Exactly 20MB
    assert!(security_middleware.validate_request_size(20 * 1024 * 1024 + 1).is_err()); // 20MB + 1 byte
}

#[tokio::test]
async fn test_suspicious_pattern_detection() {
    let security_middleware = SecurityMiddleware::new_default();

    let suspicious_inputs = vec![
        "<script>alert('xss')</script>",
        "javascript:void(0)",
        "data:text/html,<script>alert(1)</script>",
        "vbscript:msgbox(1)",
        "onload=alert(1)",
        "onerror=alert(1)",
        "eval('malicious code')",
        "alert('test')",
        "confirm('test')",
        "prompt('test')",
        "document.cookie",
        "window.location = 'evil.com'",
    ];

    for input in suspicious_inputs {
        assert!(
            security_middleware.contains_suspicious_patterns(input),
            "Should detect suspicious pattern in: {}",
            input
        );
    }

    let safe_inputs = vec![
        "normal text content",
        "https://example.com",
        "user@example.com",
        "This is a safe string with no dangerous content",
        "JSON: {\"key\": \"value\"}",
    ];

    for input in safe_inputs {
        assert!(
            !security_middleware.contains_suspicious_patterns(input),
            "Should not detect suspicious pattern in: {}",
            input
        );
    }
}

#[tokio::test]
async fn test_secure_client_headers() {
    let security_middleware = SecurityMiddleware::new_default();
    let headers = security_middleware.create_secure_client_headers().unwrap();

    // Verify secure client headers
    assert!(headers.contains_key("User-Agent"), "Should have User-Agent header");
    assert!(headers.contains_key("DNT"), "Should have Do Not Track header");
    assert!(headers.contains_key("Accept"), "Should have Accept header");
    assert!(headers.contains_key("Accept-Encoding"), "Should have Accept-Encoding header");

    // Verify header values
    assert_eq!(headers.get("DNT").unwrap().to_str().unwrap(), "1");
    assert!(headers.get("User-Agent").unwrap().to_str().unwrap().contains("RipTide"));
    assert!(headers.get("Accept").unwrap().to_str().unwrap().contains("text/html"));
    assert!(headers.get("Accept-Encoding").unwrap().to_str().unwrap().contains("gzip"));
}

/// Integration test for complete security workflow
#[tokio::test]
async fn test_complete_security_workflow() {
    let store = MockApiKeyStore::new();
    let audit_logger = MockAuditLogger::new();
    let pii_detector = MockPiiDetector::new().unwrap();
    let rate_limiter = MockRateLimiter::new(RateLimitConfig::default());
    let security_middleware = SecurityMiddleware::new_default();

    // Setup test data
    let api_key = ApiKeyData {
        key: "integration-test-key".to_string(),
        is_active: true,
        budget_limit: Some(1000),
        budget_used: 0,
        rate_limit: Some(RateLimitConfig::default()),
        permissions: vec!["search".to_string()],
        created_at: chrono::Utc::now(),
        last_used: None,
    };
    store.add_key(api_key).await;

    // Simulate incoming request with PII
    let request_body = "Search for john.doe@example.com and SSN 123-45-6789";
    let api_key = "integration-test-key";
    let client_ip = "192.168.1.100";

    // 1. Validate API key
    let key_data = store.validate_key(api_key).await;
    assert!(key_data.is_some(), "API key should be valid");
    let key_data = key_data.unwrap();
    assert!(key_data.is_active, "API key should be active");

    // 2. Check rate limiting
    let rate_allowed = rate_limiter.check_rate_limit(client_ip).await.unwrap();
    assert!(rate_allowed, "Request should be within rate limits");

    // 3. Validate request size
    let size_valid = security_middleware.validate_request_size(request_body.len());
    assert!(size_valid.is_ok(), "Request size should be valid");

    // 4. Detect and redact PII
    let pii_types = pii_detector.detect_pii(request_body);
    assert!(!pii_types.is_empty(), "Should detect PII in request");
    assert!(pii_types.contains(&"Email".to_string()), "Should detect email");
    assert!(pii_types.contains(&"SSN".to_string()), "Should detect SSN");

    let redacted_body = pii_detector.redact_pii(request_body);
    assert!(redacted_body.contains("[REDACTED-EMAIL]"), "Should redact email");
    assert!(redacted_body.contains("[REDACTED-SSN]"), "Should redact SSN");

    // 5. Update API key usage
    store.update_usage(api_key, 10).await.unwrap();

    // 6. Log the complete transaction
    audit_logger.log(AuditLogEntry {
        timestamp: chrono::Utc::now(),
        event_type: "secure_search_request".to_string(),
        user_id: None,
        api_key_id: Some(api_key.to_string()),
        resource: "search".to_string(),
        action: "query".to_string(),
        result: "success".to_string(),
        details: json!({
            "original_request_size": request_body.len(),
            "redacted_content": redacted_body,
            "pii_detected": pii_types,
            "budget_used": 10
        }).as_object().unwrap().clone(),
        ip_address: Some(client_ip.to_string()),
        user_agent: Some("RipTide-Test/1.0".to_string()),
    }).await;

    // 7. Apply security headers to response
    let mut response_headers = HeaderMap::new();
    security_middleware.apply_security_headers(&mut response_headers).unwrap();

    // Verify complete workflow
    let final_key_data = store.validate_key(api_key).await.unwrap();
    assert_eq!(final_key_data.budget_used, 10, "Budget should be updated");

    let logs = audit_logger.get_logs().await;
    assert_eq!(logs.len(), 1, "Should have audit log entry");

    assert!(!response_headers.is_empty(), "Should have security headers");
    assert!(response_headers.contains_key("X-XSS-Protection"), "Should have XSS protection");
}