//! Health monitoring system for LLM providers
//!
//! This module provides comprehensive health monitoring capabilities including:
//! - Real-time health checks
//! - Performance metrics collection
//! - Automatic failover triggers
//! - Health status reporting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::{LlmProvider, Result};

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub degraded_threshold: f64, // Error rate percentage
    pub critical_threshold: f64, // Error rate percentage
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            failure_threshold: 3,
            success_threshold: 2,
            degraded_threshold: 10.0,
            critical_threshold: 50.0,
        }
    }
}

/// Health status levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Critical,
    Unavailable,
}

/// Detailed health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub provider_name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: HealthLevel,
    pub response_time: Duration,
    pub success: bool,
    pub error_message: Option<String>,
    pub metrics: ProviderMetrics,
}

/// Provider performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time: Duration,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub error_rate: f64,
    pub requests_per_minute: f64,
    pub tokens_per_second: f64,
    pub cost_per_request: f64,
    pub uptime_percentage: f64,
    pub last_request_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for ProviderMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::from_millis(0),
            min_response_time: Duration::from_millis(u64::MAX),
            max_response_time: Duration::from_millis(0),
            error_rate: 0.0,
            requests_per_minute: 0.0,
            tokens_per_second: 0.0,
            cost_per_request: 0.0,
            uptime_percentage: 100.0,
            last_request_time: None,
        }
    }
}

