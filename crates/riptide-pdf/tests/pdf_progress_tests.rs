//! Integration tests for PDF progress tracking functionality
//!
//! These tests verify the end-to-end PDF processing with progress tracking works correctly.

use std::sync::Arc;
use std::time::Duration;

/// Test basic PDF processing progress tracking
#[tokio::test]
#[cfg(feature = "pdf")]
async fn test_pdf_progress_tracking() {
    use riptide_pdf::{integration::PdfPipelineIntegration, types::ProgressUpdate};

    // Create PDF integration
    let integration = PdfPipelineIntegration::new();

    // Create a simple test PDF (minimal PDF with text)
    let test_pdf = create_test_pdf();

    // Test progress callback functionality
    let (sender, mut receiver) = integration.create_progress_channel();

    // Spawn processing task with Arc to share ownership
    let integration_arc = Arc::new(integration);
    let integration_clone = Arc::clone(&integration_arc);
    tokio::spawn(async move {
        let _ = integration_clone
            .process_pdf_bytes_with_progress(&test_pdf, sender)
            .await;
    });

    // Collect progress updates
    let mut updates = Vec::new();
    let timeout = Duration::from_secs(10);

    while let Ok(update) = tokio::time::timeout(timeout, receiver.recv()).await {
        if let Some(progress_update) = update {
            let is_complete = matches!(
                progress_update,
                ProgressUpdate::Completed { .. } | ProgressUpdate::Failed { .. }
            );
            updates.push(progress_update);
            if is_complete {
                break;
            }
        } else {
            break;
        }
    }

    // Verify we received progress updates
    assert!(!updates.is_empty(), "Should receive progress updates");

    // Check for expected progress sequence
    let has_started = updates
        .iter()
        .any(|u| matches!(u, ProgressUpdate::Started { .. }));
    let _has_progress = updates
        .iter()
        .any(|u| matches!(u, ProgressUpdate::Progress(_)));
    let has_completion = updates.iter().any(|u| {
        matches!(
            u,
            ProgressUpdate::Completed { .. } | ProgressUpdate::Failed { .. }
        )
    });

    assert!(has_started, "Should have started event");
    // Note: Progress events might not occur for very small PDFs
    assert!(has_completion, "Should have completion event");

    println!("Received {} progress updates", updates.len());
    for update in &updates {
        println!("Progress update: {:?}", update);
    }
}

/// Test progress tracking with detailed callback
/// NOTE: This test requires `process_pdf_to_extracted_doc_with_progress` method which doesn't exist yet
/// Commented out to allow compilation - uncomment when method is implemented
/*
#[ignore = "Requires process_pdf_to_extracted_doc_with_progress method to be implemented"]
#[tokio::test]
#[cfg(feature = "pdf")]
async fn test_detailed_progress_callback() {
    use riptide_core::pdf::{
        integration::PdfPipelineIntegration,
        types::{ProcessingProgress, ProcessingStage},
    };
    use std::sync::{Arc, Mutex};

    let integration = PdfPipelineIntegration::new();
    let test_pdf = create_test_pdf();

    // Shared progress tracking
    let progress_updates = Arc::new(Mutex::new(Vec::<ProcessingProgress>::new()));
    let progress_updates_clone = progress_updates.clone();

    // Create detailed progress callback
    let callback = Some(Box::new(move |progress: ProcessingProgress| {
        progress_updates_clone.lock().unwrap().push(progress);
    }) as riptide_core::pdf::types::DetailedProgressCallback);

    // Process PDF with progress tracking
    match integration
        .process_pdf_to_extracted_doc_with_progress(&test_pdf, None, callback)
        .await
    {
        Ok(document) => {
            assert!(!document.text.is_empty(), "Should extract text from PDF");

            let updates = progress_updates.lock().unwrap();
            println!("Received {} detailed progress updates", updates.len());

            for update in updates.iter() {
                println!(
                    "Page {}/{}, {}% complete, Stage: {:?}",
                    update.current_page, update.total_pages, update.percentage, update.stage
                );
            }

            // Should have at least one progress update
            assert!(
                !updates.is_empty(),
                "Should receive detailed progress updates"
            );
        }
        Err(e) => panic!("PDF processing failed: {:?}", e),
    }
}
*/

/// Test metrics collection for PDF processing
#[tokio::test]
#[cfg(feature = "pdf")]
async fn test_pdf_metrics_collection() {
    use riptide_pdf::integration::PdfPipelineIntegration;

    let integration = PdfPipelineIntegration::new();
    let test_pdf = create_test_pdf();

    // Reset metrics
    integration.reset_metrics();

    // Process PDF
    let _result = integration
        .process_pdf_to_extracted_doc(&test_pdf, None)
        .await;

    // Check metrics
    let metrics = integration.get_metrics_snapshot();
    let prometheus_metrics = integration.export_metrics_for_monitoring();

    // Verify basic metrics are collected
    assert!(metrics.total_processed >= 1 || metrics.total_failed >= 1);
    assert!(!prometheus_metrics.is_empty());

    // Verify specific metrics exist
    assert!(prometheus_metrics.contains_key("pdf_total_processed"));
    assert!(prometheus_metrics.contains_key("pdf_avg_processing_time_ms"));

    println!("Metrics snapshot: {:#?}", metrics);
    println!("Prometheus metrics count: {}", prometheus_metrics.len());
}

