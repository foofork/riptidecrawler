/// Comprehensive WASM Extractor Test Runner
///
/// This test runner coordinates all test suites and generates comprehensive reports
/// for the WASM extractor component validation.

mod tests;

use tests::*;

#[tokio::test]
async fn run_comprehensive_test_suite() {
    println!("🚀 Starting Comprehensive WASM Extractor Test Suite");
    println!("====================================================");

    match tests::run_comprehensive_test_suite() {
        Ok(results) => {
            println!("\n✅ Test suite completed successfully!");
            println!("📊 Report generated at: /reports/last-run/wasm/index.html");

            // Assert overall success for CI/CD pipeline
            assert!(
                results.overall_success,
                "Test suite failed - check reports for details. Success rates: Golden: {:.1}%, Benchmarks: {:.1}%, Memory: {:.1}%, Cache: {:.1}%, Integration: {:.1}%",
                results.golden_tests.success_rate * 100.0,
                results.benchmarks.success_rate * 100.0,
                results.memory_tests.success_rate * 100.0,
                results.cache_tests.success_rate * 100.0,
                results.integration_tests.success_rate * 100.0
            );

            // Assert critical performance thresholds
            assert!(
                results.performance_summary.average_extraction_time_ms < 100.0,
                "Average extraction time too high: {:.2}ms (max: 100ms)",
                results.performance_summary.average_extraction_time_ms
            );

            assert!(
                results.coverage_report.coverage_percentage > 75.0,
                "Test coverage too low: {:.1}% (min: 75%)",
                results.coverage_report.coverage_percentage
            );

            assert!(
                results.performance_summary.peak_memory_usage_mb < 256.0,
                "Peak memory usage too high: {:.1}MB (max: 256MB)",
                results.performance_summary.peak_memory_usage_mb
            );
        },
        Err(e) => {
            panic!("Test suite execution failed: {}", e);
        }
    }
}

/// Individual test category runners for targeted testing

#[tokio::test]
async fn run_golden_tests_only() {
    println!("📸 Running Golden Tests Only");
    println!("============================");

    match golden::run_all_golden_tests() {
        Ok(()) => println!("✅ All golden tests passed!"),
        Err(e) => panic!("Golden tests failed: {}", e),
    }
}

#[tokio::test]
async fn run_performance_benchmarks_only() {
    println!("⚡ Running Performance Benchmarks Only");
    println!("======================================");

    match benchmarks::run_performance_benchmarks() {
        Ok(suite) => {
            println!("✅ Benchmarks completed!");

            // Assert reasonable performance
            let avg_time: f64 = suite.results.iter()
                .map(|r| r.duration_ms)
                .sum::<f64>() / suite.results.len() as f64;

            assert!(
                avg_time < 50.0,
                "Average benchmark time too high: {:.2}ms",
                avg_time
            );
        },
        Err(e) => panic!("Benchmarks failed: {}", e),
    }
}

#[tokio::test]
async fn run_memory_tests_only() {
    println!("🧠 Running Memory Tests Only");
    println!("============================");

    match memory_limiter::run_memory_limiter_tests() {
        Ok(results) => {
            let passed = results.iter().filter(|r| r.success).count();
            let total = results.len();

            println!("✅ Memory tests completed: {}/{} passed", passed, total);

            assert!(
                passed >= total * 8 / 10,  // At least 80% success
                "Too many memory test failures: {}/{}",
                total - passed,
                total
            );
        },
        Err(e) => panic!("Memory tests failed: {}", e),
    }
}

#[tokio::test]
async fn run_cache_tests_only() {
    println!("⚡ Running Cache Tests Only");
    println!("===========================");

    match aot_cache::run_aot_cache_tests() {
        Ok(results) => {
            let passed = results.iter().filter(|r| r.success).count();
            let total = results.len();

            println!("✅ Cache tests completed: {}/{} passed", passed, total);

            // Check for cache performance improvement
            if let Some(warm_test) = results.iter().find(|r| r.test_name == "warm_start_performance") {
                assert!(
                    warm_test.success,
                    "Warm start performance test failed"
                );
            }
        },
        Err(e) => panic!("Cache tests failed: {}", e),
    }
}

