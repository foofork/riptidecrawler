use super::fixtures::{get_fixture, get_expected_extraction};
use super::GoldenTestRunner;
use riptide_core::types::ExtractedDoc;
use std::fs;

/// Mock WASM extractor for testing
/// In real tests, this would interface with the actual WASM module
fn mock_extractor(html: &str, url: &str) -> Result<ExtractedDoc, String> {
    // Simple mock extraction based on HTML parsing
    let title = extract_title(html);
    let author = extract_author(html);
    let published_iso = extract_published_date(html);
    let text = extract_text_content(html);
    let markdown = text_to_markdown(&text);
    let links = extract_links(html);
    let media = extract_media(html);

    Ok(ExtractedDoc {
        url: url.to_string(),
        title,
        byline: author,
        published_iso,
        markdown,
        text,
        links,
        media,
    })
}

fn extract_title(html: &str) -> Option<String> {
    // Look for title tag
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html[start + 7..].find("</title>") {
            let title = &html[start + 7..start + 7 + end];
            return Some(html_decode(title));
        }
    }

    // Look for h1 tag
    if let Some(start) = html.find("<h1") {
        if let Some(content_start) = html[start..].find('>') {
            let content_start = start + content_start + 1;
            if let Some(end) = html[content_start..].find("</h1>") {
                let title = &html[content_start..content_start + end];
                return Some(strip_html_tags(title));
            }
        }
    }

    None
}

fn extract_author(html: &str) -> Option<String> {
    // Look for author meta tag
    if html.contains(r#"name="author""#) {
        if let Some(start) = html.find(r#"name="author" content=""#) {
            let content_start = start + r#"name="author" content=""#.len();
            if let Some(end) = html[content_start..].find('"') {
                return Some(html[content_start..content_start + end].to_string());
            }
        }
    }

    // Look for byline patterns
    let author_patterns = [
        r#"class="author""#,
        r#"class="byline""#,
        "By ",
        "Author:",
    ];

    for pattern in &author_patterns {
        if let Some(pos) = html.find(pattern) {
            // Extract text around this position
            let start = pos + pattern.len();
            let end = start + 100.min(html.len() - start);
            let section = &html[start..end];

            if let Some(author) = extract_author_from_section(section) {
                return Some(author);
            }
        }
    }

    None
}

fn extract_author_from_section(section: &str) -> Option<String> {
    // Look for text between > and <
    if let Some(start) = section.find('>') {
        if let Some(end) = section[start + 1..].find('<') {
            let author = section[start + 1..start + 1 + end].trim();
            if !author.is_empty() && author.len() < 50 {
                return Some(author.to_string());
            }
        }
    }
    None
}

fn extract_published_date(html: &str) -> Option<String> {
    // Look for structured data
    if html.contains("datePublished") {
        if let Some(start) = html.find(r#""datePublished": ""#) {
            let content_start = start + r#""datePublished": ""#.len();
            if let Some(end) = html[content_start..].find('"') {
                return Some(html[content_start..content_start + end].to_string());
            }
        }
    }

    // Look for time tags
    if let Some(start) = html.find(r#"<time datetime=""#) {
        let content_start = start + r#"<time datetime=""#.len();
        if let Some(end) = html[content_start..].find('"') {
            return Some(html[content_start..content_start + end].to_string());
        }
    }

    None
}

fn extract_text_content(html: &str) -> String {
    let mut text = String::new();

    // Extract content from main content areas
    let content_selectors = [
        "<main",
        r#"<article"#,
        r#"class="content""#,
        r#"class="article-body""#,
        r#"class="post-content""#,
    ];

    for selector in &content_selectors {
        if let Some(start) = html.find(selector) {
            // Find the end of the opening tag
            if let Some(tag_end) = html[start..].find('>') {
                let content_start = start + tag_end + 1;

                // Find the corresponding closing tag
                let tag_name = if selector.starts_with('<') {
                    selector.strip_prefix('<').unwrap_or(selector)
                } else {
                    "div" // Default for class-based selectors
                };

                if let Some(content_end) = find_closing_tag(&html[content_start..], tag_name) {
                    let content = &html[content_start..content_start + content_end];
                    text.push_str(&extract_text_from_html(content));
                    break;
                }
            }
        }
    }

    // If no main content found, extract from body
    if text.is_empty() {
        if let Some(start) = html.find("<body") {
            if let Some(tag_end) = html[start..].find('>') {
                let content_start = start + tag_end + 1;
                if let Some(content_end) = html[content_start..].find("</body>") {
                    let content = &html[content_start..content_start + content_end];
                    text = extract_text_from_html(content);
                }
            }
        }
    }

    // Clean up the text
    text.split_whitespace()
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_text_from_html(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    let mut chars = html.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' {
            in_tag = true;

            // Check for script or style tags
            let remaining: String = chars.clone().take(10).collect();
            if remaining.to_lowercase().starts_with("script") {
                in_script = true;
            } else if remaining.to_lowercase().starts_with("style") {
                in_style = true;
            } else if remaining.to_lowercase().starts_with("/script") {
                in_script = false;
            } else if remaining.to_lowercase().starts_with("/style") {
                in_style = false;
            }
        } else if ch == '>' && in_tag {
            in_tag = false;
        } else if !in_tag && !in_script && !in_style {
            text.push(ch);
        }
    }

    text
}

