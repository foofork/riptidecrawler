//! Golden Test Framework for RipTide EventMesh
//!
//! This module provides a comprehensive golden test framework for safe refactoring.
//! It captures system behavior before code changes and validates consistency after.
//!
//! ## Features
//! - Behavior capture and verification
//! - Performance baseline tracking
//! - Maximum 5% regression enforcement
//! - Memory usage monitoring (RSS ≤600MB)
//! - Latency metrics (p50, p95, p99)
//! - Throughput validation

pub mod behavior_capture;
pub mod performance_baseline;
pub mod regression_guard;
pub mod memory_monitor;
pub mod golden_runner;

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

/// Core golden test framework configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTestConfig {
    /// Maximum allowed performance regression (default: 5%)
    pub max_regression_percent: f64,
    /// Memory limit in bytes (default: 600MB)
    pub memory_limit_bytes: u64,
    /// Test timeout in seconds
    pub timeout_seconds: u64,
    /// Number of warmup iterations
    pub warmup_iterations: usize,
    /// Number of measurement iterations
    pub measurement_iterations: usize,
    /// Enable detailed logging
    pub verbose: bool,
}

impl Default for GoldenTestConfig {
    fn default() -> Self {
        Self {
            max_regression_percent: 5.0,
            memory_limit_bytes: 600 * 1024 * 1024, // 600MB
            timeout_seconds: 300, // 5 minutes
            warmup_iterations: 5,
            measurement_iterations: 10,
            verbose: false,
        }
    }
}

/// Represents captured system behavior for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSnapshot {
    pub timestamp: u64,
    pub test_name: String,
    pub performance_metrics: PerformanceMetrics,
    pub memory_metrics: MemoryMetrics,
    pub throughput_metrics: ThroughputMetrics,
    pub functional_outputs: HashMap<String, serde_json::Value>,
    pub error_patterns: Vec<String>,
}

/// Performance metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub mean_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub std_dev_ms: f64,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub rss_bytes: u64,
    pub heap_bytes: u64,
    pub peak_rss_bytes: u64,
    pub memory_efficiency: f64,
}

/// Throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub pages_per_second: f64,
    pub requests_per_second: f64,
    pub bytes_processed_per_second: u64,
    pub operations_per_second: f64,
}

/// Golden test result with comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTestResult {
    pub test_name: String,
    pub passed: bool,
    pub baseline_snapshot: BehaviorSnapshot,
    pub current_snapshot: BehaviorSnapshot,
    pub performance_delta: PerformanceDelta,
    pub memory_delta: MemoryDelta,
    pub functional_diffs: Vec<FunctionalDiff>,
    pub violations: Vec<Violation>,
}

/// Performance comparison delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDelta {
    pub p50_change_percent: f64,
    pub p95_change_percent: f64,
    pub p99_change_percent: f64,
    pub regression_detected: bool,
}

/// Memory usage comparison delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDelta {
    pub rss_change_percent: f64,
    pub heap_change_percent: f64,
    pub limit_exceeded: bool,
}

/// Functional behavior difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalDiff {
    pub field_path: String,
    pub baseline_value: serde_json::Value,
    pub current_value: serde_json::Value,
    pub diff_type: DiffType,
}

/// Type of difference detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffType {
    ValueChanged,
    FieldAdded,
    FieldRemoved,
    TypeChanged,
}

/// Test violation detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub violation_type: ViolationType,
    pub description: String,
    pub severity: Severity,
    pub measured_value: f64,
    pub threshold_value: f64,
}

/// Types of violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    PerformanceRegression,
    MemoryLimit,
    TimeoutExceeded,
    FunctionalChange,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

/// Main golden test framework entry point
pub struct GoldenTestFramework {
    config: GoldenTestConfig,
    baseline_storage: BaselineStorage,
}

impl GoldenTestFramework {
    pub fn new(config: GoldenTestConfig) -> Self {
        Self {
            baseline_storage: BaselineStorage::new(),
            config,
        }
    }

