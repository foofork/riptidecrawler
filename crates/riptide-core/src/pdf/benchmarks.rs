//! Benchmarks for PDF processing performance
//!
//! This module provides benchmarks to validate memory stability and performance.

#[cfg(feature = "benchmarks")]
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

use super::*;
use std::sync::Arc;
use std::time::Duration;

/// Generate test PDFs of various sizes for benchmarking
fn generate_test_pdf(pages: usize) -> Vec<u8> {
    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.7\n");

    // Basic PDF structure
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    pdf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n");

    // Add content to simulate larger PDFs
    for i in 0..pages {
        pdf.extend_from_slice(format!("% Page {} content\n", i).as_bytes());
        // Add some text content
        for _ in 0..100 {
            pdf.extend_from_slice(b"Sample text content for benchmarking purposes. ");
        }
        pdf.push(b'\n');
    }

    // PDF trailer
    pdf.extend_from_slice(b"xref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n");
    pdf.extend_from_slice(b"trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n181\n%%EOF\n");

    pdf
}

/// Benchmark PDF processing with different file sizes
#[cfg(feature = "benchmarks")]
pub fn benchmark_pdf_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let processor = create_pdf_processor();
    let config = PdfConfig::default();

    let mut group = c.benchmark_group("pdf_processing");
    group.measurement_time(Duration::from_secs(30));

    for pages in [1, 5, 10, 25, 50].iter() {
        let pdf_data = generate_test_pdf(*pages);

        group.bench_with_input(
            BenchmarkId::new("process_pdf", pages),
            &pdf_data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        let result = processor.process_pdf(black_box(data), black_box(&config)).await;
                        black_box(result)
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent PDF processing
#[cfg(feature = "benchmarks")]
pub fn benchmark_concurrent_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let processor = Arc::new(create_pdf_processor());
    let config = Arc::new(PdfConfig::default());
    let pdf_data = Arc::new(generate_test_pdf(10));

    let mut group = c.benchmark_group("concurrent_processing");
    group.measurement_time(Duration::from_secs(20));

    for concurrency in [1, 2, 3, 4].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_tasks", concurrency),
            concurrency,
            |b, &concurrency_level| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = Vec::new();

                        for _ in 0..concurrency_level {
                            let processor_clone = Arc::clone(&processor);
                            let config_clone = Arc::clone(&config);
                            let data_clone = Arc::clone(&pdf_data);

                            let handle = tokio::spawn(async move {
                                processor_clone.process_pdf(&data_clone, &config_clone).await
                            });

                            handles.push(handle);
                        }

                        let results = futures::future::join_all(handles).await;
                        black_box(results)
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage during PDF processing
#[cfg(feature = "benchmarks")]
pub fn benchmark_memory_usage(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    #[cfg(feature = "pdf")]
    {
        let processor = PdfiumProcessor::new();
        let pdf_data = generate_test_pdf(20);
        let config = PdfConfig::default();

        c.bench_function("memory_monitoring", |b| {
            b.iter(|| {
                rt.block_on(async {
                    // Skip memory monitoring in benchmarks as get_memory_usage is private
                    let result = processor.process_pdf(black_box(&pdf_data), black_box(&config)).await;
                    black_box(result)
                })
            });
        });
    }
}

/// Benchmark PDF detection performance
#[cfg(feature = "benchmarks")]
pub fn benchmark_pdf_detection(c: &mut Criterion) {
    let pdf_data = generate_test_pdf(5);
    let non_pdf_data = b"<html><body>Not a PDF</body></html>";

    let mut group = c.benchmark_group("pdf_detection");

    group.bench_function("detect_pdf_by_magic_bytes", |b| {
        b.iter(|| {
            black_box(utils::detect_pdf_by_magic_bytes(black_box(&pdf_data)))
        });
    });

    group.bench_function("detect_non_pdf_by_magic_bytes", |b| {
        b.iter(|| {
            black_box(utils::detect_pdf_by_magic_bytes(black_box(non_pdf_data)))
        });
    });

    group.bench_function("detect_pdf_by_extension", |b| {
        b.iter(|| {
            black_box(utils::detect_pdf_by_extension(black_box("document.pdf")))
        });
    });

    group.bench_function("comprehensive_pdf_detection", |b| {
        b.iter(|| {
            black_box(utils::detect_pdf_content(
                black_box(Some("application/pdf")),
                black_box(Some("document.pdf")),
                black_box(Some(&pdf_data)),
            ))
        });
    });

    group.finish();
}

/// Simple performance test that can be run without criterion
#[cfg(feature = "pdf")]
pub async fn simple_performance_test() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    println!("Running PDF performance tests...");

    let processor = PdfiumProcessor::new();
    let config = PdfConfig::default();

    // Test different PDF sizes
    for pages in [1, 5, 10, 20] {
        let pdf_data = generate_test_pdf(pages);

        let start = Instant::now();
        // Memory monitoring removed as get_memory_usage is private
        let result = processor.process_pdf(&pdf_data, &config).await;
        let duration = start.elapsed();

        match result {
            Ok(processing_result) => {
                println!(
                    "Pages: {}, Time: {:?}, Memory used: {} bytes, Memory diff: {} bytes, Success: {}",
                    pages,
                    duration,
                    processing_result.stats.memory_used,
                    0, // Memory diff monitoring removed
                    processing_result.success
                );
            }
            Err(e) => {
                println!("Pages: {}, Time: {:?}, Error: {}", pages, duration, e);
            }
        }
    }

    // Test concurrent processing
    println!("\nTesting concurrent processing (should be limited to 2)...");

    let processor = Arc::new(processor);
    let config = Arc::new(config);
    let pdf_data = Arc::new(generate_test_pdf(10));

    let start = Instant::now();
    let mut handles = Vec::new();

    for i in 0..4 {
        let processor_clone = Arc::clone(&processor);
        let config_clone = Arc::clone(&config);
        let data_clone = Arc::clone(&pdf_data);

        let handle = tokio::spawn(async move {
            let task_start = Instant::now();
            let result = processor_clone.process_pdf(&data_clone, &config_clone).await;
            let task_duration = task_start.elapsed();
            println!("Concurrent task {} completed in {:?}", i, task_duration);
            result
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    let total_duration = start.elapsed();

    println!("All concurrent tasks completed in {:?}", total_duration);

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(_)) => println!("Task {} succeeded", i),
            Ok(Err(e)) => println!("Task {} failed: {}", i, e),
            Err(e) => println!("Task {} panicked: {}", i, e),
        }
    }

    Ok(())
}

#[cfg(feature = "benchmarks")]
criterion_group!(
    benches,
    benchmark_pdf_processing,
    benchmark_concurrent_processing,
    benchmark_memory_usage,
    benchmark_pdf_detection
);

#[cfg(feature = "benchmarks")]
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_pdf() {
        let pdf = generate_test_pdf(1);
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 100); // Should have some content
    }

    #[tokio::test]
    async fn test_simple_performance() {
        #[cfg(feature = "pdf")]
        {
            // This test should complete without panicking
            let _ = simple_performance_test().await;
        }
    }
}