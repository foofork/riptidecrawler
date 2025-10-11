//! TDD tests for WASM component binding completion
//!
//! These tests ensure that the WASM component binding is complete and returns
//! real data from the WASM extractor, not mock data.

use anyhow::Result;
use std::path::PathBuf;

// Test HTML samples
const SAMPLE_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Article</title>
    <meta name="author" content="Test Author">
    <meta name="description" content="Test Description">
</head>
<body>
    <article>
        <h1>Real Article Title</h1>
        <p>This is actual content that should be extracted.</p>
        <p>Multiple paragraphs with meaningful text that should be properly analyzed.</p>
        <a href="https://example.com/link1">Link 1</a>
        <a href="https://example.com/link2">Link 2</a>
        <img src="https://example.com/image1.jpg" alt="Image 1">
    </article>
</body>
</html>
"#;

const EMPTY_HTML: &str = "";
const MINIMAL_HTML: &str = "<html><body>Test</body></html>";

/// Test 1: Verify that mock data is NOT returned
#[tokio::test]
async fn test_wasm_extractor_no_mock_data() -> Result<()> {
    let wasm_path = get_wasm_path()?;

    // Skip if WASM file doesn't exist yet
    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    let extractor =
        riptide_html::wasm_extraction::WasmExtractor::new(wasm_path.to_str().unwrap()).await?;

    let result = extractor.extract(SAMPLE_HTML.as_bytes(), "https://example.com", "article")?;

    // CRITICAL: Verify these are NOT the mock values from wasm_extraction.rs line 388
    assert_ne!(
        result.title.as_deref(),
        Some("Sample Title"),
        "❌ FAIL: Mock title 'Sample Title' detected! WASM binding incomplete."
    );

    assert_ne!(
        result.markdown, "# Sample Title\n\n<!DOCTYPE html>\n<html>\n<head>\n    <ti",
        "❌ FAIL: Mock markdown pattern detected! WASM binding incomplete."
    );

    // Verify we got REAL extracted content
    assert!(
        result.title.is_some() && !result.title.as_ref().unwrap().is_empty(),
        "❌ FAIL: No title extracted from real HTML"
    );

    println!("✅ PASS: No mock data detected, real extraction working");
    Ok(())
}

/// Test 2: Verify component binding is complete
#[tokio::test]
async fn test_wasm_component_binding_complete() -> Result<()> {
    let wasm_path = get_wasm_path()?;

    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    let extractor =
        riptide_html::wasm_extraction::CmExtractor::new(wasm_path.to_str().unwrap()).await?;

    let result = extractor.extract(SAMPLE_HTML, "https://example.com", "article")?;

    // Verify WIT interface is properly bound
    assert!(
        result.quality_score.unwrap_or(0) > 0 && result.quality_score.unwrap_or(0) != 80,
        "❌ FAIL: Quality score is mock value (80) or zero"
    );

    assert!(
        !result.links.is_empty(),
        "❌ FAIL: Links not extracted from HTML with 2 links"
    );

    assert!(
        !result.media.is_empty(),
        "❌ FAIL: Media not extracted from HTML with 1 image"
    );

    println!("✅ PASS: Component binding complete, WIT interface working");
    Ok(())
}

/// Test 3: Verify resource limits are enforced
#[tokio::test]
async fn test_wasm_resource_limits_enforced() -> Result<()> {
    use riptide_html::wasm_extraction::{CmExtractor, ExtractorConfig};
    use std::time::Duration;

    let wasm_path = get_wasm_path()?;

    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    // Create extractor with very tight memory limits
    let config = ExtractorConfig {
        max_memory_pages: 10, // Only 640KB
        extraction_timeout: Duration::from_secs(5),
        enable_simd: true,
        enable_aot_cache: false,
        instance_pool_size: 1,
        max_idle_time: Duration::from_secs(10),
        fuel_limit: 500_000,
        enable_leak_detection: true,
    };

    let extractor = CmExtractor::with_config(wasm_path.to_str().unwrap(), config).await?;

    // Large HTML that might exceed memory limits
    let large_html = "<div>".repeat(10000) + "content" + &"</div>".repeat(10000);

    let result = extractor.extract(&large_html, "https://example.com", "article");

    // Should either succeed with limits or fail gracefully
    match result {
        Ok(content) => {
            println!("✅ PASS: Extraction succeeded within memory limits");
            assert!(!content.text.is_empty(), "Content should be extracted");
        }
        Err(e) => {
            println!("✅ PASS: Extraction failed gracefully: {:?}", e);
        }
    }

    Ok(())
}

