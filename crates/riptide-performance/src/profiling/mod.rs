//! Memory profiling and analysis module
//!
//! This module provides comprehensive memory profiling capabilities including:
//! - Real-time memory usage tracking
//! - Memory leak detection
//! - Allocation pattern analysis
//! - Memory growth prediction
//! - Detailed memory reports

pub mod allocation_analyzer;
pub mod flamegraph_generator;
pub mod leak_detector;
pub mod memory_tracker;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use allocation_analyzer::AllocationAnalyzer;
use flamegraph_generator::FlamegraphGenerator;
use leak_detector::LeakDetector;
use memory_tracker::MemoryTracker;

/// Memory profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfileConfig {
    /// Sampling interval for memory measurements
    pub sampling_interval: Duration,
    /// Maximum number of samples to keep in memory
    pub max_samples: usize,
    /// Enable detailed allocation tracking
    pub track_allocations: bool,
    /// Enable leak detection
    pub detect_leaks: bool,
    /// Generate flamegraphs
    pub generate_flamegraphs: bool,
    /// Memory threshold for warnings (MB)
    pub warning_threshold_mb: f64,
    /// Memory threshold for alerts (MB)
    pub alert_threshold_mb: f64,
}

impl Default for MemoryProfileConfig {
    fn default() -> Self {
        Self {
            sampling_interval: Duration::from_secs(5),
            max_samples: 1000,
            track_allocations: true,
            detect_leaks: true,
            generate_flamegraphs: false, // Expensive, disabled by default
            warning_threshold_mb: 650.0,
            alert_threshold_mb: 700.0,
        }
    }
}

/// Memory usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub rss_bytes: u64,
    pub heap_bytes: u64,
    pub virtual_bytes: u64,
    pub resident_bytes: u64,
    pub shared_bytes: u64,
    pub text_bytes: u64,
    pub data_bytes: u64,
    pub stack_bytes: u64,
}

/// Memory allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub size: usize,
    pub alignment: usize,
    pub stack_trace: Vec<String>,
    pub component: String,
    pub operation: String,
}

/// Memory leak detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakAnalysis {
    pub potential_leaks: Vec<LeakInfo>,
    pub growth_rate_mb_per_hour: f64,
    pub largest_allocations: Vec<AllocationInfo>,
    pub suspicious_patterns: Vec<String>,
}

/// Information about a potential memory leak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakInfo {
    pub component: String,
    pub allocation_count: u64,
    pub total_size_bytes: u64,
    pub average_size_bytes: f64,
    pub growth_rate: f64,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Comprehensive memory analysis report
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryReport {
    pub session_id: Uuid,
    pub profiling_duration: Duration,
    pub total_samples: usize,

    // Summary statistics
    pub peak_memory_mb: f64,
    pub average_memory_mb: f64,
    pub memory_growth_rate_mb_s: f64,
    pub memory_efficiency_score: f64,

    // Detailed analysis
    pub snapshots: Vec<MemorySnapshot>,
    pub leak_analysis: LeakAnalysis,
    pub top_allocators: Vec<(String, u64)>,
    pub memory_timeline: Vec<(chrono::DateTime<chrono::Utc>, f64)>,

    // Recommendations
    pub recommendations: Vec<String>,
    pub flamegraph_path: Option<String>,
}

/// Main memory profiler
pub struct MemoryProfiler {
    config: MemoryProfileConfig,
    session_id: Uuid,
    start_time: Option<Instant>,

    tracker: Arc<RwLock<MemoryTracker>>,
    leak_detector: Arc<RwLock<LeakDetector>>,
    allocation_analyzer: Arc<RwLock<AllocationAnalyzer>>,
    flamegraph_generator: Option<Arc<RwLock<FlamegraphGenerator>>>,

    snapshots: Arc<RwLock<Vec<MemorySnapshot>>>,
    is_profiling: Arc<RwLock<bool>>,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new(session_id: Uuid) -> Result<Self> {
        Self::with_config(session_id, MemoryProfileConfig::default())
    }

    /// Create a new memory profiler with custom configuration
    pub fn with_config(session_id: Uuid, config: MemoryProfileConfig) -> Result<Self> {
        info!(
            session_id = %session_id,
            "Creating memory profiler with config: {:?}",
            config
        );

        let flamegraph_generator = if config.generate_flamegraphs {
            Some(Arc::new(RwLock::new(FlamegraphGenerator::new(session_id)?)))
        } else {
            None
        };

        Ok(Self {
            config,
            session_id,
            start_time: None,
            tracker: Arc::new(RwLock::new(MemoryTracker::new()?)),
            leak_detector: Arc::new(RwLock::new(LeakDetector::new()?)),
            allocation_analyzer: Arc::new(RwLock::new(AllocationAnalyzer::new()?)),
            flamegraph_generator,
            snapshots: Arc::new(RwLock::new(Vec::new())),
            is_profiling: Arc::new(RwLock::new(false)),
        })
    }

