use riptide_core::component::CmExtractor;
use riptide_core::types::ExtractionMode;
use std::path::Path;

#[test]
fn test_component_model_instantiation() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = Path::new("target/wasm32-wasip2/debug/riptide_extractor_wasm.wasm");

    // Skip test if WASM file doesn't exist
    if !wasm_path.exists() {
        println!("WASM component not found at {}, skipping test", wasm_path.display());
        return Ok(());
    }

    println!("Testing Component Model instantiation with WASM file: {}", wasm_path.display());

    // Test creating the extractor
    let extractor = CmExtractor::new(wasm_path.to_str().unwrap())?;
    println!("✓ Component Model extractor created successfully");

    Ok(())
}

#[test]
fn test_basic_extraction_functionality() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = Path::new("target/wasm32-wasip2/debug/riptide_extractor_wasm.wasm");

    // Skip test if WASM file doesn't exist
    if !wasm_path.exists() {
        println!("WASM component not found at {}, skipping test", wasm_path.display());
        return Ok(());
    }

    let extractor = CmExtractor::new(wasm_path.to_str().unwrap())?;

    // Test basic HTML extraction
    let html = r#"
        <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Main Title</h1>
                <p>This is a test paragraph with content.</p>
                <a href="https://example.com">Test Link</a>
            </body>
        </html>
    "#;

    let url = "https://test.example.com";

    // Test extraction with different modes
    println!("Testing article extraction mode...");
    let result = extractor.extract_typed(html, url, ExtractionMode::Article)?;
    assert_eq!(result.url, url);
    assert!(result.title.is_some());
    assert!(!result.text.is_empty());
    println!("✓ Article extraction successful");

    println!("Testing metadata extraction mode...");
    let metadata_result = extractor.extract_typed(html, url, ExtractionMode::Metadata)?;
    assert_eq!(metadata_result.url, url);
    println!("✓ Metadata extraction successful");

    Ok(())
}

#[test]
fn test_component_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = Path::new("target/wasm32-wasip2/debug/riptide_extractor_wasm.wasm");

    // Skip test if WASM file doesn't exist
    if !wasm_path.exists() {
        println!("WASM component not found at {}, skipping test", wasm_path.display());
        return Ok(());
    }

    let extractor = CmExtractor::new(wasm_path.to_str().unwrap())?;

    // Test health check
    println!("Testing component health check...");
    let health = extractor.health_check()?;

    assert!(!health.status.is_empty());
    assert!(!health.version.is_empty());
    assert!(!health.trek_version.is_empty());
    println!("✓ Health check successful: status={}, version={}, trek_version={}",
             health.status, health.version, health.trek_version);

    Ok(())
}

#[test]
fn test_component_info() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = Path::new("target/wasm32-wasip2/debug/riptide_extractor_wasm.wasm");

    // Skip test if WASM file doesn't exist
    if !wasm_path.exists() {
        println!("WASM component not found at {}, skipping test", wasm_path.display());
        return Ok(());
    }

    let extractor = CmExtractor::new(wasm_path.to_str().unwrap())?;

    // Test component info
    println!("Testing component info retrieval...");
    let info = extractor.get_info()?;

    assert!(!info.name.is_empty());
    assert!(!info.version.is_empty());
    println!("✓ Component info retrieved: name={}, version={}", info.name, info.version);

    Ok(())
}

#[test]
fn test_html_validation() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = Path::new("target/wasm32-wasip2/debug/riptide_extractor_wasm.wasm");

    // Skip test if WASM file doesn't exist
    if !wasm_path.exists() {
        println!("WASM component not found at {}, skipping test", wasm_path.display());
        return Ok(());
    }

    let extractor = CmExtractor::new(wasm_path.to_str().unwrap())?;

    // Test HTML validation
    let valid_html = "<html><body><p>Valid content</p></body></html>";
    let invalid_html = "Not HTML content at all";

    println!("Testing HTML validation...");
    let valid_result = extractor.validate_html(valid_html)?;
    let invalid_result = extractor.validate_html(invalid_html)?;

    assert!(valid_result, "Valid HTML should pass validation");
    assert!(!invalid_result, "Invalid HTML should fail validation");
    println!("✓ HTML validation working correctly");

    Ok(())
}

#[test]
fn test_extraction_modes() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = Path::new("target/wasm32-wasip2/debug/riptide_extractor_wasm.wasm");

    // Skip test if WASM file doesn't exist
    if !wasm_path.exists() {
        println!("WASM component not found at {}, skipping test", wasm_path.display());
        return Ok(());
    }

    let extractor = CmExtractor::new(wasm_path.to_str().unwrap())?;

    // Test getting supported modes
    println!("Testing extraction modes retrieval...");
    let modes = extractor.get_modes()?;

    assert!(!modes.is_empty(), "Should return at least one supported mode");
    println!("✓ Supported modes: {:?}", modes);

    Ok(())
}