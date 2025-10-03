use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::path::Path;

use riptide_extractor_wasm::{Component, ExtractedContent, ExtractionMode};

/// Serializable version of ExtractedContent for golden tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableExtractedContent {
    pub url: String,
    pub title: Option<String>,
    pub byline: Option<String>,
    pub published_iso: Option<String>,
    pub markdown: String,
    pub text: String,
    pub links: Vec<String>,
    pub media: Vec<String>,
    pub language: Option<String>,
    pub reading_time: Option<u32>,
    pub quality_score: Option<u8>,
    pub word_count: Option<u32>,
    pub categories: Vec<String>,
    pub site_name: Option<String>,
    pub description: Option<String>,
}

impl From<&ExtractedContent> for SerializableExtractedContent {
    fn from(content: &ExtractedContent) -> Self {
        Self {
            url: content.url.clone(),
            title: content.title.clone(),
            byline: content.byline.clone(),
            published_iso: content.published_iso.clone(),
            markdown: content.markdown.clone(),
            text: content.text.clone(),
            links: content.links.clone(),
            media: content.media.clone(),
            language: content.language.clone(),
            reading_time: content.reading_time,
            quality_score: content.quality_score,
            word_count: content.word_count,
            categories: content.categories.clone(),
            site_name: content.site_name.clone(),
            description: content.description.clone(),
        }
    }
}

/// Golden test framework for WASM extractor validation
///
/// This module provides a comprehensive testing framework that validates
/// extraction results against known-good snapshots stored as JSON files.
///
/// ## Test Structure
/// - HTML fixtures are stored in `tests/fixtures/`
/// - Expected results are stored in `tests/golden/snapshots/`
/// - Test runs validate current output against snapshots
///
/// ## Usage
/// ```rust
/// golden_test("news_site", ExtractionMode::Article)?;
/// ```

#[derive(Debug, Clone)]
pub struct GoldenTestCase {
    pub name: &'static str,
    pub html_file: &'static str,
    pub url: &'static str,
    pub mode: ExtractionMode,
    pub expected_features: Vec<&'static str>,
}

pub fn get_golden_test_cases() -> Vec<GoldenTestCase> {
    vec![
        GoldenTestCase {
            name: "news_site_article",
            html_file: "news_site.html",
            url: "https://news.example.com/tech/ai-breakthrough-2024",
            mode: ExtractionMode::Article,
            expected_features: vec![
                "title_extraction",
                "author_detection",
                "published_date",
                "article_content",
                "related_links",
                "category_classification",
                "reading_time",
            ],
        },
        GoldenTestCase {
            name: "news_site_full",
            html_file: "news_site.html",
            url: "https://news.example.com/tech/ai-breakthrough-2024",
            mode: ExtractionMode::Full,
            expected_features: vec![
                "full_page_content",
                "navigation_links",
                "sidebar_content",
                "footer_links",
                "all_media",
                "complete_link_graph",
            ],
        },
        GoldenTestCase {
            name: "blog_post_article",
            html_file: "blog_post.html",
            url: "https://devblog.example.com/scalable-web-apps-guide",
            mode: ExtractionMode::Article,
            expected_features: vec![
                "long_form_content",
                "code_blocks",
                "table_of_contents",
                "author_bio",
                "technical_categories",
                "tutorial_structure",
            ],
        },
        GoldenTestCase {
            name: "gallery_site_full",
            html_file: "gallery_site.html",
            url: "https://photogallery.example.com/collections/tokyo-street-life",
            mode: ExtractionMode::Full,
            expected_features: vec![
                "image_gallery_extraction",
                "media_metadata",
                "photographer_info",
                "collection_structure",
                "image_captions",
                "technical_details",
            ],
        },
        GoldenTestCase {
            name: "nav_heavy_metadata",
            html_file: "nav_heavy_site.html",
            url: "https://projectflow.example.com/dashboard",
            mode: ExtractionMode::Metadata,
            expected_features: vec![
                "site_metadata",
                "navigation_structure",
                "breadcrumb_extraction",
                "user_interface_elements",
                "application_context",
            ],
        },
    ]
}

