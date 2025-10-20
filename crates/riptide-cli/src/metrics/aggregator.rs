//! Metrics aggregation and analysis logic
//!
//! This module provides functions for computing percentiles, averages,
//! and other statistical aggregations over collected metrics.

use super::types::{CommandAggregates, CommandMetrics, MetricPoint};
use std::collections::HashMap;

/// Aggregate multiple command metrics into statistical summary
pub struct MetricsAggregator {
    /// Cached percentile calculations
    percentile_cache: HashMap<String, PercentileCache>,
}

#[derive(Debug, Clone)]
struct PercentileCache {
    durations: Vec<f64>,
    p50: f64,
    p95: f64,
    p99: f64,
    last_updated: std::time::Instant,
}

impl MetricsAggregator {
    /// Create new aggregator
    pub fn new() -> Self {
        Self {
            percentile_cache: HashMap::new(),
        }
    }

    /// Aggregate command metrics by command name
    pub fn aggregate_by_command(
        &mut self,
        metrics: &[CommandMetrics],
    ) -> HashMap<String, CommandAggregates> {
        let mut aggregates: HashMap<String, CommandAggregates> = HashMap::new();

        for metric in metrics {
            let agg = aggregates
                .entry(metric.command_name.clone())
                .or_insert_with(|| CommandAggregates::new(metric.command_name.clone()));

            self.update_aggregate(agg, metric);
        }

        // Calculate percentiles for each command
        for (command_name, agg) in aggregates.iter_mut() {
            if let Some(cache) = self.percentile_cache.get(command_name) {
                agg.p50_duration_ms = cache.p50;
                agg.p95_duration_ms = cache.p95;
                agg.p99_duration_ms = cache.p99;
            }
        }

        aggregates
    }

    /// Update aggregate with new metric
    fn update_aggregate(&mut self, agg: &mut CommandAggregates, metric: &CommandMetrics) {
        agg.total_executions += 1;
        agg.last_executed = metric.started_at;

        if metric.success {
            agg.successful_executions += 1;
        } else {
            agg.failed_executions += 1;

            if let Some(ref error) = metric.error {
                let error_type = categorize_error(error);
                *agg.error_distribution.entry(error_type).or_insert(0) += 1;
            }
        }

        agg.total_items_processed += metric.items_processed;
        agg.total_bytes_transferred += metric.bytes_transferred;
        agg.total_api_calls += metric.api_calls;

        // Update running averages
        if let Some(duration_ms) = metric.duration_ms {
            let duration_f64 = duration_ms as f64;
            update_running_avg(&mut agg.avg_duration_ms, duration_f64, agg.total_executions);

            // Cache duration for percentile calculation
            self.add_duration_to_cache(&agg.command, duration_f64);
        }

        // Update cache hit rate
        let cache_hits = metric.cache_hits;
        let total_items = metric.items_processed.max(1);
        let hit_rate = cache_hits as f64 / total_items as f64;
        update_running_avg(&mut agg.cache_hit_rate, hit_rate, agg.total_executions);

        // Update memory average
        update_running_avg(
            &mut agg.avg_memory_bytes,
            metric.peak_memory_bytes as f64,
            agg.total_executions,
        );
    }

    /// Add duration to cache for percentile calculation
    fn add_duration_to_cache(&mut self, command: &str, duration: f64) {
        let cache = self
            .percentile_cache
            .entry(command.to_string())
            .or_insert_with(|| PercentileCache {
                durations: Vec::new(),
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
                last_updated: std::time::Instant::now(),
            });

        cache.durations.push(duration);

        // Keep only last 1000 durations to avoid unbounded growth
        if cache.durations.len() > 1000 {
            cache.durations.drain(0..(cache.durations.len() - 1000));
        }

        // Recalculate percentiles every 10 entries or every 60 seconds
        let should_recalc = cache.durations.len().is_multiple_of(10)
            || cache.last_updated.elapsed().as_secs() >= 60;

        if should_recalc {
            let (p50, p95, p99) = calculate_percentiles(&cache.durations);
            cache.p50 = p50;
            cache.p95 = p95;
            cache.p99 = p99;
            cache.last_updated = std::time::Instant::now();
        }
    }

