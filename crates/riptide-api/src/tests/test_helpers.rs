//! Test helpers and builders for unit tests
//!
//! This module provides test fixtures and builders for creating test instances
//! of complex types that are difficult to construct manually in tests.

use crate::health::HealthChecker;
use crate::metrics::RipTideMetrics;
use crate::state::{AppConfig, AppState};
use anyhow::Result;
use riptide_config::ApiConfig;
use std::sync::Arc;

/// Test builder for AppState
pub struct AppStateBuilder {
    config: Option<AppConfig>,
    api_config: Option<ApiConfig>,
    metrics: Option<Arc<RipTideMetrics>>,
    health_checker: Option<Arc<HealthChecker>>,
}

impl AppStateBuilder {
    /// Create a new AppStateBuilder with defaults
    pub fn new() -> Self {
        Self {
            config: None,
            api_config: None,
            metrics: None,
            health_checker: None,
        }
    }

    /// Set custom AppConfig
    #[allow(dead_code)]
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set custom ApiConfig
    #[allow(dead_code)]
    pub fn with_api_config(mut self, api_config: RiptideApiConfig) -> Self {
        self.api_config = Some(api_config);
        self
    }

    /// Set custom metrics
    #[allow(dead_code)]
    pub fn with_metrics(mut self, metrics: Arc<RipTideMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Set custom health checker
    #[allow(dead_code)]
    pub fn with_health_checker(mut self, health_checker: Arc<HealthChecker>) -> Self {
        self.health_checker = Some(health_checker);
        self
    }

    /// Build the AppState
    ///
    /// This will use defaults for any values not explicitly set.
    /// Note: This requires Redis to be available for full functionality.
    pub async fn build(self) -> Result<AppState> {
        let config = self.config.unwrap_or_default();
        let api_config = self.api_config.unwrap_or_default();
        let metrics = self
            .metrics
            .unwrap_or_else(|| Arc::new(RipTideMetrics::new().expect("Failed to create metrics")));
        let health_checker = self
            .health_checker
            .unwrap_or_else(|| Arc::new(HealthChecker::new()));

        AppState::new_with_telemetry_and_api_config(
            config,
            api_config,
            metrics,
            health_checker,
            None,
        )
        .await
    }
}

impl Default for AppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires Redis connection"]
    async fn test_app_state_builder() {
        let state = AppStateBuilder::new()
            .build()
            .await
            .expect("Failed to build AppState");

        // Verify basic properties
        assert_eq!(state.config.max_concurrency, 16);
    }
}
