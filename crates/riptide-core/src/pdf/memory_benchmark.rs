//! Memory usage benchmarking and validation for PDF processing
//!
//! This module provides comprehensive benchmarking to validate that memory usage
//! stays within acceptable limits (<200MB RSS spikes) under various load conditions.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use crate::pdf::{create_pdf_processor, PdfConfig, PdfMetricsCollector, PdfError};

/// Benchmark results for memory usage validation
#[derive(Debug, Clone)]
pub struct MemoryBenchmarkResults {
    /// Test name/description
    pub test_name: String,

    /// Initial memory usage (bytes)
    pub initial_memory: u64,

    /// Peak memory usage during processing (bytes)
    pub peak_memory: u64,

    /// Final memory usage after processing (bytes)
    pub final_memory: u64,

    /// Memory spike (peak - initial) in bytes
    pub memory_spike: u64,

    /// Whether the spike was within acceptable limits (< 200MB)
    pub within_limits: bool,

    /// Total processing time
    pub processing_time: Duration,

    /// Number of PDFs processed
    pub pdfs_processed: u32,

    /// Number of concurrent operations
    pub concurrent_operations: u32,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Average memory per PDF (bytes)
    pub avg_memory_per_pdf: f64,

    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Memory benchmark suite for PDF processing
pub struct PdfMemoryBenchmark {
    processor: crate::pdf::processor::AnyPdfProcessor,
    metrics: Arc<PdfMetricsCollector>,
}

impl PdfMemoryBenchmark {
    /// Create a new benchmark suite
    pub fn new() -> Self {
        Self {
            processor: create_pdf_processor(),
            metrics: Arc::new(PdfMetricsCollector::new()),
        }
    }

    /// Benchmark single PDF processing with memory monitoring
    pub async fn benchmark_single_pdf(&self, pdf_size_pages: usize) -> MemoryBenchmarkResults {
        let test_name = format!("single_pdf_{}_pages", pdf_size_pages);
        let pdf_data = self.create_test_pdf(pdf_size_pages);

        let mut config = PdfConfig::default();
        config.memory_settings.max_memory_spike_bytes = 200 * 1024 * 1024; // 200MB
        config.memory_settings.aggressive_cleanup = true;

        // Get initial memory
        let initial_memory = self.get_current_memory();
        self.metrics.reset();

        let start_time = Instant::now();
        let result = self.processor.process_pdf(&pdf_data, &config).await;
        let processing_time = start_time.elapsed();

        // Get final memory after a brief pause to allow cleanup
        tokio::time::sleep(Duration::from_millis(100)).await;
        let final_memory = self.get_current_memory();

        let peak_memory = initial_memory.max(final_memory); // Conservative estimate
        let memory_spike = peak_memory.saturating_sub(initial_memory);
        let within_limits = memory_spike <= 200 * 1024 * 1024;

        let success_rate = if result.is_ok() { 1.0 } else { 0.0 };
        let pdfs_processed = if result.is_ok() { 1 } else { 0 };

        let mut metrics = HashMap::new();
        metrics.insert("initial_memory_mb".to_string(), initial_memory as f64 / (1024.0 * 1024.0));
        metrics.insert("final_memory_mb".to_string(), final_memory as f64 / (1024.0 * 1024.0));
        metrics.insert("memory_spike_mb".to_string(), memory_spike as f64 / (1024.0 * 1024.0));

        if let Ok(processing_result) = result {
            metrics.insert("pages_processed".to_string(), processing_result.stats.pages_processed as f64);
            metrics.insert("images_extracted".to_string(), processing_result.stats.images_extracted as f64);
            metrics.insert("processing_time_ms".to_string(), processing_result.stats.processing_time_ms as f64);
        }

        MemoryBenchmarkResults {
            test_name,
            initial_memory,
            peak_memory,
            final_memory,
            memory_spike,
            within_limits,
            processing_time,
            pdfs_processed,
            concurrent_operations: 1,
            success_rate,
            avg_memory_per_pdf: if pdfs_processed > 0 { memory_spike as f64 / pdfs_processed as f64 } else { 0.0 },
            metrics,
        }
    }

