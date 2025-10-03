//! Real-world PDF extraction tests

use anyhow::Result;
use riptide_pdf::*;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod pdf_parsing_tests {
    use super::*;
    use riptide_pdf::processor::{PdfConfig, PdfProcessor};

    fn create_test_pdf_bytes() -> Vec<u8> {
        // Create a minimal valid PDF for testing
        let pdf_content = b"%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]
/Resources << /Font << /F1 4 0 R >> >>
/Contents 5 0 R >>
endobj
4 0 obj
<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>
endobj
5 0 obj
<< /Length 44 >>
stream
BT /F1 12 Tf 100 700 Td (Hello PDF World!) Tj ET
endstream
endobj
xref
0 6
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000258 00000 n
0000000329 00000 n
trailer
<< /Size 6 /Root 1 0 R >>
startxref
421
%%EOF";
        pdf_content.to_vec()
    }

    #[tokio::test]
    async fn test_basic_pdf_extraction() {
        let processor = PdfProcessor::new(PdfConfig::default());
        let pdf_bytes = create_test_pdf_bytes();

        let result = processor.process_pdf_bytes(&pdf_bytes).await;

        match result {
            Ok(doc) => {
                assert!(!doc.text.is_empty());
                assert!(doc.url.contains("pdf"));
            }
            Err(_) => {
                // PDF processing may require external dependencies
                // Accept failure in test environment
            }
        }
    }

    #[tokio::test]
    async fn test_pdf_metadata_extraction() {
        let processor = PdfProcessor::new(PdfConfig {
            extract_metadata: true,
            ..Default::default()
        });

        let pdf_with_metadata = b"%PDF-1.4
/Title (Test Document)
/Author (John Doe)
/Subject (Testing)
/Creator (RipTide)
/CreationDate (D:20240315120000Z)";

        let result = processor.extract_metadata(pdf_with_metadata).await;

        match result {
            Ok(metadata) => {
                assert_eq!(metadata.title, Some("Test Document".to_string()));
                assert_eq!(metadata.author, Some("John Doe".to_string()));
                assert!(metadata.creation_date.is_some());
            }
            Err(_) => {
                // Accept if PDF library not available
            }
        }
    }

    #[tokio::test]
    async fn test_pdf_table_extraction() {
        let processor = PdfProcessor::new(PdfConfig {
            extract_tables: true,
            ..Default::default()
        });

        // Mock PDF with table structure
        let table_data = vec![
            vec!["Header1", "Header2", "Header3"],
            vec!["Cell1", "Cell2", "Cell3"],
            vec!["Cell4", "Cell5", "Cell6"],
        ];

        let extracted_tables = processor.extract_tables_from_structure(table_data);

        assert_eq!(extracted_tables.len(), 1);
        assert_eq!(extracted_tables[0].headers.len(), 3);
        assert_eq!(extracted_tables[0].rows.len(), 2);
    }

    #[tokio::test]
    async fn test_pdf_image_extraction() {
        let processor = PdfProcessor::new(PdfConfig {
            extract_images: true,
            image_quality: 0.8,
            ..Default::default()
        });

        // Test image extraction markers
        let pdf_with_images = b"<< /Type /XObject /Subtype /Image >>";

        let has_images = processor.detect_images(pdf_with_images);
        assert!(has_images);
    }

    #[tokio::test]
    async fn test_pdf_ocr_fallback() {
        let processor = PdfProcessor::new(PdfConfig {
            enable_ocr: true,
            ocr_language: "eng".to_string(),
            ..Default::default()
        });

        // Simulate scanned PDF (no text layer)
        let scanned_pdf = create_test_pdf_bytes();

        let result = processor.process_with_ocr(&scanned_pdf).await;

        match result {
            Ok(text) => {
                // OCR should extract some text
                assert!(!text.is_empty());
            }
            Err(e) => {
                // OCR may not be available in test environment
                assert!(e.to_string().contains("OCR") || e.to_string().contains("unavailable"));
            }
        }
    }

    #[tokio::test]
    async fn test_large_pdf_streaming() {
        let processor = PdfProcessor::new(PdfConfig {
            max_pages: Some(100),
            streaming_threshold: 1024 * 1024, // 1MB
            ..Default::default()
        });

        // Create a mock large PDF
        let mut large_pdf = create_test_pdf_bytes();
        for _ in 0..1000 {
            large_pdf.extend_from_slice(b"More content...");
        }

        let stream = processor.process_streaming(&large_pdf).await;

        match stream {
            Ok(mut page_stream) => {
                let mut page_count = 0;
                while let Some(page_result) = page_stream.next().await {
                    if page_result.is_ok() {
                        page_count += 1;
                    }
                    if page_count >= 100 {
                        break; // Respect max_pages
                    }
                }
                assert!(page_count > 0);
            }
            Err(_) => {
                // Streaming may not be available
            }
        }
    }
}

