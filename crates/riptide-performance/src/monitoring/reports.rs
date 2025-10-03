//! Monitoring report types

use super::alerts::PerformanceAlert;
use crate::PerformanceMetrics;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Monitoring session report
#[derive(Debug, Serialize, Deserialize)]
pub struct MonitoringReport {
    pub session_id: Uuid,
    pub monitoring_duration: Duration,
    pub total_samples: usize,
    pub metrics: PerformanceMetrics,
    pub alerts: Vec<PerformanceAlert>,
    pub system_summary: SystemSummary,
    pub application_summary: ApplicationSummary,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// System metrics summary
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSummary {
    pub avg_cpu_usage: f64,
    pub peak_cpu_usage: f64,
    pub avg_memory_mb: f64,
    pub peak_memory_mb: f64,
    pub total_disk_io_mb: f64,
    pub total_network_io_mb: f64,
    pub uptime: Duration,
}

/// Application metrics summary
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationSummary {
    pub total_requests_processed: u64,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub peak_concurrent_requests: u32,
    pub cache_efficiency: f64,
    pub ai_processing_efficiency: f64,
}
