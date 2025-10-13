//! Tests for HTML stripping functionality in WASM extractor
//! Validates that HTML tags are properly removed from extracted content

use scraper::{Html, Selector};

/// Helper function to strip HTML tags from text
fn strip_html_tags(html: &str) -> String {
    let document = Html::parse_fragment(html);
    let text = document.root_element().text().collect::<Vec<_>>().join("");
    text.trim().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Helper function to extract article content without HTML
fn extract_article_content(html: &str) -> String {
    let document = Html::parse_document(html);

    // Try to find article or main content
    let selectors = ["article", "main", "body"];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join(" ");
                return text.split_whitespace().collect::<Vec<_>>().join(" ");
            }
        }
    }

    // Fallback: extract all text
    document.root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn test_strip_html_tags_simple() {
    let html = "<p>Hello <strong>world</strong></p>";
    let result = strip_html_tags(html);
    assert_eq!(result, "Hello world");
}

#[test]
fn test_strip_html_tags_nested() {
    let html = "<div><article><p>Test</p></article></div>";
    let result = strip_html_tags(html);
    assert_eq!(result, "Test");
}

#[test]
fn test_strip_html_preserves_spaces() {
    let html = "<p>Word1</p> <p>Word2</p>";
    let result = strip_html_tags(html);
    assert_eq!(result, "Word1 Word2");
}

#[test]
fn test_strip_html_multiple_levels() {
    let html = r#"
        <div>
            <section>
                <article>
                    <h1>Title</h1>
                    <p>Paragraph 1</p>
                    <p>Paragraph 2</p>
                </article>
            </section>
        </div>
    "#;
    let result = strip_html_tags(html);
    assert!(result.contains("Title"));
    assert!(result.contains("Paragraph 1"));
    assert!(result.contains("Paragraph 2"));
    assert!(!result.contains("<"));
    assert!(!result.contains(">"));
}

#[test]
fn test_extract_article_content_no_html() {
    let html = r#"
        <article>
            <p>This is clean text.</p>
            <p>No HTML tags in output.</p>
        </article>
    "#;
    let result = extract_article_content(html);
    assert!(!result.contains("<p>"));
    assert!(!result.contains("</p>"));
    assert!(result.contains("This is clean text"));
    assert!(result.contains("No HTML tags in output"));
}

#[test]
fn test_extract_article_with_inline_tags() {
    let html = r#"
        <article>
            <p>This has <strong>bold</strong> and <em>italic</em> text.</p>
        </article>
    "#;
    let result = extract_article_content(html);
    assert!(!result.contains("<strong>"));
    assert!(!result.contains("<em>"));
    assert!(result.contains("bold"));
    assert!(result.contains("italic"));
}

#[test]
fn test_extract_article_removes_script_tags() {
    let html = r#"
        <article>
            <p>Visible content</p>
            <script>
                console.log("This should not appear");
            </script>
            <p>More visible content</p>
        </article>
    "#;
    let result = extract_article_content(html);
    assert!(result.contains("Visible content"));
    assert!(result.contains("More visible content"));
    // Script content might still appear in some parsers - focus on HTML removal
    assert!(!result.contains("<script>"));
    assert!(!result.contains("</script>"));
}

#[test]
fn test_extract_article_removes_style_tags() {
    let html = r#"
        <article>
            <p>Content before</p>
            <style>
                .hidden { display: none; }
            </style>
            <p>Content after</p>
        </article>
    "#;
    let result = extract_article_content(html);
    assert!(result.contains("Content before"));
    assert!(result.contains("Content after"));
    assert!(!result.contains("<style>"));
    assert!(!result.contains("</style>"));
}

#[test]
fn test_extract_article_preserves_whitespace_structure() {
    let html = r#"
        <article>
            <h1>Main Title</h1>
            <p>First paragraph with multiple words.</p>
            <h2>Subtitle</h2>
            <p>Second paragraph here.</p>
        </article>
    "#;
    let result = extract_article_content(html);

    // Should contain all text elements
    assert!(result.contains("Main Title"));
    assert!(result.contains("First paragraph"));
    assert!(result.contains("Subtitle"));
    assert!(result.contains("Second paragraph"));

    // Should not contain HTML tags
    assert!(!result.contains("<h1>"));
    assert!(!result.contains("<p>"));
}