    /// Start memory profiling
    pub async fn start_profiling(&mut self) -> Result<()> {
        let mut is_profiling = self.is_profiling.write().await;
        if *is_profiling {
            warn!(session_id = %self.session_id, "Memory profiling already started");
            return Ok(());
        }

        info!(session_id = %self.session_id, "Starting memory profiling");

        self.start_time = Some(Instant::now());
        *is_profiling = true;

        // Start the background profiling task
        self.start_profiling_task().await?;

        // Start flamegraph generation if enabled
        if let Some(ref flamegraph) = self.flamegraph_generator {
            let mut fg = flamegraph.write().await;
            fg.start_recording().await?;
        }

        info!(session_id = %self.session_id, "Memory profiling started successfully");
        Ok(())
    }

    /// Stop memory profiling and generate report
    pub async fn stop_profiling(&mut self) -> Result<MemoryReport> {
        let mut is_profiling = self.is_profiling.write().await;
        if !*is_profiling {
            warn!(session_id = %self.session_id, "Memory profiling not running");
            return Err(anyhow::anyhow!("Memory profiling not running"));
        }

        info!(session_id = %self.session_id, "Stopping memory profiling");

        *is_profiling = false;

        let profiling_duration = self
            .start_time
            .map(|start| start.elapsed())
            .unwrap_or_default();

        // Generate flamegraph if enabled
        let flamegraph_path = if let Some(ref flamegraph) = self.flamegraph_generator {
            let mut fg = flamegraph.write().await;
            Some(fg.stop_recording_and_generate().await?)
        } else {
            None
        };

        // Collect final data and generate report
        let report = self
            .generate_report(profiling_duration, flamegraph_path)
            .await?;

        info!(
            session_id = %self.session_id,
            duration_ms = profiling_duration.as_millis(),
            "Memory profiling stopped successfully"
        );

        Ok(report)
    }

    /// Get current memory snapshot
    pub async fn get_current_snapshot(&self) -> Result<MemorySnapshot> {
        let tracker = self.tracker.read().await;
        tracker.get_current_snapshot().await
    }

    /// Get memory usage trend
    pub async fn get_memory_trend(
        &self,
        duration: Duration,
    ) -> Result<Vec<(chrono::DateTime<chrono::Utc>, f64)>> {
        let snapshots = self.snapshots.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(duration)?;

        Ok(snapshots
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .map(|s| (s.timestamp, s.rss_bytes as f64 / 1024.0 / 1024.0))
            .collect())
    }

    /// Check if memory usage is within thresholds
    pub async fn check_memory_thresholds(&self) -> Result<Vec<String>> {
        let snapshot = self.get_current_snapshot().await?;
        let memory_mb = snapshot.rss_bytes as f64 / 1024.0 / 1024.0;

        let mut alerts = Vec::new();

        if memory_mb > self.config.alert_threshold_mb {
            alerts.push(format!(
                "CRITICAL: Memory usage {:.1}MB exceeds alert threshold {:.1}MB",
                memory_mb, self.config.alert_threshold_mb
            ));
        } else if memory_mb > self.config.warning_threshold_mb {
            alerts.push(format!(
                "WARNING: Memory usage {:.1}MB exceeds warning threshold {:.1}MB",
                memory_mb, self.config.warning_threshold_mb
            ));
        }

        Ok(alerts)
    }

    /// Start the background profiling task
    async fn start_profiling_task(&self) -> Result<()> {
        let tracker = Arc::clone(&self.tracker);
        let snapshots = Arc::clone(&self.snapshots);
        let is_profiling = Arc::clone(&self.is_profiling);
        let sampling_interval = self.config.sampling_interval;
        let max_samples = self.config.max_samples;
        let session_id = self.session_id;

        tokio::spawn(async move {
            debug!(session_id = %session_id, "Starting memory profiling background task");

            while *is_profiling.read().await {
                if let Ok(snapshot) = {
                    let tracker = tracker.read().await;
                    tracker.get_current_snapshot().await
                } {
                    let mut snapshots = snapshots.write().await;
                    snapshots.push(snapshot);

                    // Keep only the most recent samples
                    if snapshots.len() > max_samples {
                        let excess = snapshots.len() - max_samples;
                        snapshots.drain(0..excess);
                    }
                }

                tokio::time::sleep(sampling_interval).await;
            }

            debug!(session_id = %session_id, "Memory profiling background task stopped");
        });

        Ok(())
    }

