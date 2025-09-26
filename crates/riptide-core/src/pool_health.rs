use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

use crate::component::{PerformanceMetrics, ExtractorConfig};
use crate::instance_pool::AdvancedInstancePool;

/// Pool health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolHealthStatus {
    /// Overall health status
    pub status: HealthLevel,
    /// Available instances in pool
    pub available_instances: usize,
    /// Active instances currently processing
    pub active_instances: usize,
    /// Maximum pool size
    pub max_instances: usize,
    /// Pool utilization percentage
    pub utilization_percent: f64,
    /// Average semaphore wait time in milliseconds
    pub avg_semaphore_wait_ms: f64,
    /// Circuit breaker status
    pub circuit_breaker_status: String,
    /// Total extractions performed
    pub total_extractions: u64,
    /// Success rate percentage
    pub success_rate_percent: f64,
    /// Fallback usage percentage
    pub fallback_rate_percent: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryHealthStats,
    /// Last health check timestamp
    #[serde(skip)]
    pub last_check: Option<Instant>,
    /// Health trend over time
    pub trend: HealthTrend,
}

/// Memory-related health statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHealthStats {
    /// Current WASM memory pages in use
    pub wasm_memory_pages: usize,
    /// Peak WASM memory pages used
    pub peak_memory_pages: usize,
    /// Number of memory growth failures
    pub grow_failures: u64,
    /// Memory pressure level
    pub memory_pressure: MemoryPressureLevel,
}

/// Overall health levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthLevel {
    /// All systems operating normally
    Healthy,
    /// Some degradation but still functional
    Degraded,
    /// Significant issues affecting performance
    Unhealthy,
    /// Critical failure state
    Critical,
}

/// Memory pressure levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryPressureLevel {
    /// Normal memory usage
    Low,
    /// Moderate memory pressure
    Medium,
    /// High memory pressure
    High,
    /// Critical memory pressure
    Critical,
}

/// Health trend indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthTrend {
    /// Performance is improving
    Improving,
    /// Performance is stable
    Stable,
    /// Performance is degrading
    Degrading,
    /// Not enough data for trend analysis
    Unknown,
}

/// Pool health monitor with automated diagnostics
pub struct PoolHealthMonitor {
    /// Pool to monitor
    pool: Arc<AdvancedInstancePool>,
    /// Configuration settings
    config: ExtractorConfig,
    /// Health status history for trend analysis
    health_history: Arc<Mutex<Vec<PoolHealthStatus>>>,
    /// Monitoring interval
    check_interval: Duration,
}

impl PoolHealthMonitor {
    /// Create new health monitor
    pub fn new(
        pool: Arc<AdvancedInstancePool>,
        config: ExtractorConfig,
        check_interval: Duration,
    ) -> Self {
        Self {
            pool,
            config,
            health_history: Arc::new(Mutex::new(Vec::new())),
            check_interval,
        }
    }

    /// Start continuous health monitoring
    pub async fn start_monitoring(self: Arc<Self>) -> Result<()> {
        info!(
            check_interval_ms = self.check_interval.as_millis(),
            "Starting pool health monitoring"
        );

        let mut interval_timer = interval(self.check_interval);

        loop {
            interval_timer.tick().await;

            // Perform health check with timeout
            match timeout(Duration::from_secs(5), self.perform_health_check()).await {
                Ok(Ok(status)) => {
                    self.process_health_status(status).await;
                }
                Ok(Err(e)) => {
                    error!(error = %e, "Health check failed");
                }
                Err(_) => {
                    error!("Health check timed out");
                }
            }
        }
    }