/// Execute a golden test case
pub fn run_golden_test(test_case: &GoldenTestCase) -> Result<(), String> {
    // Load HTML fixture
    let fixture_path = format!("tests/fixtures/{}", test_case.html_file);
    let html = fs::read_to_string(&fixture_path)
        .map_err(|e| format!("Failed to read fixture {}: {}", fixture_path, e))?;

    // Perform extraction
    let component = Component::new();
    let result = component
        .extract(html, test_case.url.to_string(), test_case.mode.clone())
        .map_err(|e| format!("Extraction failed: {:?}", e))?;

    // Load or create golden snapshot
    let snapshot_path = format!("tests/golden/snapshots/{}.json", test_case.name);

    if Path::new(&snapshot_path).exists() {
        // Compare against existing snapshot
        validate_against_snapshot(&result, &snapshot_path, test_case)
    } else {
        // Create new snapshot
        create_snapshot(&result, &snapshot_path, test_case)
    }
}

/// Validate extraction result against stored snapshot
fn validate_against_snapshot(
    result: &ExtractedContent,
    snapshot_path: &str,
    test_case: &GoldenTestCase,
) -> Result<(), String> {
    let snapshot_json =
        fs::read_to_string(snapshot_path).map_err(|e| format!("Failed to read snapshot: {}", e))?;

    let expected: serde_json::Value = serde_json::from_str(&snapshot_json)
        .map_err(|e| format!("Failed to parse snapshot JSON: {}", e))?;

    let serializable_result = SerializableExtractedContent::from(result);
    let actual = serde_json::to_value(&serializable_result)
        .map_err(|e| format!("Failed to serialize result: {}", e))?;

    // Core field validation
    validate_field(&actual, &expected, "url", test_case.name)?;
    validate_field(&actual, &expected, "title", test_case.name)?;
    validate_optional_field(&actual, &expected, "byline", test_case.name)?;
    validate_optional_field(&actual, &expected, "published_iso", test_case.name)?;

    // Content validation (allow for minor differences)
    validate_content_similarity(&actual, &expected, "text", test_case.name, 0.95)?;
    validate_content_similarity(&actual, &expected, "markdown", test_case.name, 0.90)?;

    // Array field validation
    validate_array_field(&actual, &expected, "links", test_case.name)?;
    validate_array_field(&actual, &expected, "media", test_case.name)?;
    validate_array_field(&actual, &expected, "categories", test_case.name)?;

    // Numeric field validation (with tolerance)
    validate_numeric_field(&actual, &expected, "reading_time", test_case.name, 10.0)?;
    validate_numeric_field(&actual, &expected, "quality_score", test_case.name, 5.0)?;
    validate_numeric_field(&actual, &expected, "word_count", test_case.name, 50.0)?;

    // Feature-specific validation
    validate_expected_features(result, test_case)?;

    Ok(())
}

/// Create a new golden snapshot
fn create_snapshot(
    result: &ExtractedContent,
    snapshot_path: &str,
    test_case: &GoldenTestCase,
) -> Result<(), String> {
    // Create snapshot directory if it doesn't exist
    if let Some(parent) = Path::new(snapshot_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create snapshot directory: {}", e))?;
    }

    // Serialize result with pretty formatting
    let serializable_result = SerializableExtractedContent::from(result);
    let json = serde_json::to_string_pretty(&serializable_result)
        .map_err(|e| format!("Failed to serialize result: {}", e))?;

    // Add metadata header
    let snapshot_with_metadata = format!(
        "// Golden test snapshot for: {}\n// Generated for URL: {}\n// Extraction mode: {:?}\n// Expected features: {:?}\n{}",
        test_case.name, test_case.url, test_case.mode, test_case.expected_features, json
    );

    fs::write(snapshot_path, snapshot_with_metadata)
        .map_err(|e| format!("Failed to write snapshot: {}", e))?;

    println!("Created new golden snapshot: {}", snapshot_path);
    Ok(())
}