#[tokio::test]
async fn run_integration_tests_only() {
    println!("🔗 Running Integration Tests Only");
    println!("=================================");

    match integration::run_integration_tests() {
        Ok(results) => {
            let passed = results.iter().filter(|r| r.success).count();
            let total = results.len();

            println!("✅ Integration tests completed: {}/{} passed", passed, total);

            // Allow some failures for integration tests under stress
            assert!(
                passed >= total * 7 / 10,  // At least 70% success
                "Too many integration test failures: {}/{}",
                total - passed,
                total
            );
        },
        Err(e) => panic!("Integration tests failed: {}", e),
    }
}

/// Regression test to ensure no performance degradation
#[tokio::test]
async fn regression_test_performance_baseline() {
    println!("📉 Running Performance Regression Test");
    println!("======================================");

    // Performance baselines (update these as performance improves)
    const MAX_EXTRACTION_TIME_MS: f64 = 50.0;
    const MAX_MEMORY_USAGE_MB: f64 = 128.0;
    const MIN_THROUGHPUT_OPS_SEC: f64 = 100.0;

    match benchmarks::run_performance_benchmarks() {
        Ok(suite) => {
            // Check individual benchmark results against baselines
            for result in &suite.results {
                if result.name.contains("warm_performance") {
                    assert!(
                        result.duration_ms < MAX_EXTRACTION_TIME_MS,
                        "Regression: {} took {:.2}ms (baseline: {:.2}ms)",
                        result.name, result.duration_ms, MAX_EXTRACTION_TIME_MS
                    );
                }

                if result.name.contains("memory") && result.memory_used_bytes > 0 {
                    let memory_mb = result.memory_used_bytes as f64 / 1024.0 / 1024.0;
                    assert!(
                        memory_mb < MAX_MEMORY_USAGE_MB,
                        "Regression: {} used {:.1}MB (baseline: {:.1}MB)",
                        result.name, memory_mb, MAX_MEMORY_USAGE_MB
                    );
                }

                if result.throughput_ops_per_sec > 0.0 {
                    assert!(
                        result.throughput_ops_per_sec > MIN_THROUGHPUT_OPS_SEC,
                        "Regression: {} achieved {:.1} ops/sec (baseline: {:.1} ops/sec)",
                        result.name, result.throughput_ops_per_sec, MIN_THROUGHPUT_OPS_SEC
                    );
                }
            }

            println!("✅ No performance regressions detected");
        },
        Err(e) => panic!("Regression test failed: {}", e),
    }
}

/// Stress test for production readiness validation
#[tokio::test]
async fn stress_test_production_readiness() {
    println!("🏭 Running Production Readiness Stress Test");
    println!("===========================================");

    // This test simulates production-like conditions
    let component = riptide_extractor_wasm::Component;
    let html = std::fs::read_to_string("tests/fixtures/blog_post.html")
        .unwrap_or_else(|_| "<html><body><h1>Fallback</h1><p>Content</p></body></html>".to_string());

    let stress_iterations = 1000;
    let concurrent_threads = 4;
    let max_failures_allowed = 50; // 5% failure rate

    let mut handles = Vec::new();

    for thread_id in 0..concurrent_threads {
        let html = html.clone();
        let handle = std::thread::spawn(move || -> Result<usize, String> {
            let component = riptide_extractor_wasm::Component;
            let mut successes = 0;

            for i in 0..stress_iterations / concurrent_threads {
                match component.extract(
                    html.clone(),
                    format!("https://stress.test/{}/{}", thread_id, i),
                    riptide_extractor_wasm::ExtractionMode::Article
                ) {
                    Ok(_) => successes += 1,
                    Err(_) => {}, // Count failures implicitly
                }
            }

            Ok(successes)
        });

        handles.push(handle);
    }

    let mut total_successes = 0;
    for handle in handles {
        match handle.join() {
            Ok(Ok(successes)) => total_successes += successes,
            Ok(Err(e)) => panic!("Stress test thread error: {}", e),
            Err(_) => panic!("Stress test thread panicked"),
        }
    }

    let total_failures = stress_iterations - total_successes;

    println!("Stress test results: {}/{} succeeded ({} failures)",
             total_successes, stress_iterations, total_failures);

    assert!(
        total_failures <= max_failures_allowed,
        "Too many failures under stress: {} (max allowed: {})",
        total_failures, max_failures_allowed
    );

    println!("✅ Production readiness stress test passed");
}