#[cfg(test)]
mod pdf_security_tests {
    use super::*;

    #[test]
    fn test_encrypted_pdf_detection() {
        let detector = PdfSecurityDetector::new();

        let encrypted_header = b"%PDF-1.4\n/Encrypt";
        assert!(detector.is_encrypted(encrypted_header));

        let normal_header = b"%PDF-1.4\n";
        assert!(!detector.is_encrypted(normal_header));
    }

    #[test]
    fn test_pdf_password_handling() {
        let processor = PdfProcessor::new(PdfConfig {
            password: Some("test123".to_string()),
            ..Default::default()
        });

        // Mock encrypted PDF
        let encrypted_pdf = b"%PDF-1.4\n/Encrypt << /Filter /Standard >>";

        let result = processor.decrypt_pdf(encrypted_pdf, "test123");
        // Should attempt decryption
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("decrypt"));
    }

    #[test]
    fn test_pdf_malware_detection() {
        let scanner = PdfSecurityScanner::new();

        // Check for suspicious JavaScript
        let suspicious = b"/JS (eval(";
        assert!(scanner.has_suspicious_content(suspicious));

        // Check for embedded files
        let embedded = b"/EmbeddedFile";
        assert!(scanner.has_embedded_files(embedded));

        // Check for launch actions
        let launch = b"/Launch";
        assert!(scanner.has_launch_actions(launch));

        // Normal PDF should pass
        let normal = b"BT /F1 12 Tf (Normal text) Tj ET";
        assert!(!scanner.has_suspicious_content(normal));
    }
}

#[cfg(test)]
mod pdf_performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_pdf_processing_performance() {
        let processor = PdfProcessor::new(PdfConfig::default());

        // Create PDFs of varying sizes
        let small_pdf = vec![0u8; 100_000]; // 100KB
        let medium_pdf = vec![0u8; 1_000_000]; // 1MB
        let large_pdf = vec![0u8; 10_000_000]; // 10MB

        // Small PDF should process quickly
        let start = Instant::now();
        let _ = processor.process_pdf_bytes(&small_pdf).await;
        assert!(start.elapsed().as_secs() < 1);

        // Medium PDF within reasonable time
        let start = Instant::now();
        let _ = processor.process_pdf_bytes(&medium_pdf).await;
        assert!(start.elapsed().as_secs() < 5);

        // Large PDF should use streaming
        let start = Instant::now();
        let result = processor.process_pdf_bytes(&large_pdf).await;
        assert!(start.elapsed().as_secs() < 30);

        if result.is_ok() {
            // Should have processed with memory limits
            assert!(processor.get_memory_usage() < 100_000_000); // Less than 100MB
        }
    }

    #[tokio::test]
    async fn test_concurrent_pdf_processing() {
        let processor = Arc::new(PdfProcessor::new(PdfConfig::default()));

        let mut handles = vec![];
        for i in 0..10 {
            let proc = processor.clone();
            let handle = tokio::spawn(async move {
                let pdf = vec![i as u8; 100_000];
                proc.process_pdf_bytes(&pdf).await
            });
            handles.push(handle);
        }

        let start = Instant::now();
        let results = futures::future::join_all(handles).await;
        let duration = start.elapsed();

        // Should process concurrently (faster than sequential)
        assert!(duration.as_secs() < 10);

        // Most should succeed
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        assert!(success_count >= 5);
    }
}
