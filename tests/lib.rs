/// riptide Test Suite - London School TDD
///
/// Comprehensive test suite following the London School (mockist) approach
/// to Test-Driven Development, emphasizing behavior verification through
/// mock collaborations and contract testing.

pub mod fixtures;

// WASM Component Tests
pub mod wasm {
    pub mod wasm_extractor_integration;
}

// API Layer Tests
pub mod api {
    pub mod dynamic_rendering_tests;
}

// Chaos and Resilience Tests
pub mod chaos {
    pub mod error_resilience_tests;
}

// Performance and Benchmarking
pub mod performance {
    pub mod benchmark_tests;
}

// Integration Tests
pub mod integration {
    pub mod session_persistence_tests;
    pub mod contract_tests;
}

// Unit Tests
pub mod unit {
    pub mod component_model_tests;
    pub mod streaming_tests;
}

// Note: Stealth tests are located in crates/riptide-stealth/tests/
// and crates/riptide-stealth/src/tests.rs (not here in workspace tests/)

// Common test utilities
pub mod common {
    use std::time::Duration;
    use tokio::time::timeout;

    /// Test helper for timeout operations
    pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, &'static str>
    where
        F: std::future::Future<Output = T>,
    {
        timeout(duration, future)
            .await
            .map_err(|_| "Operation timed out")
    }

    /// Test helper for asserting error messages contain expected text
    pub fn assert_error_contains(result: &Result<impl std::fmt::Debug, String>, expected: &str) {
        match result {
            Ok(_) => panic!("Expected error containing '{}', but got success", expected),
            Err(error) => assert!(
                error.contains(expected),
                "Error '{}' does not contain expected text '{}'",
                error,
                expected
            ),
        }
    }

    /// Test helper for asserting timing constraints
    pub fn assert_timing_constraint(duration: Duration, min: Duration, max: Duration) {
        assert!(
            duration >= min && duration <= max,
            "Duration {:?} is not between {:?} and {:?}",
            duration,
            min,
            max
        );
    }

    /// Test helper for generating test data
    pub fn generate_test_html(size_kb: usize) -> String {
        let content_size = size_kb * 1024;
        let base_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Generated Test Content</title>
</head>
<body>
    <article>
        <h1>Test Article</h1>
        <div class="content">
"#;

        let footer_html = r#"
        </div>
    </article>
</body>
</html>"#;

        let paragraph = "<p>This is a test paragraph with enough content to generate meaningful test data for performance and stress testing scenarios.</p>\n";
        let paragraphs_needed = (content_size - base_html.len() - footer_html.len()) / paragraph.len();

        format!("{}{}{}", base_html, paragraph.repeat(paragraphs_needed), footer_html)
    }
}

// Test configuration and setup
pub mod config {
    use std::time::Duration;

    /// Test configuration constants
    pub struct TestConfig;

    impl TestConfig {
        /// Default timeout for integration tests
        pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

        /// Short timeout for unit tests
        pub const SHORT_TIMEOUT: Duration = Duration::from_secs(5);

        /// Performance test timeout
        pub const PERFORMANCE_TIMEOUT: Duration = Duration::from_secs(60);

        /// Chaos test timeout
        pub const CHAOS_TIMEOUT: Duration = Duration::from_secs(10);

        /// Maximum allowed TTFB for performance tests
        pub const MAX_TTFB: Duration = Duration::from_millis(500);

        /// Maximum allowed P95 latency for batch processing
        pub const MAX_P95_LATENCY: Duration = Duration::from_secs(5);

        /// Minimum required throughput (requests per second)
        pub const MIN_THROUGHPUT: f64 = 10.0;

        /// Maximum memory usage for single extraction (bytes)
        pub const MAX_MEMORY_PER_EXTRACTION: u64 = 50 * 1024 * 1024; // 50MB

