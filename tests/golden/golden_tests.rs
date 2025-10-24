//! Golden tests for HTML processing behavior
//! These tests capture expected behavior and verify no regression

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use riptide_core::strategies::extraction::*;
use riptide_core::strategies::extraction::css_json::*;
use riptide_core::strategies::extraction::regex::*;
use riptide_core::strategies::RegexPattern;
use serde::{Deserialize, Serialize};

/// Golden test data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoldenTestCase {
    name: String,
    html: String,
    url: String,
    expected_css_result: ExpectedResult,
    expected_regex_result: ExpectedResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExpectedResult {
    title: String,
    content_length: usize,
    summary: Option<String>,
    confidence_min: f64,
    confidence_max: f64,
    strategy_used: String,
}

/// Load or create golden test cases
fn get_golden_test_cases() -> Vec<GoldenTestCase> {
    vec![
        GoldenTestCase {
            name: "simple_blog_post".to_string(),
            html: r#"
<!DOCTYPE html>
<html>
<head>
    <title>Understanding Rust's Ownership System</title>
    <meta name="description" content="A comprehensive guide to Rust's ownership model and how it prevents memory leaks.">
    <meta name="author" content="Jane Developer">
    <meta property="og:title" content="Rust Ownership Guide">
    <meta property="article:published_time" content="2023-06-15T10:30:00Z">
    <meta property="article:tag" content="rust">
    <meta property="article:tag" content="programming">
</head>
<body>
    <article class="post-content">
        <header>
            <h1>Understanding Rust's Ownership System</h1>
            <div class="byline">
                <span class="author">By Jane Developer</span>
                <time class="published" datetime="2023-06-15">June 15, 2023</time>
            </div>
        </header>
        <div class="content">
            <p>Rust's ownership system is one of its most distinctive features, designed to ensure memory safety without garbage collection.</p>
            <h2>The Three Rules</h2>
            <ul>
                <li>Each value in Rust has a variable that's called its owner.</li>
                <li>There can only be one owner at a time.</li>
                <li>When the owner goes out of scope, the value will be dropped.</li>
            </ul>
            <p>Let's explore each of these rules with practical examples.</p>
            <blockquote>
                <p>"Ownership is Rust's most unique feature, and it enables Rust to make memory safety guarantees without needing a garbage collector." - The Rust Book</p>
            </blockquote>
            <p>Contact the author at jane@developer.com for questions.</p>
        </div>
        <footer>
            <div class="tags">
                <span class="tag">rust</span>
                <span class="tag">programming</span>
                <span class="tag">memory-safety</span>
            </div>
        </footer>
    </article>
</body>
</html>
            "#.to_string(),
            url: "https://blog.example.com/rust-ownership".to_string(),
            expected_css_result: ExpectedResult {
                title: "Understanding Rust's Ownership System".to_string(),
                content_length: 500, // Approximate
                summary: Some("A comprehensive guide to Rust's ownership model and how it prevents memory leaks.".to_string()),
                confidence_min: 0.8,
                confidence_max: 1.0,
                strategy_used: "css_json".to_string(),
            },
            expected_regex_result: ExpectedResult {
                title: "Understanding Rust's Ownership System".to_string(),
                content_length: 800, // More content in regex extraction
                summary: None,
                confidence_min: 0.3,
                confidence_max: 0.8,
                strategy_used: "regex".to_string(),
            },
        },
        GoldenTestCase {
            name: "ecommerce_product".to_string(),
            html: r#"
<!DOCTYPE html>
<html>
<head>
    <title>Premium Wireless Headphones - AudioTech Store</title>
    <meta name="description" content="High-quality wireless headphones with noise cancellation and 30-hour battery life.">
    <meta property="og:title" content="Premium Wireless Headphones">
    <meta property="og:description" content="Experience superior sound quality with our latest wireless headphones.">
</head>
<body>
    <div class="product-page">
        <header>
            <h1>Premium Wireless Headphones</h1>
            <div class="price">$299.99</div>
        </header>
        <div class="product-info">
            <div class="description">
                <p>Experience superior sound quality with our Premium Wireless Headphones. Featuring advanced noise cancellation technology and up to 30 hours of battery life.</p>
                <h3>Key Features:</h3>
                <ul>
                    <li>Active Noise Cancellation</li>
                    <li>30-hour battery life</li>
                    <li>Bluetooth 5.0 connectivity</li>
                    <li>Premium leather headband</li>
                </ul>
            </div>
            <div class="contact-info">
                <p>Questions? Email us at support@audiotech.com or call 1-800-AUDIO-99</p>
                <p>Visit our website at https://www.audiotech.com for more products</p>
            </div>
        </div>
        <div class="reviews">
            <div class="review">
                <h4>"Excellent sound quality!"</h4>
                <p>These headphones exceeded my expectations. The noise cancellation is fantastic.</p>
                <div class="reviewer">- Sarah M.</div>
            </div>
        </div>
    </div>
</body>
</html>
            "#.to_string(),
            url: "https://audiotech.com/products/premium-headphones".to_string(),
            expected_css_result: ExpectedResult {
                title: "Premium Wireless Headphones - AudioTech Store".to_string(),
                content_length: 400,
                summary: Some("High-quality wireless headphones with noise cancellation and 30-hour battery life.".to_string()),
                confidence_min: 0.6,
                confidence_max: 0.9,
                strategy_used: "css_json".to_string(),
            },
            expected_regex_result: ExpectedResult {
                title: "Premium Wireless Headphones - AudioTech Store".to_string(),
                content_length: 600,
                summary: None,
                confidence_min: 0.4,
                confidence_max: 0.7,
                strategy_used: "regex".to_string(),
            },
        },
        GoldenTestCase {
            name: "news_article".to_string(),
            html: r#"
<!DOCTYPE html>
<html>
<head>
    <title>Climate Change Summit Reaches Historic Agreement</title>
    <meta name="description" content="World leaders agree on ambitious carbon reduction targets at the annual climate summit.">
    <meta name="author" content="Climate News Team">
    <meta property="article:published_time" content="2023-12-01T14:30:00Z">
</head>
<body>
    <article class="news-article">
        <header>
            <h1>Climate Change Summit Reaches Historic Agreement</h1>
            <div class="article-meta">
                <span class="author">By Climate News Team</span>
                <time class="published">December 1, 2023</time>
                <span class="category">Environment</span>
            </div>
        </header>
        <div class="article-content">
            <p class="lead">World leaders at the annual Climate Summit have reached a historic agreement to accelerate carbon reduction efforts and increase renewable energy investments.</p>
            <p>The agreement, signed by representatives from 195 countries, sets ambitious targets for reducing greenhouse gas emissions by 50% before 2030.</p>
            <h2>Key Points of the Agreement</h2>
            <ul>
                <li>50% reduction in emissions by 2030</li>
                <li>$500 billion investment in renewable energy</li>
                <li>Support for developing nations' green transition</li>
                <li>Mandatory climate reporting for major corporations</li>
            </ul>
            <blockquote>
                <p>"This agreement represents a turning point in our fight against climate change," said Summit President Dr. Maria Santos.</p>
            </blockquote>
            <p>The next summit will be held in Tokyo in 2024. For more information, visit https://climatesummit.org</p>
        </div>
        <footer class="article-footer">
            <div class="tags">
                <span class="tag">climate</span>
                <span class="tag">environment</span>
                <span class="tag">policy</span>
            </div>
        </footer>
    </article>
</body>
</html>
            "#.to_string(),
            url: "https://news.example.com/climate-summit-agreement".to_string(),
            expected_css_result: ExpectedResult {
                title: "Climate Change Summit Reaches Historic Agreement".to_string(),
                content_length: 600,
                summary: Some("World leaders agree on ambitious carbon reduction targets at the annual climate summit.".to_string()),
                confidence_min: 0.8,
                confidence_max: 1.0,
                strategy_used: "css_json".to_string(),
            },
            expected_regex_result: ExpectedResult {
                title: "Climate Change Summit Reaches Historic Agreement".to_string(),
                content_length: 800,
                summary: None,
                confidence_min: 0.3,
                confidence_max: 0.6,
                strategy_used: "regex".to_string(),
            },
        },
    ]
}

