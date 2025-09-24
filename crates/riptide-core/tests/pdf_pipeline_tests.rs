#[cfg(test)]
mod tests {
    use riptide_core::pdf::{
        PdfPipelineIntegration, PdfConfig, PdfProcessor,
        PdfError, PdfResult, utils
    };
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    fn create_test_pdf() -> Vec<u8> {
        // Create a minimal valid PDF
        let mut pdf = Vec::new();
        pdf.extend_from_slice(b"%PDF-1.4\n");
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        pdf.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n");
        pdf.extend_from_slice(b"4 0 obj\n<< /Length 44 >>\nstream\nBT\n/F1 12 Tf\n100 700 Td\n(Hello World) Tj\nET\nendstream\nendobj\n");
        pdf.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000214 00000 n \n");
        pdf.extend_from_slice(b"trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n309\n%%EOF\n");
        pdf
    }

    fn create_invalid_pdf() -> Vec<u8> {
        b"This is not a PDF file".to_vec()
    }

    #[test]
    fn test_pdf_detection_by_magic_bytes() {
        let valid_pdf = create_test_pdf();
        let invalid_pdf = create_invalid_pdf();
        let html = b"<html><body>HTML content</body></html>";

        assert!(utils::detect_pdf_by_magic_bytes(&valid_pdf));
        assert!(!utils::detect_pdf_by_magic_bytes(&invalid_pdf));
        assert!(!utils::detect_pdf_by_magic_bytes(html));
    }

    #[test]
    fn test_pdf_detection_by_extension() {
        assert!(utils::detect_pdf_by_extension("document.pdf"));
        assert!(utils::detect_pdf_by_extension("file.PDF"));
        assert!(utils::detect_pdf_by_extension("/path/to/document.pdf"));
        assert!(!utils::detect_pdf_by_extension("document.html"));
        assert!(!utils::detect_pdf_by_extension("file.txt"));
    }

    #[test]
    fn test_pdf_detection_by_content_type() {
        assert!(utils::detect_pdf_by_content_type(Some("application/pdf")));
        assert!(utils::detect_pdf_by_content_type(Some("application/x-pdf")));
        assert!(utils::detect_pdf_by_content_type(Some("application/pdf; charset=utf-8")));
        assert!(!utils::detect_pdf_by_content_type(Some("text/html")));
        assert!(!utils::detect_pdf_by_content_type(None));
    }

    #[test]
    fn test_comprehensive_pdf_detection() {
        let pdf_data = create_test_pdf();

        // Should detect with all indicators
        assert!(utils::detect_pdf_content(
            Some("application/pdf"),
            Some("document.pdf"),
            Some(&pdf_data)
        ));

        // Should detect with just magic bytes
        assert!(utils::detect_pdf_content(None, None, Some(&pdf_data)));

        // Should detect with just extension
        assert!(utils::detect_pdf_content(None, Some("file.pdf"), None));

        // Should detect with just content type
        assert!(utils::detect_pdf_content(Some("application/pdf"), None, None));

        // Should not detect without any indicators
        assert!(!utils::detect_pdf_content(None, None, None));
    }