    /// Calculate percentiles for metric points
    pub fn calculate_metric_percentiles(&self, points: &[MetricPoint]) -> (f64, f64, f64) {
        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        calculate_percentiles(&values)
    }

    /// Calculate moving average over time window
    pub fn calculate_moving_average(&self, points: &[MetricPoint], window_size: usize) -> Vec<f64> {
        if points.is_empty() || window_size == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(points.len());

        for i in 0..points.len() {
            let start = if i >= window_size {
                i - window_size + 1
            } else {
                0
            };

            let window = &points[start..=i];
            let sum: f64 = window.iter().map(|p| p.value).sum();
            let avg = sum / window.len() as f64;
            result.push(avg);
        }

        result
    }

    /// Detect anomalies using simple z-score method
    pub fn detect_anomalies(&self, points: &[MetricPoint], threshold: f64) -> Vec<usize> {
        if points.len() < 3 {
            return Vec::new();
        }

        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let mut anomalies = Vec::new();

        for (i, value) in values.iter().enumerate() {
            let z_score = ((value - mean) / std_dev).abs();
            if z_score > threshold {
                anomalies.push(i);
            }
        }

        anomalies
    }

    /// Calculate rate of change between consecutive points
    pub fn calculate_rate_of_change(&self, points: &[MetricPoint]) -> Vec<f64> {
        if points.len() < 2 {
            return Vec::new();
        }

        let mut rates = Vec::with_capacity(points.len() - 1);

        for i in 1..points.len() {
            let prev = &points[i - 1];
            let curr = &points[i];

            let time_diff = (curr.timestamp.timestamp_millis() - prev.timestamp.timestamp_millis())
                as f64
                / 1000.0; // Convert to seconds

            if time_diff > 0.0 {
                let value_diff = curr.value - prev.value;
                let rate = value_diff / time_diff;
                rates.push(rate);
            } else {
                rates.push(0.0);
            }
        }

        rates
    }

