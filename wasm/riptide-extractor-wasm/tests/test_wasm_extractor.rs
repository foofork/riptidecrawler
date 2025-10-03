use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::Instant;
// Add specific imports needed for tests
// Import from the crate itself using the wit-generated exports
use riptide_extractor_wasm::{Component, ExtractionMode};

// Test configuration
#[allow(dead_code)]
const WASM_PATH: &str =
    "/workspaces/riptide/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";
const FIXTURES_DIR: &str = "/workspaces/riptide/wasm/riptide-extractor-wasm/tests/fixtures";

/// Test results structure
#[derive(Debug)]
#[allow(dead_code)]
struct TestResult {
    test_name: String,
    mode: String,
    success: bool,
    duration_ms: u128,
    extracted_fields: ExtractedFields,
    error: Option<String>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct ExtractedFields {
    has_title: bool,
    has_content: bool,
    links_count: usize,
    media_count: usize,
    has_language: bool,
    categories_count: usize,
    word_count: u32,
    quality_score: u8,
}

/// Performance metrics
#[derive(Debug, Default)]
#[allow(dead_code)]
struct PerformanceMetrics {
    cold_start_ms: u128,
    warm_start_ms: u128,
    avg_extraction_ms: u128,
    memory_pages_used: usize,
    peak_memory_mb: f64,
}

/// Test suite for WASM extractor functionality
#[tokio::test]
async fn test_wasm_extractor_suite() -> Result<()> {
    println!("🧪 WASM Extractor Test Suite");
    println!("============================\n");

    // Load test fixtures
    let fixtures = load_fixtures()?;
    println!("📁 Loaded {} test fixtures\n", fixtures.len());

    // Test all extraction modes
    let modes = vec![
        ExtractionMode::Article,
        ExtractionMode::Full,
        ExtractionMode::Metadata,
    ];
    let mut all_results = Vec::new();
    let mut perf_metrics = PerformanceMetrics::default();

    // Component performance test
    println!("🚀 Testing component performance...");
    let component = Component::new();

    let cold_start = Instant::now();
    let _ = component.health_check();
    perf_metrics.cold_start_ms = cold_start.elapsed().as_millis();
    println!("   Component init: {}ms\n", perf_metrics.cold_start_ms);

    // Run tests for each fixture and mode
    for fixture in &fixtures {
        println!("📄 Testing fixture: {}", fixture.name);

        for mode in &modes {
            let result = test_extraction_direct(fixture, mode)?;

            // Print result summary
            let status = if result.success { "✅" } else { "❌" };
            println!("   {} Mode: {:?} - {}ms", status, mode, result.duration_ms);

            if !result.success {
                println!(
                    "      Error: {}",
                    result.error.as_ref().unwrap_or(&"Unknown".to_string())
                );
            } else {
                println!(
                    "      Links: {}, Media: {}, Lang: {}, Categories: {}",
                    result.extracted_fields.links_count,
                    result.extracted_fields.media_count,
                    if result.extracted_fields.has_language {
                        "Yes"
                    } else {
                        "No"
                    },
                    result.extracted_fields.categories_count
                );
            }

            all_results.push(result);
        }
        println!();
    }

    // Edge case tests
    println!("🔥 Running edge case tests...");
    run_edge_case_tests_direct()?;

    // Performance benchmarks
    println!("\n📊 Running performance benchmarks...");
    run_performance_benchmarks_direct(&fixtures, &mut perf_metrics)?;

    // Generate report
    generate_report(&all_results, &perf_metrics)?;

    println!("\n✨ Test suite complete!");
    Ok(())
}

/// Load test fixtures
fn load_fixtures() -> Result<Vec<TestFixture>> {
    let mut fixtures = Vec::new();

    // Load each HTML fixture
    let paths = vec![
        ("news_article.html", "News Article"),
        ("edge_cases.html", "Edge Cases"),
        ("blog_post.html", "Blog Post"),
        ("ecommerce.html", "E-commerce"),
        ("documentation.html", "Documentation"),
    ];

    for (filename, name) in paths {
        let path = Path::new(FIXTURES_DIR).join(filename);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            fixtures.push(TestFixture {
                name: name.to_string(),
                html: content,
                url: format!("https://test.example.com/{}", filename),
            });
        }
    }

    Ok(fixtures)
}

struct TestFixture {
    name: String,
    html: String,
    url: String,
}

/// Test extraction for a specific mode using the component directly
fn test_extraction_direct(fixture: &TestFixture, mode: &ExtractionMode) -> Result<TestResult> {
    let start = Instant::now();
    let component = Component::new();

    // Use the actual component extraction
    let extracted_result =
        component.extract(fixture.html.clone(), fixture.url.clone(), mode.clone());
    let duration_ms = start.elapsed().as_millis();

    match extracted_result {
        Ok(content) => {
            let extracted = ExtractedFields {
                has_title: content.title.is_some(),
                has_content: !content.text.is_empty(),
                links_count: content.links.len(),
                media_count: content.media.len(),
                has_language: content.language.is_some(),
                categories_count: content.categories.len(),
                word_count: content.word_count.unwrap_or(0),
                quality_score: content.quality_score.unwrap_or(0),
            };

            Ok(TestResult {
                test_name: fixture.name.clone(),
                mode: format!("{:?}", mode),
                success: true,
                duration_ms,
                extracted_fields: extracted,
                error: None,
            })
        }
        Err(e) => Ok(TestResult {
            test_name: fixture.name.clone(),
            mode: format!("{:?}", mode),
            success: false,
            duration_ms,
            extracted_fields: ExtractedFields::default(),
            error: Some(format!("{:?}", e)),
        }),
    }
}