    /// Generate comprehensive memory report
    async fn generate_report(
        &self,
        profiling_duration: Duration,
        flamegraph_path: Option<String>,
    ) -> Result<MemoryReport> {
        let snapshots = self.snapshots.read().await;
        let total_samples = snapshots.len();

        if snapshots.is_empty() {
            return Err(anyhow::anyhow!("No memory samples collected"));
        }

        // Calculate summary statistics
        let memory_values: Vec<f64> = snapshots
            .iter()
            .map(|s| s.rss_bytes as f64 / 1024.0 / 1024.0)
            .collect();

        let peak_memory_mb = memory_values.iter().fold(0.0_f64, |max, &val| max.max(val));

        let average_memory_mb = memory_values.iter().sum::<f64>() / memory_values.len() as f64;

        // Calculate growth rate
        let memory_growth_rate_mb_s = if snapshots.len() >= 2 {
            let first = &snapshots[0];
            let last = &snapshots[snapshots.len() - 1];
            let time_diff = (last.timestamp - first.timestamp).num_seconds() as f64;
            let memory_diff = (last.rss_bytes as f64 - first.rss_bytes as f64) / 1024.0 / 1024.0;

            if time_diff > 0.0 {
                memory_diff / time_diff
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate efficiency score (inverse of growth rate and peak usage)
        let memory_efficiency_score = if peak_memory_mb > 0.0 && memory_growth_rate_mb_s.abs() < 1.0
        {
            (600.0_f64 / peak_memory_mb).min(1.0)
                * (1.0 - memory_growth_rate_mb_s.abs() / 10.0).max(0.0)
        } else {
            0.0
        };

        // Generate leak analysis
        let leak_detector = self.leak_detector.read().await;
        let leak_analysis = leak_detector.analyze_leaks().await?;

        // Get top allocators
        let allocation_analyzer = self.allocation_analyzer.read().await;
        let top_allocators = allocation_analyzer.get_top_allocators().await?;

        // Generate memory timeline
        let memory_timeline: Vec<(chrono::DateTime<chrono::Utc>, f64)> = snapshots
            .iter()
            .map(|s| (s.timestamp, s.rss_bytes as f64 / 1024.0 / 1024.0))
            .collect();

        // Generate recommendations
        let recommendations = self
            .generate_recommendations(peak_memory_mb, memory_growth_rate_mb_s, &leak_analysis)
            .await?;

        Ok(MemoryReport {
            session_id: self.session_id,
            profiling_duration,
            total_samples,
            peak_memory_mb,
            average_memory_mb,
            memory_growth_rate_mb_s,
            memory_efficiency_score,
            snapshots: snapshots.clone(),
            leak_analysis,
            top_allocators,
            memory_timeline,
            recommendations,
            flamegraph_path,
        })
    }

    /// Generate optimization recommendations
    async fn generate_recommendations(
        &self,
        peak_memory_mb: f64,
        growth_rate_mb_s: f64,
        leak_analysis: &LeakAnalysis,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Memory usage recommendations
        if peak_memory_mb > 500.0 {
            recommendations.push(
                "High memory usage detected. Consider implementing memory pooling or reducing cache sizes.".to_string()
            );
        }

        if peak_memory_mb > 600.0 {
            recommendations.push(
                "Memory usage approaching limit. Implement aggressive garbage collection or increase system limits.".to_string()
            );
        }

        // Growth rate recommendations
        if growth_rate_mb_s > 1.0 {
            recommendations.push(
                "High memory growth rate detected. Check for memory leaks and implement proper cleanup.".to_string()
            );
        }

        if growth_rate_mb_s > 5.0 {
            recommendations.push(
                "Critical memory growth rate. Immediate investigation required for memory leaks."
                    .to_string(),
            );
        }

        // Leak-specific recommendations
        if !leak_analysis.potential_leaks.is_empty() {
            recommendations.push(format!(
                "Found {} potential memory leaks. Focus on: {}",
                leak_analysis.potential_leaks.len(),
                leak_analysis
                    .potential_leaks
                    .iter()
                    .take(3)
                    .map(|leak| leak.component.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if leak_analysis.growth_rate_mb_per_hour > 50.0 {
            recommendations.push(
                "Memory leak growth rate is high. Implement automatic memory cleanup routines."
                    .to_string(),
            );
        }

        // Pattern-based recommendations
        for pattern in &leak_analysis.suspicious_patterns {
            recommendations.push(format!("Suspicious pattern detected: {}", pattern));
        }

        // General recommendations
        if recommendations.is_empty() {
            recommendations
                .push("Memory usage is within normal parameters. Continue monitoring.".to_string());
        } else {
            recommendations
                .push("Consider implementing jemalloc for better memory management.".to_string());
            recommendations.push(
                "Enable memory profiling in production with sampling to reduce overhead."
                    .to_string(),
            );
        }

        Ok(recommendations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_profiler_creation() {
        let session_id = Uuid::new_v4();
        let profiler = MemoryProfiler::new(session_id).unwrap();
        assert_eq!(profiler.session_id, session_id);
    }

    #[tokio::test]
    async fn test_memory_snapshot() {
        let session_id = Uuid::new_v4();
        let profiler = MemoryProfiler::new(session_id).unwrap();
        let snapshot = profiler.get_current_snapshot().await.unwrap();
        assert!(snapshot.rss_bytes > 0);
    }

    #[tokio::test]
    async fn test_threshold_checking() {
        let session_id = Uuid::new_v4();
        let config = MemoryProfileConfig {
            warning_threshold_mb: 1.0, // Very low threshold for testing
            alert_threshold_mb: 2.0,
            ..Default::default()
        };

        let profiler = MemoryProfiler::with_config(session_id, config).unwrap();
        let alerts = profiler.check_memory_thresholds().await.unwrap();

        // Should trigger warning since RSS is likely > 1MB
        assert!(!alerts.is_empty());
    }
}
