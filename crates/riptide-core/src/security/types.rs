//! Security module type definitions and common structures

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Tenant identification and context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TenantId(pub String);

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TenantId {
    fn from(s: String) -> Self {
        TenantId(s)
    }
}

impl From<&str> for TenantId {
    fn from(s: &str) -> Self {
        TenantId(s.to_string())
    }
}

/// Security context for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub tenant_id: TenantId,
    pub api_key_id: String,
    pub user_id: Option<String>,
    pub request_id: String,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub scopes: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl SecurityContext {
    pub fn new(tenant_id: TenantId, api_key_id: String, ip_address: String) -> Self {
        Self {
            tenant_id,
            api_key_id,
            user_id: None,
            request_id: Uuid::new_v4().to_string(),
            ip_address,
            user_agent: None,
            timestamp: Utc::now(),
            scopes: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Security event types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    ApiKeyCreated,
    ApiKeyRotated,
    ApiKeyRevoked,
    ApiKeyUsed,
    BudgetExceeded,
    BudgetWarning,
    RateLimitExceeded,
    UnauthorizedAccess,
    PiiDetected,
    PiiRedacted,
    SecurityViolation,
    AuditLogAccess,
    ConfigurationChange,
    SystemStartup,
    SystemShutdown,
}

impl fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SecurityEventType::ApiKeyCreated => "API_KEY_CREATED",
            SecurityEventType::ApiKeyRotated => "API_KEY_ROTATED",
            SecurityEventType::ApiKeyRevoked => "API_KEY_REVOKED",
            SecurityEventType::ApiKeyUsed => "API_KEY_USED",
            SecurityEventType::BudgetExceeded => "BUDGET_EXCEEDED",
            SecurityEventType::BudgetWarning => "BUDGET_WARNING",
            SecurityEventType::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            SecurityEventType::UnauthorizedAccess => "UNAUTHORIZED_ACCESS",
            SecurityEventType::PiiDetected => "PII_DETECTED",
            SecurityEventType::PiiRedacted => "PII_REDACTED",
            SecurityEventType::SecurityViolation => "SECURITY_VIOLATION",
            SecurityEventType::AuditLogAccess => "AUDIT_LOG_ACCESS",
            SecurityEventType::ConfigurationChange => "CONFIGURATION_CHANGE",
            SecurityEventType::SystemStartup => "SYSTEM_STARTUP",
            SecurityEventType::SystemShutdown => "SYSTEM_SHUTDOWN",
        };
        write!(f, "{}", s)
    }
}

/// Security event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SecuritySeverity::Low => "LOW",
            SecuritySeverity::Medium => "MEDIUM",
            SecuritySeverity::High => "HIGH",
            SecuritySeverity::Critical => "CRITICAL",
        };
        write!(f, "{}", s)
    }
}

/// Cost tracking and budget information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    pub tokens_used: u64,
    pub estimated_cost_usd: f64,
    pub model_name: String,
    pub operation_type: String,
    pub timestamp: DateTime<Utc>,
}

impl CostInfo {
    pub fn new(tokens_used: u64, estimated_cost_usd: f64, model_name: String, operation_type: String) -> Self {
        Self {
            tokens_used,
            estimated_cost_usd,
            model_name,
            operation_type,
            timestamp: Utc::now(),
        }
    }
}

/// PII detection and redaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiConfig {
    pub enable_email_detection: bool,
    pub enable_phone_detection: bool,
    pub enable_ssn_detection: bool,
    pub enable_credit_card_detection: bool,
    pub enable_custom_patterns: bool,
    pub custom_patterns: Vec<String>,
    pub redaction_method: RedactionMethod,
    pub preserve_format: bool,
}

impl Default for PiiConfig {
    fn default() -> Self {
        Self {
            enable_email_detection: true,
            enable_phone_detection: true,
            enable_ssn_detection: true,
            enable_credit_card_detection: true,
            enable_custom_patterns: false,
            custom_patterns: Vec::new(),
            redaction_method: RedactionMethod::Asterisk,
            preserve_format: true,
        }
    }
}

/// Methods for redacting PII
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedactionMethod {
    Asterisk,     // Replace with asterisks: ****
    Hash,         // Replace with hash: [REDACTED:HASH]
    Remove,       // Remove completely
    Placeholder,  // Replace with placeholder: [EMAIL], [PHONE], etc.
}

/// Rate limiting configuration per tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_allowance: u32,
    pub enable_adaptive_limits: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_allowance: 10,
            enable_adaptive_limits: false,
        }
    }
}

/// Budget limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimits {
    pub global_monthly_limit_usd: f64,
    pub per_job_limit_usd: f64,
    pub per_tenant_monthly_limit_usd: Option<f64>,
    pub warning_threshold_percent: f64,
    pub hard_stop_enabled: bool,
    pub grace_period_minutes: u32,
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            global_monthly_limit_usd: 2000.0,
            per_job_limit_usd: 10.0,
            per_tenant_monthly_limit_usd: None,
            warning_threshold_percent: 80.0,
            hard_stop_enabled: true,
            grace_period_minutes: 5,
        }
    }
}

/// Error types for security operations
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Invalid API key: {0}")]
    InvalidApiKey(String),
    
    #[error("API key expired: {0}")]
    ApiKeyExpired(String),
    
    #[error("Rate limit exceeded for tenant: {0}")]
    RateLimitExceeded(String),
    
    #[error("Budget limit exceeded: {0}")]
    BudgetLimitExceeded(String),
    
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    #[error("PII redaction failed: {0}")]
    PiiRedactionFailed(String),
    
    #[error("Audit log error: {0}")]
    AuditLogError(String),
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Unknown security error: {0}")]
    Unknown(String),
}

impl From<SecurityError> for anyhow::Error {
    fn from(err: SecurityError) -> Self {
        anyhow!(err.to_string())
    }
}

/// Result type alias for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;
