//! # RipTide Performance Optimization Suite
//!
//! This crate provides comprehensive performance monitoring, profiling, and optimization
//! capabilities for the RipTide web scraping framework.
//!
//! ## Features
//!
//! - **Memory Profiling**: Track memory usage patterns and identify leaks
//! - **Bottleneck Analysis**: Identify performance bottlenecks and suggest optimizations
//! - **Cache Optimization**: Multi-layer caching with intelligent eviction
//! - **Resource Limits**: Enforce resource constraints and prevent abuse
//! - **Real-time Monitoring**: Live performance dashboards and alerts
//!
//! ## Performance Targets
//!
//! - Latency: p50 ≤1.5s, p95 ≤5s
//! - Memory: ≤600MB RSS (alert at 650MB)
//! - Throughput: ≥70 pages/sec with AI
//! - AI Impact: ≤30% throughput reduction

pub mod profiling;
pub mod benchmarks;
pub mod monitoring;
pub mod optimization;
pub mod limits;

use thiserror::Error;

/// Performance-related errors
#[derive(Error, Debug)]
pub enum PerformanceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Profiling error: {0}")]
    ProfilingError(String),

    #[error("Monitoring error: {0}")]
    MonitoringError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, PerformanceError>;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Performance targets for the RipTide system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Maximum p50 latency in milliseconds
    pub p50_latency_ms: u64,
    /// Maximum p95 latency in milliseconds
    pub p95_latency_ms: u64,
    /// Maximum RSS memory in MB
    pub max_memory_mb: u64,
    /// Memory alert threshold in MB
    pub memory_alert_mb: u64,
    /// Minimum throughput in pages per second
    pub min_throughput_pps: f64,
    /// Maximum AI processing overhead percentage
    pub max_ai_overhead_percent: f64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            p50_latency_ms: 1500,   // 1.5s
            p95_latency_ms: 5000,   // 5s
            max_memory_mb: 600,     // 600MB
            memory_alert_mb: 650,   // 650MB alert
            min_throughput_pps: 70.0, // 70 pages/sec
            max_ai_overhead_percent: 30.0, // 30% max AI impact
        }
    }
}

/// Performance metrics collected during operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub session_id: Uuid,

    // Latency metrics
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub avg_latency_ms: f64,

    // Memory metrics
    pub memory_rss_mb: f64,
    pub memory_heap_mb: f64,
    pub memory_virtual_mb: f64,
    pub memory_growth_rate_mb_s: f64,

    // Throughput metrics
    pub throughput_pps: f64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_requests: u64,

    // AI processing metrics
    pub ai_processing_time_ms: f64,
    pub ai_overhead_percent: f64,
    pub ai_cache_hit_rate: f64,

    // Resource utilization
    pub cpu_usage_percent: f64,
    pub network_io_mbps: f64,
    pub disk_io_mbps: f64,

    // Cache metrics
    pub cache_hit_rate: f64,
    pub cache_size_mb: f64,
    pub cache_evictions: u64,
}

/// Main performance manager
pub struct PerformanceManager {
    targets: PerformanceTargets,
    profiler: RwLock<profiling::MemoryProfiler>,
    monitor: RwLock<monitoring::PerformanceMonitor>,
    optimizer: RwLock<optimization::CacheOptimizer>,
    limiter: RwLock<limits::ResourceLimiter>,
    session_id: Uuid,
}

impl PerformanceManager {
    /// Create a new performance manager with default targets
    pub fn new() -> Result<Self> {
        Self::with_targets(PerformanceTargets::default())
    }

    /// Create a new performance manager with custom targets
    pub fn with_targets(targets: PerformanceTargets) -> Result<Self> {
        let session_id = Uuid::new_v4();

        info!(
            session_id = %session_id,
            "Initializing performance manager with targets: {:?}",
            targets
        );

        Ok(Self {
            targets: targets.clone(),
            profiler: RwLock::new(profiling::MemoryProfiler::new(session_id)?),
            monitor: RwLock::new(monitoring::PerformanceMonitor::new(targets.clone())?),
            optimizer: RwLock::new(optimization::CacheOptimizer::new()?),
            limiter: RwLock::new(limits::ResourceLimiter::new(targets)?),
            session_id,
        })
    }

    /// Start performance monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        info!(session_id = %self.session_id, "Starting performance monitoring");

        let mut monitor = self.monitor.write().await;
        monitor.start().await?;

        let mut profiler = self.profiler.write().await;
        profiler.start_profiling().await?;

