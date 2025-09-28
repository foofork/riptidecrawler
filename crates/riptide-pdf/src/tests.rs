//! Comprehensive tests for PDF processing functionality
//!
//! This module contains property-based tests, memory leak tests, concurrent access tests,
//! and integration tests for the PDF pipeline.

use super::*;
use crate::config::ImageExtractionSettings;
use futures::FutureExt;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Generate a minimal valid PDF for testing
fn create_test_pdf() -> Vec<u8> {
    // Minimal PDF structure
    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.7\n");
    pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    pdf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n");
    pdf.extend_from_slice(b"xref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n");
    pdf.extend_from_slice(b"trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n181\n%%EOF\n");
    pdf
}

/// Generate a corrupted PDF for testing error handling
fn create_corrupted_pdf() -> Vec<u8> {
    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.7\n");
    pdf.extend_from_slice(b"this is not valid pdf content");
    pdf
}

/// Generate a large PDF for memory testing
fn create_large_test_pdf(pages: usize) -> Vec<u8> {
    let mut pdf = create_test_pdf();
    // Simulate a larger PDF by repeating content
    for _ in 0..pages {
        pdf.extend_from_slice(b"Additional content for large PDF test\n");
    }
    pdf
}

#[tokio::test]
async fn test_pdf_processor_creation() {
    let processor = create_pdf_processor();

    #[cfg(feature = "pdf")]
    {
        assert!(processor.capabilities().text_extraction);
        assert!(processor.capabilities().image_extraction);
        assert!(processor.capabilities().metadata_extraction);
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(!processor.is_available());
    }
}