        /// Test URLs for various scenarios
        pub fn test_urls() -> Vec<(&'static str, &'static str)> {
            vec![
                ("https://example.com/article", "article"),
                ("https://spa-app.com/dashboard", "spa"),
                ("https://docs.example.com/api.pdf", "pdf"),
                ("https://news.com/breaking-news", "news"),
                ("https://ecommerce.com/product/123", "product"),
            ]
        }

        /// Error rate thresholds for different test scenarios
        pub fn error_rate_thresholds() -> std::collections::HashMap<&'static str, f64> {
            std::collections::HashMap::from([
                ("unit_tests", 0.0),           // Unit tests should have 0% error rate
                ("integration_tests", 0.05),   // Integration tests allow 5% error rate
                ("chaos_tests", 0.50),         // Chaos tests expect up to 50% errors
                ("performance_tests", 0.10),   // Performance tests allow 10% error rate
            ])
        }
    }
}

// Test runners and utilities for different test types
pub mod runners {
    use crate::config::TestConfig;
    use std::time::{Duration, Instant};

    /// Test runner for performance benchmarks
    pub struct PerformanceTestRunner {
        pub test_name: String,
        pub iterations: usize,
        pub timeout: Duration,
    }

    impl PerformanceTestRunner {
        pub fn new(test_name: &str) -> Self {
            Self {
                test_name: test_name.to_string(),
                iterations: 100,
                timeout: TestConfig::PERFORMANCE_TIMEOUT,
            }
        }

        pub fn with_iterations(mut self, iterations: usize) -> Self {
            self.iterations = iterations;
            self
        }

        pub async fn run<F, Fut, T>(&self, test_fn: F) -> PerformanceResults<T>
        where
            F: Fn() -> Fut,
            Fut: std::future::Future<Output = Result<T, String>>,
        {
            let mut results = Vec::new();
            let mut errors = Vec::new();
            let start_time = Instant::now();

            for i in 0..self.iterations {
                let iteration_start = Instant::now();
                match test_fn().await {
                    Ok(result) => {
                        results.push(PerformanceResult {
                            iteration: i,
                            duration: iteration_start.elapsed(),
                            result,
                        });
                    }
                    Err(error) => {
                        errors.push(PerformanceError {
                            iteration: i,
                            duration: iteration_start.elapsed(),
                            error,
                        });
                    }
                }
            }

            PerformanceResults {
                test_name: self.test_name.clone(),
                total_duration: start_time.elapsed(),
                iterations: self.iterations,
                success_count: results.len(),
                error_count: errors.len(),
                results,
                errors,
            }
        }
    }

    #[derive(Debug)]
    pub struct PerformanceResults<T> {
        pub test_name: String,
        pub total_duration: Duration,
        pub iterations: usize,
        pub success_count: usize,
        pub error_count: usize,
        pub results: Vec<PerformanceResult<T>>,
        pub errors: Vec<PerformanceError>,
    }

    #[derive(Debug)]
    pub struct PerformanceResult<T> {
        pub iteration: usize,
        pub duration: Duration,
        pub result: T,
    }

    #[derive(Debug)]
    pub struct PerformanceError {
        pub iteration: usize,
        pub duration: Duration,
        pub error: String,
    }

    impl<T> PerformanceResults<T> {
        pub fn success_rate(&self) -> f64 {
            self.success_count as f64 / self.iterations as f64
        }

        pub fn average_duration(&self) -> Duration {
            if self.results.is_empty() {
                return Duration::ZERO;
            }

            let total: Duration = self.results.iter().map(|r| r.duration).sum();
            total / self.results.len() as u32
        }

        pub fn p95_duration(&self) -> Option<Duration> {
            if self.results.is_empty() {
                return None;
            }

            let mut durations: Vec<Duration> = self.results.iter().map(|r| r.duration).collect();
            durations.sort();

            let index = (durations.len() as f64 * 0.95).ceil() as usize - 1;
            durations.get(index).copied()
        }

        pub fn throughput(&self) -> f64 {
            self.success_count as f64 / self.total_duration.as_secs_f64()
        }

