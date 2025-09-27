//! RipTide Security Integration Example
//!
//! Demonstrates how to integrate and use the comprehensive security layer
//! including API keys, budget enforcement, PII redaction, and audit logging.

use riptide_core::security::*;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::init();
    
    println!("🔐 RipTide Security Integration Example");
    println!("=====================================\n");
    
    // Step 1: Create security middleware with custom configurations
    println!("📋 Step 1: Initializing Security Components");
    
    // Custom budget limits
    let budget_limits = BudgetLimits {
        global_monthly_limit_usd: 1000.0,
        per_job_limit_usd: 5.0,
        per_tenant_monthly_limit_usd: Some(200.0),
        warning_threshold_percent: 80.0,
        hard_stop_enabled: true,
        grace_period_minutes: 5,
    };
    
    // Custom PII configuration
    let pii_config = PiiConfig {
        enable_email_detection: true,
        enable_phone_detection: true,
        enable_ssn_detection: true,
        enable_credit_card_detection: true,
        enable_custom_patterns: true,
        custom_patterns: vec![
            r"\bAPI-KEY-\w{16}\b".to_string(),
            r"\bTOKEN-\d{8}\b".to_string(),
        ],
        redaction_method: RedactionMethod::Placeholder,
        preserve_format: true,
    };
    
    // Custom audit configuration
    let audit_config = AuditConfig {
        log_directory: std::path::PathBuf::from("./logs/security"),
        max_file_size_mb: 50,
        retention_days: 90,
        rotation_interval_hours: 24,
        enable_compression: true,
        log_format: AuditLogFormat::Json,
        enable_real_time_alerts: true,
        include_request_bodies: false, // For privacy
        include_response_bodies: false,
        max_payload_size_kb: 32,
        ..Default::default()
    };
    
    // Initialize components
    let api_key_manager = Arc::new(ApiKeyManager::new());
    let budget_manager = Arc::new(BudgetManager::new(Some(budget_limits)));
    let pii_middleware = Arc::new(PiiRedactionMiddleware::new(Some(pii_config))?);
    let audit_logger = Arc::new(AuditLogger::new(Some(audit_config))?);
    
    let security_middleware = SecurityMiddleware::new(
        api_key_manager.clone(),
        budget_manager.clone(),
        pii_middleware.clone(),
        audit_logger.clone(),
    );
    
    println!("✅ Security middleware initialized\n");
    
    // Step 2: Create tenant and API keys
    println!("📋 Step 2: Creating Tenant and API Keys");
    
    let tenant_id = TenantId::from("acme-corp");
    
    // Create different types of API keys
    let (production_key, prod_raw_key) = api_key_manager
        .create_api_key(
            tenant_id.clone(),
            "Production API Key".to_string(),
            Some("Main production key for ACME Corp".to_string()),
            vec!["read".to_string(), "write".to_string(), "admin".to_string()],
            Some(RateLimitConfig {
                requests_per_minute: 100,
                requests_per_hour: 5000,
                requests_per_day: 50000,
                burst_allowance: 20,
                enable_adaptive_limits: true,
            }),
            None, // No expiration
        )
        .await?;
    
    let (development_key, dev_raw_key) = api_key_manager
        .create_api_key(
            tenant_id.clone(),
            "Development API Key".to_string(),
            Some("Development and testing key".to_string()),
            vec!["read".to_string(), "write".to_string()],
            Some(RateLimitConfig {
                requests_per_minute: 20,
                requests_per_hour: 500,
                requests_per_day: 2000,
                burst_allowance: 5,
                enable_adaptive_limits: false,
            }),
            Some(chrono::Utc::now() + chrono::Duration::days(30)), // Expires in 30 days
        )
        .await?;
    
    println!("✅ Created production key: {} ({})", production_key.name, production_key.id);
    println!("✅ Created development key: {} ({})\n", development_key.name, development_key.id);
    
    // Step 3: Simulate API requests with various scenarios
    println!("📋 Step 3: Simulating API Requests");
    
    // Scenario 1: Normal request with PII
    println!("\n🔍 Scenario 1: Normal request with PII data");
    let request_with_pii = r#"{
        "task": "process_user_data",
        "user_info": {
            "name": "John Doe",
            "email": "john.doe@acmecorp.com",
            "phone": "(555) 123-4567",
            "ssn": "123-45-6789",
            "api_key": "API-KEY-ABCD1234EFGH5678"
        },
        "instructions": "Analyze customer sentiment from support tickets"
    }"#;
    
    let result = security_middleware.process_request(
        &prod_raw_key,
        "job-001",
        2000,
        "gpt-4",
        request_with_pii,
        "203.0.113.45",
        Some("ACME-Corp-App/2.1.0"),
    ).await;
    
    match result {
        Ok(context) => {
            println!("✅ Request processed successfully");
            println!("   - Tenant: {}", context.security_context.tenant_id);
            println!("   - PII detected: {}", context.pii_detected);
            println!("   - Estimated cost: ${:.4}", context.cost_info.as_ref().map(|c| c.estimated_cost_usd).unwrap_or(0.0));
            println!("   - Payload redacted: {} chars -> {} chars", 
                request_with_pii.len(), context.redacted_payload.len());
            
            // Simulate processing and response
            let response_with_pii = "Based on the analysis, customer john.doe@acmecorp.com shows positive sentiment. Contact at (555) 123-4567 for follow-up.";
            
            let response_result = security_middleware.process_response(
                "job-001",
                &context.security_context,
                1800, // actual tokens
                context.cost_info.unwrap().estimated_cost_usd * 0.9,
                "gpt-4",
                Some(response_with_pii),
                250, // duration ms
            ).await?;
            
            println!("   - Response redacted: {} chars -> {} chars", 
                response_with_pii.len(), response_result.len());
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }
    
    // Scenario 2: Request exceeding budget limits
    println!("\n🔍 Scenario 2: Request exceeding per-job budget limit");
    let expensive_request = "Process this massive dataset with complex analysis";
    
    let result = security_middleware.process_request(
        &prod_raw_key,
        "job-002",
        50000, // Very high token count
        "gpt-4", // Expensive model
        expensive_request,
        "203.0.113.45",
        Some("ACME-Corp-App/2.1.0"),
    ).await;
    
    match result {
        Ok(_) => println!("✅ Expensive request processed (unexpected)"),
        Err(e) => println!("❌ Request blocked by budget enforcement: {}", e),
    }
    
    // Scenario 3: Rate limiting test
    println!("\n🔍 Scenario 3: Testing rate limits with development key");
    
    for i in 1..=25 {
        let result = api_key_manager.validate_api_key(&dev_raw_key).await;
        match result {
            Ok(_) => println!("   Request {}: ✅ Allowed", i),
            Err(SecurityError::RateLimitExceeded(_)) => {
                println!("   Request {}: ❌ Rate limited", i);
                break;
            }
            Err(e) => {
                println!("   Request {}: ❌ Error: {}", i, e);
                break;
            }
        }
    }
    
    // Step 4: Security monitoring and health
    println!("\n📋 Step 4: Security Monitoring");
    
    let health = security_middleware.get_security_health().await;
    println!("\n🏥 Security Health Status:");
    println!("   - API Keys Active: {}", health.api_keys_active);
    println!("   - Budget Health OK: {}", health.budget_health_ok);
    println!("   - PII Redaction Active: {}", health.pii_redaction_active);
    println!("   - Audit Logging Active: {}", health.audit_logging_active);
    println!("   - Total Requests Today: {}", health.total_requests_today);
    println!("   - Security Violations Today: {}", health.security_violations_today);
    println!("   - Budget Usage: {:.1}%", health.budget_usage_percentage);
    println!("   - Remaining Budget: ${:.2}", health.remaining_budget_usd);
    
    // Budget details
    let budget_health = budget_manager.check_budget_health().await;
    println!("\n💰 Budget Details:");
    println!("   - Current Usage: ${:.2}", budget_health.current_usage_usd);
    println!("   - Monthly Limit: ${:.2}", budget_health.monthly_limit_usd);
    println!("   - Projected Monthly Cost: ${:.2}", budget_health.projected_monthly_cost);
    println!("   - Days Remaining: {}", budget_health.days_remaining_in_month);
    println!("   - Warning Threshold Reached: {}", budget_health.warning_threshold_reached);
    
    // Audit statistics
    let audit_stats = audit_logger.get_stats().await;
    println!("\n📊 Audit Statistics:");
    println!("   - Total Events: {}", audit_stats.total_events);
    println!("   - Current Log File Size: {} bytes", audit_stats.current_file_size);
    
    for (severity, count) in &audit_stats.events_by_severity {
        println!("   - {} Events: {}", severity, count);
    }
    
    for (event_type, count) in &audit_stats.events_by_type {
        println!("   - {} Events: {}", event_type, count);
    }
    
    // Step 5: API key management operations
    println!("\n📋 Step 5: API Key Management");
    
    // List tenant keys
    let tenant_keys = api_key_manager.get_tenant_keys(&tenant_id).await;
    println!("\n🔑 API Keys for tenant '{}':", tenant_id);
    for key in &tenant_keys {
        println!("   - {}: {} (Active: {}, Usage: {})", 
            key.id, key.name, key.is_active, key.usage_count);
        println!("     Scopes: {:?}", key.scopes);
        println!("     Created: {}", key.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
        if let Some(expires_at) = key.expires_at {
            println!("     Expires: {}", expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        if let Some(last_used) = key.last_used_at {
            println!("     Last Used: {}", last_used.format("%Y-%m-%d %H:%M:%S UTC"));
        }
    }
    
    // Rotate production key
    println!("\n🔄 Rotating production API key...");
    let new_prod_key = api_key_manager.rotate_api_key(&production_key.id).await?;
    println!("✅ Production key rotated successfully");
    println!("   Old key disabled, new key: {}...", &new_prod_key[..20]);
    
    // Test old key (should fail)
    let old_key_test = api_key_manager.validate_api_key(&prod_raw_key).await;
    match old_key_test {
        Ok(_) => println!("❌ Old key still works (unexpected)"),
        Err(_) => println!("✅ Old key correctly disabled"),
    }
    
    // Test new key (should work)
    let new_key_test = api_key_manager.validate_api_key(&new_prod_key).await;
    match new_key_test {
        Ok(_) => println!("✅ New key works correctly"),
        Err(e) => println!("❌ New key failed: {}", e),
    }
    
    // Step 6: PII redaction testing
    println!("\n📋 Step 6: PII Redaction Testing");
    
    let test_documents = vec![
        "Customer John Doe (john.doe@example.com) called about his order. Phone: 555-123-4567",
        "Payment failed for card 4532-1234-5678-9012. Please contact customer service.",
        "SSN verification needed: 123-45-6789. API key: API-KEY-1234567890ABCDEF",
        "Server logs at https://internal.acme.com/logs?token=TOKEN-12345678",
    ];
    
    for (i, document) in test_documents.iter().enumerate() {
        println!("\n🔍 Document {}:", i + 1);
        println!("   Original: {}", document);
        
        let redacted = pii_middleware.redact_payload(document)?;
        println!("   Redacted: {}", redacted);
        
        let contains_pii = pii_middleware.get_redactor().contains_pii(document);
        println!("   Contains PII: {}", contains_pii);
    }
    
    // Step 7: Cleanup and maintenance
    println!("\n📋 Step 7: Maintenance Operations");
    
    // Cleanup expired keys
    let expired_count = api_key_manager.cleanup_expired_keys().await;
    println!("\n🧹 Cleaned up {} expired API keys", expired_count);
    
    // Cleanup old audit logs (demo - in production this would be on a schedule)
    let removed_logs = audit_logger.cleanup_old_logs().await?;
    println!("🧹 Cleaned up {} old audit log files", removed_logs);
    
    println!("\n🎉 Security integration example completed successfully!");
    println!("\n📚 Key Features Demonstrated:");
    println!("   ✅ Multi-tenant API key management with scopes and rate limiting");
    println!("   ✅ Comprehensive budget enforcement with per-job and monthly limits");
    println!("   ✅ Advanced PII detection and redaction with custom patterns");
    println!("   ✅ Detailed audit logging with multiple output formats");
    println!("   ✅ Integrated security middleware with error handling");
    println!("   ✅ Real-time security monitoring and health checks");
    println!("   ✅ Automated maintenance and cleanup operations");
    
    Ok(())
}