    /// Perform comprehensive health check
    async fn perform_health_check(&self) -> Result<PoolHealthStatus> {
        debug!("Performing pool health check");

        // Get pool status
        let (available, active, max_size) = self.pool.get_pool_status();
        let utilization_percent = if max_size > 0 {
            (active as f64 / max_size as f64) * 100.0
        } else {
            0.0
        };

        // Get performance metrics
        let metrics = self.pool.get_metrics();

        // Calculate success rate
        let success_rate_percent = if metrics.total_extractions > 0 {
            (metrics.successful_extractions as f64 / metrics.total_extractions as f64) * 100.0
        } else {
            100.0 // Assume healthy if no extractions yet
        };

        // Calculate fallback rate
        let fallback_rate_percent = if metrics.total_extractions > 0 {
            (metrics.fallback_extractions as f64 / metrics.total_extractions as f64) * 100.0
        } else {
            0.0
        };

        // Determine memory pressure
        let memory_pressure = self.determine_memory_pressure(&metrics);

        // Determine circuit breaker status
        let circuit_breaker_status = if metrics.circuit_breaker_trips > 0 {
            "TRIPPED".to_string()
        } else {
            "CLOSED".to_string()
        };

        // Calculate overall health level
        let health_level = self.calculate_health_level(
            utilization_percent,
            success_rate_percent,
            fallback_rate_percent,
            &memory_pressure,
            &metrics,
        );

        // Calculate trend
        let trend = self.calculate_health_trend(&health_level).await;

        let status = PoolHealthStatus {
            status: health_level,
            available_instances: available,
            active_instances: active,
            max_instances: max_size,
            utilization_percent,
            avg_semaphore_wait_ms: metrics.semaphore_wait_time_ms as f64,
            circuit_breaker_status,
            total_extractions: metrics.total_extractions,
            success_rate_percent,
            fallback_rate_percent,
            memory_stats: MemoryHealthStats {
                wasm_memory_pages: metrics.wasm_memory_pages as usize,
                peak_memory_pages: metrics.wasm_peak_memory_pages as usize,
                grow_failures: metrics.wasm_grow_failed_total,
                memory_pressure,
            },
            last_check: Some(Instant::now()),
            trend,
        };

        debug!(
            status = ?status.status,
            utilization = status.utilization_percent,
            success_rate = status.success_rate_percent,
            "Health check completed"
        );

        Ok(status)
    }

    /// Process health status and take actions if needed
    async fn process_health_status(&self, status: PoolHealthStatus) {
        // Store in history for trend analysis
        {
            let mut history = self.health_history.lock().unwrap();
            history.push(status.clone());
            // Keep only last 100 entries
            if history.len() > 100 {
                history.remove(0);
            }
        }

        // Log health status based on level
        match status.status {
            HealthLevel::Healthy => {
                debug!(
                    utilization = status.utilization_percent,
                    success_rate = status.success_rate_percent,
                    "Pool health: HEALTHY"
                );
            }
            HealthLevel::Degraded => {
                warn!(
                    utilization = status.utilization_percent,
                    success_rate = status.success_rate_percent,
                    fallback_rate = status.fallback_rate_percent,
                    "Pool health: DEGRADED"
                );
            }
            HealthLevel::Unhealthy => {
                error!(
                    utilization = status.utilization_percent,
                    success_rate = status.success_rate_percent,
                    fallback_rate = status.fallback_rate_percent,
                    circuit_breaker = status.circuit_breaker_status,
                    "Pool health: UNHEALTHY"
                );
            }
            HealthLevel::Critical => {
                error!(
                    utilization = status.utilization_percent,
                    success_rate = status.success_rate_percent,
                    fallback_rate = status.fallback_rate_percent,
                    circuit_breaker = status.circuit_breaker_status,
                    memory_pressure = ?status.memory_stats.memory_pressure,
                    "Pool health: CRITICAL - Immediate attention required"
                );
            }
        }

        // TODO: Add automated remediation actions
        // - Scale pool up/down based on utilization
        // - Clear instances with high memory usage
        // - Reset circuit breaker if appropriate
        // - Send alerts/notifications
    }

