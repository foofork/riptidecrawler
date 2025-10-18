//! Core metrics data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

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

impl MetricDataPoint {
    pub fn new(value: f64) -> Self {
        Self {
            timestamp: Instant::now(),
            timestamp_utc: Utc::now(),
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(value: f64, metadata: HashMap<String, String>) -> Self {
        Self {
            timestamp: Instant::now(),
            timestamp_utc: Utc::now(),
            value,
            metadata,
        }
    }
}

/// Configuration for monitoring thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub collection_interval_secs: u64,
    pub retention_period_hours: u64,
    pub max_data_points: usize,
    pub health_thresholds: HealthThresholds,
    pub alert_cooldown_secs: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            collection_interval_secs: 30,
            retention_period_hours: 24,
            max_data_points: 10000,
            health_thresholds: HealthThresholds::default(),
            alert_cooldown_secs: 300, // 5 minutes
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    pub error_rate_warning: f64,
    pub error_rate_critical: f64,
    pub cpu_usage_warning: f32,
    pub cpu_usage_critical: f32,
    pub memory_usage_warning: u64,
    pub memory_usage_critical: u64,
    pub extraction_time_warning_ms: f64,
    pub extraction_time_critical_ms: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            error_rate_warning: 5.0,
            error_rate_critical: 10.0,
            cpu_usage_warning: 70.0,
            cpu_usage_critical: 90.0,
            memory_usage_warning: 1024 * 1024 * 1024 * 2, // 2GB
            memory_usage_critical: 1024 * 1024 * 1024 * 4, // 4GB
            extraction_time_warning_ms: 5000.0,
            extraction_time_critical_ms: 10000.0,
        }
    }
}