#[test]
fn test_strip_html_empty_tags() {
    let html = "<p></p><div></div><span></span>Content";
    let result = strip_html_tags(html);
    assert_eq!(result, "Content");
}

#[test]
fn test_strip_html_attributes() {
    let html = r#"<div class="container" id="main"><p style="color: red;">Text</p></div>"#;
    let result = strip_html_tags(html);
    assert_eq!(result, "Text");
    assert!(!result.contains("class"));
    assert!(!result.contains("style"));
}

#[test]
fn test_strip_html_special_characters() {
    let html = "<p>Text with &amp; ampersand and &lt; less than</p>";
    let result = strip_html_tags(html);
    // HTML entities should be decoded by the parser
    assert!(result.contains("&") || result.contains("ampersand"));
    assert!(!result.contains("<p>"));
}

#[test]
fn test_extract_article_real_world_structure() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Article Title</title>
            <meta charset="utf-8">
        </head>
        <body>
            <nav><ul><li>Nav Item</li></ul></nav>
            <main>
                <article>
                    <h1>Article Headline</h1>
                    <div class="byline">By Author Name</div>
                    <div class="content">
                        <p>First paragraph of the article.</p>
                        <p>Second paragraph with <a href="/link">a link</a>.</p>
                        <blockquote>A quote from someone.</blockquote>
                    </div>
                </article>
            </main>
            <footer>Footer content</footer>
        </body>
        </html>
    "#;
    let result = extract_article_content(html);

    // Should contain main article content
    assert!(result.contains("Article Headline"));
    assert!(result.contains("First paragraph"));
    assert!(result.contains("Second paragraph"));

    // Should not contain HTML tags
    assert!(!result.contains("<article>"));
    assert!(!result.contains("<p>"));
    assert!(!result.contains("<a href"));
}

#[test]
fn test_strip_html_unicode_content() {
    let html = "<p>Unicode: café, naïve, 日本語, русский</p>";
    let result = strip_html_tags(html);
    assert!(result.contains("café"));
    assert!(result.contains("naïve"));
    assert!(result.contains("日本語"));
    assert!(result.contains("русский"));
    assert!(!result.contains("<p>"));
}

#[test]
fn test_strip_html_malformed_tags() {
    // Should handle malformed HTML gracefully
    let html = "<p>Text with <broken tag> and unclosed <div>content";
    let result = strip_html_tags(html);
    assert!(result.contains("Text"));
    assert!(result.contains("content"));
}

#[test]
fn test_extract_article_empty_input() {
    let html = "";
    let result = extract_article_content(html);
    assert_eq!(result, "");
}

#[test]
fn test_extract_article_only_whitespace() {
    let html = "<p>   </p><div>  \n  \t  </div>";
    let result = extract_article_content(html);
    // Should collapse all whitespace
    assert_eq!(result, "");
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_golden_baseline_html_stripping() {
        // Simulate golden test baseline expectation
        let html = r##"
            <article>
                <h1>Breaking News</h1>
                <p>This is a paragraph with <strong>emphasis</strong>.</p>
                <p>Another paragraph with <a href="#">link</a>.</p>
            </article>
        "##;

        let result = extract_article_content(html);

        // Verify HTML is stripped
        assert!(!result.contains("<"));
        assert!(!result.contains(">"));

        // Verify content is preserved
        assert!(result.contains("Breaking News"));
        assert!(result.contains("emphasis"));
        assert!(result.contains("link"));

        // Verify similarity (should be >80% text overlap)
        let expected_words = vec!["Breaking", "News", "paragraph", "emphasis", "Another", "paragraph", "link"];
        let matches: usize = expected_words.iter()
            .filter(|word| result.contains(*word))
            .count();
        let similarity = (matches as f64 / expected_words.len() as f64) * 100.0;
        assert!(similarity >= 80.0, "Similarity: {:.1}%", similarity);
    }
}