/// Smoke test for quick validation
#[test]
fn smoke_test_basic_functionality() {
    println!("💨 Running Smoke Test");
    println!("=====================");

    let component = riptide_extractor_wasm::Component;
    let html = "<html><head><title>Test</title></head><body><p>Hello World</p></body></html>";

    // Test basic extraction
    let result = component.extract(
        html.to_string(),
        "https://example.com/smoke-test".to_string(),
        riptide_extractor_wasm::ExtractionMode::Article
    );

    assert!(result.is_ok(), "Basic extraction should succeed");

    let content = result.unwrap();
    assert!(content.title.is_some(), "Should extract title");
    assert!(!content.text.is_empty(), "Should extract text content");

    // Test component info
    let info = component.get_info();
    assert_eq!(info.name, "riptide-extractor-wasm");
    assert!(!info.features.is_empty());

    // Test health check
    let health = component.health_check();
    assert_eq!(health.status, "healthy");

    println!("✅ Smoke test passed - basic functionality works");
}

/// Compatibility test for different extraction modes
#[test]
fn compatibility_test_extraction_modes() {
    println!("🔧 Running Compatibility Test");
    println!("=============================");

    let component = riptide_extractor_wasm::Component;
    let html = std::fs::read_to_string("tests/fixtures/news_site.html")
        .unwrap_or_else(|_| "<html><body><h1>News</h1><p>Story content</p></body></html>".to_string());

    let modes = vec![
        riptide_extractor_wasm::ExtractionMode::Article,
        riptide_extractor_wasm::ExtractionMode::Full,
        riptide_extractor_wasm::ExtractionMode::Metadata,
    ];

    for mode in modes {
        let result = component.extract(
            html.clone(),
            format!("https://compat.test/{:?}", mode),
            mode
        );

        assert!(
            result.is_ok(),
            "Extraction mode {:?} should work",
            mode
        );

        let content = result.unwrap();

        // Basic validation - all modes should extract something
        assert!(
            content.title.is_some() || !content.text.is_empty(),
            "Mode {:?} should extract some content",
            mode
        );
    }

    println!("✅ Compatibility test passed - all extraction modes work");
}

/// Error handling validation
#[test]
fn error_handling_test() {
    println!("🚨 Running Error Handling Test");
    println!("==============================");

    let component = riptide_extractor_wasm::Component;

    // Test invalid HTML
    let result = component.extract(
        "".to_string(),  // Empty HTML
        "https://example.com".to_string(),
        riptide_extractor_wasm::ExtractionMode::Article
    );

    assert!(result.is_err(), "Empty HTML should fail gracefully");

    // Test malformed HTML
    let result = component.extract(
        "<html><body><p>Unclosed paragraph".to_string(),
        "https://example.com".to_string(),
        riptide_extractor_wasm::ExtractionMode::Article
    );

    // Should either succeed with fallback or fail gracefully
    match result {
        Ok(_) => println!("  Malformed HTML handled via fallback"),
        Err(_) => println!("  Malformed HTML rejected gracefully"),
    }

    println!("✅ Error handling test passed");
}

#[cfg(test)]
mod test_utilities {
    /// Utility functions for test data generation and validation

    pub fn generate_test_html(content_type: &str, size: usize) -> String {
        match content_type {
            "simple" => format!("<html><body>{}</body></html>", "Test content ".repeat(size / 13)),
            "complex" => generate_complex_test_html(size),
            _ => "<html><body>Default test content</body></html>".to_string(),
        }
    }

    fn generate_complex_test_html(target_size: usize) -> String {
        let mut html = String::with_capacity(target_size);
        html.push_str("<!DOCTYPE html><html><head><title>Complex Test</title></head><body>");

        let content_chunk = "<section><h2>Section</h2><p>Content with <a href='/link'>links</a> and <strong>formatting</strong>.</p></section>";

        while html.len() < target_size - 100 {
            html.push_str(content_chunk);
        }

        html.push_str("</body></html>");
        html
    }

    pub fn validate_extraction_result(content: &riptide_extractor_wasm::ExtractedContent, expected_type: &str) -> bool {
        match expected_type {
            "news" => content.title.is_some() && content.text.len() > 100,
            "blog" => content.text.len() > 500,
            "gallery" => !content.media.is_empty(),
            _ => !content.text.is_empty(),
        }
    }
}