        pub fn print_summary(&self) {
            println!("\n=== Performance Test Results: {} ===", self.test_name);
            println!("Iterations: {}", self.iterations);
            println!("Success Rate: {:.2}%", self.success_rate() * 100.0);
            println!("Average Duration: {:?}", self.average_duration());
            if let Some(p95) = self.p95_duration() {
                println!("P95 Duration: {:?}", p95);
            }
            println!("Throughput: {:.2} req/s", self.throughput());
            println!("Total Duration: {:?}", self.total_duration);

            if !self.errors.is_empty() {
                println!("\nErrors:");
                for error in &self.errors {
                    println!("  Iteration {}: {}", error.iteration, error.error);
                }
            }
        }
    }
}

// Test data validation and assertion helpers
pub mod assertions {
    use std::time::Duration;

    /// Assert that extracted content meets quality standards
    pub fn assert_content_quality(content: &crate::fixtures::ExtractedContent) {
        assert!(!content.url.is_empty(), "Content should have a URL");
        assert!(!content.content.is_empty(), "Content should not be empty");

        // Title should be present for most content types
        if content.url.contains("article") || content.url.contains("news") {
            assert!(content.title.is_some(), "Articles should have titles");
        }

        // Links should be valid URLs if present
        for link in &content.links {
            assert!(link.starts_with("http"), "Links should be valid URLs: {}", link);
        }

        // Images should be valid URLs if present
        for image in &content.images {
            assert!(image.starts_with("http"), "Image URLs should be valid: {}", image);
        }
    }

    /// Assert that performance metrics meet SLO requirements
    pub fn assert_performance_slo(
        duration: Duration,
        throughput: f64,
        error_rate: f64,
        test_type: &str,
    ) {
        use crate::config::TestConfig;

        // Check timing constraints based on test type
        match test_type {
            "ttfb" => {
                assert!(
                    duration <= TestConfig::MAX_TTFB,
                    "TTFB {:?} exceeds maximum {:?}",
                    duration,
                    TestConfig::MAX_TTFB
                );
            }
            "p95_latency" => {
                assert!(
                    duration <= TestConfig::MAX_P95_LATENCY,
                    "P95 latency {:?} exceeds maximum {:?}",
                    duration,
                    TestConfig::MAX_P95_LATENCY
                );
            }
            _ => {} // Other test types have flexible timing
        }

        // Check throughput requirements
        assert!(
            throughput >= TestConfig::MIN_THROUGHPUT,
            "Throughput {:.2} req/s is below minimum {:.2} req/s",
            throughput,
            TestConfig::MIN_THROUGHPUT
        );

        // Check error rate thresholds
        let max_error_rate = TestConfig::error_rate_thresholds()
            .get(test_type)
            .copied()
            .unwrap_or(0.10); // Default 10% error rate

        assert!(
            error_rate <= max_error_rate,
            "Error rate {:.2}% exceeds maximum {:.2}% for test type '{}'",
            error_rate * 100.0,
            max_error_rate * 100.0,
            test_type
        );
    }

    /// Assert that component health meets requirements
    pub fn assert_component_health(health: &crate::fixtures::HealthStatus) {
        assert!(
            ["healthy", "degraded"].contains(&health.status.as_str()),
            "Component status '{}' should be healthy or degraded",
            health.status
        );

        assert!(!health.version.is_empty(), "Version should not be empty");
        assert!(health.memory_usage > 0, "Memory usage should be positive");
    }

    /// Assert that API responses follow contract requirements
    pub fn assert_api_contract_compliance<T: std::fmt::Debug>(
        result: &Result<T, String>,
        expected_success: bool,
    ) {
        if expected_success {
            assert!(
                result.is_ok(),
                "API call should succeed but got error: {:?}",
                result.as_ref().unwrap_err()
            );
        } else {
            assert!(
                result.is_err(),
                "API call should fail but got success: {:?}",
                result.as_ref().unwrap()
            );

            let error = result.as_ref().unwrap_err();
            assert!(!error.is_empty(), "Error message should not be empty");
        }
    }
}