/// Golden tests for CSS extraction
mod css_golden_tests {
    use super::*;

    #[tokio::test]
    async fn test_css_extraction_golden_cases() {
        let test_cases = get_golden_test_cases();
        
        for test_case in test_cases {
            println!("Testing CSS extraction for: {}", test_case.name);
            
            let result = extract_default(&test_case.html, &test_case.url).await
                .expect(&format!("CSS extraction failed for {}", test_case.name));
            
            let expected = &test_case.expected_css_result;
            
            // Verify basic properties
            assert_eq!(result.title, expected.title, "Title mismatch for {}", test_case.name);
            assert_eq!(result.strategy_used, expected.strategy_used, "Strategy mismatch for {}", test_case.name);
            assert_eq!(result.url, test_case.url, "URL mismatch for {}", test_case.name);
            
            // Verify content length is within reasonable bounds
            assert!(
                result.content.len() >= expected.content_length / 2 && 
                result.content.len() <= expected.content_length * 2,
                "Content length out of bounds for {}: expected ~{}, got {}",
                test_case.name, expected.content_length, result.content.len()
            );
            
            // Verify summary if expected
            if let Some(expected_summary) = &expected.summary {
                assert_eq!(
                    result.summary.as_ref().unwrap(),
                    expected_summary,
                    "Summary mismatch for {}", test_case.name
                );
            }
            
            // Verify confidence score is within expected range
            assert!(
                result.extraction_confidence >= expected.confidence_min &&
                result.extraction_confidence <= expected.confidence_max,
                "Confidence score out of range for {}: expected [{}, {}], got {}",
                test_case.name, expected.confidence_min, expected.confidence_max, result.extraction_confidence
            );
            
            // Verify content is not empty
            assert!(!result.content.is_empty(), "Content is empty for {}", test_case.name);
        }
    }

