//! Security configuration and middleware integration for RipTide API
//!
//! This module provides centralized security configuration that integrates
//! riptide-security features with the existing tower-http middleware stack.

use anyhow::Result;
use riptide_security::{AuditLogger, SecurityMiddleware};
use std::sync::Arc;
use tracing::info;

/// Initialize security middleware with production-ready defaults
pub fn init_security_middleware() -> Result<Arc<SecurityMiddleware>> {
    // Use the with_defaults constructor which properly initializes all components
    let middleware = SecurityMiddleware::with_defaults()?;
    info!("Security middleware initialized with production settings");
    Ok(Arc::new(middleware))
}

/// Initialize audit logger for security events
pub fn init_audit_logger() -> Result<Arc<AuditLogger>> {
    // Use default configuration for audit logger
    let logger = AuditLogger::new(None)?;
    info!("Audit logger initialized for security events");
    Ok(Arc::new(logger))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_middleware_initialization() {
        let middleware = init_security_middleware();
        assert!(middleware.is_ok());
    }

    #[tokio::test]
    async fn test_audit_logger_initialization() {
        let logger = init_audit_logger();
        assert!(logger.is_ok());
    }
}
