//! Health Check Adapter for hexagonal architecture
//!
//! Adapts the concrete HealthChecker implementation to the HealthCheck port trait.

use async_trait::async_trait;
use riptide_types::ports::health::{HealthCheck, HealthStatus as PortHealthStatus};
use riptide_types::error::Result as RiptideResult;
use std::sync::Arc;

/// Adapter that implements the HealthCheck port trait for HealthChecker
pub struct HealthCheckAdapter {
    inner: Arc<crate::health::HealthChecker>,
    context: Arc<crate::context::ApplicationContext>,
}

impl HealthCheckAdapter {
    /// Create a new HealthCheckAdapter wrapping the concrete implementation
    pub fn new(
        checker: Arc<crate::health::HealthChecker>,
        context: Arc<crate::context::ApplicationContext>,
    ) -> Self {
        Self {
            inner: checker,
            context,
        }
    }
}

#[async_trait]
impl HealthCheck for HealthCheckAdapter {
    async fn check(&self) -> RiptideResult<PortHealthStatus> {
        let health = self.inner.check_health(&self.context).await;

        // Map from response health status to port health status
        if health.status == "healthy" {
            Ok(PortHealthStatus::Healthy)
        } else if health.status == "degraded" {
            Ok(PortHealthStatus::Degraded {
                reason: format!(
                    "System degraded: redis={}, extractor={}, http_client={}",
                    health.dependencies.redis.status,
                    health.dependencies.extractor.status,
                    health.dependencies.http_client.status
                ),
            })
        } else {
            Ok(PortHealthStatus::Unhealthy {
                error: format!("System unhealthy: {}", health.status),
            })
        }
    }

    fn name(&self) -> &str {
        "application_health"
    }

    fn description(&self) -> Option<&str> {
        Some("Comprehensive application health check including all dependencies")
    }
}