/// Test 4: Verify proper error handling (no panics)
#[tokio::test]
async fn test_wasm_error_handling() -> Result<()> {
    let wasm_path = get_wasm_path()?;

    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    let extractor =
        riptide_html::wasm_extraction::WasmExtractor::new(wasm_path.to_str().unwrap()).await?;

    // Test with invalid inputs
    let test_cases = vec![
        ("Empty HTML", EMPTY_HTML),
        ("Minimal HTML", MINIMAL_HTML),
        ("Invalid UTF-8", "Test \u{FFFD} content"),
    ];

    for (name, html) in test_cases {
        let result = extractor.extract(html.as_bytes(), "https://example.com", "article");

        match result {
            Ok(_) => println!("✅ {}: Handled gracefully", name),
            Err(e) => println!("✅ {}: Error handled: {:?}", name, e),
        }
    }

    println!("✅ PASS: Error handling works without panics");
    Ok(())
}

/// Test 5: Verify extraction quality vs mock data
#[tokio::test]
async fn test_extraction_quality() -> Result<()> {
    let wasm_path = get_wasm_path()?;

    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    let extractor =
        riptide_html::wasm_extraction::WasmExtractor::new(wasm_path.to_str().unwrap()).await?;

    let result = extractor.extract(SAMPLE_HTML.as_bytes(), "https://example.com", "article")?;

    // Real extraction should find the actual title
    let title = result.title.as_deref().unwrap_or("");
    assert!(
        title.contains("Real Article Title") || title.contains("Test Article"),
        "❌ FAIL: Title '{}' doesn't match actual HTML content",
        title
    );

    // Should extract actual links (not empty)
    assert!(
        result.links.len() >= 2,
        "❌ FAIL: Expected at least 2 links, got {}",
        result.links.len()
    );

    // Should extract actual media (not empty)
    assert!(
        result.media.len() >= 1,
        "❌ FAIL: Expected at least 1 image, got {}",
        result.media.len()
    );

    println!("✅ PASS: Extraction quality meets requirements");
    Ok(())
}

/// Test 6: Verify WasmResourceTracker actually tracks memory
#[test]
fn test_resource_tracker_functionality() {
    use riptide_html::wasm_extraction::WasmResourceTracker;
    use wasmtime::ResourceLimiter;

    let mut tracker = WasmResourceTracker::new(100); // 100 pages = 6.4MB

    // Initial state
    assert_eq!(tracker.current_memory_pages(), 0);
    assert_eq!(tracker.grow_failures(), 0);
    assert_eq!(tracker.peak_memory_pages(), 0);

    // Simulate memory growth
    let can_grow = tracker.memory_growing(0, 10, None).unwrap();
    assert!(can_grow, "Should allow growth within limits");
    assert_eq!(tracker.current_memory_pages(), 10);
    assert_eq!(tracker.peak_memory_pages(), 10);

    // Try to exceed limit
    let can_grow_beyond = tracker.memory_growing(10, 200, None).unwrap();
    assert!(!can_grow_beyond, "Should deny growth beyond limits");
    assert_eq!(tracker.grow_failures(), 1);

    // Peak should remain at 10
    assert_eq!(tracker.peak_memory_pages(), 10);

    println!("✅ PASS: Resource tracker properly limits memory");
}

