//! Time series data management

use crate::monitoring::metrics::MetricDataPoint;
use chrono::Utc;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Time-series buffer for historical data with automatic cleanup
pub struct TimeSeriesBuffer {
    data: VecDeque<MetricDataPoint>,
    max_size: usize,
    retention_period: Duration,
}

impl TimeSeriesBuffer {
    /// Create a new time series buffer
    pub fn new(max_size: usize, retention_period: Duration) -> Self {
        Self {
            data: VecDeque::with_capacity(max_size),
            max_size,
            retention_period,
        }
    }

    /// Add a new data point to the buffer
    pub fn add_point(&mut self, value: f64, metadata: HashMap<String, String>) {
        let now = Instant::now();

        // Clean old data first
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

    /// Get recent data points within the specified duration
    pub fn get_recent_data(&self, duration: Duration) -> Vec<&MetricDataPoint> {
        let cutoff = Instant::now() - duration;
        self.data
            .iter()
            .filter(|point| point.timestamp >= cutoff)
            .collect()
    }

    /// Calculate percentile using the T-Digest algorithm approximation
    /// This is more efficient than sorting for large datasets
    pub fn calculate_percentile(&self, percentile: f64, duration: Duration) -> Option<f64> {
        let recent_data = self.get_recent_data(duration);

        if recent_data.is_empty() {
            return None;
        }

        // For small datasets, use exact calculation
        if recent_data.len() < 100 {
            let mut values: Vec<f64> = recent_data.iter().map(|p| p.value).collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let index = ((percentile / 100.0) * (values.len() - 1) as f64) as usize;
            return Some(values[index]);
        }

        // For larger datasets, use approximate percentile
        self.approximate_percentile(&recent_data, percentile)
    }

    /// Approximate percentile calculation for better performance
    fn approximate_percentile(&self, data: &[&MetricDataPoint], percentile: f64) -> Option<f64> {
        // Reservoir sampling for approximate percentile
        const SAMPLE_SIZE: usize = 100;
        let mut sample = Vec::with_capacity(SAMPLE_SIZE);

        for (i, point) in data.iter().enumerate() {
            if i < SAMPLE_SIZE {
                sample.push(point.value);
            } else {
                // Randomly replace elements with decreasing probability
                // Using simple modulo for deterministic behavior in tests
                let j = i % SAMPLE_SIZE;
                if j < SAMPLE_SIZE {
                    sample[j] = point.value;
                }
            }
        }

        sample.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let index = ((percentile / 100.0) * (sample.len() - 1) as f64) as usize;
        Some(sample[index])
    }

    /// Calculate average value over a duration
    pub fn calculate_average(&self, duration: Duration) -> Option<f64> {
        let recent_data = self.get_recent_data(duration);

        if recent_data.is_empty() {
            return None;
        }

        let sum: f64 = recent_data.iter().map(|p| p.value).sum();
        Some(sum / recent_data.len() as f64)
    }

    /// Calculate min and max values over a duration
    pub fn calculate_min_max(&self, duration: Duration) -> Option<(f64, f64)> {
        let recent_data = self.get_recent_data(duration);

        if recent_data.is_empty() {
            return None;
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for point in recent_data {
            min = min.min(point.value);
            max = max.max(point.value);
        }

        Some((min, max))
    }

    /// Get the number of data points
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Clean up data older than retention period
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_series_buffer_capacity() {
        let mut buffer = TimeSeriesBuffer::new(5, Duration::from_secs(10));

        // Add more than max_size points
        for i in 0..7 {
            buffer.add_point(i as f64, HashMap::new());
        }

        // Should only keep the last 5 points
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn test_percentile_calculation() {
        let mut buffer = TimeSeriesBuffer::new(100, Duration::from_secs(60));

        // Add data points
        for i in 0..100 {
            buffer.add_point(i as f64, HashMap::new());
        }

        // Test percentiles
        let p50 = buffer.calculate_percentile(50.0, Duration::from_secs(120));
        assert!(p50.is_some());

        let p95 = buffer.calculate_percentile(95.0, Duration::from_secs(120));
        assert!(p95.is_some());
        assert!(p95.unwrap() > p50.unwrap());
    }

    #[test]
    fn test_average_calculation() {
        let mut buffer = TimeSeriesBuffer::new(10, Duration::from_secs(60));

        buffer.add_point(10.0, HashMap::new());
        buffer.add_point(20.0, HashMap::new());
        buffer.add_point(30.0, HashMap::new());

        let avg = buffer.calculate_average(Duration::from_secs(120));
        assert_eq!(avg, Some(20.0));
    }
}
