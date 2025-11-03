//! Benchmarking and performance testing module
//!
//! This module provides comprehensive benchmarking capabilities for the RipTide system,
//! including automated performance tests, regression detection, and comparative analysis.

pub mod extraction_benchmark;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::info;
use uuid::Uuid;

use crate::utils::safe_conversions::{
    calculate_percentile_index, count_to_f64_divisor, u128_nanos_to_u64,
};

// Re-export extraction benchmark types
pub use extraction_benchmark::{
    BenchmarkStatistics, ComparativeBenchmarkReport, ExtractionBenchmark,
    ExtractionBenchmarkRunner, ExtractionEngine,
};

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of iterations to run for each benchmark
    pub iterations: u32,
    /// Warmup iterations before actual benchmarking
    pub warmup_iterations: u32,
    /// Maximum time to spend on a single benchmark
    pub max_duration: Duration,
    /// Minimum time to spend on a single benchmark for accuracy
    pub min_duration: Duration,
    /// Target confidence level for statistical analysis
    pub confidence_level: f64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            max_duration: Duration::from_secs(300), // 5 minutes max
            min_duration: Duration::from_secs(1),   // 1 second min
            confidence_level: 0.95,
        }
    }
}

/// Individual benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: u32,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub median_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub throughput_ops_sec: f64,
    pub memory_peak_mb: f64,
    pub memory_average_mb: f64,
    pub cpu_usage_percent: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Benchmark suite results
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub session_id: Uuid,
    pub suite_name: String,
    pub config: BenchmarkConfig,
    pub results: HashMap<String, BenchmarkResult>,
    pub total_duration: Duration,
    pub baseline_comparison: Option<HashMap<String, f64>>, // Performance delta vs baseline
    pub performance_score: f64,
    pub recommendations: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    session_id: Uuid,
    baseline_results: Option<HashMap<String, BenchmarkResult>>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(session_id: Uuid) -> Self {
        Self::with_config(session_id, BenchmarkConfig::default())
    }

    /// Create a new benchmark runner with custom configuration
    pub fn with_config(session_id: Uuid, config: BenchmarkConfig) -> Self {
        Self {
            config,
            session_id,
            baseline_results: None,
        }
    }

    /// Load baseline results for comparison
    pub fn load_baseline(&mut self, baseline: HashMap<String, BenchmarkResult>) {
        info!(
            session_id = %self.session_id,
            "Loaded baseline with {} benchmarks",
            baseline.len()
        );
        self.baseline_results = Some(baseline);
    }

    /// Run a complete benchmark suite
    pub async fn run_suite(&self, suite_name: &str) -> Result<BenchmarkSuite> {
        info!(
            session_id = %self.session_id,
            suite_name = suite_name,
            "Starting benchmark suite"
        );

        let start_time = Instant::now();
        let mut results = HashMap::new();

        // Run individual benchmarks
        results.insert(
            "scraping_latency".to_string(),
            self.benchmark_scraping_latency().await?,
        );
        results.insert(
            "memory_efficiency".to_string(),
            self.benchmark_memory_efficiency().await?,
        );
        results.insert(
            "html_parsing".to_string(),
            self.benchmark_html_parsing().await?,
        );
        results.insert(
            "ai_processing".to_string(),
            self.benchmark_ai_processing().await?,
        );
        results.insert(
            "cache_performance".to_string(),
            self.benchmark_cache_performance().await?,
        );
        results.insert(
            "concurrent_processing".to_string(),
            self.benchmark_concurrent_processing().await?,
        );

        let total_duration = start_time.elapsed();

        // Calculate baseline comparison if available
        let baseline_comparison = self.baseline_results.as_ref().map(|baseline| {
            results
                .iter()
                .map(|(name, result)| {
                    let delta = if let Some(baseline_result) = baseline.get(name) {
                        // Calculate percentage change in performance (negative = regression)
                        let baseline_avg = baseline_result.average_duration.as_nanos() as f64;
                        let current_avg = result.average_duration.as_nanos() as f64;
                        if baseline_avg > 0.0 {
                            ((baseline_avg - current_avg) / baseline_avg) * 100.0
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    (name.clone(), delta)
                })
                .collect()
        });

        // Calculate overall performance score
        let performance_score = self.calculate_performance_score(&results);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&results, &baseline_comparison);

        let suite = BenchmarkSuite {
            session_id: self.session_id,
            suite_name: suite_name.to_string(),
            config: self.config.clone(),
            results,
            total_duration,
            baseline_comparison,
            performance_score,
            recommendations,
            timestamp: chrono::Utc::now(),
        };

        info!(
            session_id = %self.session_id,
            suite_name = suite_name,
            duration_ms = total_duration.as_millis(),
            performance_score = performance_score,
            "Benchmark suite completed"
        );

        Ok(suite)
    }

    /// Benchmark scraping latency
    async fn benchmark_scraping_latency(&self) -> Result<BenchmarkResult> {
        info!(session_id = %self.session_id, "Running scraping latency benchmark");

        let mut durations = Vec::new();
        let mut memory_samples = Vec::new();
        let start_time = Instant::now();

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = self.simulate_scraping_operation().await;
        }

        // Actual benchmarking
        for _ in 0..self.config.iterations {
            let iter_start = Instant::now();
            let memory_before = self.get_memory_usage().await;

            // Simulate scraping operation
            self.simulate_scraping_operation().await?;

            let iter_duration = iter_start.elapsed();
            let memory_after = self.get_memory_usage().await;

            durations.push(iter_duration);
            memory_samples.push((memory_before, memory_after));
        }

        self.calculate_benchmark_result(
            "scraping_latency",
            durations,
            memory_samples,
            start_time.elapsed(),
        )
    }

    /// Benchmark memory efficiency
    async fn benchmark_memory_efficiency(&self) -> Result<BenchmarkResult> {
        info!(session_id = %self.session_id, "Running memory efficiency benchmark");

        let mut durations = Vec::new();
        let mut memory_samples = Vec::new();
        let start_time = Instant::now();

        for _ in 0..self.config.iterations {
            let iter_start = Instant::now();
            let memory_before = self.get_memory_usage().await;

            // Simulate memory-intensive operation
            self.simulate_memory_intensive_operation().await?;

            let iter_duration = iter_start.elapsed();
            let memory_after = self.get_memory_usage().await;

            durations.push(iter_duration);
            memory_samples.push((memory_before, memory_after));
        }

        self.calculate_benchmark_result(
            "memory_efficiency",
            durations,
            memory_samples,
            start_time.elapsed(),
        )
    }

    /// Benchmark HTML parsing performance
    async fn benchmark_html_parsing(&self) -> Result<BenchmarkResult> {
        info!(session_id = %self.session_id, "Running HTML parsing benchmark");

        let mut durations = Vec::new();
        let mut memory_samples = Vec::new();
        let start_time = Instant::now();

        for _ in 0..self.config.iterations {
            let iter_start = Instant::now();
            let memory_before = self.get_memory_usage().await;

            // Simulate HTML parsing operation
            self.simulate_html_parsing_operation().await?;

            let iter_duration = iter_start.elapsed();
            let memory_after = self.get_memory_usage().await;

            durations.push(iter_duration);
            memory_samples.push((memory_before, memory_after));
        }

        self.calculate_benchmark_result(
            "html_parsing",
            durations,
            memory_samples,
            start_time.elapsed(),
        )
    }

    /// Benchmark AI processing performance
    async fn benchmark_ai_processing(&self) -> Result<BenchmarkResult> {
        info!(session_id = %self.session_id, "Running AI processing benchmark");

        let mut durations = Vec::new();
        let mut memory_samples = Vec::new();
        let start_time = Instant::now();

        for _ in 0..self.config.iterations {
            let iter_start = Instant::now();
            let memory_before = self.get_memory_usage().await;

            // Simulate AI processing operation
            self.simulate_ai_processing_operation().await?;

            let iter_duration = iter_start.elapsed();
            let memory_after = self.get_memory_usage().await;

            durations.push(iter_duration);
            memory_samples.push((memory_before, memory_after));
        }

        self.calculate_benchmark_result(
            "ai_processing",
            durations,
            memory_samples,
            start_time.elapsed(),
        )
    }

    /// Benchmark cache performance
    async fn benchmark_cache_performance(&self) -> Result<BenchmarkResult> {
        info!(session_id = %self.session_id, "Running cache performance benchmark");

        let mut durations = Vec::new();
        let mut memory_samples = Vec::new();
        let start_time = Instant::now();

        for _ in 0..self.config.iterations {
            let iter_start = Instant::now();
            let memory_before = self.get_memory_usage().await;

            // Simulate cache operations
            self.simulate_cache_operations().await?;

            let iter_duration = iter_start.elapsed();
            let memory_after = self.get_memory_usage().await;

            durations.push(iter_duration);
            memory_samples.push((memory_before, memory_after));
        }

        self.calculate_benchmark_result(
            "cache_performance",
            durations,
            memory_samples,
            start_time.elapsed(),
        )
    }

    /// Benchmark concurrent processing
    async fn benchmark_concurrent_processing(&self) -> Result<BenchmarkResult> {
        info!(session_id = %self.session_id, "Running concurrent processing benchmark");

        let mut durations = Vec::new();
        let mut memory_samples = Vec::new();
        let start_time = Instant::now();

        for _ in 0..self.config.iterations {
            let iter_start = Instant::now();
            let memory_before = self.get_memory_usage().await;

            // Simulate concurrent operations
            self.simulate_concurrent_operations().await?;

            let iter_duration = iter_start.elapsed();
            let memory_after = self.get_memory_usage().await;

            durations.push(iter_duration);
            memory_samples.push((memory_before, memory_after));
        }

        self.calculate_benchmark_result(
            "concurrent_processing",
            durations,
            memory_samples,
            start_time.elapsed(),
        )
    }

    /// Calculate benchmark result from timing data
    fn calculate_benchmark_result(
        &self,
        name: &str,
        mut durations: Vec<Duration>,
        memory_samples: Vec<(f64, f64)>,
        total_duration: Duration,
    ) -> Result<BenchmarkResult> {
        durations.sort();

        let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
        let average_duration = if durations.is_empty() {
            Duration::from_nanos(0)
        } else {
            Duration::from_nanos(u128_nanos_to_u64(total_nanos / durations.len() as u128))
        };
        let min_duration = durations.first().copied().unwrap_or_default();
        let max_duration = durations.last().copied().unwrap_or_default();
        let median_duration = durations
            .get(durations.len() / 2)
            .copied()
            .unwrap_or_default();
        let p95_index = calculate_percentile_index(durations.len(), 0.95);
        let p99_index = calculate_percentile_index(durations.len(), 0.99);
        let p95_duration = durations.get(p95_index).copied().unwrap_or_default();
        let p99_duration = durations.get(p99_index).copied().unwrap_or_default();

        let throughput_ops_sec = durations.len() as f64 / total_duration.as_secs_f64();

        let memory_peak_mb = memory_samples
            .iter()
            .map(|(_, after)| *after)
            .fold(0.0, f64::max);

        let memory_average_mb = memory_samples.iter().map(|(_, after)| *after).sum::<f64>()
            / count_to_f64_divisor(memory_samples.len());

        // Simulate CPU usage (would be measured in real implementation)
        let cpu_usage_percent = 45.0; // Placeholder

        Ok(BenchmarkResult {
            name: name.to_string(),
            iterations: self.config.iterations,
            total_duration,
            average_duration,
            min_duration,
            max_duration,
            median_duration,
            p95_duration,
            p99_duration,
            throughput_ops_sec,
            memory_peak_mb,
            memory_average_mb,
            cpu_usage_percent,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Calculate overall performance score
    fn calculate_performance_score(&self, results: &HashMap<String, BenchmarkResult>) -> f64 {
        let mut score: f64 = 100.0;

        for result in results.values() {
            // Penalize high latency
            if result.average_duration > Duration::from_millis(2000) {
                score -= 20.0;
            } else if result.average_duration > Duration::from_millis(1000) {
                score -= 10.0;
            }

            // Penalize high memory usage
            if result.memory_peak_mb > 600.0 {
                score -= 15.0;
            } else if result.memory_peak_mb > 400.0 {
                score -= 5.0;
            }

            // Penalize low throughput
            if result.throughput_ops_sec < 50.0 {
                score -= 10.0;
            } else if result.throughput_ops_sec < 100.0 {
                score -= 5.0;
            }
        }

        score.max(0.0_f64)
    }

    /// Generate performance recommendations
    fn generate_recommendations(
        &self,
        results: &HashMap<String, BenchmarkResult>,
        baseline_comparison: &Option<HashMap<String, f64>>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Analyze individual benchmark results
        for (name, result) in results {
            if result.average_duration > Duration::from_millis(1500) {
                recommendations.push(format!(
                    "High latency in {}: {:.2}ms average. Consider optimization.",
                    name,
                    result.average_duration.as_millis()
                ));
            }

            if result.memory_peak_mb > 500.0 {
                recommendations.push(format!(
                    "High memory usage in {}: {:.1}MB peak. Consider memory optimization.",
                    name, result.memory_peak_mb
                ));
            }

            if result.throughput_ops_sec < 70.0 {
                recommendations.push(format!(
                    "Low throughput in {}: {:.1} ops/sec. Consider parallelization.",
                    name, result.throughput_ops_sec
                ));
            }
        }

        // Analyze baseline comparison if available
        if let Some(comparison) = baseline_comparison {
            for (name, delta) in comparison {
                if *delta < -10.0 {
                    recommendations.push(format!(
                        "Performance regression in {}: {:.1}% slower than baseline.",
                        name,
                        delta.abs()
                    ));
                } else if *delta > 10.0 {
                    recommendations.push(format!(
                        "Performance improvement in {}: {:.1}% faster than baseline.",
                        name, *delta
                    ));
                }
            }
        }

        if recommendations.is_empty() {
            recommendations.push("All benchmarks performing within acceptable ranges.".to_string());
        }

        recommendations
    }

    // Simulation methods (would be replaced with actual operations in real implementation)
    async fn simulate_scraping_operation(&self) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    async fn simulate_memory_intensive_operation(&self) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(5)).await;
        Ok(())
    }

    async fn simulate_html_parsing_operation(&self) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(8)).await;
        Ok(())
    }

    async fn simulate_ai_processing_operation(&self) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(15)).await;
        Ok(())
    }

    async fn simulate_cache_operations(&self) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(2)).await;
        Ok(())
    }

    async fn simulate_concurrent_operations(&self) -> Result<()> {
        tokio::time::sleep(Duration::from_millis(12)).await;
        Ok(())
    }

    async fn get_memory_usage(&self) -> f64 {
        // Simulate memory usage measurement
        150.0 + (rand::random::<f64>() * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_runner_creation() {
        let session_id = Uuid::new_v4();
        let runner = BenchmarkRunner::new(session_id);
        assert_eq!(runner.session_id, session_id);
    }

    #[tokio::test]
    async fn test_benchmark_suite_execution() {
        let session_id = Uuid::new_v4();
        let config = BenchmarkConfig {
            iterations: 5, // Reduce for faster testing
            warmup_iterations: 2,
            ..Default::default()
        };

        let runner = BenchmarkRunner::with_config(session_id, config);
        let suite = runner.run_suite("test_suite").await.unwrap();

        assert_eq!(suite.session_id, session_id);
        assert_eq!(suite.suite_name, "test_suite");
        assert!(!suite.results.is_empty());
        assert!(suite.performance_score >= 0.0 && suite.performance_score <= 100.0);
    }
}