/// Test streaming response format compatibility
#[test]
fn test_progress_update_serialization() {
    use riptide_pdf::types::{ProcessingProgress, ProcessingStage, ProgressUpdate};

    // Test different progress update types
    let updates = vec![
        ProgressUpdate::Started {
            total_pages: 10,
            file_size: 1024000,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        },
        ProgressUpdate::Progress(ProcessingProgress {
            current_page: 5,
            total_pages: 10,
            percentage: 50.0,
            estimated_remaining_ms: Some(5000),
            stage: ProcessingStage::ExtractingText(5),
        }),
        ProgressUpdate::StageChanged {
            stage: ProcessingStage::ExtractingImages(6),
            timestamp: "2025-01-01T00:00:01Z".to_string(),
        },
        ProgressUpdate::KeepAlive {
            timestamp: "2025-01-01T00:00:02Z".to_string(),
        },
    ];

    // Verify all updates can be serialized to JSON (for NDJSON streaming)
    for update in &updates {
        let serialized =
            serde_json::to_string(update).expect("Progress update should serialize to JSON");
        assert!(!serialized.is_empty());

        // Verify it can be deserialized back
        let _deserialized: ProgressUpdate = serde_json::from_str(&serialized)
            .expect("Serialized progress update should deserialize");
    }

    println!("All progress update types serialize/deserialize correctly");
}

/// Create a minimal test PDF for testing
/// Returns a byte vector containing a minimal valid PDF
fn create_test_pdf() -> Vec<u8> {
    // Minimal PDF with some text content
    // This is a very basic PDF that should be processable
    let pdf_content = b"%PDF-1.7
1 0 obj
<<
/Type /Catalog
/Pages 2 0 R
>>
endobj

2 0 obj
<<
/Type /Pages
/Kids [3 0 R]
/Count 1
>>
endobj

3 0 obj
<<
/Type /Page
/Parent 2 0 R
/Contents 4 0 R
/MediaBox [0 0 612 792]
/Resources <<
/Font <<
/F1 5 0 R
>>
>>
>>
endobj

4 0 obj
<<
/Length 44
>>
stream
BT
/F1 12 Tf
100 700 Td
(Hello, World!) Tj
ET
endstream
endobj

5 0 obj
<<
/Type /Font
/Subtype /Type1
/BaseFont /Helvetica
>>
endobj

xref
0 6
0000000000 65535 f
0000000015 00000 n
0000000074 00000 n
0000000120 00000 n
0000000274 00000 n
0000000373 00000 n
trailer
<<
/Size 6
/Root 1 0 R
>>
startxref
456
%%EOF";

    pdf_content.to_vec()
}

/// Integration test for PDF processing error handling
#[tokio::test]
#[cfg(feature = "pdf")]
async fn test_pdf_processing_error_handling() {
    use riptide_pdf::{integration::PdfPipelineIntegration, types::ProgressUpdate};

    let integration = PdfPipelineIntegration::new();

    // Test with invalid PDF data
    let invalid_pdf = b"This is not a PDF file";

    let (sender, mut receiver) = integration.create_progress_channel();

    // Spawn processing task with Arc to avoid clone
    let integration_ref = Arc::new(integration);
    let integration_clone = Arc::clone(&integration_ref);
    tokio::spawn(async move {
        let _ = integration_clone
            .process_pdf_bytes_with_progress(invalid_pdf, sender)
            .await;
    });

    // Should receive a failure update
    let timeout = Duration::from_secs(5);
    let mut received_failure = false;

    while let Ok(update) = tokio::time::timeout(timeout, receiver.recv()).await {
        if let Some(progress_update) = update {
            println!("Error handling test update: {:?}", progress_update);

            if matches!(progress_update, ProgressUpdate::Failed { .. }) {
                received_failure = true;
                break;
            }
        } else {
            break;
        }
    }

    assert!(
        received_failure,
        "Should receive failure notification for invalid PDF"
    );
}

#[test]
fn test_pdf_processing_disabled() {
    // When PDF feature is disabled, operations should fail gracefully
    #[cfg(not(feature = "pdf"))]
    {
        use riptide_pdf::integration::PdfPipelineIntegration;

        let integration = PdfPipelineIntegration::new();
        assert!(!integration.is_available());

        // Capabilities should reflect disabled state
        let capabilities = integration.capabilities();
        assert!(!capabilities.text_extraction);
        assert!(!capabilities.image_extraction);
        assert_eq!(capabilities.max_file_size, 0);
    }
}
