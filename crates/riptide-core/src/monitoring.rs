use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::time::interval;

/// Comprehensive performance monitoring and metrics collection system
///
/// This module provides real-time monitoring of extractor performance,
/// resource usage, and system health with alerting capabilities.
/// Core performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // Timing metrics
    pub avg_extraction_time_ms: f64,
    pub p95_extraction_time_ms: f64,
    pub p99_extraction_time_ms: f64,

    // Throughput metrics
    pub requests_per_second: f64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,
    pub total_extractions: u64,

    // Resource metrics
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f32,
    pub pool_size: usize,
    pub active_instances: usize,
    pub idle_instances: usize,

    // Quality metrics
    pub avg_content_quality_score: f64,
    pub avg_extracted_word_count: f64,
    pub cache_hit_ratio: f64,

    // Error metrics
    pub error_rate: f64,
    pub timeout_rate: f64,
    pub circuit_breaker_trips: u64,

    // System health
    pub health_score: f32,
    pub uptime_seconds: u64,
    #[serde(skip, default = "instant_now")]
    pub last_updated: Instant,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_updated_utc: DateTime<Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_extraction_time_ms: 0.0,
            p95_extraction_time_ms: 0.0,
            p99_extraction_time_ms: 0.0,
            requests_per_second: 0.0,
            successful_extractions: 0,
            failed_extractions: 0,
            total_extractions: 0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            pool_size: 0,
            active_instances: 0,
            idle_instances: 0,
            avg_content_quality_score: 0.0,
            avg_extracted_word_count: 0.0,
            cache_hit_ratio: 0.0,
            error_rate: 0.0,
            timeout_rate: 0.0,
            circuit_breaker_trips: 0,
            health_score: 100.0,
            uptime_seconds: 0,
            last_updated: Instant::now(),
            last_updated_utc: Utc::now(),
        }
    }
}

// Helper function for serde default
fn instant_now() -> Instant {
    Instant::now()
}

/// Time-series data point for trending analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    #[serde(skip, default = "instant_now")]
    pub timestamp: Instant,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp_utc: DateTime<Utc>,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

/// Time-series buffer for historical data
pub struct TimeSeriesBuffer {
    data: VecDeque<MetricDataPoint>,
    max_size: usize,
    retention_period: Duration,
}

impl TimeSeriesBuffer {
    pub fn new(max_size: usize, retention_period: Duration) -> Self {
        Self {
            data: VecDeque::with_capacity(max_size),
            max_size,
            retention_period,
        }
    }

    pub fn add_point(&mut self, value: f64, metadata: HashMap<String, String>) {
        let now = Instant::now();

        // Clean old data
        self.cleanup_old_data(now);

        // Add new point
        let point = MetricDataPoint {
            timestamp: now,
            timestamp_utc: Utc::now(),
            value,
            metadata,
        };

        self.data.push_back(point);

        // Trim if over capacity
        while self.data.len() > self.max_size {
            self.data.pop_front();
        }
    }

    pub fn get_recent_data(&self, duration: Duration) -> Vec<&MetricDataPoint> {
        let cutoff = Instant::now() - duration;
        self.data
            .iter()
            .filter(|point| point.timestamp >= cutoff)
            .collect()
    }

    pub fn calculate_percentile(&self, percentile: f64, duration: Duration) -> Option<f64> {
        let recent_data: Vec<f64> = self
            .get_recent_data(duration)
            .iter()
            .map(|point| point.value)
            .collect();

        if recent_data.is_empty() {
            return None;
        }

        let mut sorted_data = recent_data;
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((percentile / 100.0) * (sorted_data.len() - 1) as f64) as usize;
        Some(sorted_data[index])
    }

    fn cleanup_old_data(&mut self, now: Instant) {
        let cutoff = now - self.retention_period;
        while let Some(front) = self.data.front() {
            if front.timestamp >= cutoff {
                break;
            }
            self.data.pop_front();
        }
    }
}

/// Advanced metrics collector with real-time analysis
pub struct MetricsCollector {
    // Time series data for different metrics
    extraction_times: Arc<Mutex<TimeSeriesBuffer>>,
    request_rates: Arc<Mutex<TimeSeriesBuffer>>,
    memory_usage: Arc<Mutex<TimeSeriesBuffer>>,
    error_rates: Arc<Mutex<TimeSeriesBuffer>>,

