//! Comprehensive test suite for RipTide security components
//!
//! Tests integration between API keys, budget enforcement, PII redaction,
//! and audit logging to ensure complete security functionality.

use crate::security::*;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;

/// Helper function to create test security middleware
async fn create_test_security_middleware() -> SecurityMiddleware {
    SecurityMiddleware::with_defaults().unwrap()
}

/// Helper function to create test audit config
fn create_test_audit_config() -> AuditConfig {
    let temp_dir = TempDir::new().unwrap();
    AuditConfig {
        log_directory: temp_dir.path().to_path_buf(),
        max_file_size_mb: 1,
        retention_days: 7,
        ..Default::default()
    }
}

/// Helper function to create test budget limits
fn create_test_budget_limits() -> BudgetLimits {
    BudgetLimits {
        global_monthly_limit_usd: 100.0,
        per_job_limit_usd: 5.0,
        per_tenant_monthly_limit_usd: Some(50.0),
        warning_threshold_percent: 80.0,
        hard_stop_enabled: true,
        grace_period_minutes: 1,
    }
}

#[tokio::test]
async fn test_complete_security_pipeline() {
    let middleware = create_test_security_middleware().await;
    let tenant_id = TenantId::from("integration-test-tenant");
    
    // Create API key
    let (api_key, raw_key) = middleware.get_api_key_manager()
        .create_api_key(
            tenant_id.clone(),
            "Integration Test Key".to_string(),
            Some("Test key for integration testing".to_string()),
            vec!["read".to_string(), "write".to_string()],
            None,
            None,
        )
        .await
        .unwrap();
    
    assert_eq!(api_key.tenant_id, tenant_id);
    assert!(raw_key.starts_with("rpt_"));
    
    // Test request processing with PII
    let request_payload = "Process this: Contact john.doe@example.com or call 123-456-7890";
    
    let result = middleware.process_request(
        &raw_key,
        "integration-job-1",
        1000,
        "gpt-3.5-turbo",
        request_payload,
        "192.168.1.100",
        Some("RipTide-Test/1.0"),
    ).await;
    
    assert!(result.is_ok());
    let request_context = result.unwrap();
    
    // Verify PII was redacted
    assert!(request_context.pii_detected);
    assert!(!request_context.redacted_payload.contains("john.doe@example.com"));
    assert!(!request_context.redacted_payload.contains("123-456-7890"));
    
    // Verify cost info is present
    assert!(request_context.cost_info.is_some());
    let cost_info = request_context.cost_info.unwrap();
    assert!(cost_info.estimated_cost_usd > 0.0);
    
    // Test response processing
    let response_payload = "Here's the result with contact info user@test.com";
    
    let response_result = middleware.process_response(
        "integration-job-1",
        &request_context.security_context,
        800, // actual tokens
        cost_info.estimated_cost_usd * 0.8, // actual cost
        "gpt-3.5-turbo",
        Some(response_payload),
        150, // duration in ms
    ).await;
    
    assert!(response_result.is_ok());
    let redacted_response = response_result.unwrap();
    assert!(!redacted_response.contains("user@test.com"));
    
    // Verify security health
    let health = middleware.get_security_health().await;
    assert!(health.api_keys_active);
    assert!(health.budget_health_ok);
    assert!(health.pii_redaction_active);
    assert!(health.audit_logging_active);
}

#[tokio::test]
async fn test_budget_enforcement_limits() {
    let api_key_manager = Arc::new(ApiKeyManager::new());
    let budget_manager = Arc::new(BudgetManager::new(Some(create_test_budget_limits())));
    let pii_middleware = Arc::new(PiiRedactionMiddleware::new(None).unwrap());
    let audit_logger = Arc::new(AuditLogger::new(Some(create_test_audit_config())).unwrap());
    
    let middleware = SecurityMiddleware::new(
        api_key_manager.clone(),
        budget_manager.clone(),
        pii_middleware,
        audit_logger,
    );
    
    let tenant_id = TenantId::from("budget-test-tenant");
    
    // Create API key
    let (_, raw_key) = api_key_manager
        .create_api_key(
            tenant_id.clone(),
            "Budget Test Key".to_string(),
            None,
            vec![],
            None,
            None,
        )
        .await
        .unwrap();
    
    // Test per-job limit (should fail with high token count)
    let result = middleware.process_request(
        &raw_key,
        "expensive-job",
        100000, // Very high token count
        "gpt-4", // Expensive model
        "test payload",
        "127.0.0.1",
        None,
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("per-job limit"));
    
    // Test successful request within limits
    let result = middleware.process_request(
        &raw_key,
        "normal-job",
        1000, // Reasonable token count
        "gpt-3.5-turbo", // Cheaper model
        "test payload",
        "127.0.0.1",
        None,
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rate_limiting_enforcement() {
    let rate_limits = RateLimitConfig {
        requests_per_minute: 2,
        requests_per_hour: 10,
        requests_per_day: 100,
        burst_allowance: 0,
        enable_adaptive_limits: false,
    };
    
    let api_key_manager = Arc::new(ApiKeyManager::new());
    let tenant_id = TenantId::from("rate-limit-test-tenant");
    
    // Create API key with strict rate limits
    let (_, raw_key) = api_key_manager
        .create_api_key(
            tenant_id.clone(),
            "Rate Limited Key".to_string(),
            None,
            vec![],
            Some(rate_limits),
            None,
        )
        .await
        .unwrap();
    
    // First two requests should succeed
    assert!(api_key_manager.validate_api_key(&raw_key).await.is_ok());
    assert!(api_key_manager.validate_api_key(&raw_key).await.is_ok());
    
    // Third request should fail due to rate limiting
    let result = api_key_manager.validate_api_key(&raw_key).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SecurityError::RateLimitExceeded(_)));
}

