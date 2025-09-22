use riptide_core::{
    component::CmExtractor,
    types::{ExtractedDoc, ExtractionMode, HealthStatus, ComponentInfo}
};
use std::path::Path;

const TEST_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Test Article Title</title>
    <meta name="description" content="This is a test article description">
    <meta name="author" content="Test Author">
    <meta property="article:published_time" content="2023-12-01T10:00:00Z">
</head>
<body>
    <article>
        <h1>Test Article Title</h1>
        <p class="byline">By Test Author</p>
        <div class="content">
            <p>This is the main content of the test article. It contains multiple paragraphs and provides substantial text for testing the extraction capabilities.</p>
            <p>The second paragraph continues the content and adds more text to test word counting and reading time estimation.</p>
            <a href="https://example.com/link1">External Link 1</a>
            <a href="https://example.com/link2">External Link 2</a>
            <img src="https://example.com/image1.jpg" alt="Test Image 1">
            <img src="https://example.com/image2.jpg" alt="Test Image 2">
        </div>
    </article>
</body>
</html>
"#;

const MINIMAL_HTML: &str = r#"
<html>
<head><title>Minimal</title></head>
<body><p>Minimal content</p></body>
</html>
"#;

/// Helper function to get the WASM component path
fn get_wasm_path() -> String {
    std::env::var("WASM_COMPONENT_PATH")
        .unwrap_or_else(|_| "target/wasm32-wasip2/release/riptide_extractor_wasm.wasm".to_string())
}

/// Test basic component creation and instantiation
#[test]
fn test_component_creation() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let result = CmExtractor::new(&wasm_path);
    assert!(result.is_ok(), "Failed to create CmExtractor: {:?}", result.err());
}

/// Test basic content extraction with article mode
#[test]
fn test_basic_extraction() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    let result = extractor.extract_typed(
        TEST_HTML,
        "https://test.example.com",
        ExtractionMode::Article
    );

    assert!(result.is_ok(), "Extraction failed: {:?}", result.err());

    let doc = result.unwrap();
    assert_eq!(doc.url, "https://test.example.com");
    assert_eq!(doc.title, Some("Test Article Title".to_string()));
    assert!(doc.text.contains("main content"));
    assert!(!doc.links.is_empty());
    assert!(!doc.media.is_empty());
}

/// Test extraction with different modes
#[test]
fn test_extraction_modes() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    // Test Article mode
    let article_result = extractor.extract_typed(
        TEST_HTML,
        "https://test.example.com",
        ExtractionMode::Article
    );
    assert!(article_result.is_ok());

    // Test Full mode
    let full_result = extractor.extract_typed(
        TEST_HTML,
        "https://test.example.com",
        ExtractionMode::Full
    );
    assert!(full_result.is_ok());

    // Test Metadata mode
    let metadata_result = extractor.extract_typed(
        TEST_HTML,
        "https://test.example.com",
        ExtractionMode::Metadata
    );
    assert!(metadata_result.is_ok());

    // Test Custom mode with CSS selectors
    let custom_result = extractor.extract_typed(
        TEST_HTML,
        "https://test.example.com",
        ExtractionMode::Custom(vec![".content".to_string(), "article".to_string()])
    );
    assert!(custom_result.is_ok());
}

/// Test extraction with statistics
#[test]
fn test_extraction_with_stats() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    let result = extractor.extract_with_stats(
        TEST_HTML,
        "https://test.example.com",
        ExtractionMode::Article
    );

    assert!(result.is_ok(), "Extraction with stats failed: {:?}", result.err());

    let (doc, stats) = result.unwrap();

    // Verify document content
    assert_eq!(doc.url, "https://test.example.com");
    assert!(doc.title.is_some());

    // Verify statistics
    assert!(stats.processing_time_ms > 0);
    assert!(stats.memory_used > 0);
    assert_eq!(stats.links_found, doc.links.len() as u32);
    assert_eq!(stats.images_found, doc.media.len() as u32);
}

/// Test HTML validation
#[test]
fn test_html_validation() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    // Test valid HTML
    let valid_result = extractor.validate_html(TEST_HTML);
    assert!(valid_result.is_ok());
    assert!(valid_result.unwrap(), "Valid HTML should pass validation");

    // Test minimal HTML
    let minimal_result = extractor.validate_html(MINIMAL_HTML);
    assert!(minimal_result.is_ok());
    assert!(minimal_result.unwrap(), "Minimal HTML should pass validation");

    // Test empty HTML
    let empty_result = extractor.validate_html("");
    assert!(empty_result.is_ok());
    assert!(!empty_result.unwrap(), "Empty HTML should fail validation");

    // Test invalid HTML
    let invalid_result = extractor.validate_html("Not HTML content");
    assert!(invalid_result.is_ok());
    assert!(!invalid_result.unwrap(), "Invalid HTML should fail validation");
}