    /// Capture current system behavior as baseline
    pub async fn capture_baseline<F, T>(
        &mut self,
        test_name: &str,
        test_fn: F,
    ) -> Result<BehaviorSnapshot, anyhow::Error>
    where
        F: Fn() -> T + Send + 'static,
        T: std::future::Future<Output = Result<serde_json::Value, anyhow::Error>> + Send,
    {
        behavior_capture::capture_behavior(
            test_name,
            test_fn,
            &self.config,
        ).await
    }

    /// Run golden test with comparison against baseline
    pub async fn run_golden_test<F, T>(
        &mut self,
        test_name: &str,
        test_fn: F,
    ) -> Result<GoldenTestResult, anyhow::Error>
    where
        F: Fn() -> T + Send + 'static,
        T: std::future::Future<Output = Result<serde_json::Value, anyhow::Error>> + Send,
    {
        golden_runner::run_test(
            test_name,
            test_fn,
            &self.config,
            &mut self.baseline_storage,
        ).await
    }

    /// Save baseline to persistent storage
    pub async fn save_baseline(
        &mut self,
        test_name: &str,
        snapshot: &BehaviorSnapshot,
    ) -> Result<(), anyhow::Error> {
        self.baseline_storage.save_baseline(test_name, snapshot).await
    }

    /// Load baseline from persistent storage
    pub async fn load_baseline(
        &mut self,
        test_name: &str,
    ) -> Result<Option<BehaviorSnapshot>, anyhow::Error> {
        self.baseline_storage.load_baseline(test_name).await
    }
}

/// Storage interface for baselines
pub struct BaselineStorage {
    storage_path: std::path::PathBuf,
}

impl BaselineStorage {
    pub fn new() -> Self {
        Self {
            storage_path: std::path::PathBuf::from("tests/benchmarks/baselines.json"),
        }
    }

    pub async fn save_baseline(
        &mut self,
        test_name: &str,
        snapshot: &BehaviorSnapshot,
    ) -> Result<(), anyhow::Error> {
        performance_baseline::save_baseline(&self.storage_path, test_name, snapshot).await
    }

    pub async fn load_baseline(
        &mut self,
        test_name: &str,
    ) -> Result<Option<BehaviorSnapshot>, anyhow::Error> {
        performance_baseline::load_baseline(&self.storage_path, test_name).await
    }

    pub async fn list_baselines(&self) -> Result<Vec<String>, anyhow::Error> {
        performance_baseline::list_baselines(&self.storage_path).await
    }
}

/// Utility functions for test assertions
pub mod assertions {
    use super::*;

    pub fn assert_performance_within_threshold(
        baseline: &PerformanceMetrics,
        current: &PerformanceMetrics,
        max_regression_percent: f64,
    ) -> Result<(), String> {
        let p50_change = ((current.p50_latency_ms - baseline.p50_latency_ms) / baseline.p50_latency_ms) * 100.0;
        let p95_change = ((current.p95_latency_ms - baseline.p95_latency_ms) / baseline.p95_latency_ms) * 100.0;
        
        if p50_change > max_regression_percent {
            return Err(format!(
                "P50 latency regression detected: {:.2}% > {:.2}% threshold",
                p50_change, max_regression_percent
            ));
        }
        
        if p95_change > max_regression_percent {
            return Err(format!(
                "P95 latency regression detected: {:.2}% > {:.2}% threshold",
                p95_change, max_regression_percent
            ));
        }
        
        Ok(())
    }

    pub fn assert_memory_within_limit(
        memory: &MemoryMetrics,
        limit_bytes: u64,
    ) -> Result<(), String> {
        if memory.rss_bytes > limit_bytes {
            return Err(format!(
                "Memory limit exceeded: {} > {} bytes",
                memory.rss_bytes, limit_bytes
            ));
        }
        Ok(())
    }
}
