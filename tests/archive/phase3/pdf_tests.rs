use riptide_core::pdf::{
    PdfConfig, PdfProcessor, DefaultPdfProcessor, PdfProcessingResult, PdfMetadata,
    PdfImage, PdfStats, ImageFormat, ImagePosition, ImageExtractionSettings,
    TextExtractionSettings, StructuredContent, PdfTable, PdfList, ListType,
    OutlineItem, FormField, FieldType, PdfError, PdfCapabilities, utils,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_pdf_config_creation() {
    let config = PdfConfig {
        max_size_bytes: 50 * 1024 * 1024, // 50MB
        extract_text: true,
        extract_images: true,
        extract_metadata: true,
        image_settings: ImageExtractionSettings {
            max_images: 20,
            min_dimensions: (100, 100),
            formats: vec![ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Gif],
            include_positions: true,
            base64_encode: false,
        },
        text_settings: TextExtractionSettings {
            preserve_formatting: true,
            include_coordinates: true,
            group_by_blocks: true,
            min_font_size: 8.0,
            extract_tables: true,
        },
        timeout_seconds: 60,
    };

    assert_eq!(config.max_size_bytes, 50 * 1024 * 1024);
    assert!(config.extract_text);
    assert!(config.extract_images);
    assert!(config.extract_metadata);
    assert_eq!(config.timeout_seconds, 60);

    assert_eq!(config.image_settings.max_images, 20);
    assert_eq!(config.image_settings.min_dimensions, (100, 100));
    assert_eq!(config.image_settings.formats.len(), 3);
    assert!(config.image_settings.include_positions);
    assert!(!config.image_settings.base64_encode);

    assert!(config.text_settings.preserve_formatting);
    assert!(config.text_settings.include_coordinates);
    assert!(config.text_settings.extract_tables);
    assert_eq!(config.text_settings.min_font_size, 8.0);

    // Test serialization
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: PdfConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.max_size_bytes, deserialized.max_size_bytes);
    assert_eq!(config.image_settings.max_images, deserialized.image_settings.max_images);
    assert_eq!(config.text_settings.min_font_size, deserialized.text_settings.min_font_size);
}