        info!(session_id = %self.session_id, "Performance monitoring started");
        Ok(())
    }

    /// Stop performance monitoring and generate report
    pub async fn stop_monitoring(&self) -> Result<PerformanceReport> {
        info!(session_id = %self.session_id, "Stopping performance monitoring");

        let mut monitor = self.monitor.write().await;
        let monitor_report = monitor.stop().await?;

        let mut profiler = self.profiler.write().await;
        let profile_report = profiler.stop_profiling().await?;

        let metrics_clone = monitor_report.metrics.clone();
        let report = PerformanceReport {
            session_id: self.session_id,
            targets: self.targets.clone(),
            metrics: monitor_report.metrics,
            memory_analysis: profile_report,
            recommendations: self.generate_recommendations(&metrics_clone).await?,
            timestamp: chrono::Utc::now(),
        };

        info!(
            session_id = %self.session_id,
            "Performance monitoring stopped, report generated"
        );

        Ok(report)
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> Result<PerformanceMetrics> {
        let monitor = self.monitor.read().await;
        monitor.get_current_metrics().await
    }

    /// Check if performance targets are being met
    pub async fn check_targets(&self) -> Result<TargetStatus> {
        let metrics = self.get_metrics().await?;

        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Check latency targets
        if metrics.latency_p50_ms > self.targets.p50_latency_ms as f64 {
            violations.push(format!(
                "P50 latency {}ms exceeds target {}ms",
                metrics.latency_p50_ms, self.targets.p50_latency_ms
            ));
        }

        if metrics.latency_p95_ms > self.targets.p95_latency_ms as f64 {
            violations.push(format!(
                "P95 latency {}ms exceeds target {}ms",
                metrics.latency_p95_ms, self.targets.p95_latency_ms
            ));
        }

        // Check memory targets
        if metrics.memory_rss_mb > self.targets.max_memory_mb as f64 {
            violations.push(format!(
                "Memory usage {}MB exceeds target {}MB",
                metrics.memory_rss_mb, self.targets.max_memory_mb
            ));
        } else if metrics.memory_rss_mb > self.targets.memory_alert_mb as f64 {
            warnings.push(format!(
                "Memory usage {}MB approaching limit {}MB",
                metrics.memory_rss_mb, self.targets.max_memory_mb
            ));
        }

        // Check throughput targets
        if metrics.throughput_pps < self.targets.min_throughput_pps {
            violations.push(format!(
                "Throughput {:.1} PPS below target {:.1} PPS",
                metrics.throughput_pps, self.targets.min_throughput_pps
            ));
        }

        // Check AI overhead
        if metrics.ai_overhead_percent > self.targets.max_ai_overhead_percent {
            violations.push(format!(
                "AI overhead {:.1}% exceeds target {:.1}%",
                metrics.ai_overhead_percent, self.targets.max_ai_overhead_percent
            ));
        }

        Ok(TargetStatus {
            all_targets_met: violations.is_empty(),
            violations,
            warnings,
            metrics,
        })
    }

    /// Generate optimization recommendations based on metrics
    async fn generate_recommendations(&self, metrics: &PerformanceMetrics) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Memory recommendations
        if metrics.memory_rss_mb > 500.0 {
            recommendations.push("Consider implementing more aggressive memory cleanup".to_string());
        }

        if metrics.memory_growth_rate_mb_s > 1.0 {
            recommendations.push("Memory growth rate is high, check for memory leaks".to_string());
        }

        // Cache recommendations
        if metrics.cache_hit_rate < 0.8 {
            recommendations.push("Cache hit rate is low, consider cache warming or size increase".to_string());
        }

        // Throughput recommendations
        if metrics.throughput_pps < 50.0 {
            recommendations.push("Low throughput detected, consider scaling or optimization".to_string());
        }

        // AI processing recommendations
        if metrics.ai_overhead_percent > 25.0 {
            recommendations.push("High AI overhead, consider batching or caching AI results".to_string());
        }

        Ok(recommendations)
    }
}

/// Performance monitoring report
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub session_id: Uuid,
    pub targets: PerformanceTargets,
    pub metrics: PerformanceMetrics,
    pub memory_analysis: profiling::MemoryReport,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Status of performance target compliance
#[derive(Debug, Serialize, Deserialize)]
pub struct TargetStatus {
    pub all_targets_met: bool,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
    pub metrics: PerformanceMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_manager_creation() {
        let manager = PerformanceManager::new().unwrap();
        assert_eq!(manager.targets.p50_latency_ms, 1500);
        assert_eq!(manager.targets.max_memory_mb, 600);
    }

    #[tokio::test]
    async fn test_custom_targets() {
        let targets = PerformanceTargets {
            p50_latency_ms: 1000,
            p95_latency_ms: 3000,
            max_memory_mb: 400,
            memory_alert_mb: 450,
            min_throughput_pps: 100.0,
            max_ai_overhead_percent: 20.0,
        };

        let manager = PerformanceManager::with_targets(targets).unwrap();
        assert_eq!(manager.targets.p50_latency_ms, 1000);
        assert_eq!(manager.targets.max_memory_mb, 400);
    }
}