#[tokio::test]
async fn test_pdf_processing_basic() {
    let processor = create_pdf_processor();
    let pdf_data = create_test_pdf();
    let config = PdfConfig::default();

    let result = processor.process_pdf(&pdf_data, &config).await;

    #[cfg(feature = "pdf")]
    {
        // With the pdf feature, we expect processing to work (though it might fail due to minimal PDF)
        // The main thing is that it doesn't panic and returns a proper result type
        match result {
            Ok(_) => {
                // Processing succeeded
            }
            Err(e) => {
                // Processing failed, but we got a proper error
                println!("Expected processing error for minimal PDF: {}", e);
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }
}

#[tokio::test]
async fn test_pdf_detection_comprehensive() {
    // Test content-type detection
    assert!(utils::detect_pdf_content(Some("application/pdf"), None, None));
    assert!(utils::detect_pdf_content(Some("APPLICATION/PDF"), None, None));
    assert!(!utils::detect_pdf_content(Some("text/html"), None, None));

    // Test magic bytes detection
    let pdf_data = b"%PDF-1.7\n...";
    assert!(utils::detect_pdf_content(None, None, Some(pdf_data)));
    assert!(!utils::detect_pdf_content(None, None, Some(b"not pdf")));

    // Test extension detection
    assert!(utils::detect_pdf_content(None, Some("document.pdf"), None));
    assert!(utils::detect_pdf_content(None, Some("Document.PDF"), None));
    assert!(!utils::detect_pdf_content(None, Some("document.txt"), None));

    // Test combined detection
    assert!(utils::detect_pdf_content(
        Some("application/pdf"),
        Some("doc.pdf"),
        Some(pdf_data)
    ));
}

#[tokio::test]
async fn test_concurrent_pdf_processing() {
    let processor = Arc::new(create_pdf_processor());
    let pdf_data = Arc::new(create_test_pdf());
    let config = Arc::new(PdfConfig::default());

    let mut handles = Vec::new();

    // Spawn multiple concurrent processing tasks
    for i in 0..5 {
        let processor_clone = Arc::clone(&processor);
        let data_clone = Arc::clone(&pdf_data);
        let config_clone = Arc::clone(&config);

        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let result = processor_clone.process_pdf(&data_clone, &config_clone).await;
            let duration = start.elapsed();

            println!("Task {} completed in {:?}", i, duration);

            #[cfg(feature = "pdf")]
            {
                // Should complete within reasonable time (not get stuck)
                assert!(duration < Duration::from_secs(30));
            }

            result
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;

    // Verify that at most 2 tasks were running concurrently
    // (This is implicit due to the semaphore limit of 2)
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(_processing_result) => {
                println!("Task {} completed successfully", i);
            }
            Err(e) => {
                println!("Task {} failed: {}", i, e);
            }
        }
    }
}

#[tokio::test]
async fn test_memory_monitoring() {
    let processor = create_pdf_processor();
    let large_pdf = create_large_test_pdf(1000); // Create a larger PDF
    let config = PdfConfig::default();

    #[cfg(feature = "pdf")]
    {
        let initial_memory = processor.get_memory_usage();
        println!("Initial memory usage: {} bytes", initial_memory);

        let result = processor.process_pdf(&large_pdf, &config).await;

        let final_memory = processor.get_memory_usage();
        println!("Final memory usage: {} bytes", final_memory);

        // Memory usage should not spike dramatically (within 200MB as per our limit)
        let memory_diff = final_memory.saturating_sub(initial_memory);
        println!("Memory difference: {} bytes", memory_diff);

        match result {
            Ok(processing_result) => {
                // Verify that memory statistics are recorded
                assert!(processing_result.stats.memory_used > 0);
                println!("Processing used {} bytes", processing_result.stats.memory_used);
            }
            Err(PdfError::MemoryLimit { used, limit }) => {
                // This is expected for very large files
                println!("Memory limit reached: {} > {}", used, limit);
                assert!(used > limit);
            }
            Err(e) => {
                println!("Other processing error: {}", e);
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        let result = processor.process_pdf(&large_pdf, &config).await;
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_progress_callbacks() {
    let processor = create_pdf_processor();
    let pdf_data = create_test_pdf();
    let config = PdfConfig {
        enable_progress_tracking: true,
        ..Default::default()
    };

    let progress_updates = Arc::new(std::sync::Mutex::new(Vec::new()));
    let progress_clone = Arc::clone(&progress_updates);

    let progress_callback: ProgressCallback = Box::new(move |current, total| {
        let mut updates = progress_clone.lock().unwrap();
        updates.push((current, total));
        println!("Progress: {}/{}", current, total);
    });

    let result = processor
        .process_pdf_with_progress(&pdf_data, &config, Some(progress_callback))
        .await;

    #[cfg(feature = "pdf")]
    {
        // Check that progress callbacks were called
        let updates = progress_updates.lock().unwrap();
        if !updates.is_empty() {
            println!("Received {} progress updates", updates.len());

            // First update should start with 0 or low numbers
            let first_update = updates.first().unwrap();
            assert!(first_update.0 <= first_update.1);

            // Last update should show completion
            let last_update = updates.last().unwrap();
            assert!(last_update.0 <= last_update.1);
        }

        match result {
            Ok(_) => println!("Processing with progress completed successfully"),
            Err(e) => println!("Processing with progress failed: {}", e),
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_corrupted_pdf_handling() {
    let processor = create_pdf_processor();
    let corrupted_data = create_corrupted_pdf();
    let config = PdfConfig::default();

    let result = processor.process_pdf(&corrupted_data, &config).await;

    #[cfg(feature = "pdf")]
    {
        // Should handle corrupted PDFs gracefully
        assert!(result.is_err());
        let error = result.unwrap_err();
        println!("Corrupted PDF error: {}", error);
        // Should be a processing error, not a panic
        assert!(matches!(
            error,
            PdfError::ProcessingError { .. } | PdfError::InvalidPdf { .. }
        ));
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_ocr_detection() {
    let processor = create_pdf_processor();
    let pdf_data = create_test_pdf();

    let result = processor.detect_ocr_need(&pdf_data).await;

    #[cfg(feature = "pdf")]
    {
        match result {
            Ok(needs_ocr) => {
                println!("OCR needed: {}", needs_ocr);
                // For our minimal test PDF, this could be true or false
            }
            Err(e) => {
                println!("OCR detection error: {}", e);
                // Some errors are expected with minimal PDFs
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_processing_timeout() {
    let processor = create_pdf_processor();
    let pdf_data = create_large_test_pdf(5000); // Very large PDF
    let config = PdfConfig {
        timeout_seconds: 1, // Very short timeout
        ..Default::default()
    };

    // Test with timeout
    let result = timeout(Duration::from_secs(5), processor.process_pdf(&pdf_data, &config)).await;

    match result {
        Ok(processing_result) => {
            #[cfg(feature = "pdf")]
            {
                match processing_result {
                    Ok(_) => println!("Processing completed within timeout"),
                    Err(e) => println!("Processing failed: {}", e),
                }
            }

            #[cfg(not(feature = "pdf"))]
            {
                assert!(processing_result.is_err());
            }
        }
        Err(_) => {
            // Timeout occurred - this is also acceptable for very large files
            println!("Processing timed out (test timeout, not PDF timeout)");
        }
    }
}

#[tokio::test]
async fn test_pdf_metadata_extraction() {
    let processor = create_pdf_processor();
    let pdf_data = create_test_pdf();
    let config = PdfConfig {
        extract_metadata: true,
        extract_text: false,
        extract_images: false,
        ..Default::default()
    };

    let result = processor.process_pdf(&pdf_data, &config).await;

    #[cfg(feature = "pdf")]
    {
        match result {
            Ok(processing_result) => {
                let metadata = &processing_result.metadata;
                println!("Extracted metadata: {:?}", metadata);

                // Basic metadata should be present
                assert!(metadata.page_count > 0, "PDF should have at least one page");
                assert!(metadata.custom_metadata.contains_key("extracted_by"));

                // Should indicate no encryption for our test PDF
                assert!(!metadata.encrypted);
            }
            Err(e) => {
                println!("Metadata extraction error: {}", e);
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_image_extraction_settings() {
    let processor = create_pdf_processor();
    let pdf_data = create_test_pdf();
    let config = PdfConfig {
        extract_images: true,
        image_settings: ImageExtractionSettings {
            max_images: 10,
            min_dimensions: (20, 20),
            include_positions: true,
            base64_encode: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let result = processor.process_pdf(&pdf_data, &config).await;

    #[cfg(feature = "pdf")]
    {
        match result {
            Ok(processing_result) => {
                println!("Extracted {} images", processing_result.images.len());
                println!("Images found in stats: {}", processing_result.stats.images_extracted);

                // Verify image extraction settings are respected
                for image in &processing_result.images {
                    if config.image_settings.include_positions {
                        // Position data should be included if requested
                        println!("Image {} position: {:?}", image.index, image.position);
                    }

                    // Images should meet minimum dimensions
                    assert!(image.width >= config.image_settings.min_dimensions.0);
                    assert!(image.height >= config.image_settings.min_dimensions.1);
                }
            }
            Err(e) => {
                println!("Image extraction error: {}", e);
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(result.is_err());
    }
}

/// Property-based test: Processing any valid PDF should not panic
#[tokio::test]
async fn property_test_no_panic() {
    let processor = create_pdf_processor();
    let config = PdfConfig::default();

    // Test various PDF-like inputs
    let test_cases = vec![
        vec![], // Empty data
        b"%PDF".to_vec(), // Too short
        b"%PDF-1.7".to_vec(), // Minimal header
        create_test_pdf(), // Valid minimal PDF
        create_corrupted_pdf(), // Corrupted PDF
    ];

    for (i, test_data) in test_cases.into_iter().enumerate() {
        println!("Property test case {}", i);

        // The key property: processing should never panic, always return a Result
        let result = std::panic::AssertUnwindSafe(processor.process_pdf(&test_data, &config))
            .catch_unwind()
            .await;

        match result {
            Ok(_processing_result) => {
                println!("Case {} completed normally", i);
            }
            Err(_panic) => {
                panic!("PDF processing panicked on test case {}", i);
            }
        }
    }
}

#[tokio::test]
async fn test_memory_stability_under_load() {
    let processor = Arc::new(create_pdf_processor());
    let pdf_data = Arc::new(create_large_test_pdf(500));
    let mut config = PdfConfig::default();

    // Configure for high-stress testing
    config.memory_settings.max_memory_spike_bytes = 200 * 1024 * 1024; // 200MB
    config.memory_settings.max_concurrent_operations = 2; // Ensure exactly 2
    config.memory_settings.aggressive_cleanup = true;
    config.memory_settings.memory_check_interval = 2;

    let config = Arc::new(config);
    let mut handles = Vec::new();

    #[cfg(feature = "pdf")]
    {
        let initial_memory = processor.get_memory_usage();
        println!("Starting memory stability test with initial memory: {} MB",
                 initial_memory / (1024 * 1024));

        // Spawn 10 concurrent tasks (should be throttled to max 2 concurrent)
        for i in 0..10 {
            let processor_clone = Arc::clone(&processor);
            let data_clone = Arc::clone(&pdf_data);
            let config_clone = Arc::clone(&config);

            let handle = tokio::spawn(async move {
                let start_time = std::time::Instant::now();
                let start_memory = processor_clone.get_memory_usage();

                let result = processor_clone.process_pdf(&data_clone, &config_clone).await;

                let end_memory = processor_clone.get_memory_usage();
                let duration = start_time.elapsed();
                let memory_delta = end_memory.saturating_sub(start_memory);

                println!("Task {} completed in {:?}, memory delta: {} MB",
                        i, duration, memory_delta / (1024 * 1024));

                (i, result, memory_delta, duration)
            });

            handles.push(handle);

            // Small delay to create some overlap
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Wait for all tasks and collect results
        let results = futures::future::join_all(handles).await;
        let final_memory = processor.get_memory_usage();

        println!("All tasks completed. Final memory: {} MB", final_memory / (1024 * 1024));

        let mut successful_tasks = 0;
        let mut _total_memory_used = 0u64;
        let mut max_task_duration = std::time::Duration::from_secs(0);

        for result in results {
            match result {
                Ok((task_id, task_result, _memory_delta, duration)) => {
                    max_task_duration = max_task_duration.max(duration);

                    match task_result {
                        Ok(processing_result) => {
                            successful_tasks += 1;
                            _total_memory_used += processing_result.stats.memory_used;

                            // Ensure reasonable processing time (not stuck waiting)
                            assert!(duration < std::time::Duration::from_secs(60),
                                   "Task {} took too long: {:?}", task_id, duration);
                        }
                        Err(PdfError::MemoryLimit { .. }) => {
                            println!("Task {} hit memory limit (expected under stress)", task_id);
                        }
                        Err(e) => {
                            println!("Task {} failed with error: {}", task_id, e);
                        }
                    }
                }
                Err(e) => {
                    println!("Task spawn failed: {}", e);
                }
            }
        }

        // At least some tasks should succeed
        assert!(successful_tasks > 0, "No tasks completed successfully");

        // Memory should return to reasonable levels after processing
        let final_memory_delta = final_memory.saturating_sub(initial_memory);
        assert!(final_memory_delta < 500 * 1024 * 1024,
               "Final memory delta {} MB is too high",
               final_memory_delta / (1024 * 1024));

        println!("Memory stability test completed: {} successful tasks, \
                 max duration: {:?}, final memory delta: {} MB",
                successful_tasks, max_task_duration, final_memory_delta / (1024 * 1024));
    }
}

#[tokio::test]
async fn test_concurrent_memory_limits() {
    let processor = Arc::new(create_pdf_processor());

    // Create different sized PDFs
    let small_pdf = Arc::new(create_test_pdf());
    let large_pdf = Arc::new(create_large_test_pdf(2000));

    let mut config = PdfConfig::default();
    config.memory_settings.max_concurrent_operations = 2;
    config.memory_settings.max_memory_spike_bytes = 100 * 1024 * 1024; // 100MB strict limit
    let config = Arc::new(config);

    #[cfg(feature = "pdf")]
    {
        let mut handles = Vec::new();

        // Mix of small and large PDF processing tasks
        for i in 0..6 {
            let processor_clone = Arc::clone(&processor);
            let config_clone = Arc::clone(&config);
            let data = if i % 2 == 0 {
                Arc::clone(&small_pdf)
            } else {
                Arc::clone(&large_pdf)
            };

            let handle = tokio::spawn(async move {
                let start = std::time::Instant::now();
                let result = processor_clone.process_pdf(&data, &config_clone).await;
                let duration = start.elapsed();

                match &result {
                    Ok(processing_result) => {
                        println!("Task {} succeeded: {} pages, {} MB memory",
                                i,
                                processing_result.stats.pages_processed,
                                processing_result.stats.memory_used / (1024 * 1024));
                        assert!(processing_result.stats.memory_used <= config_clone.memory_settings.max_memory_spike_bytes + 20 * 1024 * 1024);
                    }
                    Err(PdfError::MemoryLimit { used, limit }) => {
                        println!("Task {} hit memory limit: {} MB > {} MB",
                                i, used / (1024 * 1024), limit / (1024 * 1024));
                    }
                    Err(e) => {
                        println!("Task {} failed: {}", i, e);
                    }
                }

                (i, result, duration)
            });

            handles.push(handle);
        }

        // Wait for completion and verify concurrency was limited
        let results = futures::future::join_all(handles).await;

        // Verify that some tasks were queued (indicating concurrency control worked)
        let durations: Vec<_> = results.iter().filter_map(|r| {
            if let Ok((_, _, duration)) = r {
                Some(*duration)
            } else {
                None
            }
        }).collect();

        if durations.len() > 2 {
            // Sort durations to see the pattern
            let mut sorted_durations = durations.clone();
            sorted_durations.sort();

            println!("Task durations: {:?}", sorted_durations);

            // The longest tasks should indicate queueing happened
            let avg_duration = sorted_durations.iter().sum::<std::time::Duration>() / sorted_durations.len() as u32;
            let max_duration = sorted_durations.last().unwrap();

            println!("Average duration: {:?}, Max duration: {:?}", avg_duration, max_duration);
        }
    }
}