#[tokio::test]
async fn test_pdf_processing_result() {
    let result = PdfProcessingResult {
        success: true,
        text: Some("This is extracted PDF text content.".to_string()),
        images: vec![
            PdfImage {
                index: 0,
                page: 1,
                data: Some("iVBORw0KGgoAAAANS...".to_string()),
                format: ImageFormat::Png,
                width: 800,
                height: 600,
                position: Some(ImagePosition {
                    x: 100.0,
                    y: 200.0,
                    width: 400.0,
                    height: 300.0,
                }),
                alt_text: Some("Figure 1: Sample chart".to_string()),
            },
            PdfImage {
                index: 1,
                page: 2,
                data: None, // Not extracted
                format: ImageFormat::Jpeg,
                width: 1200,
                height: 900,
                position: None,
                alt_text: None,
            },
        ],
        metadata: PdfMetadata {
            title: Some("Sample PDF Document".to_string()),
            author: Some("John Doe".to_string()),
            subject: Some("Test Document".to_string()),
            keywords: Some("test, pdf, sample".to_string()),
            creator: Some("Adobe Acrobat".to_string()),
            producer: Some("Adobe PDF Library 15.0".to_string()),
            creation_date: Some("2024-01-15T10:30:00Z".to_string()),
            modification_date: Some("2024-01-15T12:45:00Z".to_string()),
            pdf_version: Some("1.7".to_string()),
            page_count: 5,
            encrypted: false,
            allows_copying: true,
            allows_printing: true,
            custom_metadata: {
                let mut meta = HashMap::new();
                meta.insert("CustomField1".to_string(), "Value1".to_string());
                meta.insert("Department".to_string(), "Engineering".to_string());
                meta
            },
        },
        structured_content: Some(StructuredContent {
            tables: vec![
                PdfTable {
                    page: 2,
                    position: Some(ImagePosition {
                        x: 50.0,
                        y: 100.0,
                        width: 500.0,
                        height: 200.0,
                    }),
                    rows: vec![
                        vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
                        vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
                        vec!["Bob".to_string(), "25".to_string(), "San Francisco".to_string()],
                    ],
                    headers: Some(vec!["Name".to_string(), "Age".to_string(), "City".to_string()]),
                    caption: Some("Employee Data".to_string()),
                },
            ],
            lists: vec![
                PdfList {
                    page: 1,
                    items: vec![
                        "First item".to_string(),
                        "Second item".to_string(),
                        "Third item".to_string(),
                    ],
                    list_type: ListType::Ordered,
                    level: 0,
                },
                PdfList {
                    page: 3,
                    items: vec![
                        "Bullet point one".to_string(),
                        "Bullet point two".to_string(),
                    ],
                    list_type: ListType::Unordered,
                    level: 1,
                },
            ],
            outline: vec![
                OutlineItem {
                    title: "Chapter 1: Introduction".to_string(),
                    page: Some(1),
                    level: 0,
                    children: vec![
                        OutlineItem {
                            title: "1.1 Overview".to_string(),
                            page: Some(2),
                            level: 1,
                            children: Vec::new(),
                        },
                    ],
                },
                OutlineItem {
                    title: "Chapter 2: Details".to_string(),
                    page: Some(3),
                    level: 0,
                    children: Vec::new(),
                },
            ],
            forms: vec![
                FormField {
                    name: "firstName".to_string(),
                    field_type: FieldType::Text,
                    value: Some("John".to_string()),
                    position: Some(ImagePosition {
                        x: 100.0,
                        y: 500.0,
                        width: 150.0,
                        height: 20.0,
                    }),
                    required: true,
                },
                FormField {
                    name: "newsletter".to_string(),
                    field_type: FieldType::Checkbox,
                    value: Some("true".to_string()),
                    position: Some(ImagePosition {
                        x: 100.0,
                        y: 450.0,
                        width: 15.0,
                        height: 15.0,
                    }),
                    required: false,
                },
            ],
        }),
        stats: PdfStats {
            processing_time_ms: 2500,
            memory_used: 1024 * 1024 * 10, // 10MB
            pages_processed: 5,
            images_extracted: 2,
            tables_found: 1,
            text_length: 1500,
            file_size: 1024 * 1024 * 5, // 5MB
        },
        error: None,
    };

    assert!(result.success);
    assert!(result.text.is_some());
    assert_eq!(result.images.len(), 2);
    assert_eq!(result.metadata.page_count, 5);
    assert!(result.structured_content.is_some());
    assert_eq!(result.stats.pages_processed, 5);
    assert!(result.error.is_none());

    // Test image data
    let first_image = &result.images[0];
    assert_eq!(first_image.index, 0);
    assert_eq!(first_image.page, 1);
    assert!(matches!(first_image.format, ImageFormat::Png));
    assert_eq!(first_image.width, 800);
    assert_eq!(first_image.height, 600);
    assert!(first_image.position.is_some());
    assert!(first_image.alt_text.is_some());

    // Test metadata
    assert_eq!(result.metadata.title, Some("Sample PDF Document".to_string()));
    assert_eq!(result.metadata.author, Some("John Doe".to_string()));
    assert_eq!(result.metadata.page_count, 5);
    assert!(!result.metadata.encrypted);
    assert_eq!(result.metadata.custom_metadata.len(), 2);

    // Test structured content
    if let Some(structured) = &result.structured_content {
        assert_eq!(structured.tables.len(), 1);
        assert_eq!(structured.lists.len(), 2);
        assert_eq!(structured.outline.len(), 2);
        assert_eq!(structured.forms.len(), 2);

        // Test table structure
        let table = &structured.tables[0];
        assert_eq!(table.page, 2);
        assert_eq!(table.rows.len(), 3);
        assert!(table.headers.is_some());
        assert!(table.caption.is_some());

        // Test list structure
        let ordered_list = &structured.lists[0];
        assert!(matches!(ordered_list.list_type, ListType::Ordered));
        assert_eq!(ordered_list.level, 0);
        assert_eq!(ordered_list.items.len(), 3);

        // Test outline structure
        let first_chapter = &structured.outline[0];
        assert_eq!(first_chapter.title, "Chapter 1: Introduction");
        assert_eq!(first_chapter.page, Some(1));
        assert_eq!(first_chapter.level, 0);
        assert_eq!(first_chapter.children.len(), 1);

        // Test form fields
        let text_field = &structured.forms[0];
        assert!(matches!(text_field.field_type, FieldType::Text));
        assert_eq!(text_field.name, "firstName");
        assert!(text_field.required);
        assert!(text_field.position.is_some());

        let checkbox_field = &structured.forms[1];
        assert!(matches!(checkbox_field.field_type, FieldType::Checkbox));
        assert!(!checkbox_field.required);
    }

    // Test serialization
    let json = serde_json::to_string(&result).expect("Should serialize");
    let deserialized: PdfProcessingResult = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(result.success, deserialized.success);
    assert_eq!(result.images.len(), deserialized.images.len());
    assert_eq!(result.metadata.title, deserialized.metadata.title);
}

