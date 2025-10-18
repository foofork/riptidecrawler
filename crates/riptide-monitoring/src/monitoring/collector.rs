//! Metrics collector with telemetry integration

use crate::monitoring::{
    error::{LockManager, Result},
    health::HealthCalculator,
    metrics::{MonitoringConfig, PerformanceMetrics},
    time_series::TimeSeriesBuffer,
};
use crate::telemetry::TelemetrySystem;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{info, warn};

/// Advanced metrics collector with real-time analysis and OpenTelemetry integration
pub struct MetricsCollector {
    // Telemetry system for observability
    telemetry: Option<Arc<TelemetrySystem>>,

    // Time series data for different metrics
    extraction_times: Arc<Mutex<TimeSeriesBuffer>>,
    request_rates: Arc<Mutex<TimeSeriesBuffer>>,
    memory_usage: Arc<Mutex<TimeSeriesBuffer>>,
    error_rates: Arc<Mutex<TimeSeriesBuffer>>,

    // Current metrics state
    current_metrics: Arc<RwLock<PerformanceMetrics>>,

    // Configuration
    config: MonitoringConfig,

    // System start time for uptime calculation
    start_time: Instant,

    // Health calculator
    health_calculator: HealthCalculator,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::with_config(MonitoringConfig::default())
    }

    /// Create a new metrics collector with custom configuration
    pub fn with_config(config: MonitoringConfig) -> Self {
        let retention_period = Duration::from_secs(config.retention_period_hours * 3600);
        let max_data_points = config.max_data_points;

        Self {
            telemetry: None,
            extraction_times: Arc::new(Mutex::new(TimeSeriesBuffer::new(
                max_data_points,
                retention_period,
            ))),
            request_rates: Arc::new(Mutex::new(TimeSeriesBuffer::new(
                max_data_points,
                retention_period,
            ))),
            memory_usage: Arc::new(Mutex::new(TimeSeriesBuffer::new(
                max_data_points,
                retention_period,
            ))),
            error_rates: Arc::new(Mutex::new(TimeSeriesBuffer::new(
                max_data_points,
                retention_period,
            ))),
            current_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            health_calculator: HealthCalculator::new(config.health_thresholds.clone()),
            config,
            start_time: Instant::now(),
        }
    }

    /// Create a new MetricsCollector with telemetry integration
    pub fn with_telemetry(telemetry: Arc<TelemetrySystem>) -> Self {
        let mut collector = Self::new();
        collector.telemetry = Some(telemetry);
        collector
    }

    /// Get a reference to the telemetry system
    pub fn telemetry(&self) -> Option<&TelemetrySystem> {
        self.telemetry.as_deref()
    }

    /// Start the metrics collection background task
    pub async fn start_collection(&self) {
        // Start metrics collection
        info!("Starting metrics collection background task");

        let collection_interval = Duration::from_secs(self.config.collection_interval_secs);
        let mut interval = interval(collection_interval);

        loop {
            interval.tick().await;
            if let Err(e) = self.collect_system_metrics().await {
                warn!("Failed to collect system metrics: {}", e);
            }
        }
    }

    /// Record extraction completion
    pub async fn record_extraction(
        &self,
        duration: Duration,
        success: bool,
        quality_score: Option<u8>,
        word_count: Option<u32>,
        was_cached: bool,
    ) -> Result<()> {
        // Record extraction metrics

        // Record extraction time
        {
            let mut times = LockManager::acquire_mutex(&self.extraction_times, "extraction_times")?;
            times.add_point(duration.as_millis() as f64, HashMap::new());
        }

        // Update current metrics
        {
            let mut metrics = LockManager::acquire_write(&self.current_metrics, "current_metrics")?;
            metrics.total_extractions += 1;

            if success {
                metrics.successful_extractions += 1;
            } else {
                metrics.failed_extractions += 1;
            }

            // Store total extractions to avoid multiple borrows
            let total_extractions = metrics.total_extractions;

            // Update quality score average
            if let Some(score) = quality_score {
                update_running_average(
                    &mut metrics.avg_content_quality_score,
                    score as f64,
                    total_extractions,
                );
            }

            // Update word count average
            if let Some(words) = word_count {
                update_running_average(
                    &mut metrics.avg_extracted_word_count,
                    words as f64,
                    total_extractions,
                );
            }

            // Update cache hit ratio
            if was_cached {
                update_running_average(&mut metrics.cache_hit_ratio, 1.0, total_extractions);
            } else {
                update_running_average(&mut metrics.cache_hit_ratio, 0.0, total_extractions);
            }

            metrics.last_updated = Instant::now();
            metrics.last_updated_utc = Utc::now();
        }

        Ok(())
    }

    /// Record error occurrence
    pub async fn record_error(&self, error_type: &str, is_timeout: bool) -> Result<()> {
        // Record error metrics

        let mut error_metadata = HashMap::new();
        error_metadata.insert("type".to_string(), error_type.to_string());
        error_metadata.insert("is_timeout".to_string(), is_timeout.to_string());

        {
            let mut errors = LockManager::acquire_mutex(&self.error_rates, "error_rates")?;
            errors.add_point(1.0, error_metadata);
        }

        if is_timeout {
            let mut metrics = LockManager::acquire_write(&self.current_metrics, "timeout_metrics")?;
            let total = metrics.total_extractions.max(1);
            update_running_average(&mut metrics.timeout_rate, 1.0, total);
            metrics.last_updated = Instant::now();
            metrics.last_updated_utc = Utc::now();
        }

        Ok(())
    }

    /// Record circuit breaker trip
    pub async fn record_circuit_breaker_trip(&self) -> Result<()> {
        let mut metrics = LockManager::acquire_write(&self.current_metrics, "circuit_breaker")?;
        metrics.circuit_breaker_trips += 1;
        metrics.last_updated = Instant::now();
        metrics.last_updated_utc = Utc::now();
        Ok(())
    }

    /// Update pool statistics
    pub async fn update_pool_stats(
        &self,
        pool_size: usize,
        active: usize,
        idle: usize,
    ) -> Result<()> {
        let mut metrics = LockManager::acquire_write(&self.current_metrics, "pool_stats")?;
        metrics.pool_size = pool_size;
        metrics.active_instances = active;
        metrics.idle_instances = idle;
        metrics.last_updated = Instant::now();
        metrics.last_updated_utc = Utc::now();
        Ok(())
    }

    /// Get current metrics snapshot
    pub async fn get_current_metrics(&self) -> Result<PerformanceMetrics> {
        let metrics = LockManager::acquire_read(&self.current_metrics, "get_metrics")?;
        Ok(metrics.clone())
    }

    /// Collect system-level metrics
    async fn collect_system_metrics(&self) -> Result<()> {
        // Collect memory usage
        let memory_usage = self.get_memory_usage();
        {
            let mut memory = LockManager::acquire_mutex(&self.memory_usage, "memory_usage")?;
            memory.add_point(memory_usage as f64, HashMap::new());
        }

        // Calculate request rate
        let request_rate = self.calculate_request_rate().await?;
        {
            let mut rates = LockManager::acquire_mutex(&self.request_rates, "request_rates")?;
            rates.add_point(request_rate, HashMap::new());
        }

        // Update metrics with calculated values
        {
            let mut metrics = LockManager::acquire_write(&self.current_metrics, "system_metrics")?;

            // Calculate percentiles
            let times = LockManager::acquire_mutex(&self.extraction_times, "percentiles")?;
            metrics.p95_extraction_time_ms = times
                .calculate_percentile(95.0, Duration::from_secs(5 * 60))
                .unwrap_or(0.0);
            metrics.p99_extraction_time_ms = times
                .calculate_percentile(99.0, Duration::from_secs(5 * 60))
                .unwrap_or(0.0);
            metrics.avg_extraction_time_ms = times
                .calculate_average(Duration::from_secs(5 * 60))
                .unwrap_or(0.0);

            // Update system metrics
            metrics.memory_usage_bytes = memory_usage;
            metrics.cpu_usage_percent = self.get_cpu_usage();
            metrics.requests_per_second = request_rate;
            metrics.uptime_seconds = self.start_time.elapsed().as_secs();

            // Calculate error rate
            let error_count = self
                .error_rates
                .lock()
                .ok()
                .map(|rates| rates.get_recent_data(Duration::from_secs(5 * 60)).len())
                .unwrap_or(0);

            let total_count = times
                .get_recent_data(Duration::from_secs(5 * 60))
                .len()
                .max(1);
            metrics.error_rate = (error_count as f64 / total_count as f64) * 100.0;

            // Calculate health score
            metrics.health_score = self.health_calculator.calculate_health(&metrics);

            metrics.last_updated = Instant::now();
            metrics.last_updated_utc = Utc::now();
        }

        Ok(())
    }

    fn get_memory_usage(&self) -> u64 {
        // Implement actual memory usage collection using sysinfo
        use std::process;
        use sysinfo::{Pid, ProcessesToUpdate, System};

        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        // Get current process memory usage
        let current_pid = process::id();
        if let Some(process) = sys.process(Pid::from(current_pid as usize)) {
            // Return RSS (Resident Set Size) in bytes
            process.memory() * 1024 // sysinfo returns KB, convert to bytes
        } else {
            // Fallback: get system memory usage if process not found
            (sys.total_memory() - sys.available_memory()) * 1024
        }
    }

    fn get_cpu_usage(&self) -> f32 {
        // Implement actual CPU usage collection using sysinfo
        use std::process;
        use sysinfo::{Pid, ProcessesToUpdate, System};

        let mut sys = System::new_all();
        sys.refresh_cpu_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        // Wait a bit for CPU usage calculation to be accurate
        std::thread::sleep(std::time::Duration::from_millis(100));
        sys.refresh_cpu_all();

        // Get current process CPU usage first
        let current_pid = process::id();
        if let Some(process) = sys.process(Pid::from(current_pid as usize)) {
            process.cpu_usage()
        } else {
            // Fallback: get system-wide CPU usage average
            if !sys.cpus().is_empty() {
                let total_cpu: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
                total_cpu / sys.cpus().len() as f32
            } else {
                0.0
            }
        }
    }

    async fn calculate_request_rate(&self) -> Result<f64> {
        let metrics = LockManager::acquire_read(&self.current_metrics, "request_rate")?;
        let duration_secs = self.start_time.elapsed().as_secs() as f64;

        Ok(if duration_secs > 0.0 {
            metrics.total_extractions as f64 / duration_secs
        } else {
            0.0
        })
    }

    /// Record pool operation metrics (for event handlers)
    pub async fn record_pool_operation(
        &self,
        operation: &str,
        value: f64,
        timestamp: u64,
    ) -> Result<()> {
        // For now, just log the operation - can be expanded later
        tracing::debug!(operation = %operation, value = %value, timestamp = %timestamp, "Pool operation recorded");
        Ok(())
    }

    /// Record pool state metrics (for event handlers)
    pub async fn record_pool_state(
        &self,
        available: usize,
        active: usize,
        total: usize,
    ) -> Result<()> {
        self.update_pool_stats(total, active, available.saturating_sub(active))
            .await
    }

    /// Record extraction time metrics (for event handlers)
    pub async fn record_extraction_time(&self, duration: Duration, success: bool) -> Result<()> {
        self.record_extraction(duration, success, None, None, false)
            .await
    }

    /// Record extraction outcome (for event handlers)
    pub async fn record_extraction_outcome(&self, operation: &str, url: &str) -> Result<()> {
        tracing::debug!(operation = %operation, url = %url, "Extraction outcome recorded");
        Ok(())
    }

    /// Record custom metric (for event handlers)
    pub async fn record_custom_metric(
        &self,
        metric_name: &str,
        metric_value: f64,
        tags: &HashMap<String, String>,
    ) -> Result<()> {
        tracing::debug!(metric_name = %metric_name, metric_value = %metric_value, ?tags, "Custom metric recorded");
        Ok(())
    }

    /// Record health status (for event handlers)
    pub async fn record_health_status(&self, component: &str, health_score: f64) -> Result<()> {
        tracing::debug!(component = %component, health_score = %health_score, "Health status recorded");
        Ok(())
    }
}