/// Test component health check
#[test]
fn test_health_check() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    let health_result = extractor.health_check();
    assert!(health_result.is_ok(), "Health check failed: {:?}", health_result.err());

    let health = health_result.unwrap();
    assert_eq!(health.status, "healthy");
    assert!(!health.version.is_empty());
    assert_eq!(health.trek_version, "0.2.1");
    assert!(!health.capabilities.is_empty());
}

/// Test component info retrieval
#[test]
fn test_component_info() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    let info_result = extractor.get_info();
    assert!(info_result.is_ok(), "Get info failed: {:?}", info_result.err());

    let info = info_result.unwrap();
    assert!(!info.name.is_empty());
    assert!(!info.version.is_empty());
    assert_eq!(info.component_model_version, "0.2.0");
    assert!(!info.features.is_empty());
    assert!(!info.supported_modes.is_empty());
}

/// Test supported modes retrieval
#[test]
fn test_get_modes() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    let modes_result = extractor.get_modes();
    assert!(modes_result.is_ok(), "Get modes failed: {:?}", modes_result.err());

    let modes = modes_result.unwrap();
    assert!(!modes.is_empty());
    assert!(modes.len() >= 4); // Should have at least article, full, metadata, custom

    // Check that expected modes are present
    let modes_text = modes.join(" ");
    assert!(modes_text.contains("article"));
    assert!(modes_text.contains("full"));
    assert!(modes_text.contains("metadata"));
    assert!(modes_text.contains("custom"));
}

/// Test component state reset
#[test]
fn test_reset_state() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    let reset_result = extractor.reset_state();
    assert!(reset_result.is_ok(), "Reset state failed: {:?}", reset_result.err());

    let message = reset_result.unwrap();
    assert!(message.contains("reset"));
}

/// Test error handling with invalid inputs
#[test]
fn test_error_handling() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    // Test with empty HTML
    let empty_result = extractor.extract_typed(
        "",
        "https://test.example.com",
        ExtractionMode::Article
    );
    assert!(empty_result.is_err(), "Empty HTML should result in error");

    // Test with invalid URL
    let invalid_url_result = extractor.extract_typed(
        TEST_HTML,
        "not-a-valid-url",
        ExtractionMode::Article
    );
    assert!(invalid_url_result.is_err(), "Invalid URL should result in error");
}

/// Test legacy string-based mode compatibility
#[test]
fn test_legacy_mode_compatibility() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    // Test legacy string modes
    let article_result = extractor.extract(TEST_HTML, "https://test.example.com", "article");
    assert!(article_result.is_ok());

    let full_result = extractor.extract(TEST_HTML, "https://test.example.com", "full");
    assert!(full_result.is_ok());

    let metadata_result = extractor.extract(TEST_HTML, "https://test.example.com", "metadata");
    assert!(metadata_result.is_ok());

    // Test unknown mode (should default to article)
    let unknown_result = extractor.extract(TEST_HTML, "https://test.example.com", "unknown");
    assert!(unknown_result.is_ok());
}

/// Performance benchmark test
#[test]
fn test_performance_benchmark() {
    let wasm_path = get_wasm_path();

    if !Path::new(&wasm_path).exists() {
        println!("Skipping test: WASM component not found at {}", wasm_path);
        return;
    }

    let extractor = CmExtractor::new(&wasm_path).expect("Failed to create extractor");

    let start = std::time::Instant::now();

    // Perform multiple extractions to test performance
    for i in 0..10 {
        let url = format!("https://test{}.example.com", i);
        let result = extractor.extract_typed(
            TEST_HTML,
            &url,
            ExtractionMode::Article
        );
        assert!(result.is_ok(), "Extraction {} failed", i);
    }

    let duration = start.elapsed();
    println!("10 extractions completed in: {:?}", duration);
    println!("Average per extraction: {:?}", duration / 10);

    // Basic performance assertion (should complete 10 extractions in reasonable time)
    assert!(duration.as_secs() < 30, "Performance test took too long: {:?}", duration);
}