    // Current metrics state
    current_metrics: Arc<RwLock<PerformanceMetrics>>,

    // Configuration
    collection_interval: Duration,
    #[allow(dead_code)]
    retention_period: Duration,
    #[allow(dead_code)]
    max_data_points: usize,

    // System start time for uptime calculation
    start_time: Instant,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        let retention_period = Duration::from_secs(24 * 60 * 60);
        let max_data_points = 10000;

        Self {
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
            collection_interval: Duration::from_secs(30),
            retention_period,
            max_data_points,
            start_time: Instant::now(),
        }
    }

    /// Start the metrics collection background task
    pub async fn start_collection(&self) {
        let mut interval = interval(self.collection_interval);

        loop {
            interval.tick().await;
            self.collect_system_metrics().await;
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
    ) {
        // Record extraction time
        {
            let mut times = self.extraction_times.lock().unwrap();
            times.add_point(duration.as_millis() as f64, HashMap::new());
        }

        // Update current metrics
        {
            let mut metrics = self.current_metrics.write().unwrap();
            metrics.total_extractions += 1;

            if success {
                metrics.successful_extractions += 1;
            } else {
                metrics.failed_extractions += 1;
            }

            if let Some(score) = quality_score {
                // Update running average of quality score
                let total_weight = metrics.total_extractions as f64;
                let current_weight = (total_weight - 1.0) / total_weight;
                let new_weight = 1.0 / total_weight;

                metrics.avg_content_quality_score =
                    metrics.avg_content_quality_score * current_weight + score as f64 * new_weight;
            }

            if let Some(words) = word_count {
                // Update running average of word count
                let total_weight = metrics.total_extractions as f64;
                let current_weight = (total_weight - 1.0) / total_weight;
                let new_weight = 1.0 / total_weight;

                metrics.avg_extracted_word_count =
                    metrics.avg_extracted_word_count * current_weight + words as f64 * new_weight;
            }

            // Update cache hit ratio
            let total_weight = metrics.total_extractions as f64;
            if was_cached {
                // This is a simplified calculation - in practice you'd track cache hits/misses separately
                metrics.cache_hit_ratio =
                    (metrics.cache_hit_ratio * (total_weight - 1.0) + 1.0) / total_weight;
            } else {
                metrics.cache_hit_ratio =
                    (metrics.cache_hit_ratio * (total_weight - 1.0)) / total_weight;
            }

            metrics.last_updated = Instant::now();
            metrics.last_updated_utc = Utc::now();
        }
    }

    /// Record error occurrence
    pub async fn record_error(&self, error_type: &str, is_timeout: bool) {
        let mut error_metadata = HashMap::new();
        error_metadata.insert("type".to_string(), error_type.to_string());
        error_metadata.insert("is_timeout".to_string(), is_timeout.to_string());

        {
            let mut errors = self.error_rates.lock().unwrap();
            errors.add_point(1.0, error_metadata);
        }

        {
            let mut metrics = self.current_metrics.write().unwrap();
            if is_timeout {
                // Update timeout rate - this is simplified, in practice you'd track more precisely
                let total = metrics.total_extractions as f64;
                metrics.timeout_rate = (metrics.timeout_rate * (total - 1.0) + 1.0) / total;
            }
            metrics.last_updated = Instant::now();
            metrics.last_updated_utc = Utc::now();
        }
    }

    /// Record circuit breaker trip
    pub async fn record_circuit_breaker_trip(&self) {
        let mut metrics = self.current_metrics.write().unwrap();
        metrics.circuit_breaker_trips += 1;
        metrics.last_updated = Instant::now();
        metrics.last_updated_utc = Utc::now();
    }

    /// Update pool statistics
    pub async fn update_pool_stats(&self, pool_size: usize, active: usize, idle: usize) {
        let mut metrics = self.current_metrics.write().unwrap();
        metrics.pool_size = pool_size;
        metrics.active_instances = active;
        metrics.idle_instances = idle;
        metrics.last_updated = Instant::now();
        metrics.last_updated_utc = Utc::now();
    }

    /// Get current metrics snapshot
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        self.current_metrics.read().unwrap().clone()
    }

    /// Get detailed performance report
    pub async fn get_performance_report(&self, duration: Duration) -> PerformanceReport {
        let times = self.extraction_times.lock().unwrap();
        let _rates = self.request_rates.lock().unwrap();
        let memory = self.memory_usage.lock().unwrap();
        let errors = self.error_rates.lock().unwrap();

        let current = self.current_metrics.read().unwrap().clone();

        PerformanceReport {
            current_metrics: current.clone(),
            avg_extraction_time: times
                .get_recent_data(duration)
                .iter()
                .map(|p| p.value)
                .sum::<f64>()
                / times.get_recent_data(duration).len() as f64,
            p95_extraction_time: times.calculate_percentile(95.0, duration).unwrap_or(0.0),
            p99_extraction_time: times.calculate_percentile(99.0, duration).unwrap_or(0.0),
            peak_memory_usage: memory
                .get_recent_data(duration)
                .iter()
                .map(|p| p.value)
                .fold(0.0, f64::max),
            error_count: errors.get_recent_data(duration).len(),
            health_summary: self.calculate_health_summary(&current),
            recommendations: self.generate_recommendations(&current),
        }
    }

    /// Collect system-level metrics
    async fn collect_system_metrics(&self) {
        // Collect memory usage
        let memory_usage = self.get_memory_usage();
        {
            let mut memory = self.memory_usage.lock().unwrap();
            memory.add_point(memory_usage as f64, HashMap::new());
        }

        // Calculate request rate
        let request_rate = self.calculate_request_rate().await;
        {
            let mut rates = self.request_rates.lock().unwrap();
            rates.add_point(request_rate, HashMap::new());
        }

        // Update metrics with calculated values
        {
            let mut metrics = self.current_metrics.write().unwrap();

            // Calculate percentiles
            let times = self.extraction_times.lock().unwrap();
            metrics.p95_extraction_time_ms = times
                .calculate_percentile(95.0, Duration::from_secs(5 * 60))
                .unwrap_or(0.0);
            metrics.p99_extraction_time_ms = times
                .calculate_percentile(99.0, Duration::from_secs(5 * 60))
                .unwrap_or(0.0);

            // Update system metrics
            metrics.memory_usage_bytes = memory_usage;
            metrics.cpu_usage_percent = self.get_cpu_usage();
            metrics.requests_per_second = request_rate;
            metrics.uptime_seconds = self.start_time.elapsed().as_secs();

            // Calculate error rate
            let recent_errors = self
                .error_rates
                .lock()
                .unwrap()
                .get_recent_data(Duration::from_secs(5 * 60))
                .len();
            let recent_total =
                std::cmp::max(1, times.get_recent_data(Duration::from_secs(5 * 60)).len());
            metrics.error_rate = (recent_errors as f64 / recent_total as f64) * 100.0;

            // Calculate health score
            metrics.health_score = self.calculate_health_score(&metrics);

            metrics.last_updated = Instant::now();
            metrics.last_updated_utc = Utc::now();
        }
    }

    fn get_memory_usage(&self) -> u64 {
        // In a real implementation, this would use system APIs
        // For now, return a placeholder value
        std::process::id() as u64 * 1024 * 1024 // Simplified
    }

    fn get_cpu_usage(&self) -> f32 {
        // In a real implementation, this would use system APIs
        // For now, return a placeholder value
        15.5 // Simplified
    }

    async fn calculate_request_rate(&self) -> f64 {
        let metrics = self.current_metrics.read().unwrap();
        let duration_minutes = self.start_time.elapsed().as_secs() as f64 / 60.0;

        if duration_minutes > 0.0 {
            metrics.total_extractions as f64 / (duration_minutes * 60.0)
        } else {
            0.0
        }
    }

    fn calculate_health_score(&self, metrics: &PerformanceMetrics) -> f32 {
        let mut score = 100.0;

        // Deduct points for high error rates
        if metrics.error_rate > 5.0 {
            score -= (metrics.error_rate - 5.0) * 2.0;
        }

        // Deduct points for high CPU usage
        if metrics.cpu_usage_percent > 80.0 {
            score -= (metrics.cpu_usage_percent - 80.0) as f64 * 0.5;
        }

        // Deduct points for circuit breaker trips
        if metrics.circuit_breaker_trips > 0 {
            score -= (metrics.circuit_breaker_trips as f64).min(20.0);
        }

        // Deduct points for high timeout rate
        if metrics.timeout_rate > 1.0 {
            score -= (metrics.timeout_rate - 1.0) * 10.0;
        }

        score.max(0.0) as f32
    }

    fn calculate_health_summary(&self, metrics: &PerformanceMetrics) -> String {
        if metrics.health_score >= 95.0 {
            "Excellent - System performing optimally".to_string()
        } else if metrics.health_score >= 85.0 {
            "Good - Minor performance issues detected".to_string()
        } else if metrics.health_score >= 70.0 {
            "Fair - Performance degradation observed".to_string()
        } else if metrics.health_score >= 50.0 {
            "Poor - Significant performance issues".to_string()
        } else {
            "Critical - System requires immediate attention".to_string()
        }
    }

    fn generate_recommendations(&self, metrics: &PerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.error_rate > 5.0 {
            recommendations.push("High error rate detected - investigate error patterns and implement circuit breakers".to_string());
        }

        if metrics.cpu_usage_percent > 80.0 {
            recommendations
                .push("High CPU usage - consider scaling out or optimizing algorithms".to_string());
        }

        if metrics.avg_extraction_time_ms > 5000.0 {
            recommendations.push(
                "High extraction latency - optimize parsing algorithms or increase instance pool"
                    .to_string(),
            );
        }

        if metrics.cache_hit_ratio < 0.5 {
            recommendations
                .push("Low cache hit ratio - review caching strategy and TTL settings".to_string());
        }

        if metrics.pool_size < metrics.active_instances {
            recommendations.push(
                "Instance pool exhausted - increase pool size or implement queue management"
                    .to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations.push("System is performing well - continue monitoring".to_string());
        }

        recommendations
    }
}

