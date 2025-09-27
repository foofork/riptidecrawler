# RipTide Security Layer Implementation

## Overview

This document describes the comprehensive security layer implemented for RipTide, providing enterprise-grade security features including API key management, budget enforcement, PII redaction, and audit logging.

## 🔐 Security Components

### 1. API Key Management System (`api_keys.rs`)

**Features:**
- Per-tenant API key isolation
- Cryptographically secure key generation (SHA-256 hashing)
- Configurable rate limiting per key
- Key rotation capabilities
- Automatic expiration and cleanup
- Comprehensive usage tracking

**Key Classes:**
- `ApiKeyManager`: Central management for all API keys
- `ApiKey`: Individual key with metadata and permissions
- `RateLimitConfig`: Configurable rate limiting per key

**Usage Example:**
```rust
use riptide_core::security::*;

let manager = ApiKeyManager::new();
let tenant_id = TenantId::from("acme-corp");

// Create API key with rate limits
let (api_key, raw_key) = manager.create_api_key(
    tenant_id,
    "Production Key".to_string(),
    Some("Main production API key".to_string()),
    vec!["read".to_string(), "write".to_string()],
    Some(RateLimitConfig {
        requests_per_minute: 100,
        requests_per_hour: 5000,
        requests_per_day: 50000,
        burst_allowance: 20,
        enable_adaptive_limits: true,
    }),
    None, // No expiration
).await?;

// Validate key and check rate limits
let (validated_key, security_context) = manager.validate_api_key(&raw_key).await?;
```

### 2. Budget Enforcement System (`budget.rs`)

**Features:**
- Global monthly budget limits ($2,000/month default)
- Per-job budget limits ($10/job default)
- Per-tenant budget tracking
- Real-time cost calculation for different models
- Circuit breaker pattern for budget violations
- Automatic budget warnings and hard stops

**Key Classes:**
- `BudgetManager`: Central budget enforcement
- `BudgetLimits`: Configurable budget constraints
- `CostInfo`: Token and cost tracking
- `BudgetCircuitBreaker`: Automatic protection against overspend

**Usage Example:**
```rust
let budget_limits = BudgetLimits {
    global_monthly_limit_usd: 2000.0,
    per_job_limit_usd: 10.0,
    per_tenant_monthly_limit_usd: Some(500.0),
    warning_threshold_percent: 80.0,
    hard_stop_enabled: true,
    grace_period_minutes: 5,
};

let budget_manager = BudgetManager::new(Some(budget_limits));

// Check if job can proceed within budget
let cost_info = budget_manager.check_budget_for_job(
    "job-123",
    &tenant_id,
    1000, // estimated tokens
    "gpt-4",
).await?;

// Record actual usage after completion
budget_manager.record_usage(
    "job-123",
    &tenant_id,
    actual_cost_info,
).await?;
```

### 3. PII Redaction System (`pii.rs`)

**Features:**
- Automatic detection of emails, phone numbers, SSNs, credit cards
- Custom pattern support with regex
- Multiple redaction methods (asterisk, hash, remove, placeholder)
- Format preservation options
- Comprehensive detection statistics

**Supported PII Types:**
- Email addresses
- Phone numbers (multiple formats)
- Social Security Numbers
- Credit card numbers (Visa, MasterCard, Amex, Discover)
- IP addresses
- URLs
- Custom patterns via regex

**Usage Example:**
```rust
let pii_config = PiiConfig {
    enable_email_detection: true,
    enable_phone_detection: true,
    enable_ssn_detection: true,
    enable_credit_card_detection: true,
    enable_custom_patterns: true,
    custom_patterns: vec![r"\bAPI-KEY-\w{16}\b".to_string()],
    redaction_method: RedactionMethod::Placeholder,
    preserve_format: true,
};

let redactor = PiiRedactor::new(Some(pii_config))?;

let text = "Contact john.doe@example.com or call 555-123-4567";
let result = redactor.redact_text(text)?;

println!("Original: {}", text);
println!("Redacted: {}", result.redacted_text);
println!("Detections: {:?}", result.detections);
```

### 4. Audit Logging System (`audit.rs`)

**Features:**
- Comprehensive security event logging
- Multiple output formats (JSON, CSV, Syslog)
- Configurable log rotation and retention
- Real-time alerting for security violations
- Compliance-ready audit trails
- Searchable audit history

**Event Types:**
- API key creation, rotation, revocation, usage
- Budget violations and warnings
- Rate limit violations
- PII detections
- Security violations
- System events