/// Validate a required field matches
fn validate_field(
    actual: &serde_json::Value,
    expected: &serde_json::Value,
    field: &str,
    test_name: &str,
) -> Result<(), String> {
    let actual_val = actual.get(field);
    let expected_val = expected.get(field);

    match (actual_val, expected_val) {
        (Some(a), Some(e)) => {
            if a != e {
                return Err(format!(
                    "Field '{}' mismatch in {}: expected {:?}, got {:?}",
                    field, test_name, e, a
                ));
            }
        }
        (None, Some(e)) => {
            return Err(format!(
                "Missing required field '{}' in {}: expected {:?}",
                field, test_name, e
            ));
        }
        (Some(a), None) => {
            return Err(format!(
                "Unexpected field '{}' in {}: got {:?}",
                field, test_name, a
            ));
        }
        (None, None) => {} // Both missing is OK for optional fields
    }

    Ok(())
}

/// Validate an optional field (None is acceptable)
fn validate_optional_field(
    actual: &serde_json::Value,
    expected: &serde_json::Value,
    field: &str,
    test_name: &str,
) -> Result<(), String> {
    let actual_val = actual.get(field);
    let expected_val = expected.get(field);

    if let (Some(a), Some(e)) = (actual_val, expected_val) {
        if a != e {
            return Err(format!(
                "Optional field '{}' mismatch in {}: expected {:?}, got {:?}",
                field, test_name, e, a
            ));
        }
    }

    Ok(())
}

/// Validate content similarity with tolerance for minor differences
fn validate_content_similarity(
    actual: &serde_json::Value,
    expected: &serde_json::Value,
    field: &str,
    test_name: &str,
    similarity_threshold: f64,
) -> Result<(), String> {
    let actual_str = actual.get(field).and_then(|v| v.as_str()).unwrap_or("");
    let expected_str = expected.get(field).and_then(|v| v.as_str()).unwrap_or("");

    if actual_str.is_empty() && expected_str.is_empty() {
        return Ok(());
    }

    let similarity = calculate_similarity(actual_str, expected_str);
    if similarity < similarity_threshold {
        return Err(format!(
            "Content similarity too low for field '{}' in {}: {:.2}% (threshold: {:.2}%)",
            field,
            test_name,
            similarity * 100.0,
            similarity_threshold * 100.0
        ));
    }

    Ok(())
}

/// Validate array fields (order-independent)
fn validate_array_field(
    actual: &serde_json::Value,
    expected: &serde_json::Value,
    field: &str,
    test_name: &str,
) -> Result<(), String> {
    let actual_arr = actual.get(field).and_then(|v| v.as_array());
    let expected_arr = expected.get(field).and_then(|v| v.as_array());

    match (actual_arr, expected_arr) {
        (Some(a), Some(e)) => {
            if a.len() != e.len() {
                return Err(format!(
                    "Array '{}' length mismatch in {}: expected {}, got {}",
                    field,
                    test_name,
                    e.len(),
                    a.len()
                ));
            }

            // Check that all expected items are present (order-independent)
            for expected_item in e {
                if !a.contains(expected_item) {
                    return Err(format!(
                        "Missing array item in '{}' for {}: {:?}",
                        field, test_name, expected_item
                    ));
                }
            }
        }
        (None, Some(e)) if !e.is_empty() => {
            return Err(format!(
                "Missing array field '{}' in {}: expected {} items",
                field,
                test_name,
                e.len()
            ));
        }
        _ => {} // Other combinations are acceptable
    }

    Ok(())
}

/// Validate numeric fields with tolerance
fn validate_numeric_field(
    actual: &serde_json::Value,
    expected: &serde_json::Value,
    field: &str,
    test_name: &str,
    tolerance: f64,
) -> Result<(), String> {
    let actual_num = actual.get(field).and_then(|v| v.as_f64());
    let expected_num = expected.get(field).and_then(|v| v.as_f64());

    if let (Some(a), Some(e)) = (actual_num, expected_num) {
        let diff = (a - e).abs();
        if diff > tolerance {
            return Err(format!(
                "Numeric field '{}' difference too large in {}: expected {:.2}, got {:.2} (diff: {:.2}, tolerance: {:.2})",
                field, test_name, e, a, diff, tolerance
            ));
        }
    }

    Ok(())
}