/// Helper function to update running average
fn update_running_average(current: &mut f64, new_value: f64, total_count: u64) {
    let weight = total_count as f64;
    let current_weight = (weight - 1.0) / weight;
    let new_weight = 1.0 / weight;
    *current = *current * current_weight + new_value * new_weight;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        // Record some extractions
        collector
            .record_extraction(Duration::from_millis(100), true, Some(85), Some(500), false)
            .await
            .unwrap();
        collector
            .record_extraction(Duration::from_millis(200), true, Some(90), Some(750), true)
            .await
            .unwrap();
        collector
            .record_extraction(Duration::from_millis(150), false, None, None, false)
            .await
            .unwrap();

        let metrics = collector.get_current_metrics().await.unwrap();

        assert_eq!(metrics.total_extractions, 3);
        assert_eq!(metrics.successful_extractions, 2);
        assert_eq!(metrics.failed_extractions, 1);
        assert!(metrics.avg_content_quality_score > 0.0);
        assert!(metrics.avg_extracted_word_count > 0.0);
    }

    #[tokio::test]
    async fn test_error_recording() {
        let collector = MetricsCollector::new();

        collector
            .record_error("network_error", false)
            .await
            .unwrap();
        collector.record_error("timeout_error", true).await.unwrap();

        let metrics = collector.get_current_metrics().await.unwrap();
        assert!(metrics.timeout_rate > 0.0);
    }
}
