use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::time::{interval, timeout, Duration};
use tracing::{debug, error, info, warn};

// Core imports needed throughout the file
use crate::config::{ExtractorConfig, PerformanceMetrics};
use riptide_events::{EventBus, HealthEvent, HealthStatus, MetricType, MetricsEvent};

// Feature-gated pool import
#[cfg(feature = "wasm-pool")]
use crate::AdvancedInstancePool;

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
#[cfg(feature = "wasm-pool")]
pub struct PoolHealthMonitor {
    /// Pool to monitor
    pool: Arc<AdvancedInstancePool>,
    /// Configuration settings
    config: ExtractorConfig,
    /// Health status history for trend analysis
    health_history: Arc<Mutex<Vec<PoolHealthStatus>>>,
    /// Monitoring interval
    check_interval: Duration,
    /// Optional event bus for health event emission
    event_bus: Option<Arc<EventBus>>,
    /// Monitor ID for event emission
    monitor_id: String,
}

#[cfg(feature = "wasm-pool")]
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
            event_bus: None,
            monitor_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Set event bus for health event emission
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>) {
        self.event_bus = Some(event_bus);
    }

    /// Create new health monitor with event bus
    pub fn with_event_bus(
        pool: Arc<AdvancedInstancePool>,
        config: ExtractorConfig,
        check_interval: Duration,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            pool,
            config,
            health_history: Arc::new(Mutex::new(Vec::new())),
            check_interval,
            event_bus: Some(event_bus),
            monitor_id: uuid::Uuid::new_v4().to_string(),
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
                    // Emit health check completed event
                    self.emit_health_check_completed(&status).await;
                    self.process_health_status(status).await;
                }
                Ok(Err(e)) => {
                    error!(error = %e, "Health check failed");
                    // Emit health check failed event
                    self.emit_health_check_failed(&e).await;
                }
                Err(_) => {
                    error!("Health check timed out");
                    // Emit health check timeout event
                    self.emit_health_check_timeout().await;
                }
            }
        }
    }

    /// Perform comprehensive health check
    async fn perform_health_check(&self) -> Result<PoolHealthStatus> {
        debug!("Performing pool health check");

        // Get pool status
        let (available, active, max_size) = self.pool.get_pool_status().await;
        let utilization_percent = if max_size > 0 {
            (active as f64 / max_size as f64) * 100.0
        } else {
            0.0
        };

        // Get performance metrics
        let metrics = self.pool.get_metrics().await;

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
            avg_semaphore_wait_ms: metrics.semaphore_wait_time_ms,
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
            match self.health_history.lock() {
                Ok(mut history) => {
                    history.push(status.clone());
                    // Keep only last 100 entries
                    if history.len() > 100 {
                        history.remove(0);
                    }
                }
                Err(poisoned) => {
                    warn!("Health history mutex poisoned, attempting recovery");
                    let mut history = poisoned.into_inner();
                    history.push(status.clone());
                    if history.len() > 100 {
                        history.remove(0);
                    }
                }
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

        // Emit health status as event
        self.emit_health_status_event(&status).await;

        // Emit metrics events
        self.emit_metrics_events(&status).await;

        // Automated remediation actions based on health status
        self.perform_automated_remediation(&status).await;
    }

    /// Determine memory pressure level
    fn determine_memory_pressure(&self, metrics: &PerformanceMetrics) -> MemoryPressureLevel {
        let memory_limit = self.config.memory_limit_pages.unwrap_or(256);
        let memory_usage_percent = if memory_limit > 0 {
            (metrics.wasm_memory_pages as f64 / memory_limit as f64) * 100.0
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
            || metrics.epoch_timeouts > 10
        {
            return HealthLevel::Critical;
        }

        // Unhealthy conditions
        if success_rate_percent < 75.0
            || *memory_pressure == MemoryPressureLevel::High
            || utilization_percent > 85.0
            || fallback_rate_percent > 30.0
            || metrics.circuit_breaker_trips > 5
        {
            return HealthLevel::Unhealthy;
        }

        // Degraded conditions
        if success_rate_percent < 90.0
            || *memory_pressure == MemoryPressureLevel::Medium
            || utilization_percent > 75.0
            || fallback_rate_percent > 10.0
            || metrics.avg_processing_time_ms > 5000.0
        {
            return HealthLevel::Degraded;
        }

        // Default to healthy
        HealthLevel::Healthy
    }

    /// Calculate health trend based on recent history
    async fn calculate_health_trend(&self, _current_level: &HealthLevel) -> HealthTrend {
        let history = match self.health_history.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Health history mutex poisoned during trend calculation, recovering");
                poisoned.into_inner()
            }
        };

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
        match self.health_history.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => {
                warn!("Health history mutex poisoned during get_health_history, recovering");
                poisoned.into_inner().clone()
            }
        }
    }

    /// Get health summary for external monitoring systems
    pub fn get_health_summary(&self) -> Result<serde_json::Value> {
        let history = match self.health_history.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Health history mutex poisoned during get_health_summary, recovering");
                poisoned.into_inner()
            }
        };

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

    /// Emit health check completed event
    async fn emit_health_check_completed(&self, status: &PoolHealthStatus) {
        if let Some(event_bus) = &self.event_bus {
            let health_status = match status.status {
                HealthLevel::Healthy => HealthStatus::Healthy,
                HealthLevel::Degraded => HealthStatus::Degraded,
                HealthLevel::Unhealthy => HealthStatus::Unhealthy,
                HealthLevel::Critical => HealthStatus::Critical,
            };

            // Create detailed health check completed event with healthy/unhealthy counts
            let healthy_instances = status.available_instances;
            let unhealthy_instances =
                status.max_instances - status.available_instances - status.active_instances;

            let mut event = HealthEvent::new(
                format!("pool_health_{}", self.monitor_id),
                health_status,
                &format!("health_monitor_{}", self.monitor_id),
            )
            .with_details(format!(
                "Health check completed. Pool utilization: {:.1}%, Success rate: {:.1}%, Healthy: {}, Unhealthy: {}",
                status.utilization_percent, status.success_rate_percent, healthy_instances, unhealthy_instances
            ));

            // Add detailed metadata for health check results
            let mut health_metrics = std::collections::HashMap::new();
            health_metrics.insert("healthy_instances".to_string(), healthy_instances as f64);
            health_metrics.insert(
                "unhealthy_instances".to_string(),
                unhealthy_instances as f64,
            );
            health_metrics.insert(
                "utilization_percent".to_string(),
                status.utilization_percent,
            );
            health_metrics.insert(
                "success_rate_percent".to_string(),
                status.success_rate_percent,
            );
            health_metrics.insert(
                "fallback_rate_percent".to_string(),
                status.fallback_rate_percent,
            );
            health_metrics.insert(
                "avg_semaphore_wait_ms".to_string(),
                status.avg_semaphore_wait_ms,
            );
            health_metrics.insert(
                "total_extractions".to_string(),
                status.total_extractions as f64,
            );
            health_metrics.insert(
                "memory_pressure_level".to_string(),
                match status.memory_stats.memory_pressure {
                    MemoryPressureLevel::Low => 1.0,
                    MemoryPressureLevel::Medium => 2.0,
                    MemoryPressureLevel::High => 3.0,
                    MemoryPressureLevel::Critical => 4.0,
                },
            );

            event = event.with_metrics(health_metrics);

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit health check completed event");
            }
        }
    }

    /// Emit health check failed event
    async fn emit_health_check_failed(&self, error: &anyhow::Error) {
        if let Some(event_bus) = &self.event_bus {
            let event = HealthEvent::new(
                format!("pool_health_{}", self.monitor_id),
                HealthStatus::Critical,
                &format!("health_monitor_{}", self.monitor_id),
            )
            .with_details(format!("Health check failed: {}", error));

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit health check failed event");
            }
        }
    }

    /// Emit health check timeout event
    async fn emit_health_check_timeout(&self) {
        if let Some(event_bus) = &self.event_bus {
            let event = HealthEvent::new(
                format!("pool_health_{}", self.monitor_id),
                HealthStatus::Critical,
                &format!("health_monitor_{}", self.monitor_id),
            )
            .with_details("Health check timed out".to_string());

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit health check timeout event");
            }
        }
    }

    /// Emit health status event
    async fn emit_health_status_event(&self, status: &PoolHealthStatus) {
        if let Some(event_bus) = &self.event_bus {
            let health_status = match status.status {
                HealthLevel::Healthy => HealthStatus::Healthy,
                HealthLevel::Degraded => HealthStatus::Degraded,
                HealthLevel::Unhealthy => HealthStatus::Unhealthy,
                HealthLevel::Critical => HealthStatus::Critical,
            };

            let mut metrics = HashMap::new();
            metrics.insert(
                "utilization_percent".to_string(),
                status.utilization_percent,
            );
            metrics.insert(
                "success_rate_percent".to_string(),
                status.success_rate_percent,
            );
            metrics.insert(
                "available_instances".to_string(),
                status.available_instances as f64,
            );
            metrics.insert(
                "active_instances".to_string(),
                status.active_instances as f64,
            );
            metrics.insert("max_instances".to_string(), status.max_instances as f64);

            let event = HealthEvent::new(
                format!("pool_{}", self.monitor_id),
                health_status,
                &format!("health_monitor_{}", self.monitor_id),
            )
            .with_metrics(metrics);

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit health status event");
            }
        }
    }

    /// Emit metrics events
    async fn emit_metrics_events(&self, status: &PoolHealthStatus) {
        if let Some(event_bus) = &self.event_bus {
            // Pool utilization metric
            let util_event = MetricsEvent::new(
                "pool_utilization".to_string(),
                status.utilization_percent,
                MetricType::Gauge,
                &format!("health_monitor_{}", self.monitor_id),
            );

            if let Err(e) = event_bus.emit(util_event).await {
                warn!(error = %e, "Failed to emit utilization metric");
            }

            // Success rate metric
            let success_event = MetricsEvent::new(
                "pool_success_rate".to_string(),
                status.success_rate_percent,
                MetricType::Gauge,
                &format!("health_monitor_{}", self.monitor_id),
            );

            if let Err(e) = event_bus.emit(success_event).await {
                warn!(error = %e, "Failed to emit success rate metric");
            }

            // Total extractions counter
            let extraction_event = MetricsEvent::new(
                "total_extractions".to_string(),
                status.total_extractions as f64,
                MetricType::Counter,
                &format!("health_monitor_{}", self.monitor_id),
            );

            if let Err(e) = event_bus.emit(extraction_event).await {
                warn!(error = %e, "Failed to emit extraction counter");
            }
        }
    }

    /// Perform automated remediation actions based on health status
    async fn perform_automated_remediation(&self, status: &PoolHealthStatus) {
        match status.status {
            HealthLevel::Critical => {
                error!(
                    "CRITICAL health status detected - performing emergency remediation actions"
                );
                self.emergency_remediation(status).await;
            }
            HealthLevel::Unhealthy => {
                warn!("UNHEALTHY status detected - performing remediation actions");
                self.unhealthy_remediation(status).await;
            }
            HealthLevel::Degraded => {
                info!("DEGRADED status detected - performing optimization actions");
                self.degraded_remediation(status).await;
            }
            HealthLevel::Healthy => {
                // Monitor for optimization opportunities
                self.optimization_check(status).await;
            }
        }
    }

    /// Emergency remediation for critical health status
    async fn emergency_remediation(&self, status: &PoolHealthStatus) {
        // 1. Try to clear high memory usage instances
        if status.memory_stats.memory_pressure == MemoryPressureLevel::Critical {
            warn!("Critical memory pressure - clearing instances with high memory usage");
            if let Err(e) = self.pool.clear_high_memory_instances().await {
                error!(error = %e, "Failed to clear high memory instances");
            }
        }

        // 2. Scale down if utilization is too high
        if status.utilization_percent > 95.0 {
            warn!("Critical utilization - attempting to scale down active instances");
            // This would be implementation-specific based on your pool design
            info!("Scaling down would be implemented based on pool capabilities");
        }

        // 3. Reset circuit breaker if it's tripped and conditions have improved
        if status.circuit_breaker_status == "TRIPPED" && status.success_rate_percent > 80.0 {
            warn!("Attempting to reset circuit breaker after conditions improved");
            // Circuit breaker reset would be implementation-specific
            info!("Circuit breaker reset would be triggered here");
        }

        // 4. Emit emergency alert
        self.emit_emergency_alert(status).await;
    }

    /// Remediation for unhealthy status
    async fn unhealthy_remediation(&self, status: &PoolHealthStatus) {
        // 1. Clear instances if memory pressure is high
        if matches!(
            status.memory_stats.memory_pressure,
            MemoryPressureLevel::High | MemoryPressureLevel::Critical
        ) {
            info!("High memory pressure - clearing some instances");
            if let Err(e) = self.pool.clear_some_instances(2).await {
                warn!(error = %e, "Failed to clear instances for memory pressure");
            }
        }

        // 2. Reduce concurrency if utilization is too high
        if status.utilization_percent > 85.0 {
            info!("High utilization - recommending concurrency reduction");
            // This would typically involve adjusting semaphore limits
        }

        // 3. Check if fallback rate is too high
        if status.fallback_rate_percent > 30.0 {
            warn!(
                fallback_rate = status.fallback_rate_percent,
                "High fallback rate detected - may indicate system issues"
            );
        }
    }

    /// Remediation for degraded status
    async fn degraded_remediation(&self, status: &PoolHealthStatus) {
        // 1. Proactive memory management
        if status.memory_stats.memory_pressure == MemoryPressureLevel::Medium {
            debug!("Medium memory pressure - performing proactive cleanup");
            if let Err(e) = self.pool.trigger_memory_cleanup().await {
                debug!(error = %e, "Memory cleanup failed");
            }
        }

        // 2. Performance optimization
        if status.avg_semaphore_wait_ms > 1000.0 {
            info!(
                wait_time = status.avg_semaphore_wait_ms,
                "High semaphore wait times - may need pool size adjustment"
            );
        }
    }

    /// Check for optimization opportunities in healthy state
    async fn optimization_check(&self, status: &PoolHealthStatus) {
        // Only log optimization opportunities for healthy state
        if status.utilization_percent < 30.0 && status.available_instances > 3 {
            debug!(
                utilization = status.utilization_percent,
                available = status.available_instances,
                "Low utilization - pool could potentially be scaled down"
            );
        }

        if status.memory_stats.memory_pressure == MemoryPressureLevel::Low
            && status.success_rate_percent > 95.0
        {
            debug!("Optimal conditions - system is performing well");
        }
    }

    /// Emit emergency alert for critical conditions
    async fn emit_emergency_alert(&self, status: &PoolHealthStatus) {
        if let Some(event_bus) = &self.event_bus {
            let alert_event = HealthEvent::new(
                format!("emergency_alert_{}", self.monitor_id),
                HealthStatus::Critical,
                &format!("health_monitor_{}", self.monitor_id),
            )
            .with_details(format!(
                "EMERGENCY: Critical health status detected. Utilization: {:.1}%, \
                Success Rate: {:.1}%, Memory Pressure: {:?}, Circuit Breaker: {}",
                status.utilization_percent,
                status.success_rate_percent,
                status.memory_stats.memory_pressure,
                status.circuit_breaker_status
            ));

            if let Err(e) = event_bus.emit(alert_event).await {
                error!(error = %e, "Failed to emit emergency alert");
            }
        }

        // Also log the emergency to ensure it's captured
        error!(
            utilization = status.utilization_percent,
            success_rate = status.success_rate_percent,
            memory_pressure = ?status.memory_stats.memory_pressure,
            circuit_breaker = status.circuit_breaker_status,
            "EMERGENCY ALERT: Critical pool health status requires immediate attention"
        );
    }
}
