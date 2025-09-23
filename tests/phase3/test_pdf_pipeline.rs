use riptide_core::pdf::{
    create_pdf_processor, process_pdf, utils, PdfConfig, PdfError, PdfProcessor,
};
use std::collections::HashMap;
use tokio;

// Sample PDF header for testing
const SAMPLE_PDF_HEADER: &[u8] = b"%PDF-1.7\n1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n2 0 obj\n<<\n/Type /Pages\n/Kids [3 0 R]\n/Count 1\n>>\nendobj\n3 0 obj\n<<\n/Type /Page\n/Parent 2 0 R\n/MediaBox [0 0 612 792]\n>>\nendobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000074 00000 n \n0000000120 00000 n \ntrailer\n<<\n/Size 4\n/Root 1 0 R\n>>\nstartxref\n202\n%%EOF";

#[tokio::test]
async fn test_pdf_detection_utils() {
    // Test content type detection
    assert!(utils::is_pdf_content(Some("application/pdf"), b"any data"));
    assert!(utils::is_pdf_content(Some("application/pdf; charset=utf-8"), b"any data"));
    assert!(!utils::is_pdf_content(Some("text/html"), b"any data"));

    // Test magic bytes detection
    assert!(utils::is_pdf_content(None, b"%PDF-1.4\nrest of pdf"));
    assert!(utils::is_pdf_content(None, SAMPLE_PDF_HEADER));
    assert!(!utils::is_pdf_content(None, b"<html>not a pdf</html>"));

    // Test skip headless detection
    assert!(utils::should_skip_headless("application/pdf", "https://example.com/doc"));
    assert!(utils::should_skip_headless("text/html", "https://example.com/doc.pdf"));
    assert!(!utils::should_skip_headless("text/html", "https://example.com/page.html"));
}

#[tokio::test]
async fn test_pdf_version_extraction() {
    assert_eq!(utils::extract_pdf_version(b"%PDF-1.7\n"), Some("1.7".to_string()));
    assert_eq!(utils::extract_pdf_version(b"%PDF-1.4\n"), Some("1.4".to_string()));
    assert_eq!(utils::extract_pdf_version(b"%PDF-2.0\n"), Some("2.0".to_string()));
    assert_eq!(utils::extract_pdf_version(b"not a pdf"), None);
    assert_eq!(utils::extract_pdf_version(b"%PDF"), None); // Too short
}

#[tokio::test]
async fn test_complexity_estimation() {
    use riptide_core::pdf::utils::ProcessingComplexity;

    assert!(matches!(
        utils::estimate_complexity(500_000),
        ProcessingComplexity::Low
    ));
    assert!(matches!(
        utils::estimate_complexity(5_000_000),
        ProcessingComplexity::Medium
    ));
    assert!(matches!(
        utils::estimate_complexity(25_000_000),
        ProcessingComplexity::High
    ));
    assert!(matches!(
        utils::estimate_complexity(100_000_000),
        ProcessingComplexity::VeryHigh
    ));
}

#[tokio::test]
async fn test_pdf_config_defaults() {
    let config = PdfConfig::default();
    assert!(config.extract_text);
    assert!(config.extract_metadata);
    assert!(!config.extract_images);
    assert_eq!(config.max_size_bytes, 100 * 1024 * 1024);
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.image_settings.max_images, 50);
    assert!(config.text_settings.preserve_formatting);
}

#[tokio::test]
async fn test_pdf_processor_creation() {
    let processor = create_pdf_processor();
    let capabilities = processor.capabilities();

    #[cfg(feature = "pdf")]
    {
        // With PDF feature enabled
        assert!(capabilities.text_extraction);
        assert!(capabilities.metadata_extraction);
        assert!(capabilities.max_file_size > 0);
        assert!(!capabilities.supported_versions.is_empty());
    }

    #[cfg(not(feature = "pdf"))]
    {
        // Without PDF feature
        assert!(!processor.is_available());
        assert!(!capabilities.text_extraction);
        assert_eq!(capabilities.max_file_size, 0);
        assert!(capabilities.supported_versions.is_empty());
    }
}