**Usage Example:**
```rust
let audit_config = AuditConfig {
    log_directory: PathBuf::from("./logs/security"),
    max_file_size_mb: 100,
    retention_days: 90,
    log_format: AuditLogFormat::Json,
    enable_real_time_alerts: true,
    ..Default::default()
};

let audit_logger = AuditLogger::new(Some(audit_config))?;

// Log API key usage
let entry = audit_logger.create_api_key_usage_entry(
    &security_context,
    AuditOutcome::Success,
    audit_details,
);
audit_logger.log_event(entry).await?;
```

### 5. Integrated Security Middleware (`middleware.rs`)

**Features:**
- Single point of integration for all security components
- Automatic request/response processing
- Error handling and fallback strategies
- Security health monitoring
- Performance optimizations

**Usage Example:**
```rust
// Create integrated security middleware
let security_middleware = SecurityMiddleware::with_defaults()?;

// Process incoming request
let request_context = security_middleware.process_request(
    &api_key,
    "job-id",
    estimated_tokens,
    "gpt-4",
    request_payload,
    source_ip,
    user_agent,
).await?;

// Process response
let redacted_response = security_middleware.process_response(
    "job-id",
    &request_context.security_context,
    actual_tokens,
    actual_cost,
    "gpt-4",
    Some(response_payload),
    duration_ms,
).await?;
```

## 🛡️ Security Architecture

### Security Flow

1. **Authentication**: API key validation and rate limiting
2. **Authorization**: Budget check and job approval
3. **Input Processing**: PII detection and redaction
4. **Execution**: Protected LLM operation
5. **Output Processing**: Response PII redaction
6. **Audit**: Comprehensive logging of all activities

### Multi-Tenant Isolation

- Each tenant has isolated API keys and budgets
- Tenant-specific rate limits and permissions
- Isolated audit logs and usage tracking
- Configurable per-tenant budget limits

### Circuit Breaker Protection

- Automatic protection against budget overruns
- Configurable grace periods for budget violations
- Exponential backoff for repeated violations
- Health monitoring and automatic recovery

## 📊 Configuration

### Environment Variables

```bash
# Budget Configuration
RIPTIDE_GLOBAL_MONTHLY_BUDGET=2000.0
RIPTIDE_PER_JOB_BUDGET=10.0
RIPTIDE_BUDGET_WARNING_THRESHOLD=80.0

# Audit Configuration
RIPTIDE_AUDIT_LOG_DIR=/var/log/riptide/security
RIPTIDE_AUDIT_RETENTION_DAYS=90
RIPTIDE_AUDIT_MAX_FILE_SIZE_MB=100

# PII Configuration
RIPTIDE_PII_REDACTION_METHOD=placeholder
RIPTIDE_PII_PRESERVE_FORMAT=true
```

### Programmatic Configuration

```rust
use riptide_core::security::*;

// Custom budget limits
let budget_limits = BudgetLimits {
    global_monthly_limit_usd: 5000.0,
    per_job_limit_usd: 25.0,
    per_tenant_monthly_limit_usd: Some(1000.0),
    warning_threshold_percent: 85.0,
    hard_stop_enabled: true,
    grace_period_minutes: 10,
};

// Custom PII configuration
let pii_config = PiiConfig {
    enable_email_detection: true,
    enable_phone_detection: true,
    enable_ssn_detection: true,
    enable_credit_card_detection: true,
    enable_custom_patterns: true,
    custom_patterns: vec![
        r"\bCUST-\d{8}\b".to_string(),
        r"\bORDER-[A-Z0-9]{12}\b".to_string(),
    ],
    redaction_method: RedactionMethod::Hash,
    preserve_format: false,
};

// Custom audit configuration
let audit_config = AuditConfig {
    log_directory: PathBuf::from("/var/log/riptide"),
    max_file_size_mb: 200,
    retention_days: 365,
    log_format: AuditLogFormat::Json,
    enable_real_time_alerts: true,
    alert_thresholds: AlertThresholds {
        failed_auth_attempts_per_minute: 5,
        rate_limit_violations_per_hour: 25,
        budget_violations_per_hour: 3,
        critical_events_per_hour: 1,
        pii_detections_per_hour: 50,
    },
    ..Default::default()
};
```

## 🔍 Monitoring and Alerts

### Security Health Monitoring

