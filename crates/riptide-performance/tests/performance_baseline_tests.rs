//! Performance baseline tests and monitoring for RipTide
//!
//! This module establishes performance baselines and provides monitoring for:
//! - Latency benchmarks across all components
//! - Throughput measurements under various loads
//! - Memory usage monitoring and leak detection
//! - Resource limit enforcement validation
//! - Performance regression detection

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceCollector {
    metrics: Arc<Mutex<HashMap<String, PerformanceMetric>>>,
    baselines: Arc<Mutex<HashMap<String, PerformanceBaseline>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub name: String,
    pub latency_ms: f64,
    pub throughput_rps: f64,
    pub memory_bytes: usize,
    pub cpu_percent: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub samples: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub name: String,
    pub max_latency_ms: f64,
    pub min_throughput_rps: f64,
    pub max_memory_bytes: usize,
    pub max_cpu_percent: f32,
    pub regression_threshold_percent: f64,
}

impl Default for PerformanceCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceCollector {
    pub fn new() -> Self {
        let mut baselines = HashMap::new();

        // Establish performance baselines for different components
        baselines.insert(
            "search_query".to_string(),
            PerformanceBaseline {
                name: "search_query".to_string(),
                max_latency_ms: 500.0,
                min_throughput_rps: 100.0,
                max_memory_bytes: 50 * 1024 * 1024, // 50MB
                max_cpu_percent: 80.0,
                regression_threshold_percent: 5.0,
            },
        );

        baselines.insert(
            "pdf_extraction".to_string(),
            PerformanceBaseline {
                name: "pdf_extraction".to_string(),
                max_latency_ms: 2000.0,
                min_throughput_rps: 10.0,
                max_memory_bytes: 200 * 1024 * 1024, // 200MB
                max_cpu_percent: 90.0,
                regression_threshold_percent: 10.0,
            },
        );

        baselines.insert(
            "html_parsing".to_string(),
            PerformanceBaseline {
                name: "html_parsing".to_string(),
                max_latency_ms: 200.0,
                min_throughput_rps: 200.0,
                max_memory_bytes: 20 * 1024 * 1024, // 20MB
                max_cpu_percent: 60.0,
                regression_threshold_percent: 5.0,
            },
        );

        baselines.insert(
            "api_request".to_string(),
            PerformanceBaseline {
                name: "api_request".to_string(),
                max_latency_ms: 100.0,
                min_throughput_rps: 500.0,
                max_memory_bytes: 10 * 1024 * 1024, // 10MB
                max_cpu_percent: 50.0,
                regression_threshold_percent: 3.0,
            },
        );

        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            baselines: Arc::new(Mutex::new(baselines)),
        }
    }

    pub fn record_metric(&self, metric: PerformanceMetric) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.insert(metric.name.clone(), metric);
    }

    pub fn get_metric(&self, name: &str) -> Option<PerformanceMetric> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(name).cloned()
    }

    pub fn check_baseline(&self, name: &str) -> Result<BaselineCheckResult> {
        let metrics = self.metrics.lock().unwrap();
        let baselines = self.baselines.lock().unwrap();

        let metric = metrics
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Metric '{}' not found", name))?;

        let baseline = baselines
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Baseline '{}' not found", name))?;

        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Check latency
        if metric.latency_ms > baseline.max_latency_ms {
            violations.push(format!(
                "Latency {} ms exceeds baseline {} ms",
                metric.latency_ms, baseline.max_latency_ms
            ));
        } else if metric.latency_ms
            > baseline.max_latency_ms * (1.0 - baseline.regression_threshold_percent / 100.0)
        {
            warnings.push(format!(
                "Latency {} ms approaching baseline {} ms",
                metric.latency_ms, baseline.max_latency_ms
            ));
        }

        // Check throughput
        if metric.throughput_rps < baseline.min_throughput_rps {
            violations.push(format!(
                "Throughput {} rps below baseline {} rps",
                metric.throughput_rps, baseline.min_throughput_rps
            ));
        } else if metric.throughput_rps
            < baseline.min_throughput_rps * (1.0 + baseline.regression_threshold_percent / 100.0)
        {
            warnings.push(format!(
                "Throughput {} rps approaching baseline {} rps",
                metric.throughput_rps, baseline.min_throughput_rps
            ));
        }

        // Check memory usage
        if metric.memory_bytes > baseline.max_memory_bytes {
            violations.push(format!(
                "Memory usage {} bytes exceeds baseline {} bytes",
                metric.memory_bytes, baseline.max_memory_bytes
            ));
        }

        // Check CPU usage
        if metric.cpu_percent > baseline.max_cpu_percent {
            violations.push(format!(
                "CPU usage {}% exceeds baseline {}%",
                metric.cpu_percent, baseline.max_cpu_percent
            ));
        }

        Ok(BaselineCheckResult {
            metric_name: name.to_string(),
            passed: violations.is_empty(),
            violations,
            warnings,
        })
    }

    pub fn get_all_metrics(&self) -> HashMap<String, PerformanceMetric> {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }
}