#[tokio::test]
async fn test_pdf_error_types() {
    let errors = vec![
        PdfError::InvalidPdf {
            message: "Missing PDF header".to_string(),
        },
        PdfError::EncryptedPdf,
        PdfError::FileTooLarge {
            size: 200 * 1024 * 1024,
            max_size: 100 * 1024 * 1024,
        },
        PdfError::CorruptedPdf {
            message: "Invalid xref table".to_string(),
        },
        PdfError::Timeout {
            timeout_seconds: 30,
        },
        PdfError::MemoryLimit {
            used: 512 * 1024 * 1024,
            limit: 256 * 1024 * 1024,
        },
        PdfError::UnsupportedVersion {
            version: "2.0".to_string(),
        },
        PdfError::ProcessingError {
            message: "Failed to parse fonts".to_string(),
        },
        PdfError::IoError {
            message: "File read error".to_string(),
        },
    ];

    for error in errors {
        // Test display formatting
        let error_string = error.to_string();
        assert!(!error_string.is_empty());

        // Test that it implements Error trait
        let _: &dyn std::error::Error = &error;

        // Test serialization
        let json = serde_json::to_string(&error).expect("Should serialize");
        let deserialized: PdfError = serde_json::from_str(&json).expect("Should deserialize");

        assert!(std::mem::discriminant(&error) == std::mem::discriminant(&deserialized));
    }
}

#[tokio::test]
async fn test_default_pdf_processor() {
    let processor = DefaultPdfProcessor::new();

    // Test availability
    assert!(processor.is_available());

    // Test capabilities
    let capabilities = processor.capabilities();
    assert!(capabilities.text_extraction);
    assert!(capabilities.metadata_extraction);
    assert!(!capabilities.image_extraction); // Default processor doesn't support images
    assert!(!capabilities.table_extraction);
    assert!(!capabilities.form_extraction);
    assert!(!capabilities.encrypted_pdfs);
    assert_eq!(capabilities.max_file_size, 100 * 1024 * 1024);
    assert!(!capabilities.supported_versions.is_empty());
}

#[tokio::test]
async fn test_pdf_processor_with_valid_pdf() {
    let processor = DefaultPdfProcessor::new();
    let config = PdfConfig::default();

    // Create a minimal valid PDF structure
    let pdf_data = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n2 0 obj\n<<\n/Type /Pages\n/Kids [3 0 R]\n/Count 1\n>>\nendobj\n3 0 obj\n<<\n/Type /Page\n/Parent 2 0 R\n/MediaBox [0 0 612 792]\n>>\nendobj\nxref\n0 4\n0000000000 65535 f \n0000000015 00000 n \n0000000074 00000 n \n0000000120 00000 n \ntrailer\n<<\n/Size 4\n/Root 1 0 R\n>>\nstartxref\n203\n%%EOF";

    let result = processor.process_pdf(pdf_data, &config).await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    assert!(result.text.is_some());
    assert_eq!(result.metadata.page_count, 1);
    assert_eq!(result.stats.file_size, pdf_data.len() as u64);
}

#[tokio::test]
async fn test_pdf_processor_with_invalid_data() {
    let processor = DefaultPdfProcessor::new();
    let config = PdfConfig::default();

    // Test with non-PDF data
    let invalid_data = b"This is not a PDF file";

    let result = processor.process_pdf(invalid_data, &config).await;
    assert!(result.is_err());

    if let Err(error) = result {
        assert!(matches!(error, PdfError::InvalidPdf { .. }));
        assert!(error.to_string().contains("PDF header"));
    }
}

#[tokio::test]
async fn test_pdf_processor_with_large_file() {
    let processor = DefaultPdfProcessor::new();
    let config = PdfConfig {
        max_size_bytes: 1024, // 1KB limit
        ..Default::default()
    };

    // Create data larger than the limit
    let large_data = vec![0u8; 2048]; // 2KB

    let result = processor.process_pdf(&large_data, &config).await;
    assert!(result.is_err());

    if let Err(error) = result {
        assert!(matches!(error, PdfError::FileTooLarge { .. }));
        assert!(error.to_string().contains("too large"));
    }
}

#[tokio::test]
async fn test_pdf_capabilities() {
    let capabilities = PdfCapabilities {
        text_extraction: true,
        image_extraction: true,
        metadata_extraction: true,
        table_extraction: false,
        form_extraction: true,
        encrypted_pdfs: false,
        max_file_size: 50 * 1024 * 1024,
        supported_versions: vec![
            "1.4".to_string(),
            "1.5".to_string(),
            "1.6".to_string(),
            "1.7".to_string(),
        ],
    };

    assert!(capabilities.text_extraction);
    assert!(capabilities.image_extraction);
    assert!(capabilities.metadata_extraction);
    assert!(!capabilities.table_extraction);
    assert!(capabilities.form_extraction);
    assert!(!capabilities.encrypted_pdfs);
    assert_eq!(capabilities.max_file_size, 50 * 1024 * 1024);
    assert_eq!(capabilities.supported_versions.len(), 4);

    // Test serialization
    let json = serde_json::to_string(&capabilities).expect("Should serialize");
    let deserialized: PdfCapabilities = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(capabilities.text_extraction, deserialized.text_extraction);
    assert_eq!(capabilities.max_file_size, deserialized.max_file_size);
    assert_eq!(capabilities.supported_versions, deserialized.supported_versions);
}