#[tokio::test]
async fn test_api_key_lifecycle() {
    let manager = ApiKeyManager::new();
    let tenant_id = TenantId::from("lifecycle-test-tenant");
    
    // Create API key
    let (api_key, raw_key) = manager
        .create_api_key(
            tenant_id.clone(),
            "Lifecycle Test Key".to_string(),
            Some("Testing key lifecycle".to_string()),
            vec!["read".to_string()],
            None,
            Some(Utc::now() + Duration::hours(1)), // Expires in 1 hour
        )
        .await
        .unwrap();
    
    // Key should be valid initially
    let validation_result = manager.validate_api_key(&raw_key).await;
    assert!(validation_result.is_ok());
    
    // Rotate the key
    let new_raw_key = manager.rotate_api_key(&api_key.id).await.unwrap();
    assert_ne!(raw_key, new_raw_key);
    
    // Old key should no longer work
    assert!(manager.validate_api_key(&raw_key).await.is_err());
    
    // New key should work
    assert!(manager.validate_api_key(&new_raw_key).await.is_ok());
    
    // Revoke the key
    manager.revoke_api_key(&api_key.id).await.unwrap();
    
    // Both keys should now fail
    assert!(manager.validate_api_key(&raw_key).await.is_err());
    assert!(manager.validate_api_key(&new_raw_key).await.is_err());
}

#[tokio::test]
async fn test_pii_detection_comprehensive() {
    let redactor = PiiRedactor::new(None).unwrap();
    
    let test_text = r#"
    Here's my personal info:
    Email: john.doe@company.com
    Phone: (555) 123-4567
    SSN: 123-45-6789
    Credit Card: 4532 1234 5678 9012
    IP: 192.168.1.1
    URL: https://secret.internal.com/api/key
    "#;
    
    let result = redactor.redact_text(test_text).unwrap();
    
    // Verify all PII was detected and redacted
    assert!(result.detections.len() >= 6); // At least 6 different PII types
    assert!(!result.redacted_text.contains("john.doe@company.com"));
    assert!(!result.redacted_text.contains("(555) 123-4567"));
    assert!(!result.redacted_text.contains("123-45-6789"));
    assert!(!result.redacted_text.contains("4532 1234 5678 9012"));
    assert!(!result.redacted_text.contains("192.168.1.1"));
    assert!(!result.redacted_text.contains("https://secret.internal.com"));
    
    // Verify statistics
    assert!(result.stats.total_detections >= 6);
    assert!(result.stats.detections_by_type.contains_key(&PiiType::Email));
    assert!(result.stats.detections_by_type.contains_key(&PiiType::Phone));
    assert!(result.stats.detections_by_type.contains_key(&PiiType::SSN));
    assert!(result.stats.detections_by_type.contains_key(&PiiType::CreditCard));
}

#[tokio::test]
async fn test_audit_logging_comprehensive() {
    let audit_logger = AuditLogger::new(Some(create_test_audit_config())).unwrap();
    let tenant_id = TenantId::from("audit-test-tenant");
    
    // Create various audit entries
    let security_context = SecurityContext::new(
        tenant_id.clone(),
        "test-api-key".to_string(),
        "127.0.0.1".to_string(),
    );
    
    // API key usage
    let api_usage_entry = audit_logger.create_api_key_usage_entry(
        &security_context,
        AuditOutcome::Success,
        AuditDetails {
            description: "API key used successfully".to_string(),
            error_message: None,
            request_payload: None,
            response_payload: None,
            duration_ms: Some(100),
            bytes_processed: Some(1024),
            cost_usd: Some(0.01),
            tokens_used: Some(500),
            model_name: Some("gpt-3.5-turbo".to_string()),
            rate_limit_info: None,
            pii_redacted: false,
            pii_detections: None,
        },
    );
    
    audit_logger.log_event(api_usage_entry).await.unwrap();
    
    // Budget event
    let cost_info = CostInfo::new(
        1000,
        0.05,
        "gpt-4".to_string(),
        "llm_request".to_string(),
    );
    
    let budget_entry = audit_logger.create_budget_event_entry(
        SecurityEventType::BudgetWarning,
        &tenant_id,
        &cost_info,
        AuditDetails {
            description: "Budget warning threshold reached".to_string(),
            error_message: None,
            request_payload: None,
            response_payload: None,
            duration_ms: None,
            bytes_processed: None,
            cost_usd: Some(cost_info.estimated_cost_usd),
            tokens_used: Some(cost_info.tokens_used),
            model_name: Some(cost_info.model_name.clone()),
            rate_limit_info: None,
            pii_redacted: false,
            pii_detections: None,
        },
    );
    
    audit_logger.log_event(budget_entry).await.unwrap();
    
    // PII detection event
    let pii_entry = audit_logger.create_pii_detection_entry(
        &tenant_id,
        3,
        AuditDetails {
            description: "PII detected in request".to_string(),
            error_message: None,
            request_payload: None,
            response_payload: None,
            duration_ms: Some(50),
            bytes_processed: Some(512),
            cost_usd: None,
            tokens_used: None,
            model_name: None,
            rate_limit_info: None,
            pii_redacted: true,
            pii_detections: Some(3),
        },
    );
    
    audit_logger.log_event(pii_entry).await.unwrap();
    
    // Verify statistics
    let stats = audit_logger.get_stats().await;
    assert_eq!(stats.total_events, 3);
    assert!(stats.events_by_type.contains_key(&SecurityEventType::ApiKeyUsed.to_string()));
    assert!(stats.events_by_type.contains_key(&SecurityEventType::BudgetWarning.to_string()));
    assert!(stats.events_by_type.contains_key(&SecurityEventType::PiiDetected.to_string()));
}