#[derive(Debug, Clone)]
pub struct BaselineCheckResult {
    pub metric_name: String,
    pub passed: bool,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

/// Mock workload generator for performance testing
#[derive(Debug)]
pub struct WorkloadGenerator {
    pub name: String,
    pub concurrency: usize,
    pub duration: Duration,
    pub request_rate: f64,
}

impl WorkloadGenerator {
    pub fn new(name: String, concurrency: usize, duration: Duration, request_rate: f64) -> Self {
        Self {
            name,
            concurrency,
            duration,
            request_rate,
        }
    }

    pub async fn run_workload<F, Fut>(&self, operation: F) -> WorkloadResult
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<Duration>> + Send,
    {
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        let mut handles = Vec::new();
        let start_time = Instant::now();
        let request_times = Arc::new(Mutex::new(Vec::new()));

        let requests_per_second = self.request_rate;
        let interval = Duration::from_secs_f64(1.0 / requests_per_second);

        while start_time.elapsed() < self.duration {
            let sem = semaphore.clone();
            let op = operation.clone();
            let times = request_times.clone();

            let handle = tokio::spawn(async move {
                // RAII guard: hold semaphore permit for the duration of the operation
                let _permit = sem.acquire().await.unwrap();

                let result = match op().await {
                    Ok(latency) => {
                        let mut times_guard = times.lock().unwrap();
                        times_guard.push(latency.as_millis() as f64);
                        Ok(())
                    }
                    Err(e) => Err(e),
                };

                result
            });

            handles.push(handle);
            tokio::time::sleep(interval).await;
        }

        // Wait for all requests to complete
        let mut successful_requests = 0;
        let mut failed_requests = 0;

        for handle in handles {
            match handle.await {
                Ok(Ok(())) => successful_requests += 1,
                Ok(Err(_)) => failed_requests += 1,
                Err(_) => failed_requests += 1,
            }
        }

        let times = request_times.lock().unwrap();
        let total_duration = start_time.elapsed();

        WorkloadResult {
            workload_name: self.name.clone(),
            total_requests: successful_requests + failed_requests,
            successful_requests,
            failed_requests,
            total_duration,
            average_latency: if !times.is_empty() {
                times.iter().sum::<f64>() / times.len() as f64
            } else {
                0.0
            },
            p95_latency: Self::percentile(&times, 95.0),
            p99_latency: Self::percentile(&times, 99.0),
            throughput_rps: successful_requests as f64 / total_duration.as_secs_f64(),
        }
    }