/// Health monitoring events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthEvent {
    StatusChanged {
        provider_name: String,
        old_status: HealthLevel,
        new_status: HealthLevel,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ProviderAdded {
        provider_name: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ProviderRemoved {
        provider_name: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    HealthCheckFailed {
        provider_name: String,
        error: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    MetricsUpdated {
        provider_name: String,
        metrics: ProviderMetrics,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Health monitoring system
pub struct HealthMonitor {
    providers: Arc<RwLock<HashMap<String, Arc<dyn LlmProvider>>>>,
    health_status: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    metrics: Arc<RwLock<HashMap<String, ProviderMetrics>>>,
    config: HealthCheckConfig,
    event_tx: mpsc::UnboundedSender<HealthEvent>,
    event_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<HealthEvent>>>>,
    running: Arc<RwLock<bool>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthCheckConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Add a provider to monitor
    pub async fn add_provider(&self, name: String, provider: Arc<dyn LlmProvider>) {
        {
            let mut providers = self.providers.write().await;
            providers.insert(name.clone(), provider);
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.insert(name.clone(), ProviderMetrics::default());
        }

        let event = HealthEvent::ProviderAdded {
            provider_name: name.clone(),
            timestamp: chrono::Utc::now(),
        };

        let _ = self.event_tx.send(event);

        info!("Added provider {} to health monitoring", name);
    }

    /// Remove a provider from monitoring
    pub async fn remove_provider(&self, name: &str) {
        {
            let mut providers = self.providers.write().await;
            providers.remove(name);
        }

        {
            let mut health_status = self.health_status.write().await;
            health_status.remove(name);
        }

        {
            let mut metrics = self.metrics.write().await;
            metrics.remove(name);
        }

        let event = HealthEvent::ProviderRemoved {
            provider_name: name.to_string(),
            timestamp: chrono::Utc::now(),
        };

        let _ = self.event_tx.send(event);

        info!("Removed provider {} from health monitoring", name);
    }

    /// Start the health monitoring loop
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        let providers = Arc::clone(&self.providers);
        let health_status = Arc::clone(&self.health_status);
        let metrics = Arc::clone(&self.metrics);
        let config = self.config.clone();
        let event_tx = self.event_tx.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            let mut interval = interval(config.interval);

            while *running.read().await {
                interval.tick().await;

                let provider_list: Vec<(String, Arc<dyn LlmProvider>)> =
                    { providers.read().await.clone().into_iter().collect() };

                for (name, provider) in provider_list {
                    let health_check_result =
                        Self::perform_health_check(&name, &provider, &config).await;

                    // Update health status
                    let old_status = {
                        let status_map = health_status.read().await;
                        status_map.get(&name).map(|s| s.level.clone())
                    };

                    {
                        let mut status_map = health_status.write().await;
                        status_map.insert(name.clone(), health_check_result.clone());
                    }

                    // Check if status changed
                    if let Some(old_level) = old_status {
                        if old_level != health_check_result.level {
                            let event = HealthEvent::StatusChanged {
                                provider_name: name.clone(),
                                old_status: old_level,
                                new_status: health_check_result.level.clone(),
                                timestamp: chrono::Utc::now(),
                            };
                            let _ = event_tx.send(event);
                        }
                    }

                    // Update metrics
                    {
                        let mut metrics_map = metrics.write().await;
                        if let Some(provider_metrics) = metrics_map.get_mut(&name) {
                            Self::update_metrics(provider_metrics, &health_check_result);

                            let event = HealthEvent::MetricsUpdated {
                                provider_name: name.clone(),
                                metrics: provider_metrics.clone(),
                                timestamp: chrono::Utc::now(),
                            };
                            let _ = event_tx.send(event);
                        }
                    }

                    if !health_check_result.success {
                        let event = HealthEvent::HealthCheckFailed {
                            provider_name: name,
                            error: health_check_result
                                .error_message
                                .unwrap_or_else(|| "Unknown error".to_string()),
                            timestamp: chrono::Utc::now(),
                        };
                        let _ = event_tx.send(event);
                    }
                }
            }
        });

        info!(
            "Health monitor started with interval: {:?}",
            self.config.interval
        );
        Ok(())
    }

    /// Stop the health monitoring loop
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Health monitor stopped");
    }

    /// Get current health status for all providers
    pub async fn get_health_status(&self) -> HashMap<String, HealthCheckResult> {
        self.health_status.read().await.clone()
    }

    /// Get health status for a specific provider
    pub async fn get_provider_health(&self, name: &str) -> Option<HealthCheckResult> {
        self.health_status.read().await.get(name).cloned()
    }

    /// Get metrics for all providers
    pub async fn get_metrics(&self) -> HashMap<String, ProviderMetrics> {
        self.metrics.read().await.clone()
    }

    /// Get metrics for a specific provider
    pub async fn get_provider_metrics(&self, name: &str) -> Option<ProviderMetrics> {
        self.metrics.read().await.get(name).cloned()
    }

    /// Get healthy providers only
    pub async fn get_healthy_providers(&self) -> Vec<String> {
        let status_map = self.health_status.read().await;
        status_map
            .iter()
            .filter(|(_, result)| result.level == HealthLevel::Healthy)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get providers by health level
    pub async fn get_providers_by_health(&self, level: HealthLevel) -> Vec<String> {
        let status_map = self.health_status.read().await;
        status_map
            .iter()
            .filter(|(_, result)| result.level == level)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Take the event receiver for processing health events
    pub async fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<HealthEvent>> {
        self.event_rx.write().await.take()
    }

    /// Perform a manual health check on a specific provider
    pub async fn check_provider(&self, name: &str) -> Option<HealthCheckResult> {
        let provider = {
            let providers = self.providers.read().await;
            providers.get(name).cloned()
        };

        if let Some(provider) = provider {
            Some(Self::perform_health_check(name, &provider, &self.config).await)
        } else {
            None
        }
    }

    /// Perform health check on a provider
    async fn perform_health_check(
        name: &str,
        provider: &Arc<dyn LlmProvider>,
        config: &HealthCheckConfig,
    ) -> HealthCheckResult {
        let start_time = Instant::now();
        let timestamp = chrono::Utc::now();

        debug!("Performing health check for provider: {}", name);

        // Perform the actual health check with timeout
        let health_check_future = provider.health_check();
        let timeout_future = tokio::time::timeout(config.timeout, health_check_future);

        let (success, error_message) = match timeout_future.await {
            Ok(Ok(_)) => {
                debug!("Health check succeeded for provider: {}", name);
                (true, None)
            }
            Ok(Err(e)) => {
                warn!("Health check failed for provider {}: {}", name, e);
                (false, Some(e.to_string()))
            }
            Err(_) => {
                warn!("Health check timed out for provider: {}", name);
                (false, Some("Health check timeout".to_string()))
            }
        };

        let response_time = start_time.elapsed();

        // Determine health level based on success and other factors
        let level = if success {
            HealthLevel::Healthy
        } else {
            HealthLevel::Unavailable
        };

        HealthCheckResult {
            provider_name: name.to_string(),
            timestamp,
            level,
            response_time,
            success,
            error_message,
            metrics: ProviderMetrics::default(), // This would be populated with actual metrics
        }
    }

    /// Update provider metrics based on health check result
    fn update_metrics(metrics: &mut ProviderMetrics, result: &HealthCheckResult) {
        metrics.total_requests += 1;

        if result.success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        // Update response times
        if result.response_time < metrics.min_response_time
            || metrics.min_response_time == Duration::from_millis(u64::MAX)
        {
            metrics.min_response_time = result.response_time;
        }
        if result.response_time > metrics.max_response_time {
            metrics.max_response_time = result.response_time;
        }

        // Update average response time
        let total_time = metrics.avg_response_time.as_millis() as f64
            * (metrics.total_requests - 1) as f64
            + result.response_time.as_millis() as f64;
        metrics.avg_response_time =
            Duration::from_millis((total_time / metrics.total_requests as f64) as u64);

        // Calculate error rate
        metrics.error_rate =
            (metrics.failed_requests as f64 / metrics.total_requests as f64) * 100.0;

        // Update uptime percentage
        metrics.uptime_percentage =
            (metrics.successful_requests as f64 / metrics.total_requests as f64) * 100.0;

        metrics.last_request_time = Some(result.timestamp);
    }
}

/// Health monitor builder for easy configuration
pub struct HealthMonitorBuilder {
    config: HealthCheckConfig,
}

impl HealthMonitorBuilder {
    pub fn new() -> Self {
        Self {
            config: HealthCheckConfig::default(),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.config.interval = interval;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.config.failure_threshold = threshold;
        self
    }

    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.config.success_threshold = threshold;
        self
    }

    pub fn with_degraded_threshold(mut self, threshold: f64) -> Self {
        self.config.degraded_threshold = threshold;
        self
    }

    pub fn with_critical_threshold(mut self, threshold: f64) -> Self {
        self.config.critical_threshold = threshold;
        self
    }

    pub fn build(self) -> HealthMonitor {
        HealthMonitor::new(self.config)
    }
}

impl Default for HealthMonitorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_provider::MockLlmProvider;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new(HealthCheckConfig::default());
        assert_eq!(monitor.get_health_status().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_remove_provider() {
        let monitor = HealthMonitor::new(HealthCheckConfig::default());
        let provider = Arc::new(MockLlmProvider::new());

        monitor.add_provider("test".to_string(), provider).await;
        assert_eq!(monitor.get_health_status().await.len(), 0); // No health checks run yet

        monitor.remove_provider("test").await;
        assert_eq!(monitor.get_health_status().await.len(), 0);
    }

    #[tokio::test]
    async fn test_health_monitor_builder() {
        let monitor = HealthMonitorBuilder::new()
            .with_interval(Duration::from_secs(10))
            .with_timeout(Duration::from_secs(5))
            .with_failure_threshold(5)
            .build();

        assert_eq!(monitor.config.interval, Duration::from_secs(10));
        assert_eq!(monitor.config.timeout, Duration::from_secs(5));
        assert_eq!(monitor.config.failure_threshold, 5);
    }
}