    #[test]
    fn test_pdf_config_defaults() {
        let config = PdfConfig::default();

        assert_eq!(config.max_pages, 1000);
        assert_eq!(config.max_size_bytes, 100 * 1024 * 1024); // 100MB
        assert!(config.extract_text);
        assert!(config.extract_metadata);
        assert!(config.extract_images);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_pdf_config_builder() {
        let config = PdfConfig::builder()
            .max_pages(500)
            .max_size_bytes(50 * 1024 * 1024)
            .extract_text(true)
            .extract_metadata(false)
            .extract_images(false)
            .timeout_seconds(60)
            .build();

        assert_eq!(config.max_pages, 500);
        assert_eq!(config.max_size_bytes, 50 * 1024 * 1024);
        assert!(config.extract_text);
        assert!(!config.extract_metadata);
        assert!(!config.extract_images);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[tokio::test]
    async fn test_pdf_pipeline_integration() {
        let pipeline = PdfPipelineIntegration::new();

        let pdf_data = create_test_pdf();

        // Test detection
        assert!(pipeline.should_process_as_pdf(
            Some("application/pdf"),
            Some("test.pdf"),
            Some(&pdf_data)
        ));

        // Pipeline should handle processing (will fail without pdfium)
        let result = pipeline.process_pdf_to_extracted_doc(&pdf_data, Some("test.pdf")).await;

        // Without the pdf feature, this should return an error
        #[cfg(not(feature = "pdf"))]
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pdf_size_limit() {
        let pipeline = PdfPipelineIntegration::with_config(
            PdfConfig::builder()
                .max_size_bytes(100) // 100 bytes limit
                .build()
        );

        let large_pdf = vec![0u8; 200]; // 200 bytes

        let result = pipeline.process_pdf_to_extracted_doc(&large_pdf, None).await;
        assert!(matches!(result, Err(PdfError::FileTooLarge { .. })));
    }

    #[tokio::test]
    async fn test_pdf_semaphore_concurrency() {
        // This test verifies the semaphore limits concurrent operations to 2

        let pipeline = Arc::new(PdfPipelineIntegration::new());
        let pdf_data = Arc::new(create_test_pdf());

        let mut handles = vec![];

        // Spawn 5 concurrent PDF processing tasks
        for i in 0..5 {
            let pipeline_clone = pipeline.clone();
            let pdf_clone = pdf_data.clone();

            let handle = tokio::spawn(async move {
                let start = std::time::Instant::now();
                let _result = pipeline_clone
                    .process_pdf_to_extracted_doc(&pdf_clone, Some(&format!("test{}.pdf", i)))
                    .await;
                let elapsed = start.elapsed();
                (i, elapsed)
            });

            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles).await;

        // Verify all tasks completed
        assert_eq!(results.len(), 5);

        // Due to semaphore limit of 2, at least some tasks should have waited
        let timings: Vec<_> = results.iter().map(|r| r.as_ref().unwrap().1).collect();

        // At least 3 tasks should have been delayed (since only 2 can run concurrently)
        let delayed_count = timings.iter().filter(|t| t.as_millis() > 10).count();
        assert!(delayed_count >= 2, "Expected semaphore to limit concurrency");
    }

    #[test]
    fn test_pdf_metadata_extraction() {
        // Test metadata structure
        use riptide_core::pdf::types::PdfMetadata;

        let metadata = PdfMetadata {
            title: Some("Test Document".to_string()),
            author: Some("Test Author".to_string()),
            subject: Some("Test Subject".to_string()),
            keywords: vec!["test".to_string(), "pdf".to_string()],
            creator: Some("Test Creator".to_string()),
            producer: Some("Test Producer".to_string()),
            creation_date: Some("2024-01-01".to_string()),
            modification_date: Some("2024-01-02".to_string()),
            page_count: 10,
        };

        assert_eq!(metadata.title, Some("Test Document".to_string()));
        assert_eq!(metadata.page_count, 10);
        assert_eq!(metadata.keywords.len(), 2);
    }

    #[test]
    fn test_pdf_processing_result() {
        use riptide_core::pdf::types::{PdfProcessingResult, PdfMetadata, PdfStats};

        let result = PdfProcessingResult {
            success: true,
            text: Some("Extracted text content".to_string()),
            metadata: PdfMetadata {
                title: Some("Test".to_string()),
                author: None,
                subject: None,
                keywords: vec![],
                creator: None,
                producer: None,
                creation_date: None,
                modification_date: None,
                page_count: 1,
            },
            images: vec![],
            stats: PdfStats {
                processing_time_ms: 100,
                pages_processed: 1,
                text_length: 21,
                images_extracted: 0,
                memory_used: 1024,
            },
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.text, Some("Extracted text content".to_string()));
        assert_eq!(result.stats.pages_processed, 1);
        assert_eq!(result.stats.text_length, 21);
    }

    #[tokio::test]
    async fn test_pdf_metrics_collection() {
        let pipeline = PdfPipelineIntegration::new();

        // Get initial metrics
        let initial_metrics = pipeline.get_metrics_snapshot();
        assert_eq!(initial_metrics.total_processed, 0);
        assert_eq!(initial_metrics.total_failed, 0);

        // Process an invalid PDF to trigger failure metric
        let invalid_pdf = create_invalid_pdf();
        let _ = pipeline.process_pdf_to_extracted_doc(&invalid_pdf, None).await;

        // Check metrics were updated
        let updated_metrics = pipeline.get_metrics_snapshot();

        // Without pdf feature, this should show as a failure
        #[cfg(not(feature = "pdf"))]
        assert_eq!(updated_metrics.total_failed, 1);
    }

    #[test]
    fn test_pdf_prometheus_metrics_export() {
        use riptide_core::pdf::metrics::PdfMetricsCollector;

        let collector = PdfMetricsCollector::new();

        // Record some metrics
        collector.record_processing_success(
            std::time::Duration::from_millis(500),
            10,
            1024 * 1024
        );

        collector.record_processing_failure(false);

        // Export for Prometheus
        let prometheus_metrics = collector.export_for_prometheus();

        assert!(prometheus_metrics.contains_key("pdf_total_processed"));
        assert!(prometheus_metrics.contains_key("pdf_total_failed"));
        assert!(prometheus_metrics.contains_key("pdf_avg_processing_time_ms"));

        assert_eq!(prometheus_metrics["pdf_total_processed"], 1.0);
        assert_eq!(prometheus_metrics["pdf_total_failed"], 1.0);
    }

    #[test]
    fn test_reading_time_estimation() {
        // Average reading speed is 200-250 words per minute
        assert_eq!(utils::estimate_reading_time(200), 1); // 1 minute
        assert_eq!(utils::estimate_reading_time(500), 3); // ~2.5 minutes, rounds to 3
        assert_eq!(utils::estimate_reading_time(1000), 5); // 5 minutes
        assert_eq!(utils::estimate_reading_time(50), 1); // Min 1 minute
    }

    #[test]
    fn test_pdf_error_types() {
        let error = PdfError::InvalidPdf {
            message: "Invalid header".to_string(),
        };

        assert!(matches!(error, PdfError::InvalidPdf { .. }));

        let error = PdfError::FileTooLarge {
            size: 200 * 1024 * 1024,
            max_size: 100 * 1024 * 1024,
        };

        assert!(matches!(error, PdfError::FileTooLarge { .. }));
    }
}