/// Comprehensive performance report
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub current_metrics: PerformanceMetrics,
    pub avg_extraction_time: f64,
    pub p95_extraction_time: f64,
    pub p99_extraction_time: f64,
    pub peak_memory_usage: f64,
    pub error_count: usize,
    pub health_summary: String,
    pub recommendations: Vec<String>,
}

/// Alert system for monitoring critical conditions
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub metric_name: String,
    pub threshold: f64,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert manager for monitoring and notifications
pub struct AlertManager {
    rules: Vec<AlertRule>,
    active_alerts: HashMap<String, Instant>,
    cooldown_period: Duration,
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            rules: Self::default_alert_rules(),
            active_alerts: HashMap::new(),
            cooldown_period: Duration::from_secs(5 * 60),
        }
    }

    pub async fn check_alerts(&mut self, metrics: &PerformanceMetrics) -> Vec<Alert> {
        let mut triggered_alerts = Vec::new();
        let now = Instant::now();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let current_value = self.get_metric_value(metrics, &rule.metric_name);
            let should_alert = match rule.condition {
                AlertCondition::GreaterThan => current_value > rule.threshold,
                AlertCondition::LessThan => current_value < rule.threshold,
                AlertCondition::Equals => (current_value - rule.threshold).abs() < 0.001,
            };

            if should_alert {
                // Check cooldown
                if let Some(last_alert) = self.active_alerts.get(&rule.name) {
                    if now.duration_since(*last_alert) < self.cooldown_period {
                        continue;
                    }
                }

                let alert = Alert {
                    rule_name: rule.name.clone(),
                    message: format!(
                        "Alert: {} is {:.2}, threshold is {:.2}",
                        rule.metric_name, current_value, rule.threshold
                    ),
                    severity: rule.severity.clone(),
                    timestamp: now,
                    timestamp_utc: Utc::now(),
                    current_value,
                    threshold: rule.threshold,
                };

                triggered_alerts.push(alert);
                self.active_alerts.insert(rule.name.clone(), now);
            }
        }

        triggered_alerts
    }

    fn default_alert_rules() -> Vec<AlertRule> {
        vec![
            AlertRule {
                name: "high_error_rate".to_string(),
                metric_name: "error_rate".to_string(),
                threshold: 10.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Error,
                enabled: true,
            },
            AlertRule {
                name: "high_cpu_usage".to_string(),
                metric_name: "cpu_usage_percent".to_string(),
                threshold: 90.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                name: "low_health_score".to_string(),
                metric_name: "health_score".to_string(),
                threshold: 50.0,
                condition: AlertCondition::LessThan,
                severity: AlertSeverity::Critical,
                enabled: true,
            },
            AlertRule {
                name: "high_extraction_time".to_string(),
                metric_name: "p99_extraction_time_ms".to_string(),
                threshold: 10000.0,
                condition: AlertCondition::GreaterThan,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
        ]
    }

    fn get_metric_value(&self, metrics: &PerformanceMetrics, metric_name: &str) -> f64 {
        match metric_name {
            "error_rate" => metrics.error_rate,
            "cpu_usage_percent" => metrics.cpu_usage_percent as f64,
            "health_score" => metrics.health_score as f64,
            "p99_extraction_time_ms" => metrics.p99_extraction_time_ms,
            "memory_usage_bytes" => metrics.memory_usage_bytes as f64,
            "requests_per_second" => metrics.requests_per_second,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub rule_name: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: Instant,
    pub timestamp_utc: DateTime<Utc>,
    pub current_value: f64,
    pub threshold: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        // Record some extractions
        collector
            .record_extraction(Duration::from_millis(100), true, Some(85), Some(500), false)
            .await;
        collector
            .record_extraction(Duration::from_millis(200), true, Some(90), Some(750), true)
            .await;
        collector
            .record_extraction(Duration::from_millis(150), false, None, None, false)
            .await;

        let metrics = collector.get_current_metrics().await;

        assert_eq!(metrics.total_extractions, 3);
        assert_eq!(metrics.successful_extractions, 2);
        assert_eq!(metrics.failed_extractions, 1);
        assert!(metrics.avg_content_quality_score > 0.0);
        assert!(metrics.avg_extracted_word_count > 0.0);
    }

    #[tokio::test]
    async fn test_performance_report() {
        let collector = MetricsCollector::new();

        // Record multiple extractions with different timings
        for i in 0..10 {
            let duration = Duration::from_millis(100 + i * 10);
            collector
                .record_extraction(duration, true, Some(80 + i as u8), Some(500), i % 2 == 0)
                .await;
        }

        sleep(Duration::from_millis(100)).await;

        let report = collector
            .get_performance_report(Duration::from_secs(60))
            .await;

        assert!(report.avg_extraction_time > 0.0);
        assert!(report.p95_extraction_time >= report.avg_extraction_time);
        assert!(report.p99_extraction_time >= report.p95_extraction_time);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_time_series_buffer() {
        let mut buffer = TimeSeriesBuffer::new(5, Duration::from_secs(10));

        // Add some data points
        for i in 0..7 {
            buffer.add_point(i as f64, HashMap::new());
        }

        // Should only keep the last 5 points due to max_size
        assert_eq!(buffer.data.len(), 5);

        // Test percentile calculation
        let p50 = buffer.calculate_percentile(50.0, Duration::from_secs(15));
        assert!(p50.is_some());
    }

    #[tokio::test]
    async fn test_alert_system() {
        let mut alert_manager = AlertManager::new();

        let high_error_metrics = PerformanceMetrics {
            error_rate: 15.0,        // Above threshold of 10.0
            cpu_usage_percent: 95.0, // Above threshold of 90.0
            health_score: 40.0,      // Below threshold of 50.0
            ..Default::default()
        };

        let alerts = alert_manager.check_alerts(&high_error_metrics).await;

        // Should trigger multiple alerts
        assert!(alerts.len() >= 3);

        // Check that error rate alert is triggered
        assert!(alerts.iter().any(|a| a.rule_name == "high_error_rate"));
        assert!(alerts.iter().any(|a| a.rule_name == "high_cpu_usage"));
        assert!(alerts.iter().any(|a| a.rule_name == "low_health_score"));
    }
}