/// Validate that expected features are present
fn validate_expected_features(
    result: &ExtractedContent,
    test_case: &GoldenTestCase,
) -> Result<(), String> {
    for feature in &test_case.expected_features {
        match *feature {
            "title_extraction" => {
                if result.title.is_none() || result.title.as_ref().unwrap().is_empty() {
                    return Err(format!(
                        "Feature '{}' missing in {}",
                        feature, test_case.name
                    ));
                }
            }
            "author_detection" => {
                if result.byline.is_none() || result.byline.as_ref().unwrap().is_empty() {
                    return Err(format!(
                        "Feature '{}' missing in {}",
                        feature, test_case.name
                    ));
                }
            }
            "published_date" => {
                if result.published_iso.is_none() {
                    return Err(format!(
                        "Feature '{}' missing in {}",
                        feature, test_case.name
                    ));
                }
            }
            "article_content" => {
                if result.text.is_empty() && result.markdown.is_empty() {
                    return Err(format!(
                        "Feature '{}' missing in {}",
                        feature, test_case.name
                    ));
                }
            }
            "related_links" => {
                if result.links.is_empty() {
                    return Err(format!(
                        "Feature '{}' missing in {}",
                        feature, test_case.name
                    ));
                }
            }
            "category_classification" => {
                if result.categories.is_empty() {
                    return Err(format!(
                        "Feature '{}' missing in {}",
                        feature, test_case.name
                    ));
                }
            }
            "reading_time" => {
                if result.reading_time.is_none() {
                    return Err(format!(
                        "Feature '{}' missing in {}",
                        feature, test_case.name
                    ));
                }
            }
            "all_media" => {
                if result.media.is_empty() {
                    return Err(format!(
                        "Feature '{}' missing in {}",
                        feature, test_case.name
                    ));
                }
            }
            _ => {} // Other features are optional
        }
    }

    Ok(())
}

/// Calculate text similarity using a simple algorithm
fn calculate_similarity(text1: &str, text2: &str) -> f64 {
    if text1 == text2 {
        return 1.0;
    }

    if text1.is_empty() || text2.is_empty() {
        return 0.0;
    }

    // Simple word-based similarity
    let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
    let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Run all golden tests
pub fn run_all_golden_tests() -> Result<(), String> {
    let mut failures = Vec::new();
    let test_cases = get_golden_test_cases();

    for test_case in &test_cases {
        match run_golden_test(test_case) {
            Ok(()) => println!("✓ Golden test passed: {}", test_case.name),
            Err(e) => {
                println!("✗ Golden test failed: {}: {}", test_case.name, e);
                failures.push(format!("{}: {}", test_case.name, e));
            }
        }
    }

    if failures.is_empty() {
        println!("All {} golden tests passed!", test_cases.len());
        Ok(())
    } else {
        Err(format!("Golden tests failed:\n{}", failures.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news_site_article_extraction() {
        let test_cases = get_golden_test_cases();
        let test_case = &test_cases[0];
        run_golden_test(test_case).expect("News site article extraction should pass");
    }

    #[test]
    fn test_blog_post_article_extraction() {
        let test_cases = get_golden_test_cases();
        let test_case = &test_cases[2];
        run_golden_test(test_case).expect("Blog post article extraction should pass");
    }

    #[test]
    fn test_gallery_full_extraction() {
        let test_cases = get_golden_test_cases();
        let test_case = &test_cases[3];
        run_golden_test(test_case).expect("Gallery full extraction should pass");
    }

    #[test]
    fn test_nav_heavy_metadata_extraction() {
        let test_cases = get_golden_test_cases();
        let test_case = &test_cases[4];
        run_golden_test(test_case).expect("Nav-heavy metadata extraction should pass");
    }

    #[test]
    fn test_all_golden_tests() {
        run_all_golden_tests().expect("All golden tests should pass");
    }

    #[test]
    fn test_similarity_calculation() {
        assert_eq!(calculate_similarity("hello world", "hello world"), 1.0);
        assert_eq!(calculate_similarity("", ""), 1.0);
        assert_eq!(calculate_similarity("hello", ""), 0.0);

        let sim = calculate_similarity("hello world foo", "hello world bar");
        assert!(sim > 0.5 && sim < 1.0);
    }
}