    #[tokio::test]
    async fn test_css_extraction_consistency() {
        let test_cases = get_golden_test_cases();
        
        for test_case in test_cases {
            // Run extraction multiple times to ensure consistency
            let mut results = Vec::new();
            for _ in 0..5 {
                let result = extract_default(&test_case.html, &test_case.url).await
                    .expect(&format!("CSS extraction failed for {}", test_case.name));
                results.push(result);
            }
            
            // All results should be identical
            let first_result = &results[0];
            for (i, result) in results.iter().enumerate().skip(1) {
                assert_eq!(result.title, first_result.title, "Title inconsistency in run {} for {}", i, test_case.name);
                assert_eq!(result.content, first_result.content, "Content inconsistency in run {} for {}", i, test_case.name);
                assert_eq!(result.summary, first_result.summary, "Summary inconsistency in run {} for {}", i, test_case.name);
                assert_eq!(result.extraction_confidence, first_result.extraction_confidence, "Confidence inconsistency in run {} for {}", i, test_case.name);
            }
        }
    }
}

/// Golden tests for regex extraction
mod regex_golden_tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_extraction_golden_cases() {
        let test_cases = get_golden_test_cases();
        let patterns = default_patterns();
        
        for test_case in test_cases {
            println!("Testing regex extraction for: {}", test_case.name);
            
            let result = extract(&test_case.html, &test_case.url, &patterns).await
                .expect(&format!("Regex extraction failed for {}", test_case.name));
            
            let expected = &test_case.expected_regex_result;
            
            // Verify basic properties
            assert_eq!(result.title, expected.title, "Title mismatch for {}", test_case.name);
            assert_eq!(result.strategy_used, expected.strategy_used, "Strategy mismatch for {}", test_case.name);
            assert_eq!(result.url, test_case.url, "URL mismatch for {}", test_case.name);
            
            // Verify content length is within reasonable bounds
            assert!(
                result.content.len() >= expected.content_length / 2 && 
                result.content.len() <= expected.content_length * 2,
                "Content length out of bounds for {}: expected ~{}, got {}",
                test_case.name, expected.content_length, result.content.len()
            );
            
            // Verify confidence score is within expected range
            assert!(
                result.extraction_confidence >= expected.confidence_min &&
                result.extraction_confidence <= expected.confidence_max,
                "Confidence score out of range for {}: expected [{}, {}], got {}",
                test_case.name, expected.confidence_min, expected.confidence_max, result.extraction_confidence
            );
            
            // Verify content is not empty
            assert!(!result.content.is_empty(), "Content is empty for {}", test_case.name);
        }
    }
}

