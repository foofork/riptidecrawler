//! System and application metrics types

use serde::{Deserialize, Serialize};

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage_percent: f64,
    pub memory_rss_mb: f64,
    pub memory_heap_mb: f64,
    pub memory_virtual_mb: f64,
    pub disk_read_mbps: f64,
    pub disk_write_mbps: f64,
    pub network_in_mbps: f64,
    pub network_out_mbps: f64,
    pub open_file_descriptors: u32,
    pub thread_count: u32,
}

/// Application-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub active_requests: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub cache_size_mb: f64,
    pub ai_processing_queue_size: u32,
    pub ai_avg_processing_time_ms: f64,
}