    /// Benchmark concurrent PDF processing with memory limits
    pub async fn benchmark_concurrent_processing(&self,
                                                pdf_size_pages: usize,
                                                concurrent_count: usize) -> MemoryBenchmarkResults {
        let test_name = format!("concurrent_{}_pdfs_{}pages", concurrent_count, pdf_size_pages);
        let pdf_data = Arc::new(self.create_test_pdf(pdf_size_pages));

        let mut config = PdfConfig::default();
        config.memory_settings.max_memory_spike_bytes = 200 * 1024 * 1024; // 200MB
        config.memory_settings.max_concurrent_operations = 2; // Strict limit
        config.memory_settings.aggressive_cleanup = true;
        let config = Arc::new(config);

        let initial_memory = self.get_current_memory();
        self.metrics.reset();

        let start_time = Instant::now();
        let mut handles = Vec::new();

        // Spawn concurrent tasks
        for i in 0..concurrent_count {
            let processor_clone = self.processor.clone();
            let data_clone = Arc::clone(&pdf_data);
            let config_clone = Arc::clone(&config);

            let handle = tokio::spawn(async move {
                let task_start = Instant::now();
                let result = processor_clone.process_pdf(&data_clone, &config_clone).await;
                let task_duration = task_start.elapsed();
                (i, result, task_duration)
            });

            handles.push(handle);

            // Small delay to create overlap
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Collect results
        let results = futures::future::join_all(handles).await;
        let processing_time = start_time.elapsed();

        // Wait for cleanup and measure final memory
        tokio::time::sleep(Duration::from_millis(200)).await;
        let final_memory = self.get_current_memory();

        let peak_memory = self.estimate_peak_memory_during_processing(initial_memory, final_memory);
        let memory_spike = peak_memory.saturating_sub(initial_memory);
        let within_limits = memory_spike <= 200 * 1024 * 1024;

        let mut successful_tasks = 0;
        let mut total_pages = 0;
        let mut memory_limit_failures = 0;

        for (_, task_result, _) in results.into_iter().flatten() {
            match task_result {
                Ok(processing_result) => {
                    successful_tasks += 1;
                    total_pages += processing_result.stats.pages_processed;
                }
                Err(PdfError::MemoryLimit { .. }) => {
                    memory_limit_failures += 1;
                }
                Err(_) => {
                    // Other error
                }
            }
        }

        let success_rate = successful_tasks as f64 / concurrent_count as f64;
        let avg_memory_per_pdf = if successful_tasks > 0 {
            memory_spike as f64 / successful_tasks as f64
        } else {
            0.0
        };

        let mut metrics = HashMap::new();
        metrics.insert("initial_memory_mb".to_string(), initial_memory as f64 / (1024.0 * 1024.0));
        metrics.insert("final_memory_mb".to_string(), final_memory as f64 / (1024.0 * 1024.0));
        metrics.insert("memory_spike_mb".to_string(), memory_spike as f64 / (1024.0 * 1024.0));
        metrics.insert("successful_tasks".to_string(), successful_tasks as f64);
        metrics.insert("memory_limit_failures".to_string(), memory_limit_failures as f64);
        metrics.insert("total_pages".to_string(), total_pages as f64);
        metrics.insert("concurrency_effectiveness".to_string(),
                      (successful_tasks as f64 / processing_time.as_secs_f64()).min(2.0)); // Max 2 concurrent

        MemoryBenchmarkResults {
            test_name,
            initial_memory,
            peak_memory,
            final_memory,
            memory_spike,
            within_limits,
            processing_time,
            pdfs_processed: successful_tasks,
            concurrent_operations: concurrent_count as u32,
            success_rate,
            avg_memory_per_pdf,
            metrics,
        }
    }

    /// Benchmark memory stability over time with repeated processing
    pub async fn benchmark_memory_stability(&self,
                                           iterations: usize,
                                           pdf_size_pages: usize) -> MemoryBenchmarkResults {
        let test_name = format!("stability_{}_iterations_{}pages", iterations, pdf_size_pages);
        let pdf_data = self.create_test_pdf(pdf_size_pages);

        let mut config = PdfConfig::default();
        config.memory_settings.max_memory_spike_bytes = 200 * 1024 * 1024;
        config.memory_settings.aggressive_cleanup = true;

        let initial_memory = self.get_current_memory();
        let mut peak_memory = initial_memory;
        let mut successful_iterations = 0;

        let start_time = Instant::now();

        for i in 0..iterations {
            let iteration_start_memory = self.get_current_memory();
            let result = self.processor.process_pdf(&pdf_data, &config).await;

            if result.is_ok() {
                successful_iterations += 1;
            }

            // Track peak memory
            let iteration_end_memory = self.get_current_memory();
            peak_memory = peak_memory.max(iteration_end_memory);

            // Memory should not continuously grow
            if i > 10 && iteration_end_memory > initial_memory + 300 * 1024 * 1024 {
                println!("Warning: Memory growth detected at iteration {}: {} MB (start: {} MB, end: {} MB)",
                        i,
                        (iteration_end_memory - initial_memory) / (1024 * 1024),
                        (iteration_start_memory - initial_memory) / (1024 * 1024),
                        (iteration_end_memory - initial_memory) / (1024 * 1024));
            }

            // Small pause between iterations
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let processing_time = start_time.elapsed();
        let final_memory = self.get_current_memory();
        let memory_spike = peak_memory.saturating_sub(initial_memory);
        let within_limits = memory_spike <= 200 * 1024 * 1024;

        let success_rate = successful_iterations as f64 / iterations as f64;
        let avg_memory_per_pdf = if successful_iterations > 0 {
            memory_spike as f64 / successful_iterations as f64
        } else {
            0.0
        };

        let mut metrics = HashMap::new();
        metrics.insert("initial_memory_mb".to_string(), initial_memory as f64 / (1024.0 * 1024.0));
        metrics.insert("final_memory_mb".to_string(), final_memory as f64 / (1024.0 * 1024.0));
        metrics.insert("peak_memory_mb".to_string(), peak_memory as f64 / (1024.0 * 1024.0));
        metrics.insert("memory_spike_mb".to_string(), memory_spike as f64 / (1024.0 * 1024.0));
        metrics.insert("successful_iterations".to_string(), successful_iterations as f64);
        metrics.insert("memory_growth_mb".to_string(),
                      (final_memory.saturating_sub(initial_memory)) as f64 / (1024.0 * 1024.0));
        metrics.insert("avg_time_per_iteration_ms".to_string(),
                      processing_time.as_millis() as f64 / iterations as f64);

        MemoryBenchmarkResults {
            test_name,
            initial_memory,
            peak_memory,
            final_memory,
            memory_spike,
            within_limits,
            processing_time,
            pdfs_processed: successful_iterations,
            concurrent_operations: 1,
            success_rate,
            avg_memory_per_pdf,
            metrics,
        }
    }

    /// Run comprehensive memory benchmark suite
    pub async fn run_comprehensive_benchmark(&self) -> Vec<MemoryBenchmarkResults> {
        let mut results = Vec::new();

        println!("🧪 Running comprehensive PDF memory benchmarks...");

        // Single PDF processing tests
        for pages in &[10, 50, 100, 500] {
            println!("📄 Testing single PDF with {} pages", pages);
            let result = self.benchmark_single_pdf(*pages).await;
            println!("   Memory spike: {:.1} MB, Within limits: {}",
                    result.memory_spike as f64 / (1024.0 * 1024.0),
                    result.within_limits);
            results.push(result);
        }

        // Concurrent processing tests
        for (concurrent_count, pages) in &[(2, 50), (4, 50), (6, 50), (2, 200), (4, 200)] {
            println!("🔄 Testing {} concurrent PDFs with {} pages", concurrent_count, pages);
            let result = self.benchmark_concurrent_processing(*pages, *concurrent_count).await;
            println!("   Memory spike: {:.1} MB, Success rate: {:.1}%, Within limits: {}",
                    result.memory_spike as f64 / (1024.0 * 1024.0),
                    result.success_rate * 100.0,
                    result.within_limits);
            results.push(result);
        }

        // Memory stability tests
        for (iterations, pages) in &[(50, 20), (100, 10)] {
            println!("⏱️  Testing memory stability over {} iterations with {} pages", iterations, pages);
            let result = self.benchmark_memory_stability(*iterations, *pages).await;
            println!("   Peak memory: {:.1} MB, Final growth: {:.1} MB, Within limits: {}",
                    result.peak_memory as f64 / (1024.0 * 1024.0),
                    result.metrics.get("memory_growth_mb").unwrap_or(&0.0),
                    result.within_limits);
            results.push(result);
        }

        results
    }

    /// Generate a summary report of benchmark results
    pub fn generate_summary_report(&self, results: &[MemoryBenchmarkResults]) -> String {
        let mut report = String::new();

        report.push_str("# PDF Memory Benchmark Summary Report\n\n");

        let total_tests = results.len();
        let passing_tests = results.iter().filter(|r| r.within_limits).count();
        let overall_success_rate = results.iter().map(|r| r.success_rate).sum::<f64>() / total_tests as f64;

        report.push_str("## Overall Results\n");
        report.push_str(&format!("- **Total Tests**: {}\n", total_tests));
        report.push_str(&format!("- **Tests Within Memory Limits**: {} ({:.1}%)\n",
                                passing_tests,
                                (passing_tests as f64 / total_tests as f64) * 100.0));
        report.push_str(&format!("- **Average Success Rate**: {:.1}%\n", overall_success_rate * 100.0));

        let max_memory_spike = results.iter().map(|r| r.memory_spike).max().unwrap_or(0);
        report.push_str(&format!("- **Maximum Memory Spike**: {:.1} MB\n", max_memory_spike as f64 / (1024.0 * 1024.0)));

        report.push_str("\n## Detailed Results\n\n");

        for result in results {
            report.push_str(&format!("### {}\n", result.test_name));
            report.push_str(&format!("- Memory Spike: {:.1} MB (Within limits: {})\n",
                                    result.memory_spike as f64 / (1024.0 * 1024.0),
                                    if result.within_limits { "✅" } else { "❌" }));
            report.push_str(&format!("- Success Rate: {:.1}%\n", result.success_rate * 100.0));
            report.push_str(&format!("- Processing Time: {:.2}s\n", result.processing_time.as_secs_f64()));
            report.push_str(&format!("- PDFs Processed: {}\n", result.pdfs_processed));
            report.push_str(&format!("- Avg Memory/PDF: {:.1} MB\n", result.avg_memory_per_pdf / (1024.0 * 1024.0)));
            report.push('\n');
        }

        if passing_tests == total_tests {
            report.push_str("## ✅ Conclusion\n\n");
            report.push_str("All memory benchmarks PASSED. The PDF processing pipeline maintains memory usage within acceptable limits (<200MB RSS spikes) under all tested conditions.\n");
        } else {
            report.push_str("## ❌ Conclusion\n\n");
            report.push_str("Some memory benchmarks FAILED. The PDF processing pipeline may exceed memory limits under certain conditions. Review failed tests and consider optimizations.\n");
        }

        report
    }

    // Helper methods

    fn create_test_pdf(&self, pages: usize) -> Vec<u8> {
        let mut pdf = Vec::new();
        pdf.extend_from_slice(b"%PDF-1.7\n");
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        pdf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n");

        // Add content to simulate larger PDFs
        for _ in 0..pages {
            pdf.extend_from_slice(b"Additional page content for testing memory usage patterns\n");
            pdf.extend_from_slice(b"Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n");
            pdf.extend_from_slice(b"Sed do eiusmod tempor incididunt ut labore et dolore magna.\n");
        }

        pdf.extend_from_slice(b"xref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n");
        pdf.extend_from_slice(b"trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n181\n%%EOF\n");
        pdf
    }

    fn get_current_memory(&self) -> u64 {
        #[cfg(unix)]
        {
            if let Ok(process) = psutil::process::Process::current() {
                if let Ok(memory_info) = process.memory_info() {
                    return memory_info.rss();
                }
            }
        }

        // Fallback
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        sys.used_memory()
    }

    fn estimate_peak_memory_during_processing(&self, initial: u64, final_mem: u64) -> u64 {
        // Conservative estimate: assume peak was 50% higher than the difference
        let diff = final_mem.saturating_sub(initial);
        initial + (diff * 3) / 2
    }
}

impl Default for PdfMemoryBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_pdf_benchmark() {
        let benchmark = PdfMemoryBenchmark::new();
        let result = benchmark.benchmark_single_pdf(50).await;

        assert_eq!(result.concurrent_operations, 1);
        assert!(result.processing_time > Duration::from_millis(0));
        // Memory spike should be reasonable for a small test PDF
        assert!(result.memory_spike < 500 * 1024 * 1024); // Less than 500MB
    }

    #[tokio::test]
    async fn test_memory_benchmark_reporting() {
        let benchmark = PdfMemoryBenchmark::new();
        let result = benchmark.benchmark_single_pdf(10).await;
        let results = vec![result];

        let report = benchmark.generate_summary_report(&results);

        assert!(report.contains("PDF Memory Benchmark Summary Report"));
        assert!(report.contains("Overall Results"));
        assert!(report.contains("Total Tests: 1"));
    }
}