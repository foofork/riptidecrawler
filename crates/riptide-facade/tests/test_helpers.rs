//! Test helpers and utilities for facade integration tests.

use riptide_facade::{Riptide, RiptideConfig, ScraperFacade};

/// Helper function to create a test scraper with default config
pub async fn create_test_scraper() -> Result<ScraperFacade, Box<dyn std::error::Error>> {
    let _config = RiptideConfig::default();
    let scraper = Riptide::builder().build_scraper().await?;
    Ok(scraper)
}

/// Helper to validate URL format
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Helper to measure execution time
pub async fn time_async<F, T>(f: F) -> (T, std::time::Duration)
where
    F: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = f.await;
    (result, start.elapsed())
}

/// HTML test fixtures
pub mod fixtures {
    pub const SIMPLE_HTML: &str = r#"
        <!DOCTYPE html>
        <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Hello World</h1>
                <p>This is a test page.</p>
            </body>
        </html>
    "#;

    pub const ARTICLE_HTML: &str = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Test Article</title>
                <meta name="description" content="Test description">
                <meta name="author" content="Test Author">
            </head>
            <body>
                <article>
                    <h1>Article Title</h1>
                    <p>Article content here.</p>
                    <p>More content.</p>
                </article>
            </body>
        </html>
    "#;

    pub const LINKS_HTML: &str = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <a href="https://example.com/page1">Link 1</a>
                <a href="https://example.com/page2">Link 2</a>
                <a href="/relative-link">Relative Link</a>
            </body>
        </html>
    "#;

    pub const IMAGES_HTML: &str = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <img src="https://example.com/image1.jpg" alt="Image 1">
                <img src="/images/image2.png" alt="Image 2">
            </body>
        </html>
    "#;

    pub const TABLE_HTML: &str = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <table>
                    <tr><th>Name</th><th>Value</th></tr>
                    <tr><td>Item 1</td><td>100</td></tr>
                    <tr><td>Item 2</td><td>200</td></tr>
                </table>
            </body>
        </html>
    "#;

    pub const JSON_LD_HTML: &str = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <script type="application/ld+json">
                {
                    "@context": "https://schema.org",
                    "@type": "Article",
                    "headline": "Test Headline",
                    "author": "Test Author"
                }
                </script>
            </head>
            <body>
                <h1>Test Page</h1>
            </body>
        </html>
    "#;

    pub const MALFORMED_HTML: &str = r#"
        <html>
            <body>
                <p>Unclosed paragraph
                <div>Unclosed div
            </body>
    "#;
}

/// Assertion helpers
pub fn assert_contains_text(html: &str, text: &str) {
    assert!(
        html.contains(text),
        "HTML does not contain expected text: {}",
        text
    );
}

pub fn assert_valid_html(html: &str) {
    assert!(!html.is_empty(), "HTML is empty");
    assert!(
        html.contains("<html") || html.contains("<!DOCTYPE"),
        "HTML does not contain html tag or doctype"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://test.com/path"));
        assert!(!is_valid_url("not a url"));
        assert!(!is_valid_url(""));
    }

    #[test]
    #[allow(clippy::absurd_extreme_comparisons)]
    fn test_fixtures_are_valid() {
        assert!(!fixtures::SIMPLE_HTML.is_empty());
        assert!(!fixtures::ARTICLE_HTML.is_empty());
        assert!(!fixtures::LINKS_HTML.is_empty());
        assert!(!fixtures::IMAGES_HTML.is_empty());
    }

    #[test]
    fn test_assert_contains_text() {
        let html = "<html><body>Hello World</body></html>";
        assert_contains_text(html, "Hello");
        assert_contains_text(html, "World");
    }

    #[test]
    fn test_assert_valid_html() {
        assert_valid_html("<!DOCTYPE html><html></html>");
        assert_valid_html("<html><body></body></html>");
    }
}
