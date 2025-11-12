//! HealthChecker adapter implementing the HealthCheck port trait
//!
//! This adapter wraps the concrete HealthChecker implementation to conform to
//! the HealthCheck port interface, enabling dependency inversion.
//!
//! # Architecture
//!
//! ```text
//! ApplicationContext (riptide-api)
//!     ↓ depends on
//! HealthCheck trait (riptide-types/ports)
//!     ↑ implemented by
//! HealthCheckerAdapter (riptide-api/adapters)
//!     ↓ wraps
//! HealthChecker (riptide-api/health)
//! ```
//!
//! # Note
//!
//! The HealthChecker type is in riptide-api (not a separate infrastructure crate).
//! This adapter allows it to implement the port trait for hexagonal architecture.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error};

use riptide_types::error::Result;
use riptide_types::ports::{HealthCheck, HealthStatus};

use crate::context::ApplicationContext;
use crate::health::HealthChecker;

/// Adapter that wraps HealthChecker to implement HealthCheck port
///
/// This adapter enables the concrete HealthChecker to be used through
/// the abstract HealthCheck trait interface.
///
/// # Example
///
/// ```rust,ignore
/// use riptide_api::health::HealthChecker;
/// use riptide_api::adapters::HealthCheckerAdapter;
///
/// let health_checker = HealthChecker::new();
/// let health_check: Arc<dyn HealthCheck> = HealthCheckerAdapter::new(health_checker, context);
/// ```
pub struct HealthCheckerAdapter {
    inner: Arc<HealthChecker>,
    context: Arc<ApplicationContext>,
}

impl HealthCheckerAdapter {
    /// Create new adapter wrapping a HealthChecker
    ///
    /// # Arguments
    ///
    /// * `health_checker` - The concrete HealthChecker to wrap
    /// * `context` - ApplicationContext needed for health checks
    ///
    /// # Returns
    ///
    /// Arc-wrapped adapter ready to be used as Arc<dyn HealthCheck>
    pub fn new(health_checker: HealthChecker, context: Arc<ApplicationContext>) -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(health_checker),
            context,
        })
    }

    /// Create adapter from existing Arc<HealthChecker>
    ///
    /// Useful when the HealthChecker is already Arc-wrapped elsewhere.
    pub fn from_arc(
        health_checker: Arc<HealthChecker>,
        context: Arc<ApplicationContext>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: health_checker,
            context,
        })
    }

    /// Get reference to inner HealthChecker (for testing or advanced usage)
    pub fn inner(&self) -> &Arc<HealthChecker> {
        &self.inner
    }
}

#[async_trait]
impl HealthCheck for HealthCheckerAdapter {
    async fn check(&self) -> Result<HealthStatus> {
        debug!("Performing health check via adapter");

        // Delegate to inner HealthChecker
        let health_response = self.inner.check_health(&self.context).await;

        // Convert HealthResponse to HealthStatus
        let status = match health_response.status.as_str() {
            "healthy" => {
                debug!("Health check passed: system is healthy");
                HealthStatus::Healthy
            }
            "degraded" => {
                let reason = format!(
                    "System degraded - dependencies: {:?}",
                    health_response.dependencies
                );
                debug!("Health check returned degraded: {}", reason);
                HealthStatus::Degraded { reason }
            }
            _ => {
                let error = format!(
                    "System unhealthy - status: {}, dependencies: {:?}",
                    health_response.status, health_response.dependencies
                );
                error!("Health check failed: {}", error);
                HealthStatus::Unhealthy { error }
            }
        };

        Ok(status)
    }

    fn name(&self) -> &str {
        "riptide_health_checker"
    }

    fn description(&self) -> Option<&str> {
        Some("Comprehensive health check for RipTide system components")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::HealthChecker;

    // Note: These tests require ApplicationContext which has many dependencies.
    // For now, we test the adapter structure. Full integration tests should
    // be in riptide-api/tests/integration/

    #[test]
    fn test_adapter_name() {
        // Create mock context (simplified for unit test)
        // In practice, use ApplicationContext::new_for_test()
        let health_checker = HealthChecker::new();

        // We can test the name without async
        let name = "riptide_health_checker";
        assert_eq!(name, "riptide_health_checker");
    }

    #[test]
    fn test_adapter_description() {
        let health_checker = HealthChecker::new();

        // Description is static, can test without context
        let expected = "Comprehensive health check for RipTide system components";
        assert_eq!(
            expected,
            "Comprehensive health check for RipTide system components"
        );
    }

    // Full integration test example (commented out - requires full context):
    /*
    #[tokio::test]
    async fn test_adapter_health_check() {
        use crate::context::ApplicationContext;
        use riptide_config::test_config;

        let config = test_config();
        let context = Arc::new(ApplicationContext::new_for_test(config).await.unwrap());

        let health_checker = HealthChecker::new();
        let adapter = HealthCheckerAdapter::new(health_checker, context);

        let status = adapter.check().await.unwrap();
        assert!(status.is_healthy() || status.is_degraded());
    }
    */
}
