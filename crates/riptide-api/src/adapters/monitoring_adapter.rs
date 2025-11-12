//! Monitoring System Adapter for hexagonal architecture
//!
//! Adapts the concrete MonitoringSystem to monitoring port traits.

use riptide_types::ports::monitoring::MonitoringSystem as MonitoringPort;
use riptide_types::error::Result as RiptideResult;
use std::sync::Arc;

/// Monitoring backend trait for dependency inversion
pub trait MonitoringBackend: Send + Sync {
    /// Calculate current health score
    fn health_score(&self) -> impl std::future::Future<Output = RiptideResult<f32>> + Send;

    /// Get current system status
    fn status(&self) -> impl std::future::Future<Output = String> + Send;

    /// Check if monitoring is healthy
    fn is_healthy(&self) -> impl std::future::Future<Output = bool> + Send;
}

/// Adapter that implements MonitoringBackend for MonitoringSystem
pub struct MonitoringAdapter {
    inner: Arc<crate::context::MonitoringSystem>,
}

impl MonitoringAdapter {
    /// Create a new MonitoringAdapter wrapping the concrete implementation
    pub fn new(system: Arc<crate::context::MonitoringSystem>) -> Self {
        Self { inner: system }
    }
}

impl MonitoringBackend for MonitoringAdapter {
    async fn health_score(&self) -> RiptideResult<f32> {
        self.inner.calculate_health_score().await
    }

    async fn status(&self) -> String {
        match self.inner.generate_performance_report().await {
            Ok(report) => report.health_summary,
            Err(e) => format!("Error getting status: {}", e),
        }
    }

    async fn is_healthy(&self) -> bool {
        match self.health_score().await {
            Ok(score) => score > 0.7,
            Err(_) => false,
        }
    }
}

impl MonitoringPort for MonitoringAdapter {
    fn health_score(&self) -> impl std::future::Future<Output = RiptideResult<f32>> + Send {
        MonitoringBackend::health_score(self)
    }

    fn status(&self) -> impl std::future::Future<Output = String> + Send {
        MonitoringBackend::status(self)
    }
}
