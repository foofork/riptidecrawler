//! Performance reporting and analysis

use crate::monitoring::{
    collector::MetricsCollector,
    error::{LockManager, Result},
    health::HealthCalculator,
    metrics::PerformanceMetrics,
    time_series::TimeSeriesBuffer,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub current_metrics: PerformanceMetrics,
    pub avg_extraction_time: f64,
    pub p95_extraction_time: f64,
    pub p99_extraction_time: f64,
    pub peak_memory_usage: f64,
    pub error_count: usize,
    pub health_summary: String,
    pub recommendations: Vec<String>,
    pub trend_analysis: TrendAnalysis,
}

/// Trend analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub extraction_time_trend: TrendDirection,
    pub error_rate_trend: TrendDirection,
    pub memory_usage_trend: TrendDirection,
    pub request_rate_trend: TrendDirection,
    pub health_score_trend: TrendDirection,
}

/// Trend direction indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

/// Report generator for creating comprehensive performance reports
pub struct ReportGenerator {
    health_calculator: HealthCalculator,
}

impl ReportGenerator {
    /// Create a new report generator
    #[must_use]
    pub fn new(health_calculator: HealthCalculator) -> Self {
        Self { health_calculator }
    }

    /// Generate a performance report
    ///
    /// # Errors
    /// Returns error if metrics cannot be retrieved or locks cannot be acquired
    pub async fn generate_report(
        &self,
        collector: &MetricsCollector,
        extraction_times: &Arc<Mutex<TimeSeriesBuffer>>,
        error_rates: &Arc<Mutex<TimeSeriesBuffer>>,
        memory_usage: &Arc<Mutex<TimeSeriesBuffer>>,
        request_rates: &Arc<Mutex<TimeSeriesBuffer>>,
        duration: Duration,
    ) -> Result<PerformanceReport> {
        // Get current metrics
        let current_metrics = collector.get_current_metrics().await?;

        // Calculate extraction time statistics
        let times = LockManager::acquire_mutex(extraction_times, "extraction_times")?;
        let avg_extraction_time = times.calculate_average(duration).unwrap_or(0.0);
        let p95_extraction_time = times.calculate_percentile(95.0, duration).unwrap_or(0.0);
        let p99_extraction_time = times.calculate_percentile(99.0, duration).unwrap_or(0.0);

        // Calculate memory statistics
        let memory = LockManager::acquire_mutex(memory_usage, "memory_usage")?;
        let peak_memory_usage = memory
            .calculate_min_max(duration)
            .map(|(_, max)| max)
            .unwrap_or(0.0);

        // Calculate error statistics
        let errors = LockManager::acquire_mutex(error_rates, "error_rates")?;
        let error_count = errors.get_recent_data(duration).len();

        // Generate health summary and recommendations
        let health_summary = self
            .health_calculator
            .generate_health_summary(&current_metrics);
        let recommendations = self
            .health_calculator
            .generate_recommendations(&current_metrics);

        // Analyze trends
        let trend_analysis = self.analyze_trends(
            &times,
            &errors,
            &memory,
            &*LockManager::acquire_mutex(request_rates, "request_rates")?,
            duration,
        );

        Ok(PerformanceReport {
            current_metrics,
            avg_extraction_time,
            p95_extraction_time,
            p99_extraction_time,
            peak_memory_usage,
            error_count,
            health_summary,
            recommendations,
            trend_analysis,
        })
    }

    /// Analyze performance trends
    fn analyze_trends(
        &self,
        extraction_times: &TimeSeriesBuffer,
        error_rates: &TimeSeriesBuffer,
        memory_usage: &TimeSeriesBuffer,
        request_rates: &TimeSeriesBuffer,
        duration: Duration,
    ) -> TrendAnalysis {
        TrendAnalysis {
            extraction_time_trend: self.calculate_trend(extraction_times, duration),
            error_rate_trend: self.calculate_trend(error_rates, duration),
            memory_usage_trend: self.calculate_trend(memory_usage, duration),
            request_rate_trend: self.calculate_trend_inverted(request_rates, duration), // Higher is better
            health_score_trend: TrendDirection::Unknown, // Would need historical health scores
        }
    }