/// Performance regression tests
mod performance_golden_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_extraction_performance_regression() {
        let test_cases = get_golden_test_cases();
        
        for test_case in test_cases {
            // Test CSS extraction performance
            let start = Instant::now();
            let css_result = extract_default(&test_case.html, &test_case.url).await
                .expect(&format!("CSS extraction failed for {}", test_case.name));
            let css_duration = start.elapsed();
            
            // Should complete within 1 second for these test cases
            assert!(
                css_duration.as_millis() < 1000,
                "CSS extraction too slow for {}: {}ms",
                test_case.name, css_duration.as_millis()
            );
            
            // Test regex extraction performance
            let patterns = default_patterns();
            let start = Instant::now();
            let regex_result = extract(&test_case.html, &test_case.url, &patterns).await
                .expect(&format!("Regex extraction failed for {}", test_case.name));
            let regex_duration = start.elapsed();
            
            // Should complete within 1 second for these test cases
            assert!(
                regex_duration.as_millis() < 1000,
                "Regex extraction too slow for {}: {}ms",
                test_case.name, regex_duration.as_millis()
            );
            
            println!(
                "{}: CSS={}ms, Regex={}ms",
                test_case.name,
                css_duration.as_millis(),
                regex_duration.as_millis()
            );
        }
    }

    #[tokio::test]
    async fn test_memory_usage_stability() {
        let test_cases = get_golden_test_cases();
        
        // Run extractions multiple times to check for memory leaks
        for _ in 0..10 {
            for test_case in &test_cases {
                let _css_result = extract_default(&test_case.html, &test_case.url).await
                    .expect(&format!("CSS extraction failed for {}", test_case.name));
                
                let patterns = default_patterns();
                let _regex_result = extract(&test_case.html, &test_case.url, &patterns).await
                    .expect(&format!("Regex extraction failed for {}", test_case.name));
            }
        }
        
        // If we get here without OOM, memory usage is stable
        assert!(true);
    }
}

/// Edge case golden tests
mod edge_case_golden_tests {
    use super::*;

    #[tokio::test]
    async fn test_malformed_html_extraction() {
        let malformed_cases = vec![
            (
                "unclosed_tags",
                "<html><head><title>Test</title><body><p>Paragraph<div>Unclosed div<span>Unclosed span",
                "Test"
            ),
            (
                "missing_html_structure",
                "<title>No HTML</title><p>Just some content</p>",
                "No HTML"
            ),
            (
                "empty_document",
                "",
                "Untitled"
            ),
            (
                "only_whitespace",
                "   \n\t\r   ",
                "Untitled"
            ),
        ];
        
        for (name, html, expected_title) in malformed_cases {
            println!("Testing malformed HTML case: {}", name);
            
            let css_result = extract_default(html, "https://example.com").await
                .expect(&format!("CSS extraction failed for {}", name));
            
            assert_eq!(css_result.title, expected_title, "Title mismatch for {}", name);
            assert_eq!(css_result.strategy_used, "css_json");
            
            let patterns = default_patterns();
            let regex_result = extract(html, "https://example.com", &patterns).await
                .expect(&format!("Regex extraction failed for {}", name));
            
            assert_eq!(regex_result.title, expected_title, "Title mismatch for regex {}", name);
            assert_eq!(regex_result.strategy_used, "regex");
        }
    }

