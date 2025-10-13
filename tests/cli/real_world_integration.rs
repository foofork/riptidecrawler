/// Real-World Integration Tests using TestHarness
///
/// Comprehensive CLI testing with content validation and baseline comparison.
///
/// Prerequisites:
/// 1. Start Redis: `docker run -d -p 6379:6379 redis:alpine`
/// 2. Start API: `cargo run --bin riptide-api`
/// 3. Run tests: `cargo test --test real_world_integration -- --test-threads=1`

use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;

// Import common test utilities
mod common;
use common::{
    TestHarness, ContentValidator, BaselineManager,
    ValidationRule, RuleType,
};

const API_URL: &str = "http://localhost:8080";

/// Helper to create a test harness instance
fn create_test_harness() -> TestHarness {
    let output_dir = PathBuf::from("test-results");
    let binary_path = PathBuf::from(env!("CARGO_BIN_EXE_riptide"));
    TestHarness::new(output_dir, binary_path)
}

/// Helper to create a baseline manager
fn create_baseline_manager() -> Result<BaselineManager> {
    BaselineManager::new(PathBuf::from("test-results/baselines"))
}

#[tokio::test]
#[ignore] // Run with: cargo test --test real_world_integration -- --ignored
async fn test_wikipedia_extraction_with_validation() -> Result<()> {
    let harness = create_test_harness();
    let url = "https://en.wikipedia.org/wiki/Web_scraping";

    println!("🧪 Testing Wikipedia extraction with content validation...");

    let (content, duration) = harness
        .run_extraction("trek", url, 30)
        .await?;

    // Create validator with expected criteria
    let mut validator = ContentValidator::new();

    validator.add_rule(ValidationRule {
        name: "min_content_length".to_string(),
        rule_type: RuleType::ContentLength { min: 2000, max: None },
        threshold: None,
        expected_value: Some(serde_json::json!(2000)),
        required: true,
    });

    validator.add_rule(ValidationRule {
        name: "wikipedia_keywords".to_string(),
        rule_type: RuleType::KeywordPresence {
            keywords: vec![
                "web".to_string(),
                "scraping".to_string(),
                "data".to_string(),
            ],
            min_matches: 2,
        },
        threshold: None,
        expected_value: None,
        required: true,
    });

    validator.add_rule(ValidationRule {
        name: "extraction_time".to_string(),
        rule_type: RuleType::ExtractionTime { max_ms: 5000 },
        threshold: Some(5000.0),
        expected_value: Some(serde_json::json!(5000)),
        required: false,
    });

    let metadata = HashMap::new();
    let results = validator.validate(&content, &metadata, duration.as_millis() as u64);

    println!("\n📊 Validation Results:");
    for result in &results {
        if result.passed {
            println!("  ✅ {}: {}", result.rule_name, result.message);
        } else {
            println!("  ❌ {}: {}", result.rule_name, result.message);
        }
    }

    let all_passed = results.iter().all(|r| r.passed);
    assert!(all_passed, "Not all validation rules passed");

    println!("\n✅ Wikipedia extraction validation passed!");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_documentation_with_baseline_comparison() -> Result<()> {
    let harness = create_test_harness();
    let baseline_manager = create_baseline_manager()?;

    let test_id = "docs-rust-book";
    let method = "trek";
    let url = "https://doc.rust-lang.org/book/ch01-00-getting-started.html";

    println!("🧪 Testing Rust docs with baseline comparison...");

    let (content, duration) = harness
        .run_extraction(method, url, 30)
        .await?;

    let mut metadata = HashMap::new();
    metadata.insert("quality_score".to_string(), serde_json::json!(0.85));
    metadata.insert("has_code_examples".to_string(), serde_json::json!(true));

    // Check if baseline exists
    if !baseline_manager.baseline_exists(test_id, method) {
        println!("📝 Creating baseline for {}/{}", test_id, method);
        let baseline = baseline_manager.generate_baseline(
            test_id,
            method,
            url,
            &content,
            &metadata,
            duration.as_millis() as u64,
        )?;
        baseline_manager.save_baseline(&baseline)?;
        println!("✅ Baseline created successfully");
        return Ok(());
    }

    // Load and compare against baseline
    let baseline = baseline_manager.load_baseline(test_id, method)?;
    let comparison = baseline_manager.compare_against_baseline(
        &baseline,
        &content,
        &metadata,
        duration.as_millis() as u64,
    );

    println!("\n📊 Baseline Comparison:");
    println!("  {}", comparison.summary);

    if !comparison.differences.is_empty() {
        println!("\n  Differences:");
        for diff in &comparison.differences {
            println!("    • {}: {:?} → {:?} ({:?})",
                diff.field,
                diff.baseline_value,
                diff.current_value,
                diff.severity
            );
        }
    }

    assert!(comparison.passed, "Baseline comparison failed: {}", comparison.summary);

    println!("\n✅ Baseline comparison passed!");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_example_com_simple_validation() -> Result<()> {
    let harness = create_test_harness();
    let url = "https://example.com";

    println!("🧪 Testing example.com with simple validation...");

    let (content, duration) = harness
        .run_extraction("trek", url, 30)
        .await?;

    // Create default validator from expected values
    let mut expected = HashMap::new();
    expected.insert("has_title".to_string(), serde_json::json!(true));
    expected.insert("min_content_length".to_string(), serde_json::json!(50));

    let validator = ContentValidator::create_default(&expected);

    let metadata = HashMap::new();
    let results = validator.validate(&content, &metadata, duration.as_millis() as u64);

    println!("\n📊 Validation Results:");
    for result in &results {
        if result.passed {
            println!("  ✅ {}: {}", result.rule_name, result.message);
        } else {
            println!("  ❌ {}: {}", result.rule_name, result.message);
        }
    }

    let all_required_passed = results.iter()
        .filter(|r| r.rule_name == "content_length" || r.rule_name == "title_presence")
        .all(|r| r.passed);

    assert!(all_required_passed, "Required validation rules failed");

    println!("\n✅ Example.com validation passed!");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_multiple_urls_with_test_suite() -> Result<()> {
    let harness = create_test_harness();

    // Load test URLs from JSON file
    let test_urls_path = PathBuf::from("tests/webpage-extraction/test-urls.json");

    if !test_urls_path.exists() {
        println!("⚠️  Test URLs file not found, skipping suite test");
        return Ok(());
    }

    let test_urls = harness.load_test_urls(&test_urls_path).await?;

    // Run test suite with first 5 URLs only (to keep test time reasonable)
    let limited_urls = common::TestUrls {
        test_urls: test_urls.test_urls.into_iter().take(5).collect(),
    };

    println!("🧪 Running test suite with {} URLs...", limited_urls.test_urls.len());

    let methods = vec!["trek".to_string()];
    let session = harness.run_test_suite(&limited_urls, &methods).await?;

    println!("\n📊 Test Suite Results:");
    println!("  Total tests: {}", session.total_tests);
    println!("  Successful: {}", session.successful_tests);
    println!("  Failed: {}", session.failed_tests);
    println!("  Pass rate: {:.1}%",
        (session.successful_tests as f64 / session.total_tests as f64) * 100.0
    );

    // Require at least 80% pass rate
    let pass_rate = session.successful_tests as f64 / session.total_tests as f64;
    assert!(pass_rate >= 0.8, "Pass rate {:.1}% below 80% threshold", pass_rate * 100.0);

    println!("\n✅ Test suite completed successfully!");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_json_api_extraction() -> Result<()> {
    let harness = create_test_harness();
    let url = "https://jsonplaceholder.typicode.com/posts/1";

    println!("🧪 Testing JSON API extraction...");

    let (content, duration) = harness
        .run_extraction("trek", url, 30)
        .await?;

    // Validate JSON structure
    let is_json = content.trim_start().starts_with('{');
    assert!(is_json, "Expected JSON response");

    // Parse JSON to validate structure
    let json: serde_json::Value = serde_json::from_str(&content)?;
    assert!(json.get("title").is_some(), "JSON missing 'title' field");
    assert!(json.get("body").is_some(), "JSON missing 'body' field");

    println!("  ✅ JSON structure valid");
    println!("  ✅ Duration: {} ms", duration.as_millis());
    println!("  ✅ Content length: {} bytes", content.len());

    println!("\n✅ JSON API extraction passed!");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_error_handling_404() -> Result<()> {
    let harness = create_test_harness();
    let url = "https://httpstat.us/404";

    println!("🧪 Testing error handling for 404 status...");

    let result = harness.run_extraction("trek", url, 30).await;

    // We expect this to either fail or return minimal content
    match result {
        Ok((content, duration)) => {
            println!("  ⚠️  Extraction succeeded with {} bytes", content.len());
            println!("  Duration: {} ms", duration.as_millis());
            // This is acceptable - some tools handle 404s gracefully
        }
        Err(e) => {
            println!("  ✅ Extraction failed as expected: {}", e);
        }
    }

    println!("\n✅ Error handling test completed!");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_performance_benchmark() -> Result<()> {
    let harness = create_test_harness();
    let url = "https://example.com";
    let iterations = 3;

    println!("🧪 Running performance benchmark ({} iterations)...", iterations);

    let mut durations = Vec::new();

    for i in 1..=iterations {
        println!("  Iteration {}/{}...", i, iterations);
        let (_, duration) = harness
            .run_extraction("trek", url, 30)
            .await?;
        durations.push(duration.as_millis() as u64);
    }

    let avg_duration = durations.iter().sum::<u64>() / durations.len() as u64;
    let min_duration = *durations.iter().min().unwrap();
    let max_duration = *durations.iter().max().unwrap();

    println!("\n📊 Performance Metrics:");
    println!("  Average: {} ms", avg_duration);
    println!("  Minimum: {} ms", min_duration);
    println!("  Maximum: {} ms", max_duration);

    // Assert performance is reasonable (< 5 seconds average)
    assert!(avg_duration < 5000, "Average duration {} ms exceeds 5000 ms threshold", avg_duration);

    println!("\n✅ Performance benchmark passed!");
    Ok(())
}

// Helper module to re-export common types
mod common {
    pub use super::super::common::{
        TestHarness, TestUrl, TestUrls, ExtractionResult, TestSession,
        ContentValidator, ValidationRule, ValidationResult, RuleType,
        BaselineManager, Baseline, ComparisonResult,
    };
}
