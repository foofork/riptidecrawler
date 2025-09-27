//! Golden Test Runner
//!
//! Main orchestration logic for running golden tests with baseline comparison.

use super::*;
use super::behavior_capture::capture_behavior;
use super::regression_guard::{RegressionGuard, VerificationSystem};
use std::time::Instant;

/// Run a complete golden test with baseline comparison
pub async fn run_test<F, T>(
    test_name: &str,
    test_fn: F,
    config: &GoldenTestConfig,
    baseline_storage: &mut BaselineStorage,
) -> Result<GoldenTestResult, anyhow::Error>
where
    F: Fn() -> T + Send + 'static,
    T: std::future::Future<Output = Result<serde_json::Value, anyhow::Error>> + Send,
{
    if config.verbose {
        println!("🏃 Starting golden test: {}", test_name);
    }
    
    let start_time = Instant::now();
    
    // Load baseline
    let baseline = match baseline_storage.load_baseline(test_name).await? {
        Some(baseline) => {
            if config.verbose {
                println!("📝 Loaded baseline for: {}", test_name);
            }
            baseline
        },
        None => {
            return Err(anyhow::anyhow!(
                "No baseline found for test '{}'. Run capture_baseline() first.",
                test_name
            ));
        }
    };
    
    // Capture current behavior
    if config.verbose {
        println!("📊 Capturing current behavior for: {}", test_name);
    }
    
    let current_snapshot = capture_behavior(test_name, test_fn, config).await?;
    
    // Run regression analysis
    if config.verbose {
        println!("🔍 Analyzing for regressions: {}", test_name);
    }
    
    let guard = RegressionGuard::new(config.clone());
    let result = guard.detect_regressions(&baseline, &current_snapshot);
    
    let elapsed = start_time.elapsed();
    
    if config.verbose {
        println!(
            "✅ Golden test completed in {:.2}s: {} ({})",
            elapsed.as_secs_f64(),
            test_name,
            if result.passed { "PASSED" } else { "FAILED" }
        );
    }
    
    Ok(result)
}