    /// Calculate trend direction (lower values are better)
    fn calculate_trend(&self, buffer: &TimeSeriesBuffer, duration: Duration) -> TrendDirection {
        let _half_duration = duration / 2;

        // Get all data for the full period
        let all_data = buffer.get_recent_data(duration);
        if all_data.len() < 2 {
            return TrendDirection::Unknown;
        }

        // Split data into first half and second half
        let midpoint = all_data.len() / 2;
        let first_half_data = &all_data[..midpoint];
        let second_half_data = &all_data[midpoint..];

        if first_half_data.is_empty() || second_half_data.is_empty() {
            return TrendDirection::Unknown;
        }

        let first_half: f64 = first_half_data.iter().map(|p| p.value).sum::<f64>();
        #[allow(clippy::cast_precision_loss)]
        let first_half_avg = first_half / first_half_data.len() as f64;

        let second_half: f64 = second_half_data.iter().map(|p| p.value).sum::<f64>();
        #[allow(clippy::cast_precision_loss)]
        let second_half_avg = second_half / second_half_data.len() as f64;

        let change_percent =
            ((second_half_avg - first_half_avg) / first_half_avg.abs().max(0.001_f64)) * 100.0;

        if change_percent < -5.0 {
            TrendDirection::Improving // Values decreasing
        } else if change_percent > 5.0 {
            TrendDirection::Degrading // Values increasing
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate trend direction (higher values are better)
    fn calculate_trend_inverted(
        &self,
        buffer: &TimeSeriesBuffer,
        duration: Duration,
    ) -> TrendDirection {
        match self.calculate_trend(buffer, duration) {
            TrendDirection::Improving => TrendDirection::Degrading,
            TrendDirection::Degrading => TrendDirection::Improving,
            other => other,
        }
    }
}

/// Summary statistics for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSummary {
    pub name: String,
    pub current_value: f64,
    pub average: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub trend: TrendDirection,
}

impl MetricSummary {
    /// Create a metric summary from time series data
    #[must_use]
    pub fn from_time_series(
        name: String,
        current_value: f64,
        buffer: &TimeSeriesBuffer,
        duration: Duration,
    ) -> Self {
        let (min, max) = buffer.calculate_min_max(duration).unwrap_or((0.0, 0.0));

        Self {
            name,
            current_value,
            average: buffer.calculate_average(duration).unwrap_or(0.0),
            minimum: min,
            maximum: max,
            p50: buffer.calculate_percentile(50.0, duration).unwrap_or(0.0),
            p95: buffer.calculate_percentile(95.0, duration).unwrap_or(0.0),
            p99: buffer.calculate_percentile(99.0, duration).unwrap_or(0.0),
            trend: TrendDirection::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::metrics::HealthThresholds;
    use std::collections::HashMap;

    #[test]
    fn test_trend_analysis() {
        let mut buffer = TimeSeriesBuffer::new(100, Duration::from_secs(3600));

        // Add improving trend data (decreasing values)
        // Add more data points to have a proper first half and second half
        for i in (0..20).rev() {
            buffer.add_point(f64::from(i) * 10.0, HashMap::new());
        }

        let generator = ReportGenerator::new(HealthCalculator::new(HealthThresholds::default()));
        let trend = generator.calculate_trend(&buffer, Duration::from_secs(3600));

        match trend {
            TrendDirection::Improving => {} // Expected
            _ => panic!("Expected improving trend, got: {:?}", trend),
        }
    }

    #[test]
    fn test_metric_summary() {
        let mut buffer = TimeSeriesBuffer::new(100, Duration::from_secs(3600));

        // Add test data
        for i in 0..100 {
            buffer.add_point(f64::from(i), HashMap::new());
        }

        let summary = MetricSummary::from_time_series(
            "test_metric".to_string(),
            99.0,
            &buffer,
            Duration::from_secs(3600),
        );

        assert_eq!(summary.current_value, 99.0);
        assert!(summary.average > 0.0);
        assert!(summary.p99 > summary.p50);
        assert!(summary.maximum >= summary.minimum);
    }
}