    /// Determine memory pressure level
    fn determine_memory_pressure(&self, metrics: &PerformanceMetrics) -> MemoryPressureLevel {
        let memory_usage_percent = if self.config.memory_limit_pages > 0 {
            (metrics.wasm_memory_pages as f64 / self.config.memory_limit_pages as f64) * 100.0
        } else {
            0.0
        };

        match memory_usage_percent {
            p if p < 50.0 => MemoryPressureLevel::Low,
            p if p < 75.0 => MemoryPressureLevel::Medium,
            p if p < 90.0 => MemoryPressureLevel::High,
            _ => MemoryPressureLevel::Critical,
        }
    }

    /// Calculate overall health level based on various metrics
    fn calculate_health_level(
        &self,
        utilization_percent: f64,
        success_rate_percent: f64,
        fallback_rate_percent: f64,
        memory_pressure: &MemoryPressureLevel,
        metrics: &PerformanceMetrics,
    ) -> HealthLevel {
        // Critical conditions
        if success_rate_percent < 50.0
            || *memory_pressure == MemoryPressureLevel::Critical
            || utilization_percent > 95.0
            || metrics.epoch_timeouts > 10 {
            return HealthLevel::Critical;
        }

        // Unhealthy conditions
        if success_rate_percent < 75.0
            || *memory_pressure == MemoryPressureLevel::High
            || utilization_percent > 85.0
            || fallback_rate_percent > 30.0
            || metrics.circuit_breaker_trips > 5 {
            return HealthLevel::Unhealthy;
        }

        // Degraded conditions
        if success_rate_percent < 90.0
            || *memory_pressure == MemoryPressureLevel::Medium
            || utilization_percent > 75.0
            || fallback_rate_percent > 10.0
            || metrics.avg_processing_time_ms > 5000.0 {
            return HealthLevel::Degraded;
        }

        // Default to healthy
        HealthLevel::Healthy
    }

    /// Calculate health trend based on recent history
    async fn calculate_health_trend(&self, _current_level: &HealthLevel) -> HealthTrend {
        let history = self.health_history.lock().unwrap();

        if history.len() < 3 {
            return HealthTrend::Unknown;
        }

        // Get recent health levels
        let recent_levels: Vec<&HealthLevel> = history
            .iter()
            .rev()
            .take(5)
            .map(|status| &status.status)
            .collect();

        // Simple trend analysis based on health level changes
        let health_scores: Vec<i32> = recent_levels
            .iter()
            .map(|level| match level {
                HealthLevel::Healthy => 4,
                HealthLevel::Degraded => 3,
                HealthLevel::Unhealthy => 2,
                HealthLevel::Critical => 1,
            })
            .collect();

        if health_scores.len() < 2 {
            return HealthTrend::Unknown;
        }

        let first_score = health_scores[health_scores.len() - 1];
        let last_score = health_scores[0];

        match last_score.cmp(&first_score) {
            std::cmp::Ordering::Greater => HealthTrend::Improving,
            std::cmp::Ordering::Less => HealthTrend::Degrading,
            std::cmp::Ordering::Equal => HealthTrend::Stable,
        }
    }

    /// Get current health status
    pub async fn get_current_health(&self) -> Result<PoolHealthStatus> {
        self.perform_health_check().await
    }

    /// Get health history for analysis
    pub fn get_health_history(&self) -> Vec<PoolHealthStatus> {
        self.health_history.lock().unwrap().clone()
    }

    /// Get health summary for external monitoring systems
    pub fn get_health_summary(&self) -> Result<serde_json::Value> {
        let history = self.health_history.lock().unwrap();

        if let Some(latest) = history.last() {
            Ok(serde_json::json!({
                "status": latest.status,
                "utilization_percent": latest.utilization_percent,
                "success_rate_percent": latest.success_rate_percent,
                "fallback_rate_percent": latest.fallback_rate_percent,
                "memory_pressure": latest.memory_stats.memory_pressure,
                "trend": latest.trend,
                "total_extractions": latest.total_extractions
            }))
        } else {
            Ok(serde_json::json!({
                "status": "Unknown",
                "message": "No health data available"
            }))
        }
    }
}