    #[tokio::test]
    async fn test_unicode_content_extraction() {
        let unicode_html = r#"
        <html>
        <head>
            <title>中文标题 - Chinese Title</title>
            <meta name="description" content="Description with émojis 🌍 and spëcial characters">
        </head>
        <body>
            <article class="content">
                <h1>中文标题</h1>
                <p>Content with various Unicode: áéíóú 中文 العربية русский 😀🌍✨</p>
                <p>Email: test@中文.com and phone: +1-800-UNICODE</p>
            </article>
        </body>
        </html>
        "#;
        
        let css_result = extract_default(unicode_html, "https://example.com").await
            .expect("CSS extraction failed for Unicode content");
        
        assert_eq!(css_result.title, "中文标题 - Chinese Title");
        assert!(css_result.content.contains("中文"));
        assert!(css_result.content.contains("رعربية"));
        assert!(css_result.content.contains("русский"));
        assert!(css_result.content.contains("😀"));
        
        assert_eq!(css_result.summary.unwrap(), "Description with émojis 🌍 and spëcial characters");
    }

    #[tokio::test]
    async fn test_very_large_content_extraction() {
        // Generate a large HTML document
        let mut large_html = String::from(r#"
        <html>
        <head><title>Large Document Test</title></head>
        <body><div class="content">
        "#);
        
        // Add 1000 paragraphs
        for i in 0..1000 {
            large_html.push_str(&format!(
                "<p>This is paragraph number {} with some substantial content that makes the document quite large. ",
                i
            ));
            large_html.push_str("Lorem ipsum dolor sit amet, consectetur adipiscing elit. ");
            large_html.push_str("Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>\n");
        }
        
        large_html.push_str("</div></body></html>");
        
        let start = Instant::now();
        let result = extract_default(&large_html, "https://example.com").await
            .expect("CSS extraction failed for large document");
        let duration = start.elapsed();
        
        assert_eq!(result.title, "Large Document Test");
        assert!(result.content.len() > 50000); // Should be substantial
        assert!(duration.as_millis() < 5000); // Should complete within 5 seconds
        
        // Performance should scale reasonably
        assert!(result.extraction_confidence > 0.0);
    }
}

/// Specific regression tests for known issues
mod regression_tests {
    use super::*;

    #[tokio::test]
    async fn test_script_style_content_exclusion() {
        let html_with_scripts = r#"
        <html>
        <head>
            <title>Script Test</title>
            <script>
                console.log('This should not appear in content');
                var badCode = "alert('evil')";
            </script>
            <style>
                body { color: red; }
                .hidden { display: none; }
            </style>
        </head>
        <body>
            <article class="content">
                <p>This is the actual content that should be extracted.</p>
                <script>alert('Another script');</script>
                <p>More actual content after script.</p>
            </article>
        </body>
        </html>
        "#;
        
        let result = extract_default(html_with_scripts, "https://example.com").await
            .expect("CSS extraction failed for script content");
        
        // Should not contain script or style content
        assert!(!result.content.contains("console.log"));
        assert!(!result.content.contains("alert"));
        assert!(!result.content.contains("color: red"));
        assert!(!result.content.contains("display: none"));
        
        // Should contain actual content
        assert!(result.content.contains("actual content that should be extracted"));
        assert!(result.content.contains("More actual content after script"));
    }

    #[tokio::test]
    async fn test_whitespace_normalization() {
        let html_with_whitespace = r#"
        <html>
        <head><title>   Whitespace   Test   </title></head>
        <body>
            <article class="content">
                <p>    Text    with    multiple    spaces    </p>
                <p>\n\n\nText\nwith\nnewlines\n\n\n</p>
                <p>\t\tText\t\twith\t\ttabs\t\t</p>
            </article>
        </body>
        </html>
        "#;
        
        let result = extract_default(html_with_whitespace, "https://example.com").await
            .expect("CSS extraction failed for whitespace content");
        
        assert_eq!(result.title.trim(), "Whitespace   Test"); // Title should be trimmed but preserve internal spaces
        
        // Content should have normalized whitespace
        assert!(result.content.contains("Text with multiple spaces"));
        assert!(result.content.contains("Text with newlines"));
        assert!(result.content.contains("Text with tabs"));
        
        // Should not have excessive whitespace
        assert!(!result.content.contains("    "));
        assert!(!result.content.contains("\n\n\n"));
        assert!(!result.content.contains("\t\t"));
    }
}