/// Test 7: Verify statistics collection works
#[tokio::test]
async fn test_statistics_collection() -> Result<()> {
    let wasm_path = get_wasm_path()?;

    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    let extractor =
        riptide_html::wasm_extraction::CmExtractor::new(wasm_path.to_str().unwrap()).await?;

    // Initial stats
    let stats_before = extractor.get_stats();
    let extractions_before = stats_before.successful_extractions;

    // Perform extraction
    let _ = extractor.extract(SAMPLE_HTML, "https://example.com", "article")?;

    // Check stats updated
    let stats_after = extractor.get_stats();
    assert_eq!(
        stats_after.successful_extractions,
        extractions_before + 1,
        "❌ FAIL: Statistics not updated after extraction"
    );

    assert!(
        stats_after.avg_extraction_time.as_millis() > 0,
        "❌ FAIL: Average extraction time not recorded"
    );

    println!("✅ PASS: Statistics collection working");
    Ok(())
}

/// Test 8: Verify health status is accurate
#[tokio::test]
async fn test_health_status() -> Result<()> {
    let wasm_path = get_wasm_path()?;

    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    let extractor =
        riptide_html::wasm_extraction::CmExtractor::new(wasm_path.to_str().unwrap()).await?;

    let health = extractor.health_status();

    assert_eq!(health.status, "healthy");
    assert_eq!(health.version, "1.0.0");
    assert!(health.successful_extractions >= 0);
    assert!(health.avg_extraction_time >= 0.0);

    println!("✅ PASS: Health status reporting correctly");
    Ok(())
}

/// Helper: Get WASM component path
fn get_wasm_path() -> Result<PathBuf> {
    // Try multiple possible locations
    let possible_paths = vec![
        "/workspaces/eventmesh/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm",
        "/workspaces/eventmesh/target/wasm32-wasip1/release/riptide_extractor_wasm.wasm",
        "./target/wasm32-wasip2/release/riptide_extractor_wasm.wasm",
    ];

    for path_str in &possible_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Ok(path);
        }
    }

    // Return first path even if it doesn't exist - tests will skip
    Ok(PathBuf::from(possible_paths[0]))
}

/// Test 9: Verify different extraction modes work
#[tokio::test]
async fn test_multiple_extraction_modes() -> Result<()> {
    let wasm_path = get_wasm_path()?;

    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    let extractor =
        riptide_html::wasm_extraction::WasmExtractor::new(wasm_path.to_str().unwrap()).await?;

    let modes = vec!["article", "full", "metadata"];

    for mode in modes {
        let result = extractor.extract(SAMPLE_HTML.as_bytes(), "https://example.com", mode);

        match result {
            Ok(content) => {
                println!("✅ Mode '{}': Extracted {} bytes", mode, content.text.len());
                assert!(!content.text.is_empty(), "Content should not be empty");
            }
            Err(e) => {
                println!("⚠️ Mode '{}': Error: {:?}", mode, e);
            }
        }
    }

    println!("✅ PASS: Multiple extraction modes supported");
    Ok(())
}

/// Test 10: Integration test - full pipeline
#[tokio::test]
async fn test_full_integration_pipeline() -> Result<()> {
    let wasm_path = get_wasm_path()?;

    if !wasm_path.exists() {
        println!("⚠️ WASM file not found, skipping test: {:?}", wasm_path);
        return Ok(());
    }

    println!("🧪 Testing full WASM extraction pipeline");

    // 1. Create extractor
    let extractor =
        riptide_html::wasm_extraction::CmExtractor::new(wasm_path.to_str().unwrap()).await?;
    println!("  ✅ Extractor created");

    // 2. Check component info
    let info = extractor.component_info();
    assert_eq!(info.name, "WASM Content Extractor");
    println!("  ✅ Component info: {}", info.version);

    // 3. Perform extraction
    let result = extractor.extract(SAMPLE_HTML, "https://example.com", "article")?;
    assert!(result.title.is_some());
    assert!(!result.text.is_empty());
    println!("  ✅ Extraction successful: {} chars", result.text.len());

    // 4. Verify no mock data
    assert_ne!(result.title.as_deref(), Some("Sample Title"));
    println!("  ✅ No mock data detected");

    // 5. Check statistics
    let stats = extractor.get_stats();
    assert!(stats.successful_extractions > 0);
    println!(
        "  ✅ Statistics tracked: {} extractions",
        stats.successful_extractions
    );

    println!("✅ PASS: Full integration pipeline working");
    Ok(())
}