    /// Group metrics by time bucket (e.g., hourly, daily)
    pub fn group_by_time_bucket(
        &self,
        metrics: &[CommandMetrics],
        bucket_hours: i64,
    ) -> HashMap<String, Vec<CommandMetrics>> {
        let mut buckets: HashMap<String, Vec<CommandMetrics>> = HashMap::new();

        for metric in metrics {
            let timestamp = metric.started_at.timestamp();
            let bucket_timestamp = (timestamp / (bucket_hours * 3600)) * (bucket_hours * 3600);
            let bucket_key = chrono::DateTime::from_timestamp(bucket_timestamp, 0)
                .unwrap_or(metric.started_at)
                .to_rfc3339();

            buckets.entry(bucket_key).or_default().push(metric.clone());
        }

        buckets
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate P50, P95, and P99 percentiles from a set of values
fn calculate_percentiles(values: &[f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p50 = percentile(&sorted, 50.0);
    let p95 = percentile(&sorted, 95.0);
    let p99 = percentile(&sorted, 99.0);

    (p50, p95, p99)
}

/// Calculate a specific percentile from sorted values
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let idx = (p / 100.0 * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[idx.min(sorted_values.len() - 1)]
}

/// Update running average with new value
fn update_running_avg(current: &mut f64, new_value: f64, count: u64) {
    if count == 0 {
        *current = new_value;
        return;
    }

    let weight = count as f64;
    let current_weight = (weight - 1.0) / weight;
    let new_weight = 1.0 / weight;
    *current = *current * current_weight + new_value * new_weight;
}

/// Categorize error message into error type
#[allow(dead_code)]
fn categorize_error(error: &str) -> String {
    let error_lower = error.to_lowercase();

    if error_lower.contains("timeout") || error_lower.contains("timed out") {
        "timeout".to_string()
    } else if error_lower.contains("network") || error_lower.contains("connection") {
        "network".to_string()
    } else if error_lower.contains("permission") || error_lower.contains("denied") {
        "permission".to_string()
    } else if error_lower.contains("not found") || error_lower.contains("404") {
        "not_found".to_string()
    } else if error_lower.contains("parse") || error_lower.contains("invalid") {
        "parse".to_string()
    } else if error_lower.contains("rate limit") || error_lower.contains("too many") {
        "rate_limit".to_string()
    } else if error_lower.contains("auth") || error_lower.contains("unauthorized") {
        "authentication".to_string()
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    fn create_test_metric(command: &str, duration_ms: u64, success: bool) -> CommandMetrics {
        let mut metric = CommandMetrics::new(command);
        if success {
            metric.complete(Duration::from_millis(duration_ms));
        } else {
            metric.fail(Duration::from_millis(duration_ms), "test error");
        }
        metric.items_processed = 10;
        metric.cache_hits = 5;
        metric
    }

    #[test]
    fn test_percentile_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let (p50, p95, p99) = calculate_percentiles(&values);

        // P50 formula: (p/100) * (len - 1) = 0.5 * 9 = 4.5 rounds to index 5 (or 4), value = 5.0 or 6.0
        assert!((p50 - 5.0).abs() < 1.5); // P50 should be around 5.0-6.0
        assert!(p95 >= 9.0); // P95 should be high
        assert!(p99 >= 9.0); // P99 should be very high
    }

    #[test]
    fn test_aggregation() {
        let mut aggregator = MetricsAggregator::new();

        let metrics = vec![
            create_test_metric("extract", 100, true),
            create_test_metric("extract", 200, true),
            create_test_metric("extract", 150, false),
            create_test_metric("crawl", 300, true),
        ];

        let aggregates = aggregator.aggregate_by_command(&metrics);

        assert_eq!(aggregates.len(), 2);

        let extract_stats = &aggregates["extract"];
        assert_eq!(extract_stats.total_executions, 3);
        assert_eq!(extract_stats.successful_executions, 2);
        assert_eq!(extract_stats.failed_executions, 1);
        assert!(extract_stats.avg_duration_ms > 0.0);
    }

    #[test]
    fn test_moving_average() {
        let aggregator = MetricsAggregator::new();

        let points = vec![
            MetricPoint::new(10.0),
            MetricPoint::new(20.0),
            MetricPoint::new(30.0),
            MetricPoint::new(40.0),
        ];

        let ma = aggregator.calculate_moving_average(&points, 2);

        assert_eq!(ma.len(), 4);
        assert_eq!(ma[0], 10.0); // First point
        assert_eq!(ma[1], 15.0); // (10 + 20) / 2
        assert_eq!(ma[2], 25.0); // (20 + 30) / 2
        assert_eq!(ma[3], 35.0); // (30 + 40) / 2
    }

    #[test]
    fn test_anomaly_detection() {
        let aggregator = MetricsAggregator::new();

        let points = vec![
            MetricPoint::new(10.0),
            MetricPoint::new(11.0),
            MetricPoint::new(12.0),
            MetricPoint::new(100.0), // Anomaly
            MetricPoint::new(11.0),
            MetricPoint::new(10.0),
        ];

        let anomalies = aggregator.detect_anomalies(&points, 2.0);

        assert!(!anomalies.is_empty());
        assert!(anomalies.contains(&3)); // Index 3 should be detected
    }

    #[test]
    fn test_rate_of_change() {
        let aggregator = MetricsAggregator::new();

        let mut points = vec![
            MetricPoint::new(0.0),
            MetricPoint::new(10.0),
            MetricPoint::new(25.0),
        ];

        // Manually set timestamps for predictable rate calculation
        let base_time = Utc::now();
        points[0].timestamp = base_time;
        points[1].timestamp = base_time + chrono::Duration::seconds(1);
        points[2].timestamp = base_time + chrono::Duration::seconds(2);

        let rates = aggregator.calculate_rate_of_change(&points);

        assert_eq!(rates.len(), 2);
        assert!(rates[0] > 0.0); // Positive rate
        assert!(rates[1] > rates[0]); // Increasing rate
    }

    #[test]
    fn test_error_categorization() {
        assert_eq!(categorize_error("Connection timeout"), "timeout");
        assert_eq!(categorize_error("Network error"), "network");
        assert_eq!(categorize_error("Permission denied"), "permission");
        assert_eq!(categorize_error("404 not found"), "not_found");
        assert_eq!(categorize_error("Rate limit exceeded"), "rate_limit");
        assert_eq!(categorize_error("Unauthorized access"), "authentication");
        assert_eq!(categorize_error("Something weird"), "unknown");
    }
}