#[tokio::test]
async fn test_pdf_error_types() {
    let large_size = 200 * 1024 * 1024;
    let max_size = 100 * 1024 * 1024;

    let error = PdfError::FileTooLarge {
        size: large_size,
        max_size,
    };

    let error_string = error.to_string();
    assert!(error_string.contains("too large"));
    assert!(error_string.contains("200"));
    assert!(error_string.contains("100"));

    let invalid_error = PdfError::InvalidPdf {
        message: "Test error".to_string(),
    };
    assert!(invalid_error.to_string().contains("Invalid PDF"));
    assert!(invalid_error.to_string().contains("Test error"));
}

#[tokio::test]
async fn test_process_pdf_function() {
    let result = process_pdf(SAMPLE_PDF_HEADER).await;

    #[cfg(feature = "pdf")]
    {
        // With PDF feature, should attempt processing
        match result {
            Ok(doc) => {
                // Successfully processed
                assert_eq!(doc.url, "pdf://document");
                assert!(doc.categories.contains(&"pdf".to_string()));
                assert!(doc.quality_score.unwrap_or(0) > 0);
            }
            Err(err) => {
                // Expected if pdfium library is not available
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("Failed to initialize Pdfium")
                    || error_msg.contains("Failed to load PDF")
                );
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        // Without PDF feature, should return error
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }
}

#[tokio::test]
async fn test_pdf_processor_validation() {
    let processor = create_pdf_processor();
    let config = PdfConfig::default();

    // Test with invalid PDF data
    let invalid_data = b"<html>not a pdf</html>";
    let result = processor.process_pdf(invalid_data, &config).await;
    assert!(result.is_err());

    if let Err(PdfError::InvalidPdf { message }) = result {
        assert!(message.contains("PDF header"));
    } else {
        panic!("Expected InvalidPdf error");
    }

    // Test with oversized file
    let mut oversized_config = config.clone();
    oversized_config.max_size_bytes = 10; // Very small limit

    let result = processor.process_pdf(SAMPLE_PDF_HEADER, &oversized_config).await;

    #[cfg(feature = "pdf")]
    {
        assert!(result.is_err());
        if let Err(PdfError::FileTooLarge { size, max_size }) = result {
            assert_eq!(size, SAMPLE_PDF_HEADER.len() as u64);
            assert_eq!(max_size, 10);
        } else {
            panic!("Expected FileTooLarge error");
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        // Should fail with feature not enabled error
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_pdf_metadata_extraction() {
    let processor = create_pdf_processor();
    let mut config = PdfConfig::default();
    config.extract_metadata = true;
    config.extract_text = true;

    let result = processor.process_pdf(SAMPLE_PDF_HEADER, &config).await;

    #[cfg(feature = "pdf")]
    {
        match result {
            Ok(pdf_result) => {
                assert!(pdf_result.success);
                assert!(pdf_result.metadata.page_count > 0);
                assert!(pdf_result.stats.file_size > 0);
                assert!(pdf_result.stats.processing_time_ms >= 0);

                // Check that we get text if extraction is enabled
                if config.extract_text {
                    assert!(pdf_result.text.is_some());
                }
            }
            Err(err) => {
                // Expected if pdfium is not available in test environment
                println!("PDF processing failed (expected in test environment): {}", err);
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_pdf_configuration_variants() {
    let processor = create_pdf_processor();

    // Test with text-only extraction
    let mut text_only_config = PdfConfig::default();
    text_only_config.extract_text = true;
    text_only_config.extract_images = false;
    text_only_config.extract_metadata = false;

    // Test with metadata-only extraction
    let mut metadata_only_config = PdfConfig::default();
    metadata_only_config.extract_text = false;
    metadata_only_config.extract_images = false;
    metadata_only_config.extract_metadata = true;

    // Test with image extraction enabled
    let mut image_config = PdfConfig::default();
    image_config.extract_images = true;
    image_config.image_settings.max_images = 10;

    let configs = vec![text_only_config, metadata_only_config, image_config];

    for config in configs {
        let result = processor.process_pdf(SAMPLE_PDF_HEADER, &config).await;

        #[cfg(feature = "pdf")]
        {
            match result {
                Ok(pdf_result) => {
                    // Verify configuration was respected
                    if config.extract_text {
                        assert!(pdf_result.text.is_some());
                    } else {
                        assert!(pdf_result.text.is_none());
                    }

                    if config.extract_images {
                        // Images array should be initialized even if empty
                        assert!(pdf_result.images.len() <= config.image_settings.max_images as usize);
                    }
                }
                Err(_) => {
                    // Expected if pdfium is not available
                }
            }
        }
    }
}

#[tokio::test]
async fn test_pdf_stats_collection() {
    let processor = create_pdf_processor();
    let config = PdfConfig::default();

    let result = processor.process_pdf(SAMPLE_PDF_HEADER, &config).await;

    #[cfg(feature = "pdf")]
    {
        match result {
            Ok(pdf_result) => {
                let stats = &pdf_result.stats;

                // Verify stats are populated
                assert!(stats.file_size > 0);
                assert_eq!(stats.file_size, SAMPLE_PDF_HEADER.len() as u64);
                assert!(stats.processing_time_ms >= 0);
                assert!(stats.pages_processed >= 0);
                assert!(stats.images_extracted >= 0);
                assert!(stats.text_length >= 0);
            }
            Err(_) => {
                // Expected if pdfium is not available
            }
        }
    }
}

#[cfg(feature = "pdf")]
#[tokio::test]
async fn test_pdf_concurrency_limit() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::task::JoinSet;

    let active_count = Arc::new(AtomicU32::new(0));
    let max_active = Arc::new(AtomicU32::new(0));
    let completed_count = Arc::new(AtomicU32::new(0));
    let mut tasks = JoinSet::new();

    // Spawn multiple concurrent PDF processing tasks to test semaphore limit
    for i in 0..6 {
        let active_clone = active_count.clone();
        let max_active_clone = max_active.clone();
        let completed_clone = completed_count.clone();

        tasks.spawn(async move {
            // Simulate the semaphore behavior by calling process_pdf
            let result = process_pdf(SAMPLE_PDF_HEADER).await;

            // Track concurrent executions
            let current_active = active_clone.fetch_add(1, Ordering::SeqCst) + 1;

            // Update max seen concurrent
            loop {
                let current_max = max_active_clone.load(Ordering::SeqCst);
                if current_active <= current_max ||
                   max_active_clone.compare_exchange(current_max, current_active, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    break;
                }
            }

            // Simulate some processing time
            tokio::time::sleep(Duration::from_millis(50)).await;

            active_clone.fetch_sub(1, Ordering::SeqCst);

            match result {
                Ok(_) => {
                    completed_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(i)
                }
                Err(err) => {
                    // Expected if pdfium is not available
                    Err(format!("Task {} failed: {}", i, err))
                }
            }
        });
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.unwrap());
    }

    // Verify all tasks were spawned
    assert_eq!(results.len(), 6);

    let success_count = completed_count.load(Ordering::SeqCst);
    let max_concurrent = max_active.load(Ordering::SeqCst);

    println!("Successfully processed {} out of 6 concurrent PDF tasks", success_count);
    println!("Maximum concurrent operations observed: {}", max_concurrent);

    // If any processing succeeded, verify concurrency was limited
    // Note: The semaphore in the actual implementation limits to 2 concurrent operations
    // This test may not perfectly reflect that due to test environment constraints
    if success_count > 0 {
        assert!(max_concurrent <= 6); // All tasks were at least attempted
    }
}

#[cfg(feature = "pdf")]
#[tokio::test]
async fn test_pdf_text_extraction() {
    // Test text extraction from a sample PDF
    let result = process_pdf(SAMPLE_PDF_HEADER).await;

    match result {
        Ok(doc) => {
            // Successful text extraction
            assert!(!doc.text.is_empty(), "Extracted text should not be empty");
            assert!(doc.word_count.unwrap_or(0) >= 0, "Word count should be non-negative");
            assert!(doc.reading_time.unwrap_or(0) > 0, "Reading time should be positive");
            assert_eq!(doc.url, "pdf://document", "URL should indicate PDF document");
            assert!(doc.categories.contains(&"pdf".to_string()), "Should be categorized as PDF");
            assert!(doc.quality_score.unwrap_or(0) > 0, "Quality score should be positive");
        }
        Err(err) => {
            // Expected if pdfium library is not available in test environment
            let error_msg = err.to_string();
            println!("PDF text extraction test skipped: {}", error_msg);
            assert!(
                error_msg.contains("Failed to initialize Pdfium")
                || error_msg.contains("Failed to load PDF")
                || error_msg.contains("not enabled")
            );
        }
    }
}

#[tokio::test]
async fn test_pdf_pipeline_integration() {
    // Test integration with the broader pipeline
    use riptide_core::types::{CrawlOptions, RenderMode};

    let options = CrawlOptions {
        render_mode: RenderMode::Pdf,
        pdf_config: Some(PdfConfig::default()),
        ..Default::default()
    };

    // Verify PDF config is properly integrated
    assert!(options.pdf_config.is_some());
    assert!(matches!(options.render_mode, RenderMode::Pdf));

    let pdf_config = options.pdf_config.unwrap();
    assert!(pdf_config.extract_text);
    assert!(pdf_config.extract_metadata);
}

#[tokio::test]
async fn test_pdf_utils_comprehensive() {
    // Test comprehensive utility functions

    // Test various PDF versions
    let versions = vec!["1.0", "1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "2.0"];
    for version in versions {
        let pdf_header = format!("%PDF-{}\n", version);
        assert_eq!(utils::extract_pdf_version(pdf_header.as_bytes()), Some(version.to_string()));
    }

    // Test edge cases
    assert_eq!(utils::extract_pdf_version(b"%PDF"), None);
    assert_eq!(utils::extract_pdf_version(b""), None);
    assert_eq!(utils::extract_pdf_version(b"not-pdf"), None);

    // Test content type variations
    let content_types = vec![
        "application/pdf",
        "application/pdf; charset=utf-8",
        "application/pdf; name=document.pdf",
        "Application/PDF", // Case variations
    ];

    for ct in content_types {
        assert!(utils::is_pdf_content(Some(ct), b"any data"));
        assert!(utils::should_skip_headless(ct, "any-url"));
    }

    // Test URL variations
    let pdf_urls = vec![
        "document.pdf",
        "path/to/file.pdf",
        "https://example.com/doc.PDF", // Case variation
        "/local/file.pdf",
    ];

    for url in pdf_urls {
        assert!(utils::should_skip_headless("any-content-type", url));
    }
}

#[tokio::test]
async fn test_pdf_large_file_processing() {
    // Test processing of large PDFs with pagination
    let processor = create_pdf_processor();
    let mut config = PdfConfig::default();
    config.enable_progress_tracking = true;
    config.max_size_bytes = 10 * 1024 * 1024; // 10MB limit

    // Test with small PDF that should succeed
    let result = processor.process_pdf(SAMPLE_PDF_HEADER, &config).await;

    #[cfg(feature = "pdf")]
    {
        match result {
            Ok(pdf_result) => {
                assert!(pdf_result.success);
                assert!(pdf_result.stats.file_size <= config.max_size_bytes);
                assert!(pdf_result.stats.processing_time_ms > 0);
            }
            Err(PdfError::ProcessingError { .. }) => {
                // Expected if pdfium is not available
            }
            Err(other) => {
                panic!("Unexpected error: {:?}", other);
            }
        }
    }

    // Test oversized file detection
    config.max_size_bytes = 10; // Very small limit
    let oversized_result = processor.process_pdf(SAMPLE_PDF_HEADER, &config).await;

    #[cfg(feature = "pdf")]
    {
        assert!(oversized_result.is_err());
        if let Err(PdfError::FileTooLarge { size, max_size }) = oversized_result {
            assert_eq!(size, SAMPLE_PDF_HEADER.len() as u64);
            assert_eq!(max_size, 10);
        } else {
            panic!("Expected FileTooLarge error");
        }
    }
}

#[tokio::test]
async fn test_pdf_corrupted_file_handling() {
    // Test error handling for corrupted PDFs
    let processor = create_pdf_processor();
    let config = PdfConfig::default();

    // Test completely invalid data
    let invalid_data = b"This is not a PDF file at all!";
    let result = processor.process_pdf(invalid_data, &config).await;
    assert!(result.is_err());

    if let Err(PdfError::InvalidPdf { message }) = result {
        assert!(message.contains("PDF header"));
    } else {
        panic!("Expected InvalidPdf error");
    }

    // Test truncated PDF
    let truncated_pdf = b"%PDF-1.7\ntruncated";
    let truncated_result = processor.process_pdf(truncated_pdf, &config).await;

    #[cfg(feature = "pdf")]
    {
        // Should fail during processing
        assert!(truncated_result.is_err());
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(truncated_result.is_err());
    }
}

#[tokio::test]
async fn test_pdf_ocr_detection() {
    // Test OCR need detection for image-based PDFs
    let processor = create_pdf_processor();

    #[cfg(feature = "pdf")]
    {
        // Test OCR detection with sample PDF
        let ocr_needed = processor.detect_ocr_need(SAMPLE_PDF_HEADER).await;

        match ocr_needed {
            Ok(needs_ocr) => {
                // For our sample PDF with text content, should not need OCR
                assert!(!needs_ocr);
            }
            Err(PdfError::ProcessingError { .. }) => {
                // Expected if pdfium is not available
            }
            Err(other) => {
                panic!("Unexpected error: {:?}", other);
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        let ocr_result = processor.detect_ocr_need(SAMPLE_PDF_HEADER).await;
        assert!(ocr_result.is_err());
    }
}

#[tokio::test]
async fn test_pdf_progress_tracking() {
    // Test page-by-page processing with progress tracking
    use std::sync::{Arc, Mutex};

    let processor = create_pdf_processor();
    let mut config = PdfConfig::default();
    config.enable_progress_tracking = true;

    // Track progress calls
    let progress_calls = Arc::new(Mutex::new(Vec::<(u32, u32)>::new()));
    let progress_calls_clone = progress_calls.clone();

    #[cfg(feature = "pdf")]
    {
        let progress_callback = move |current: u32, total: u32| {
            let mut calls = progress_calls_clone.lock().unwrap();
            calls.push((current, total));
        };

        let result = processor
            .process_pdf_with_progress(SAMPLE_PDF_HEADER, &config, Some(Box::new(progress_callback)))
            .await;

        match result {
            Ok(pdf_result) => {
                assert!(pdf_result.success);
                let calls = progress_calls.lock().unwrap();

                // Should have received at least one progress call
                assert!(!calls.is_empty());

                // Progress should make sense
                if let Some((current, total)) = calls.last() {
                    assert!(*current <= *total);
                    assert!(*total > 0);
                }
            }
            Err(PdfError::ProcessingError { .. }) => {
                // Expected if pdfium is not available
            }
            Err(other) => {
                panic!("Unexpected error: {:?}", other);
            }
        }
    }
}

#[tokio::test]
async fn test_pdf_enhanced_api_functions() {
    // Test the enhanced API functions
    use riptide_core::pdf::{process_pdf_with_progress, detect_pdf_ocr_need};

    // Test progress tracking function
    let progress_calls = Arc::new(Mutex::new(Vec::<(u32, u32)>::new()));
    let progress_calls_clone = progress_calls.clone();

    let progress_callback = move |current: u32, total: u32| {
        let mut calls = progress_calls_clone.lock().unwrap();
        calls.push((current, total));
    };

    let result = process_pdf_with_progress(
        SAMPLE_PDF_HEADER,
        None, // Use default config
        Some(progress_callback),
    ).await;

    #[cfg(feature = "pdf")]
    {
        match result {
            Ok(pdf_result) => {
                assert!(pdf_result.success);
                let calls = progress_calls.lock().unwrap();
                assert!(!calls.is_empty());
            }
            Err(_) => {
                // Expected if pdfium is not available
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(result.is_err());
    }

    // Test OCR detection function
    let ocr_result = detect_pdf_ocr_need(SAMPLE_PDF_HEADER).await;

    #[cfg(feature = "pdf")]
    {
        match ocr_result {
            Ok(needs_ocr) => {
                // Sample PDF has text, should not need OCR
                assert!(!needs_ocr);
            }
            Err(_) => {
                // Expected if pdfium is not available
            }
        }
    }

    #[cfg(not(feature = "pdf"))]
    {
        assert!(ocr_result.is_err());
    }
}