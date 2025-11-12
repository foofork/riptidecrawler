//! Security configuration and middleware integration for RipTide API
//!
//! This module provides centralized security configuration that integrates
//! riptide-security features with the existing tower-http middleware stack.

use anyhow::Result;
use riptide_security::{
    AuditConfig, AuditLogger, PiiRedactionMiddleware, SecurityConfig, SecurityMiddleware,
};
use std::sync::Arc;
use tracing::info;

/// Initialize security middleware with production-ready defaults
pub fn init_security_middleware() -> Result<Arc<SecurityMiddleware>> {
    let config = SecurityConfig {
        enable_cors: false, // We use tower-http CorsLayer instead
        cors_allowed_origins: vec![],
        enable_xss_protection: true,
        enable_content_type_protection: true,
        enable_frame_protection: true,
        enable_hsts: true,
        max_request_size: 20 * 1024 * 1024, // 20MB
        rate_limit: None, // We use custom rate_limit_middleware instead
    };

    let middleware = SecurityMiddleware::new(config)?;
    info!("Security middleware initialized with production settings");
    Ok(Arc::new(middleware))
}

/// Initialize PII redaction middleware for sensitive endpoints
pub fn init_pii_redaction() -> Arc<PiiRedactionMiddleware> {
    let middleware = PiiRedactionMiddleware::with_defaults();
    info!("PII redaction middleware initialized");
    Arc::new(middleware)
}

/// Initialize audit logger for security events
pub fn init_audit_logger() -> Result<Arc<AuditLogger>> {
    let config = AuditConfig {
        enabled: true,
        log_level: "info".to_string(),
        include_request_body: false, // Don't log potentially sensitive request bodies
        include_response_body: false,
        max_body_size: 1024,
    };

    let logger = AuditLogger::new(config)?;
    info!("Audit logger initialized for security events");
    Ok(Arc::new(logger))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_middleware_initialization() {
        let middleware = init_security_middleware();
        assert!(middleware.is_ok());
    }

    #[test]
    fn test_pii_redaction_initialization() {
        let _middleware = init_pii_redaction();
        // Test passes if no panic
    }

    #[test]
    fn test_audit_logger_initialization() {
        let logger = init_audit_logger();
        assert!(logger.is_ok());
    }
}