```rust
let health = security_middleware.get_security_health().await;

println!("Security Health:");
println!("  API Keys Active: {}", health.api_keys_active);
println!("  Budget Health OK: {}", health.budget_health_ok);
println!("  PII Redaction Active: {}", health.pii_redaction_active);
println!("  Audit Logging Active: {}", health.audit_logging_active);
println!("  Budget Usage: {:.1}%", health.budget_usage_percentage);
println!("  Remaining Budget: ${:.2}", health.remaining_budget_usd);
```

### Real-time Alerts

The system provides real-time alerts for:
- Failed authentication attempts
- Rate limit violations
- Budget threshold breaches
- Critical security events
- Excessive PII detections

### Audit Queries

```rust
// Search audit logs
let events = audit_logger.search_logs(
    start_time,
    end_time,
    Some(vec![SecurityEventType::ApiKeyUsed]),
    Some(&tenant_id),
    Some(SecuritySeverity::High),
).await?;

// Export compliance reports
let exported_count = audit_logger.export_logs(
    start_time,
    end_time,
    AuditLogFormat::Csv,
    Path::new("./compliance_report.csv"),
).await?;
```

## 🧪 Testing

### Unit Tests

Each security component includes comprehensive unit tests:

```bash
cd /workspaces/eventmesh/crates/riptide-core
cargo test security:: --lib
```

### Integration Tests

Comprehensive integration tests are available in `src/security/tests/mod.rs`:

```bash
cargo test security::tests:: --lib
```

### Example Application

Run the security integration example:

```bash
cargo run --example security_integration
```

## 🔧 Maintenance

### Automated Cleanup

```rust
// Cleanup expired API keys
let expired_count = api_key_manager.cleanup_expired_keys().await;

// Cleanup old audit logs
let removed_logs = audit_logger.cleanup_old_logs().await?;

// Reset monthly budgets (typically scheduled)
budget_manager.reset_monthly_usage().await?;
```

### Key Rotation

```rust
// Rotate API key
let new_key = api_key_manager.rotate_api_key(&key_id).await?;

// Update client with new key
// Old key is automatically disabled
```

### Backup and Recovery

```rust
// Backup audit logs
audit_logger.export_logs(
    month_start,
    month_end,
    AuditLogFormat::Json,
    &backup_path,
).await?;

// Backup memory stores
budget_manager.backup_memory_stores(&backup_path).await?;
```

## 🔒 Security Best Practices

### API Key Management
- Generate keys with sufficient entropy (64+ characters)
- Use secure hashing (SHA-256 minimum)
- Implement automatic key rotation
- Monitor key usage patterns
- Revoke compromised keys immediately

### Budget Protection
- Set conservative budget limits initially
- Monitor usage patterns and adjust limits
- Implement alerts before hard limits
- Use circuit breakers to prevent runaway costs
- Review budget usage regularly

### PII Protection
- Enable all PII detection types
- Test custom patterns thoroughly
- Use hash redaction for audit trails
- Implement format preservation where needed
- Regular review of detection patterns

### Audit Compliance
- Enable comprehensive audit logging
- Implement log rotation and retention
- Export regular compliance reports
- Monitor for security violations
- Implement real-time alerting

## 📈 Performance Considerations

### Optimizations
- Efficient regex compilation and caching
- Async processing for all I/O operations
- Memory-efficient data structures
- Batch processing for bulk operations
- Connection pooling for database operations

### Scalability
- Horizontal scaling support
- Distributed rate limiting
- Sharded audit logs
- Load balancing for security checks
- Caching for frequent operations

## 🎯 Implementation Status

✅ **COMPLETED**:
- [x] API Key Management System (SEC-001)
- [x] Budget Enforcement System (SEC-002)
- [x] PII Redaction System (SEC-003)
- [x] Audit Logging System (SEC-004)
- [x] Integrated Security Middleware
- [x] Comprehensive Test Suite
- [x] Documentation and Examples
- [x] Performance Optimizations

## 🚀 Next Steps

1. **Integration Testing**: Test with real LLM providers
2. **Performance Tuning**: Optimize for high-throughput scenarios
3. **Monitoring Dashboard**: Create web interface for security monitoring
4. **Advanced Analytics**: Implement ML-based anomaly detection
5. **Compliance Features**: Add SOC2, GDPR, HIPAA compliance tools

---

**Implementation Date**: September 27, 2025  
**Version**: 1.0.0  
**Status**: ✅ Complete  
**Next Review**: October 15, 2025
