//! Memory stability test for PDF Pipeline optimization
//!
//! This test validates the ROADMAP requirements:
//! - PDF: ≤2 concurrent operations
//! - No >200MB RSS spikes per worker
//! - Stable memory under sustained load

use riptide_core::pdf::{
    create_pdf_processor, PdfConfig, PdfMetricsCollector, PdfPipelineIntegration,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tracing::{info, warn};

/// Test configuration for memory stability validation
#[derive(Debug, Clone)]
pub struct MemoryStabilityTestConfig {
    /// Duration to run the sustained load test
    pub test_duration: Duration,
    /// Number of PDF processing tasks to spawn
    pub total_operations: usize,
    /// Size of test PDF in pages
    pub pdf_size_pages: usize,
    /// Expected concurrency limit
    pub max_concurrent: usize,
    /// Memory spike limit in bytes
    pub memory_spike_limit: u64,
}

impl Default for MemoryStabilityTestConfig {
    fn default() -> Self {
        Self {
            test_duration: Duration::from_secs(30),
            total_operations: 50,
            pdf_size_pages: 10,
            max_concurrent: 2,                     // ROADMAP requirement
            memory_spike_limit: 200 * 1024 * 1024, // ROADMAP requirement: 200MB
        }
    }
}

/// Results from memory stability testing
#[derive(Debug)]
pub struct MemoryStabilityResults {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub memory_spike_violations: usize,
    pub max_concurrent_observed: usize,
    pub max_memory_spike_bytes: u64,
    pub average_processing_time_ms: f64,
    pub memory_stability_score: f64,
    pub concurrency_compliance: bool,
    pub memory_compliance: bool,
}

/// Memory stability test runner
pub struct MemoryStabilityTester {
    config: MemoryStabilityTestConfig,
    integration: Arc<PdfPipelineIntegration>,
}

impl MemoryStabilityTester {
    /// Create new memory stability tester
    pub fn new(config: MemoryStabilityTestConfig) -> Self {
        // Create PDF integration with ROADMAP-compliant configuration
        let pdf_config = PdfConfig {
            max_size_bytes: 100 * 1024 * 1024, // 100MB max
            extract_text: true,
            extract_images: false, // Disable to focus on memory stability
            extract_metadata: true,
            memory_settings: riptide_core::pdf::config::MemorySettings {
                max_memory_spike_bytes: config.memory_spike_limit,
                max_concurrent_operations: config.max_concurrent,
                memory_check_interval: 3,
                cleanup_interval: 10,
                memory_pressure_threshold: 0.8,
                aggressive_cleanup: true,
            },
            ..Default::default()
        };

        let integration = Arc::new(
            riptide_core::pdf::integration::PdfPipelineIntegration::with_config(pdf_config),
        );

        Self {
            config,
            integration,
        }
    }

    /// Generate synthetic PDF data for testing
    fn generate_test_pdf(&self, pages: usize) -> Vec<u8> {
        let mut pdf_data = b"%PDF-1.7\n".to_vec();

        // Add minimal PDF structure with content for specified pages
        for page_num in 1..=pages {
            let page_content = format!(
                "1 0 obj<< /Type/Page /Contents 2 0 R >>endobj\n\
                2 0 obj<< /Length {} >>stream\n\
                BT /F1 12 Tf 72 720 Td (Test PDF Page {}) Tj ET\n\
                endstream endobj\n",
                format!("Test PDF Page {}", page_num).len() + 30,
                page_num
            );
            pdf_data.extend_from_slice(page_content.as_bytes());
        }

        // Add minimal PDF trailer
        pdf_data.extend_from_slice(b"\nxref\n0 3\ntrailer<< /Size 3 /Root 1 0 R >>startxref\n");
        pdf_data.extend_from_slice(format!("{}\n%%EOF", pdf_data.len()).as_bytes());

        pdf_data
    }

    /// Run sustained load test to validate memory stability
    pub async fn run_sustained_load_test(&self) -> MemoryStabilityResults {
        info!(
            "Starting PDF memory stability test with {} operations over {:?}",
            self.config.total_operations, self.config.test_duration
        );

        let start_time = Instant::now();
        let mut join_set = JoinSet::new();
        let mut successful = 0;
        let mut failed = 0;
        let mut processing_times = Vec::new();

        // Get initial memory baseline
        let initial_memory = self.get_current_memory_usage();
        let mut max_memory_spike = 0u64;
        let mut memory_spike_violations = 0;
        let mut max_concurrent = 0;

        // Reset metrics before test
        self.integration.reset_metrics();

        // Spawn PDF processing tasks with controlled rate
        let test_pdf = self.generate_test_pdf(self.config.pdf_size_pages);
        let operations_per_second =
            self.config.total_operations as f64 / self.config.test_duration.as_secs_f64();
        let task_interval = Duration::from_millis((1000.0 / operations_per_second) as u64);

        for task_id in 0..self.config.total_operations {
            let integration = self.integration.clone();
            let pdf_data = test_pdf.clone();

            join_set.spawn(async move {
                let task_start = Instant::now();
                let url_str = format!("test://pdf/task/{}", task_id);
                let url = Some(url_str.as_str());

                match integration
                    .process_pdf_to_extracted_doc(&pdf_data, url)
                    .await
                {
                    Ok(_) => (task_id, true, task_start.elapsed()),
                    Err(e) => {
                        warn!("Task {} failed: {:?}", task_id, e);
                        (task_id, false, task_start.elapsed())
                    }
                }
            });

            // Monitor memory and concurrency periodically
            if task_id % 5 == 0 {
                let current_memory = self.get_current_memory_usage();
                let memory_spike = current_memory.saturating_sub(initial_memory);

                if memory_spike > max_memory_spike {
                    max_memory_spike = memory_spike;
                }

                if memory_spike > self.config.memory_spike_limit {
                    memory_spike_violations += 1;
                    warn!(
                        "Memory spike violation: {} MB (limit: {} MB)",
                        memory_spike / (1024 * 1024),
                        self.config.memory_spike_limit / (1024 * 1024)
                    );
                }

                // Check concurrent operations (approximation)
                let current_concurrent = join_set.len().min(10); // Reasonable upper bound
                if current_concurrent > max_concurrent {
                    max_concurrent = current_concurrent;
                }
            }

            // Rate limiting
            if task_id < self.config.total_operations - 1 {
                tokio::time::sleep(task_interval).await;
            }

            // Stop if we've exceeded test duration
            if start_time.elapsed() > self.config.test_duration {
                break;
            }
        }

        // Wait for all tasks to complete with timeout
        let completion_timeout = self.config.test_duration + Duration::from_secs(30);
        let completion_deadline = Instant::now() + completion_timeout;

        while !join_set.is_empty() && Instant::now() < completion_deadline {
            if let Some(result) = join_set.join_next().await {
                match result {
                    Ok((_task_id, success, duration)) => {
                        if success {
                            successful += 1;
                        } else {
                            failed += 1;
                        }
                        processing_times.push(duration.as_millis() as f64);
                    }
                    Err(e) => {
                        warn!("Task join failed: {:?}", e);
                        failed += 1;
                    }
                }
            }
        }

        // Abort remaining tasks if timeout
        join_set.abort_all();

        let total_operations = successful + failed;
        let average_processing_time = if !processing_times.is_empty() {
            processing_times.iter().sum::<f64>() / processing_times.len() as f64
        } else {
            0.0
        };

        // Get final metrics
        let metrics_snapshot = self.integration.get_metrics_snapshot();

        // Calculate compliance scores
        let concurrency_compliance = max_concurrent <= self.config.max_concurrent;
        let memory_compliance = memory_spike_violations == 0;
        let memory_stability_score = if memory_compliance && concurrency_compliance {
            1.0 - (memory_spike_violations as f64 / total_operations as f64).min(1.0)
        } else {
            0.0
        };

        let results = MemoryStabilityResults {
            total_operations,
            successful_operations: successful,
            failed_operations: failed,
            memory_spike_violations,
            max_concurrent_observed: max_concurrent,
            max_memory_spike_bytes: max_memory_spike,
            average_processing_time_ms: average_processing_time,
            memory_stability_score,
            concurrency_compliance,
            memory_compliance,
        };

        info!("Memory stability test completed: {:#?}", results);
        info!("PDF Metrics: {:#?}", metrics_snapshot);

        results
    }

    /// Get current memory usage (simplified implementation)
    fn get_current_memory_usage(&self) -> u64 {
        // Use sysinfo to get current process memory usage
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        sys.used_memory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_memory_stability_roadmap_requirements() {
        let config = MemoryStabilityTestConfig {
            test_duration: Duration::from_secs(15), // Shorter for testing
            total_operations: 20,
            pdf_size_pages: 5,
            ..Default::default()
        };

        let tester = MemoryStabilityTester::new(config);
        let results = tester.run_sustained_load_test().await;

        // Validate ROADMAP requirements
        assert!(
            results.concurrency_compliance,
            "Concurrency violation: observed {} concurrent operations, limit is {}",
            results.max_concurrent_observed, 2
        );

        assert!(
            results.memory_compliance,
            "Memory spike violations: {} (max spike: {} MB)",
            results.memory_spike_violations,
            results.max_memory_spike_bytes / (1024 * 1024)
        );

        assert!(
            results.memory_stability_score > 0.8,
            "Memory stability score too low: {}",
            results.memory_stability_score
        );

        // Performance validations
        assert!(
            results.successful_operations > results.failed_operations,
            "More failures than successes: {} failed vs {} successful",
            results.failed_operations,
            results.successful_operations
        );
    }

    #[test]
    async fn test_concurrent_pdf_limit_enforcement() {
        // Test specifically that we never exceed 2 concurrent PDF operations
        let config = MemoryStabilityTestConfig {
            test_duration: Duration::from_secs(10),
            total_operations: 30, // High operation count
            pdf_size_pages: 3,    // Small PDFs
            max_concurrent: 2,
            ..Default::default()
        };

        let tester = MemoryStabilityTester::new(config);
        let results = tester.run_sustained_load_test().await;

        assert!(
            results.concurrency_compliance,
            "PDF concurrency limit violated: {} concurrent operations observed (limit: 2)",
            results.max_concurrent_observed
        );
    }

    #[test]
    async fn test_memory_spike_prevention() {
        // Test specifically for memory spike prevention
        let config = MemoryStabilityTestConfig {
            test_duration: Duration::from_secs(20),
            total_operations: 15,
            pdf_size_pages: 15,                    // Larger PDFs to stress memory
            memory_spike_limit: 200 * 1024 * 1024, // 200MB limit
            ..Default::default()
        };

        let tester = MemoryStabilityTester::new(config);
        let results = tester.run_sustained_load_test().await;

        assert_eq!(
            results.memory_spike_violations,
            0,
            "Memory spike violations detected: {} spikes, max spike: {} MB",
            results.memory_spike_violations,
            results.max_memory_spike_bytes / (1024 * 1024)
        );
    }
}