#[tokio::test]
async fn test_security_error_handling() {
    let middleware = create_test_security_middleware().await;
    
    // Test invalid API key
    let result = middleware.process_request(
        "invalid_key",
        "test-job",
        1000,
        "gpt-3.5-turbo",
        "test payload",
        "127.0.0.1",
        None,
    ).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SecurityError::InvalidApiKey(_) => {},
        _ => panic!("Expected InvalidApiKey error"),
    }
    
    // Test expired API key
    let tenant_id = TenantId::from("expired-test-tenant");
    let expired_time = Utc::now() - Duration::minutes(1);
    
    let (_, expired_key) = middleware.get_api_key_manager()
        .create_api_key(
            tenant_id,
            "Expired Key".to_string(),
            None,
            vec![],
            None,
            Some(expired_time),
        )
        .await
        .unwrap();
    
    let result = middleware.process_request(
        &expired_key,
        "test-job",
        1000,
        "gpt-3.5-turbo",
        "test payload",
        "127.0.0.1",
        None,
    ).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SecurityError::ApiKeyExpired(_) => {},
        _ => panic!("Expected ApiKeyExpired error"),
    }
}

#[tokio::test]
async fn test_multi_tenant_isolation() {
    let manager = ApiKeyManager::new();
    
    // Create keys for different tenants
    let tenant1 = TenantId::from("tenant-1");
    let tenant2 = TenantId::from("tenant-2");
    
    let (key1, raw_key1) = manager
        .create_api_key(
            tenant1.clone(),
            "Tenant 1 Key".to_string(),
            None,
            vec!["read".to_string()],
            None,
            None,
        )
        .await
        .unwrap();
    
    let (key2, raw_key2) = manager
        .create_api_key(
            tenant2.clone(),
            "Tenant 2 Key".to_string(),
            None,
            vec!["write".to_string()],
            None,
            None,
        )
        .await
        .unwrap();
    
    // Verify tenant isolation
    assert_eq!(key1.tenant_id, tenant1);
    assert_eq!(key2.tenant_id, tenant2);
    assert_ne!(key1.tenant_id, key2.tenant_id);
    
    // Verify different scopes
    assert_eq!(key1.scopes, vec!["read"]);
    assert_eq!(key2.scopes, vec!["write"]);
    
    // Get tenant keys
    let tenant1_keys = manager.get_tenant_keys(&tenant1).await;
    let tenant2_keys = manager.get_tenant_keys(&tenant2).await;
    
    assert_eq!(tenant1_keys.len(), 1);
    assert_eq!(tenant2_keys.len(), 1);
    assert_eq!(tenant1_keys[0].id, key1.id);
    assert_eq!(tenant2_keys[0].id, key2.id);
}

#[tokio::test]
async fn test_security_configuration_validation() {
    // Test PII configuration
    let pii_config = PiiConfig {
        enable_email_detection: true,
        enable_phone_detection: false,
        enable_ssn_detection: true,
        enable_credit_card_detection: false,
        enable_custom_patterns: true,
        custom_patterns: vec![r"\bCONF-\d{4}\b".to_string()],
        redaction_method: RedactionMethod::Hash,
        preserve_format: false,
    };
    
    let redactor = PiiRedactor::new(Some(pii_config)).unwrap();
    
    let test_text = "Email: test@example.com, Phone: 123-456-7890, SSN: 123-45-6789, Conf: CONF-1234";
    let result = redactor.redact_text(test_text).unwrap();
    
    // Should detect email, SSN, and custom pattern, but not phone
    assert!(result.detections.iter().any(|d| d.pii_type == PiiType::Email));
    assert!(result.detections.iter().any(|d| d.pii_type == PiiType::SSN));
    assert!(result.detections.iter().any(|d| matches!(d.pii_type, PiiType::Custom(_))));
    assert!(!result.detections.iter().any(|d| d.pii_type == PiiType::Phone));
}

use std::sync::Arc;
