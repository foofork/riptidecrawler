//! Health check port definition
//!
//! This module defines the abstract health check interfaces for monitoring
//! system component health and overall service availability.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;

/// Health status of a component
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Component is fully operational
    Healthy,
    /// Component is operational but degraded
    Degraded { reason: String },
    /// Component is not operational
    Unhealthy { error: String },
}

impl HealthStatus {
    /// Checks if the status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Checks if the status is degraded
    pub fn is_degraded(&self) -> bool {
        matches!(self, HealthStatus::Degraded { .. })
    }

    /// Checks if the status is unhealthy
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy { .. })
    }

    /// Gets a human-readable description
    pub fn description(&self) -> String {
        match self {
            HealthStatus::Healthy => "Healthy".to_string(),
            HealthStatus::Degraded { reason } => format!("Degraded: {}", reason),
            HealthStatus::Unhealthy { error } => format!("Unhealthy: {}", error),
        }
    }
}

/// Health check port interface
///
/// Implementations should perform actual health checks (e.g., database ping,
/// cache connectivity, external service availability).
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Performs the health check
    async fn check(&self) -> Result<HealthStatus>;

    /// Returns the name of this health check
    fn name(&self) -> &str;

    /// Returns optional description of what this check verifies
    fn description(&self) -> Option<&str> {
        None
    }
}

/// Health registry for managing multiple health checks
#[async_trait]
pub trait HealthRegistry: Send + Sync {
    /// Registers a new health check
    async fn register(&mut self, check: Arc<dyn HealthCheck>);

    /// Removes a health check by name
    async fn unregister(&mut self, name: &str) -> bool;

    /// Runs all registered health checks
    async fn check_all(&self) -> HashMap<String, HealthStatus>;

    /// Checks if all components are healthy
    async fn is_healthy(&self) -> bool {
        self.check_all()
            .await
            .values()
            .all(|status| status.is_healthy())
    }

    /// Gets the overall system health status
    async fn overall_status(&self) -> HealthStatus {
        let checks = self.check_all().await;

        if checks.is_empty() {
            return HealthStatus::Unhealthy {
                error: "No health checks registered".to_string(),
            };
        }

        let unhealthy: Vec<_> = checks
            .iter()
            .filter(|(_, status)| status.is_unhealthy())
            .collect();

        if !unhealthy.is_empty() {
            return HealthStatus::Unhealthy {
                error: format!(
                    "{} components unhealthy: {}",
                    unhealthy.len(),
                    unhealthy
                        .iter()
                        .map(|(name, _)| name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            };
        }

        let degraded: Vec<_> = checks
            .iter()
            .filter(|(_, status)| status.is_degraded())
            .collect();

        if !degraded.is_empty() {
            return HealthStatus::Degraded {
                reason: format!(
                    "{} components degraded: {}",
                    degraded.len(),
                    degraded
                        .iter()
                        .map(|(name, _)| name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            };
        }

        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    struct MockHealthCheck {
        name: String,
        status: HealthStatus,
    }

    #[async_trait]
    impl HealthCheck for MockHealthCheck {
        async fn check(&self) -> Result<HealthStatus> {
            Ok(self.status.clone())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_health_status() {
        let healthy = HealthStatus::Healthy;
        assert!(healthy.is_healthy());
        assert!(!healthy.is_degraded());
        assert!(!healthy.is_unhealthy());

        let degraded = HealthStatus::Degraded {
            reason: "High latency".to_string(),
        };
        assert!(!degraded.is_healthy());
        assert!(degraded.is_degraded());
        assert!(!degraded.is_unhealthy());

        let unhealthy = HealthStatus::Unhealthy {
            error: "Connection failed".to_string(),
        };
        assert!(!unhealthy.is_healthy());
        assert!(!unhealthy.is_degraded());
        assert!(unhealthy.is_unhealthy());
    }

    #[test]
    fn test_health_status_description() {
        assert_eq!(HealthStatus::Healthy.description(), "Healthy");
        assert_eq!(
            HealthStatus::Degraded {
                reason: "Test".to_string()
            }
            .description(),
            "Degraded: Test"
        );
        assert_eq!(
            HealthStatus::Unhealthy {
                error: "Failed".to_string()
            }
            .description(),
            "Unhealthy: Failed"
        );
    }
}