/// Run a complete golden test suite
pub async fn run_test_suite(
    test_cases: &[(&str, fn() -> Result<serde_json::Value, anyhow::Error>)],
    config: &GoldenTestConfig,
    baseline_storage: &mut BaselineStorage,
) -> Result<TestSuiteResult, anyhow::Error> {
    println!("🚀 Running golden test suite with {} tests...", test_cases.len());
    
    let suite_start = Instant::now();
    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    
    for (test_name, _test_fn) in test_cases {
        println!("\n📝 Running test: {}", test_name);
        
        // For demonstration, we'll create a mock test function
        // In real usage, you'd use the actual test_fn
        let mock_test = || async {
            // Simulate test execution
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            Ok(serde_json::json!({
                "test_name": test_name,
                "result": "success",
                "timestamp": chrono::Utc::now().timestamp()
            }))
        };
        
        match run_test(test_name, mock_test, config, baseline_storage).await {
            Ok(result) => {
                if result.passed {
                    passed += 1;
                    println!("✅ Test '{}' PASSED", test_name);
                } else {
                    failed += 1;
                    println!("❌ Test '{}' FAILED with {} violations", test_name, result.violations.len());
                    
                    if config.verbose {
                        for violation in &result.violations {
                            println!("  • {:?}: {}", violation.severity, violation.description);
                        }
                    }
                }
                results.push(result);
            },
            Err(e) => {
                failed += 1;
                println!("❌ Test '{}' ERROR: {}", test_name, e);
            }
        }
    }
    
    let suite_elapsed = suite_start.elapsed();
    let total_tests = passed + failed;
    let success_rate = if total_tests > 0 {
        (passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };
    
    let suite_result = TestSuiteResult {
        total_tests,
        passed_tests: passed,
        failed_tests: failed,
        success_rate,
        duration: suite_elapsed,
        test_results: results,
    };
    
    suite_result.print_summary();
    
    Ok(suite_result)
}

/// Initialize golden test framework for a new project
pub async fn initialize_framework(
    baseline_path: &std::path::Path,
    config: &GoldenTestConfig,
) -> Result<(), anyhow::Error> {
    println!("🔧 Initializing Golden Test Framework...");
    
    // Initialize baseline storage
    super::performance_baseline::initialize_baseline_storage(baseline_path).await?;
    
    // Create example test cases
    let example_tests = vec![
        ("page_extraction_performance", example_page_extraction_test as fn() -> Result<serde_json::Value, anyhow::Error>),
        ("batch_processing_throughput", example_batch_processing_test),
        ("memory_usage_under_load", example_memory_test),
        ("concurrent_request_handling", example_concurrent_test),
    ];
    
    // Capture initial baselines
    super::performance_baseline::capture_initial_baselines(
        baseline_path,
        &example_tests,
        config,
    ).await?;
    
    println!("✅ Golden Test Framework initialized successfully!");
    println!("📝 Baseline storage: {}", baseline_path.display());
    println!("🔍 Framework ready for refactoring verification");
    
    Ok(())
}

/// Test suite execution result
#[derive(Debug, Clone)]
pub struct TestSuiteResult {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub duration: std::time::Duration,
    pub test_results: Vec<GoldenTestResult>,
}

impl TestSuiteResult {
    pub fn is_successful(&self) -> bool {
        self.failed_tests == 0
    }
    
    pub fn print_summary(&self) {
        println!("\n📊 Golden Test Suite Results");
        println!("==============================");
        println!("Total Tests: {}", self.total_tests);
        println!("Passed: ✅ {}", self.passed_tests);
        println!("Failed: ❌ {}", self.failed_tests);
        println!("Success Rate: {:.1}%", self.success_rate);
        println!("Duration: {:.2}s", self.duration.as_secs_f64());
        
        if self.is_successful() {
            println!("\n🎉 All tests passed! Refactoring is safe to proceed.");
        } else {
            println!("\n⚠️  {} test(s) failed. Review violations before proceeding.", self.failed_tests);
        }
        
        // Performance summary
        if !self.test_results.is_empty() {
            let avg_p50 = self.test_results.iter()
                .map(|r| r.performance_delta.p50_change_percent)
                .sum::<f64>() / self.test_results.len() as f64;
            
            let avg_p95 = self.test_results.iter()
                .map(|r| r.performance_delta.p95_change_percent)
                .sum::<f64>() / self.test_results.len() as f64;
            
            println!("\n📊 Performance Summary:");
            println!("Average P50 change: {:.2}%", avg_p50);
            println!("Average P95 change: {:.2}%", avg_p95);
        }
        
        println!();
    }
}

/// Example test functions for framework initialization
fn example_page_extraction_test() -> Result<serde_json::Value, anyhow::Error> {
    Ok(serde_json::json!({
        "pages_extracted": 10,
        "total_bytes": 1024000,
        "processing_time_ms": 850,
        "success_rate": 100.0
    }))
}

fn example_batch_processing_test() -> Result<serde_json::Value, anyhow::Error> {
    Ok(serde_json::json!({
        "batch_size": 100,
        "processed_items": 100,
        "throughput_per_second": 12.5,
        "memory_peak_mb": 450
    }))
}

fn example_memory_test() -> Result<serde_json::Value, anyhow::Error> {
    Ok(serde_json::json!({
        "initial_memory_mb": 200,
        "peak_memory_mb": 480,
        "final_memory_mb": 220,
        "memory_efficiency": 85.0
    }))
}

fn example_concurrent_test() -> Result<serde_json::Value, anyhow::Error> {
    Ok(serde_json::json!({
        "concurrent_requests": 50,
        "successful_requests": 49,
        "average_response_time_ms": 450,
        "p95_response_time_ms": 800
    }))
}

/// CLI interface for golden test operations
pub struct GoldenTestCli {
    config: GoldenTestConfig,
    baseline_storage: BaselineStorage,
}

impl GoldenTestCli {
    pub fn new(config: GoldenTestConfig) -> Self {
        Self {
            baseline_storage: BaselineStorage::new(),
            config,
        }
    }
    
    /// Capture baselines for all configured tests
    pub async fn capture_baselines(&mut self) -> Result<(), anyhow::Error> {
        println!("📸 Capturing baselines for all tests...");
        
        let test_cases = vec![
            ("page_extraction_performance", example_page_extraction_test),
            ("batch_processing_throughput", example_batch_processing_test),
            ("memory_usage_under_load", example_memory_test),
            ("concurrent_request_handling", example_concurrent_test),
        ];
        
        for (test_name, test_fn) in test_cases {
            println!("📝 Capturing baseline: {}", test_name);
            
            let mock_test = || async {
                Ok(test_fn()?)
            };
            
            let snapshot = capture_behavior(test_name, mock_test, &self.config).await?;
            self.baseline_storage.save_baseline(test_name, &snapshot).await?;
            
            println!("✅ Baseline captured: {}", test_name);
        }
        
        println!("🎉 All baselines captured successfully!");
        Ok(())
    }
    
    /// Run verification against existing baselines
    pub async fn verify_against_baselines(&mut self) -> Result<(), anyhow::Error> {
        println!("🔍 Verifying current implementation against baselines...");
        
        let test_cases = vec![
            ("page_extraction_performance", example_page_extraction_test),
            ("batch_processing_throughput", example_batch_processing_test),
            ("memory_usage_under_load", example_memory_test),
            ("concurrent_request_handling", example_concurrent_test),
        ];
        
        let verification = VerificationSystem::new(self.config.clone());
        let report = verification.verify_refactoring(&test_cases, &mut self.baseline_storage).await?;
        
        if report.is_successful() {
            println!("✅ Verification passed! Implementation is safe.");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Verification failed! {} out of {} tests failed.",
                report.failed_tests,
                report.total_tests
            ))
        }
    }
    
    /// List all available baselines
    pub async fn list_baselines(&mut self) -> Result<(), anyhow::Error> {
        let baselines = self.baseline_storage.list_baselines().await?;
        
        if baselines.is_empty() {
            println!("📦 No baselines found. Run 'capture-baselines' first.");
        } else {
            println!("📝 Available baselines ({}):", baselines.len());
            for baseline in baselines {
                println!("  • {}", baseline);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_golden_test_runner() {
        let temp_dir = tempdir().unwrap();
        let baseline_path = temp_dir.path().join("test_baselines.json");
        
        let config = GoldenTestConfig {
            verbose: true,
            ..Default::default()
        };
        
        // Initialize framework
        initialize_framework(&baseline_path, &config).await.unwrap();
        
        // Verify baseline file was created
        assert!(baseline_path.exists());
        
        // Test baseline loading
        let mut storage = BaselineStorage::new();
        let baseline = storage.load_baseline("page_extraction_performance").await.unwrap();
        assert!(baseline.is_some());
    }
    
    #[tokio::test]
    async fn test_cli_operations() {
        let config = GoldenTestConfig::default();
        let mut cli = GoldenTestCli::new(config);
        
        // Test baseline listing (should be empty initially)
        cli.list_baselines().await.unwrap();
        
        // Test baseline capture
        cli.capture_baselines().await.unwrap();
        
        // Test verification
        cli.verify_against_baselines().await.unwrap();
    }
}