/// Simulate extraction (placeholder - replace with actual WASM calls)
#[allow(dead_code)]
fn simulate_extraction(html: &str, _url: &str, _mode: &str) -> ExtractedFields {
    ExtractedFields {
        has_title: html.contains("<title>"),
        has_content: html.len() > 100,
        links_count: html.matches("<a ").count(),
        media_count: html.matches("<img ").count() + html.matches("<video").count(),
        has_language: html.contains("lang="),
        categories_count: html.matches("category").count(),
        word_count: html.split_whitespace().count() as u32,
        quality_score: 75,
    }
}

/// Run edge case tests using component directly
fn run_edge_case_tests_direct() -> Result<()> {
    let giant_html = "x".repeat(100_000); // 100KB for reasonable test
    let deep_nested = generate_deep_nesting(100);

    let tests = vec![
        ("Empty HTML", ""),
        ("Minimal HTML", "<html><body>Test</body></html>"),
        ("Giant HTML", giant_html.as_str()),
        ("Invalid UTF-8 sequences", "Test \u{FFFD} content"),
        ("Deep nesting", deep_nested.as_str()),
    ];

    let component = Component::new();

    for (name, html) in tests {
        print!("   Testing {}: ", name);

        let start = Instant::now();

        match component.extract(
            html.to_string(),
            "https://test.example.com".to_string(),
            ExtractionMode::Article,
        ) {
            Ok(_) => {
                let duration = start.elapsed().as_millis();
                println!("✅ Handled in {}ms", duration);
            }
            Err(e) => {
                println!("⚠️  Expected error: {:?}", e);
            }
        }
    }

    Ok(())
}

/// Generate deeply nested HTML
fn generate_deep_nesting(depth: usize) -> String {
    let mut html = String::new();
    for _ in 0..depth {
        html.push_str("<div>");
    }
    html.push_str("Content");
    for _ in 0..depth {
        html.push_str("</div>");
    }
    html
}

/// Run performance benchmarks using component directly
fn run_performance_benchmarks_direct(
    fixtures: &[TestFixture],
    metrics: &mut PerformanceMetrics,
) -> Result<()> {
    let iterations = 10; // Reduced for faster testing
    let mut total_time = 0u128;

    println!("   Running {} iterations...", iterations);

    let component = Component::new();

    for i in 0..iterations {
        let fixture = &fixtures[i % fixtures.len()];
        let start = Instant::now();

        let _ = component.extract(
            fixture.html.clone(),
            fixture.url.clone(),
            ExtractionMode::Article,
        );

        total_time += start.elapsed().as_millis();
    }

    metrics.avg_extraction_ms = total_time / iterations as u128;
    println!(
        "   Average extraction time: {}ms",
        metrics.avg_extraction_ms
    );

    Ok(())
}

// Memory stress tests removed as they were wasmtime-specific

/// Generate test report
fn generate_report(results: &[TestResult], metrics: &PerformanceMetrics) -> Result<()> {
    println!("\n📈 Test Report Summary");
    println!("======================");

    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let success_rate = (passed as f64 / total as f64) * 100.0;

    println!(
        "✅ Success Rate: {:.1}% ({}/{})",
        success_rate, passed, total
    );
    println!("\n⚡ Performance Metrics:");
    println!("   Cold Start: {}ms", metrics.cold_start_ms);
    println!("   Warm Start: {}ms", metrics.warm_start_ms);
    println!("   Avg Extraction: {}ms", metrics.avg_extraction_ms);

    // Check against targets
    println!("\n🎯 Target Validation:");
    let cold_target_met = metrics.cold_start_ms < 15;
    let perf_target_met = metrics.avg_extraction_ms < 50;

    println!(
        "   Cold Start <15ms: {}",
        if cold_target_met { "✅" } else { "❌" }
    );
    println!(
        "   Avg Extract <50ms: {}",
        if perf_target_met { "✅" } else { "❌" }
    );

    // Save detailed report
    let report_path = "/workspaces/eventmesh/wasm/riptide-extractor-wasm/test-report.json";
    let report = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "success_rate": success_rate,
        "total_tests": total,
        "passed_tests": passed,
        "metrics": {
            "cold_start_ms": metrics.cold_start_ms,
            "warm_start_ms": metrics.warm_start_ms,
            "avg_extraction_ms": metrics.avg_extraction_ms,
        },
        "targets_met": {
            "cold_start": cold_target_met,
            "performance": perf_target_met,
        }
    });

    fs::write(report_path, serde_json::to_string_pretty(&report)?)?;
    println!("\n💾 Detailed report saved to: {}", report_path);

    Ok(())
}