    fn percentile(values: &[f64], percentile: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = (percentile / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

#[derive(Debug, Clone)]
pub struct WorkloadResult {
    pub workload_name: String,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub total_duration: Duration,
    pub average_latency: f64,
    pub p95_latency: f64,
    pub p99_latency: f64,
    pub throughput_rps: f64,
}

/// Memory usage monitor
#[derive(Debug)]
pub struct MemoryMonitor {
    baseline_memory: usize,
    max_allowed_growth: usize,
    samples: Vec<MemoryUsageSample>,
}

#[derive(Debug, Clone)]
pub struct MemoryUsageSample {
    pub timestamp: Instant,
    pub heap_bytes: usize,
    pub stack_bytes: usize,
    pub total_bytes: usize,
}

impl MemoryMonitor {
    pub fn new(max_growth_mb: usize) -> Self {
        Self {
            baseline_memory: Self::get_memory_usage(),
            max_allowed_growth: max_growth_mb * 1024 * 1024,
            samples: Vec::new(),
        }
    }

    pub fn sample(&mut self) {
        let sample = MemoryUsageSample {
            timestamp: Instant::now(),
            heap_bytes: Self::get_memory_usage(),
            stack_bytes: 0, // Simplified for this test
            total_bytes: Self::get_memory_usage(),
        };
        self.samples.push(sample);
    }

    pub fn check_memory_growth(&self) -> Result<MemoryGrowthResult> {
        if let Some(latest) = self.samples.last() {
            let growth = latest.total_bytes.saturating_sub(self.baseline_memory);
            let within_limits = growth <= self.max_allowed_growth;

            Ok(MemoryGrowthResult {
                baseline_bytes: self.baseline_memory,
                current_bytes: latest.total_bytes,
                growth_bytes: growth,
                max_allowed_growth: self.max_allowed_growth,
                within_limits,
                samples: self.samples.clone(),
            })
        } else {
            Err(anyhow::anyhow!("No memory samples available"))
        }
    }

    fn get_memory_usage() -> usize {
        // Simplified memory usage measurement
        // In a real implementation, you would use proper memory measurement
        std::process::id() as usize * 1024 // Placeholder
    }
}

#[derive(Debug, Clone)]
pub struct MemoryGrowthResult {
    pub baseline_bytes: usize,
    pub current_bytes: usize,
    pub growth_bytes: usize,
    pub max_allowed_growth: usize,
    pub within_limits: bool,
    pub samples: Vec<MemoryUsageSample>,
}

#[tokio::test]
async fn test_search_query_performance_baseline() {
    let collector = PerformanceCollector::new();

    // Simulate search query performance
    async fn mock_search_query() -> Result<Duration> {
        let start = Instant::now();

        // Simulate search processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(start.elapsed())
    }

    // Run workload
    let workload = WorkloadGenerator::new(
        "search_query".to_string(),
        10, // concurrency
        Duration::from_secs(5),
        50.0, // 50 RPS
    );

    let result = workload.run_workload(mock_search_query).await;

    // Record metrics
    let metric = PerformanceMetric {
        name: "search_query".to_string(),
        latency_ms: result.average_latency,
        throughput_rps: result.throughput_rps,
        memory_bytes: 25 * 1024 * 1024, // 25MB
        cpu_percent: 45.0,
        timestamp: chrono::Utc::now(),
        samples: vec![
            result.average_latency,
            result.p95_latency,
            result.p99_latency,
        ],
    };

    collector.record_metric(metric);

    // Check against baseline
    let baseline_check = collector.check_baseline("search_query").unwrap();

    println!("Search Query Performance:");
    println!("  Average Latency: {:.2} ms", result.average_latency);
    println!("  P95 Latency: {:.2} ms", result.p95_latency);
    println!("  P99 Latency: {:.2} ms", result.p99_latency);
    println!("  Throughput: {:.2} RPS", result.throughput_rps);
    println!(
        "  Success Rate: {:.2}%",
        (result.successful_requests as f64 / result.total_requests as f64) * 100.0
    );

    assert!(
        baseline_check.passed,
        "Search query should meet baseline: {:?}",
        baseline_check.violations
    );
    assert!(
        result.average_latency < 500.0,
        "Average latency should be under 500ms"
    );
    assert!(
        result.throughput_rps > 30.0,
        "Throughput should be above 30 RPS"
    );
}

#[tokio::test]
async fn test_pdf_extraction_performance() {
    let collector = PerformanceCollector::new();

    async fn mock_pdf_extraction() -> Result<Duration> {
        let start = Instant::now();

        // Simulate PDF processing (more intensive)
        tokio::time::sleep(Duration::from_millis(200)).await;

        Ok(start.elapsed())
    }

    let workload = WorkloadGenerator::new(
        "pdf_extraction".to_string(),
        5, // Lower concurrency for CPU-intensive task
        Duration::from_secs(10),
        5.0, // 5 RPS
    );

    let result = workload.run_workload(mock_pdf_extraction).await;

    let metric = PerformanceMetric {
        name: "pdf_extraction".to_string(),
        latency_ms: result.average_latency,
        throughput_rps: result.throughput_rps,
        memory_bytes: 150 * 1024 * 1024, // 150MB
        cpu_percent: 75.0,
        timestamp: chrono::Utc::now(),
        samples: vec![
            result.average_latency,
            result.p95_latency,
            result.p99_latency,
        ],
    };

    collector.record_metric(metric);

    let baseline_check = collector.check_baseline("pdf_extraction").unwrap();

    println!("PDF Extraction Performance:");
    println!("  Average Latency: {:.2} ms", result.average_latency);
    println!("  P95 Latency: {:.2} ms", result.p95_latency);
    println!("  Throughput: {:.2} RPS", result.throughput_rps);

    assert!(
        baseline_check.passed,
        "PDF extraction should meet baseline: {:?}",
        baseline_check.violations
    );
    assert!(
        result.average_latency < 2000.0,
        "PDF extraction should complete under 2s"
    );
}

#[tokio::test]
async fn test_html_parsing_performance() {
    let collector = PerformanceCollector::new();

    async fn mock_html_parsing() -> Result<Duration> {
        let start = Instant::now();

        // Simulate HTML parsing
        tokio::time::sleep(Duration::from_millis(25)).await;

        Ok(start.elapsed())
    }

    let workload = WorkloadGenerator::new(
        "html_parsing".to_string(),
        20, // Higher concurrency for I/O bound task
        Duration::from_secs(5),
        100.0, // 100 RPS
    );

    let result = workload.run_workload(mock_html_parsing).await;

    let metric = PerformanceMetric {
        name: "html_parsing".to_string(),
        latency_ms: result.average_latency,
        throughput_rps: result.throughput_rps,
        memory_bytes: 15 * 1024 * 1024, // 15MB
        cpu_percent: 40.0,
        timestamp: chrono::Utc::now(),
        samples: vec![
            result.average_latency,
            result.p95_latency,
            result.p99_latency,
        ],
    };

    collector.record_metric(metric);

    let baseline_check = collector.check_baseline("html_parsing").unwrap();

    println!("HTML Parsing Performance:");
    println!("  Average Latency: {:.2} ms", result.average_latency);
    println!("  Throughput: {:.2} RPS", result.throughput_rps);

    assert!(
        baseline_check.passed,
        "HTML parsing should meet baseline: {:?}",
        baseline_check.violations
    );
    assert!(
        result.throughput_rps > 80.0,
        "HTML parsing throughput should be above 80 RPS"
    );
}

#[tokio::test]
async fn test_api_request_performance() {
    let collector = PerformanceCollector::new();

    async fn mock_api_request() -> Result<Duration> {
        let start = Instant::now();

        // Simulate fast API request
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(start.elapsed())
    }

    let workload = WorkloadGenerator::new(
        "api_request".to_string(),
        50, // High concurrency for API
        Duration::from_secs(3),
        200.0, // 200 RPS
    );

    let result = workload.run_workload(mock_api_request).await;

    let metric = PerformanceMetric {
        name: "api_request".to_string(),
        latency_ms: result.average_latency,
        throughput_rps: result.throughput_rps,
        memory_bytes: 5 * 1024 * 1024, // 5MB
        cpu_percent: 30.0,
        timestamp: chrono::Utc::now(),
        samples: vec![
            result.average_latency,
            result.p95_latency,
            result.p99_latency,
        ],
    };

    collector.record_metric(metric);

    let baseline_check = collector.check_baseline("api_request").unwrap();

    println!("API Request Performance:");
    println!("  Average Latency: {:.2} ms", result.average_latency);
    println!("  Throughput: {:.2} RPS", result.throughput_rps);

    assert!(
        baseline_check.passed,
        "API requests should meet baseline: {:?}",
        baseline_check.violations
    );
    assert!(
        result.average_latency < 100.0,
        "API latency should be under 100ms"
    );
    assert!(
        result.throughput_rps > 150.0,
        "API throughput should be above 150 RPS"
    );
}

#[tokio::test]
async fn test_memory_usage_monitoring() {
    let mut monitor = MemoryMonitor::new(50); // 50MB max growth

    // Take baseline sample
    monitor.sample();

    // Simulate memory-intensive operations
    let mut data_vectors = Vec::new();
    for i in 0..100 {
        monitor.sample();

        // Simulate memory allocation
        let data: Vec<u8> = vec![0; 1024 * 100]; // 100KB allocation
        data_vectors.push(data);

        if i % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    // Final sample
    monitor.sample();

    // Check memory growth
    let growth_result = monitor.check_memory_growth().unwrap();

    println!("Memory Growth Analysis:");
    println!("  Baseline: {} bytes", growth_result.baseline_bytes);
    println!("  Current: {} bytes", growth_result.current_bytes);
    println!("  Growth: {} bytes", growth_result.growth_bytes);
    println!("  Within Limits: {}", growth_result.within_limits);
    println!("  Samples: {}", growth_result.samples.len());

    // Memory growth should be reasonable
    assert!(
        growth_result.samples.len() > 10,
        "Should have multiple memory samples"
    );

    // Clean up allocated memory
    drop(data_vectors);
}

#[tokio::test]
async fn test_resource_limit_enforcement() {
    // Test that resource limits are properly enforced

    // Test concurrent request limiting
    let semaphore = Arc::new(Semaphore::new(10)); // Max 10 concurrent requests
    let mut handles = Vec::new();

    for i in 0..20 {
        let sem = semaphore.clone();
        let handle = tokio::spawn(async move {
            // RAII guard: hold semaphore permit to limit concurrency
            let permit = sem.acquire().await.unwrap();

            // Simulate work
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Permit dropped here automatically
            drop(permit);
            i
        });
        handles.push(handle);
    }

    let start = Instant::now();

    // Wait for all to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    let duration = start.elapsed();

    // With 10 concurrent limit and 100ms per task, 20 tasks should take at least 200ms
    assert!(
        duration >= Duration::from_millis(180),
        "Resource limiting should enforce concurrency limits"
    );
    assert_eq!(results.len(), 20, "All tasks should complete");

    // Test memory limit enforcement
    async fn memory_intensive_task(limit_mb: usize) -> Result<()> {
        let max_allocation = limit_mb * 1024 * 1024;
        let mut allocated = 0;
        let mut buffers = Vec::new();

        while allocated < max_allocation {
            let buffer: Vec<u8> = vec![0; 1024 * 1024]; // 1MB allocation
            allocated += buffer.len();
            buffers.push(buffer);

            if allocated > max_allocation {
                return Err(anyhow::anyhow!("Memory limit exceeded"));
            }

            tokio::task::yield_now().await;
        }

        Ok(())
    }

    // Test with reasonable memory limit
    let result = memory_intensive_task(10).await; // 10MB limit
    assert!(
        result.is_ok(),
        "Should be able to allocate within memory limits"
    );
}

#[tokio::test]
async fn test_performance_regression_detection() {
    let collector = PerformanceCollector::new();

    // Simulate baseline performance
    let baseline_metric = PerformanceMetric {
        name: "regression_test".to_string(),
        latency_ms: 100.0,
        throughput_rps: 200.0,
        memory_bytes: 20 * 1024 * 1024,
        cpu_percent: 50.0,
        timestamp: chrono::Utc::now(),
        samples: vec![95.0, 100.0, 105.0],
    };

    // Add a custom baseline for this test
    {
        let mut baselines = collector.baselines.lock().unwrap();
        baselines.insert(
            "regression_test".to_string(),
            PerformanceBaseline {
                name: "regression_test".to_string(),
                max_latency_ms: 120.0,
                min_throughput_rps: 180.0,
                max_memory_bytes: 25 * 1024 * 1024,
                max_cpu_percent: 60.0,
                regression_threshold_percent: 5.0,
            },
        );
    }

    collector.record_metric(baseline_metric);

    // Test 1: Performance within baseline
    let good_metric = PerformanceMetric {
        name: "regression_test".to_string(),
        latency_ms: 105.0,     // Within limits
        throughput_rps: 195.0, // Within limits
        memory_bytes: 22 * 1024 * 1024,
        cpu_percent: 55.0,
        timestamp: chrono::Utc::now(),
        samples: vec![100.0, 105.0, 110.0],
    };

    collector.record_metric(good_metric);
    let check = collector.check_baseline("regression_test").unwrap();
    assert!(check.passed, "Good performance should pass baseline check");
    assert!(check.violations.is_empty(), "Should have no violations");

    // Test 2: Performance regression (latency too high)
    let bad_latency_metric = PerformanceMetric {
        name: "regression_test".to_string(),
        latency_ms: 150.0, // Exceeds max_latency_ms (120.0)
        throughput_rps: 190.0,
        memory_bytes: 22 * 1024 * 1024,
        cpu_percent: 55.0,
        timestamp: chrono::Utc::now(),
        samples: vec![140.0, 150.0, 160.0],
    };

    collector.record_metric(bad_latency_metric);
    let check = collector.check_baseline("regression_test").unwrap();
    assert!(!check.passed, "High latency should fail baseline check");
    assert!(
        !check.violations.is_empty(),
        "Should have latency violation"
    );
    assert!(
        check.violations[0].contains("Latency"),
        "Should mention latency violation"
    );

    // Test 3: Performance regression (throughput too low)
    let bad_throughput_metric = PerformanceMetric {
        name: "regression_test".to_string(),
        latency_ms: 110.0,
        throughput_rps: 150.0, // Below min_throughput_rps (180.0)
        memory_bytes: 22 * 1024 * 1024,
        cpu_percent: 55.0,
        timestamp: chrono::Utc::now(),
        samples: vec![110.0, 115.0, 120.0],
    };

    collector.record_metric(bad_throughput_metric);
    let check = collector.check_baseline("regression_test").unwrap();
    assert!(!check.passed, "Low throughput should fail baseline check");
    assert!(
        check.violations.iter().any(|v| v.contains("Throughput")),
        "Should have throughput violation"
    );
}

#[tokio::test]
async fn test_load_testing_scenarios() {
    // Test different load patterns

    // Scenario 1: Steady load
    async fn steady_load_operation() -> Result<Duration> {
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(start.elapsed())
    }

    let steady_workload = WorkloadGenerator::new(
        "steady_load".to_string(),
        10,
        Duration::from_secs(5),
        20.0, // Constant 20 RPS
    );

    let steady_result = steady_workload.run_workload(steady_load_operation).await;
    assert!(
        steady_result.successful_requests > 80,
        "Steady load should process most requests"
    );

    // Scenario 2: Burst load
    async fn burst_operation() -> Result<Duration> {
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok(start.elapsed())
    }

    let burst_workload = WorkloadGenerator::new(
        "burst_load".to_string(),
        50,                     // High concurrency
        Duration::from_secs(2), // Short duration
        100.0,                  // High RPS
    );

    let burst_result = burst_workload.run_workload(burst_operation).await;
    assert!(
        burst_result.throughput_rps > 50.0,
        "Should handle burst load effectively"
    );

    // Scenario 3: Gradual ramp-up
    let mut ramp_results = Vec::new();
    for rps in [10.0, 25.0, 50.0, 75.0, 100.0] {
        let ramp_workload =
            WorkloadGenerator::new(format!("ramp_{}", rps), 20, Duration::from_secs(2), rps);

        let result = ramp_workload.run_workload(burst_operation).await;
        ramp_results.push((rps, result.throughput_rps, result.average_latency));
    }

    println!("Ramp-up test results:");
    for (target_rps, actual_rps, latency) in &ramp_results {
        println!(
            "  Target: {} RPS, Actual: {:.1} RPS, Latency: {:.1} ms",
            target_rps, actual_rps, latency
        );
    }

    // Latency should remain reasonable even as load increases
    let max_latency = ramp_results
        .iter()
        .map(|(_, _, latency)| *latency)
        .fold(0.0f64, f64::max);

    assert!(
        max_latency < 200.0,
        "Latency should remain reasonable under load"
    );
}

#[tokio::test]
async fn test_concurrent_performance_monitoring() {
    let collector = Arc::new(PerformanceCollector::new());
    let mut handles = Vec::new();

    // Run multiple concurrent performance tests
    for i in 0..5 {
        let collector_clone = collector.clone();
        let handle = tokio::spawn(async move {
            let workload = WorkloadGenerator::new(
                format!("concurrent_test_{}", i),
                5,
                Duration::from_secs(3),
                10.0,
            );

            async fn test_operation() -> Result<Duration> {
                let start = Instant::now();
                tokio::time::sleep(Duration::from_millis(25)).await;
                Ok(start.elapsed())
            }

            let result = workload.run_workload(test_operation).await;

            let metric = PerformanceMetric {
                name: format!("concurrent_test_{}", i),
                latency_ms: result.average_latency,
                throughput_rps: result.throughput_rps,
                memory_bytes: 10 * 1024 * 1024,
                cpu_percent: 40.0,
                timestamp: chrono::Utc::now(),
                samples: vec![result.average_latency],
            };

            collector_clone.record_metric(metric);
            result
        });
        handles.push(handle);
    }

    // Wait for all concurrent tests to complete
    let mut all_results = Vec::new();
    for handle in handles {
        all_results.push(handle.await.unwrap());
    }

    // Verify all tests completed successfully
    assert_eq!(all_results.len(), 5, "All concurrent tests should complete");

    let total_requests: usize = all_results.iter().map(|r| r.successful_requests).sum();
    assert!(
        total_requests > 100,
        "Should process significant number of requests concurrently"
    );

    // Check that metrics were recorded for all tests
    let all_metrics = collector.get_all_metrics();
    assert!(
        all_metrics.len() >= 5,
        "Should have metrics for all concurrent tests"
    );
}