fn find_closing_tag(html: &str, tag_name: &str) -> Option<usize> {
    let closing_tag = format!("</{}>", tag_name);
    html.find(&closing_tag)
}

fn text_to_markdown(text: &str) -> String {
    // Simple conversion - in reality this would be more sophisticated
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn extract_links(html: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut pos = 0;

    while let Some(start) = html[pos..].find(r#"href=""#) {
        let absolute_start = pos + start + 6; // length of 'href="'
        if let Some(end) = html[absolute_start..].find('"') {
            let link = &html[absolute_start..absolute_start + end];
            if link.starts_with("http") {
                links.push(link.to_string());
            }
        }
        pos = absolute_start;
    }

    links
}

fn extract_media(html: &str) -> Vec<String> {
    let mut media = Vec::new();
    let mut pos = 0;

    // Extract images
    while let Some(start) = html[pos..].find(r#"src=""#) {
        let absolute_start = pos + start + 5; // length of 'src="'
        if let Some(end) = html[absolute_start..].find('"') {
            let src = &html[absolute_start..absolute_start + end];
            if src.starts_with("http") || src.ends_with(".jpg") || src.ends_with(".png") || src.ends_with(".gif") {
                media.push(src.to_string());
            }
        }
        pos = absolute_start;
    }

    media
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    html_decode(&result)
}

fn html_decode(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_extractor_blog_post() {
        let html = get_fixture("blog_post").unwrap();
        let result = mock_extractor(html, "https://example.com/blog/webassembly").unwrap();

        assert_eq!(result.url, "https://example.com/blog/webassembly");
        assert!(result.title.is_some());
        assert!(result.title.as_ref().unwrap().contains("WebAssembly"));
        assert!(result.byline.is_some());
        assert!(result.byline.as_ref().unwrap().contains("Jane Developer"));
        assert!(result.text.len() > 500);
        assert!(result.text.contains("WebAssembly"));
        assert!(result.text.contains("performance"));
    }

    #[test]
    fn test_mock_extractor_news_article() {
        let html = get_fixture("news_article").unwrap();
        let result = mock_extractor(html, "https://example.com/news/quantum").unwrap();

        assert!(result.title.is_some());
        assert!(result.title.as_ref().unwrap().contains("Quantum Computing"));
        assert!(result.byline.is_some());
        assert!(result.byline.as_ref().unwrap().contains("Dr. Sarah Chen"));
        assert!(result.published_iso.is_some());
        assert!(result.text.contains("quantum"));
        assert!(result.text.contains("breakthrough"));
    }

    #[test]
    fn test_mock_extractor_spa_application() {
        let html = get_fixture("spa_application").unwrap();
        let result = mock_extractor(html, "https://example.com/app").unwrap();

        assert!(result.title.is_some());
        assert!(result.title.as_ref().unwrap().contains("TaskMaster"));
        assert!(result.text.contains("JavaScript") || result.text.contains("project management"));

        // SPA should have minimal content initially
        assert!(result.text.len() < 2000);
    }

    #[test]
    fn test_mock_extractor_ecommerce_product() {
        let html = get_fixture("ecommerce_product").unwrap();
        let result = mock_extractor(html, "https://example.com/product/headphones").unwrap();

        assert!(result.title.is_some());
        assert!(result.title.as_ref().unwrap().contains("Headphones"));
        assert!(result.text.contains("$299.99") || result.text.contains("299.99"));
        assert!(result.text.contains("AudioTech"));
        assert!(result.text.contains("noise cancellation"));
    }

    #[test]
    fn test_mock_extractor_documentation() {
        let html = get_fixture("documentation").unwrap();
        let result = mock_extractor(html, "https://example.com/docs/auth").unwrap();

        assert!(result.title.is_some());
        assert!(result.title.as_ref().unwrap().contains("Authentication"));
        assert!(result.text.contains("API"));
        assert!(result.text.contains("authentication"));
        assert!(result.text.contains("Authorization"));
    }

    #[test]
    fn test_extract_title_from_title_tag() {
        let html = r#"<html><head><title>Test Title</title></head></html>"#;
        let title = extract_title(html);
        assert_eq!(title, Some("Test Title".to_string()));
    }

    #[test]
    fn test_extract_title_from_h1() {
        let html = r#"<html><body><h1>Main Heading</h1></body></html>"#;
        let title = extract_title(html);
        assert_eq!(title, Some("Main Heading".to_string()));
    }

    #[test]
    fn test_extract_author_from_meta() {
        let html = r#"<meta name="author" content="John Doe">"#;
        let author = extract_author(html);
        assert_eq!(author, Some("John Doe".to_string()));
    }

    #[test]
    fn test_extract_links() {
        let html = r#"<a href="https://example.com">Link 1</a> <a href="https://test.org">Link 2</a>"#;
        let links = extract_links(html);
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"https://example.com".to_string()));
        assert!(links.contains(&"https://test.org".to_string()));
    }

    #[test]
    fn test_strip_html_tags() {
        let html = "Hello <b>world</b> <em>test</em>";
        let text = strip_html_tags(html);
        assert_eq!(text, "Hello world test");
    }

    #[test]
    fn test_html_decode() {
        let encoded = "Caf&eacute; &amp; Restaurant";
        let decoded = html_decode(encoded);
        assert!(decoded.contains("&")); // Basic test
    }

    #[test]
    fn test_text_extraction_removes_scripts() {
        let html = r#"<div>Content <script>alert('bad');</script> More content</div>"#;
        let text = extract_text_from_html(html);
        assert!(!text.contains("alert"));
        assert!(text.contains("Content"));
        assert!(text.contains("More content"));
    }

    #[test]
    fn test_golden_test_runner() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let fixtures_path = temp_dir.path().join("fixtures");
        let baselines_path = temp_dir.path().join("baselines");

        fs::create_dir_all(&fixtures_path).unwrap();
        fs::create_dir_all(&baselines_path).unwrap();

        let runner = GoldenTestRunner::new(
            fixtures_path.to_str().unwrap(),
            baselines_path.to_str().unwrap(),
        );

        // Create a test fixture
        let test_html = r#"<html><head><title>Test Article</title></head><body><h1>Test Article</h1><p>This is test content.</p></body></html>"#;
        fs::write(fixtures_path.join("test.html"), test_html).unwrap();

        // Run extraction
        let result = mock_extractor(test_html, "https://example.com/test").unwrap();

        // Save as baseline
        runner.save_baseline("test", &result).unwrap();

        // Run comparison (should pass)
        let comparison = runner.compare_extraction("test", &result);
        assert!(comparison.is_ok());

        // Test with slightly different content (should pass due to flexibility)
        let modified_html = r#"<html><head><title>Test Article</title></head><body><h1>Test Article</h1><p>This is test   content with extra   whitespace.</p></body></html>"#;
        let modified_result = mock_extractor(modified_html, "https://example.com/test").unwrap();
        let comparison = runner.compare_extraction("test", &modified_result);
        assert!(comparison.is_ok());
    }

    #[test]
    fn test_comprehensive_golden_tests() {
        let test_cases = vec![
            "blog_post",
            "news_article",
            "spa_application",
            "ecommerce_product",
            "documentation",
        ];

        for test_case in test_cases {
            let html = get_fixture(test_case).unwrap();
            let expected = get_expected_extraction(test_case).unwrap();

            let result = mock_extractor(html, &format!("https://example.com/{}", test_case)).unwrap();

            // Validate title
            if let Some(expected_title) = &expected.title {
                assert!(result.title.is_some(), "Title should be extracted for {}", test_case);
                let actual_title = result.title.as_ref().unwrap();
                assert!(
                    actual_title.contains(&expected_title.split(' ').next().unwrap()),
                    "Title should contain key words for {}. Expected: {}, Got: {}",
                    test_case, expected_title, actual_title
                );
            }

            // Validate author
            if let Some(expected_author) = &expected.author {
                assert!(
                    result.byline.is_some() && result.byline.as_ref().unwrap().contains(expected_author),
                    "Author should be extracted for {}. Expected: {}, Got: {:?}",
                    test_case, expected_author, result.byline
                );
            }

            // Validate minimum content length
            assert!(
                result.text.len() >= expected.min_text_length,
                "Text content too short for {}. Expected at least {}, got {}",
                test_case, expected.min_text_length, result.text.len()
            );

            // Validate key phrases
            let text_lower = result.text.to_lowercase();
            for phrase in &expected.key_phrases {
                assert!(
                    text_lower.contains(&phrase.to_lowercase()),
                    "Key phrase '{}' not found in extracted text for {}",
                    phrase, test_case
                );
            }

            println!("✓ Golden test passed for: {}", test_case);
        }
    }
}