#[test]
fn test_pdf_content_detection() {
    // Test with valid PDF magic bytes
    let pdf_data = b"%PDF-1.7\nsome content";
    assert!(utils::is_pdf_content(None, pdf_data));
    assert!(utils::is_pdf_content(Some("application/pdf"), pdf_data));
    assert!(utils::is_pdf_content(Some("application/pdf; charset=utf-8"), pdf_data));

    // Test with content type but no magic bytes
    let non_pdf_data = b"<html><body>Not a PDF</body></html>";
    assert!(utils::is_pdf_content(Some("application/pdf"), non_pdf_data));
    assert!(!utils::is_pdf_content(Some("text/html"), non_pdf_data));
    assert!(!utils::is_pdf_content(None, non_pdf_data));

    // Test with empty data
    assert!(!utils::is_pdf_content(None, b""));
    assert!(!utils::is_pdf_content(Some("application/pdf"), b""));
}

#[test]
fn test_pdf_version_extraction() {
    // Test valid PDF headers
    assert_eq!(utils::extract_pdf_version(b"%PDF-1.4\n"), Some("1.4".to_string()));
    assert_eq!(utils::extract_pdf_version(b"%PDF-1.7\n"), Some("1.7".to_string()));
    assert_eq!(utils::extract_pdf_version(b"%PDF-2.0\nmore content"), Some("2.0".to_string()));

    // Test invalid headers
    assert_eq!(utils::extract_pdf_version(b"not a pdf"), None);
    assert_eq!(utils::extract_pdf_version(b"%PDF-"), None);
    assert_eq!(utils::extract_pdf_version(b""), None);

    // Test truncated headers
    assert_eq!(utils::extract_pdf_version(b"%PDF-1."), None);
}

#[test]
fn test_processing_complexity_estimation() {
    use utils::ProcessingComplexity;

    // Test different file sizes
    assert!(matches!(utils::estimate_complexity(500_000), ProcessingComplexity::Low)); // 500KB
    assert!(matches!(utils::estimate_complexity(5_000_000), ProcessingComplexity::Medium)); // 5MB
    assert!(matches!(utils::estimate_complexity(25_000_000), ProcessingComplexity::High)); // 25MB
    assert!(matches!(utils::estimate_complexity(100_000_000), ProcessingComplexity::VeryHigh)); // 100MB

    // Test boundary conditions
    assert!(matches!(utils::estimate_complexity(1_048_576), ProcessingComplexity::Medium)); // Exactly 1MB
    assert!(matches!(utils::estimate_complexity(10_485_760), ProcessingComplexity::High)); // Exactly 10MB
    assert!(matches!(utils::estimate_complexity(52_428_800), ProcessingComplexity::VeryHigh)); // Exactly 50MB
}

#[tokio::test]
async fn test_image_format_serialization() {
    let formats = vec![
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Gif,
        ImageFormat::Bmp,
        ImageFormat::Tiff,
    ];

    for format in formats {
        let json = serde_json::to_string(&format).expect("Should serialize");
        let deserialized: ImageFormat = serde_json::from_str(&json).expect("Should deserialize");

        assert!(std::mem::discriminant(&format) == std::mem::discriminant(&deserialized));
    }
}

#[tokio::test]
async fn test_field_type_serialization() {
    let field_types = vec![
        FieldType::Text,
        FieldType::Checkbox,
        FieldType::Radio,
        FieldType::ComboBox,
        FieldType::ListBox,
        FieldType::Button,
        FieldType::Signature,
    ];

    for field_type in field_types {
        let json = serde_json::to_string(&field_type).expect("Should serialize");
        let deserialized: FieldType = serde_json::from_str(&json).expect("Should deserialize");

        assert!(std::mem::discriminant(&field_type) == std::mem::discriminant(&deserialized));
    }
}

#[tokio::test]
async fn test_list_type_serialization() {
    let list_types = vec![
        ListType::Ordered,
        ListType::Unordered,
        ListType::Definition,
    ];

    for list_type in list_types {
        let json = serde_json::to_string(&list_type).expect("Should serialize");
        let deserialized: ListType = serde_json::from_str(&json).expect("Should deserialize");

        assert!(std::mem::discriminant(&list_type) == std::mem::discriminant(&deserialized));
    }
}