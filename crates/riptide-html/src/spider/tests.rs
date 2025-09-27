//! Tests for DOM spider functionality

use super::*;
use crate::spider::traits::{DomSpider, DomSpiderConfig};
use crate::spider::dom_crawler::HtmlDomCrawler;
use url::Url;

#[tokio::test]
async fn test_link_extraction() {
    let html = r##"
        <html>
            <head><title>Test Page</title></head>
            <body>
                <a href="https://example.com/page1">Link 1</a>
                <a href="/relative/path">Relative Link</a>
                <a href="#fragment">Fragment Link</a>
                <a href="javascript:void(0)">JS Link</a>
                <img src="/image.jpg" alt="Image">
                <link rel="stylesheet" href="/style.css">
            </body>
        </html>
    "##;

    let base_url = Url::parse("https://example.com").unwrap();
    let crawler = HtmlDomCrawler::default();

    let links = crawler.extract_links(html, &base_url).await.unwrap();

    // Should extract valid links but filter out fragments and javascript
    assert!(links.len() >= 2);
    assert!(links.contains(&Url::parse("https://example.com/page1").unwrap()));
    assert!(links.contains(&Url::parse("https://example.com/relative/path").unwrap()));
}

#[tokio::test]
async fn test_form_extraction() {
    let html = r#"
        <html>
            <body>
                <form action="/search" method="GET">
                    <input type="text" name="q" placeholder="Search...">
                    <input type="submit" value="Search">
                </form>

                <form action="/login" method="POST">
                    <input type="email" name="email" required>
                    <input type="password" name="password" required>
                    <button type="submit">Login</button>
                </form>
            </body>
        </html>
    "#;

    let base_url = Url::parse("https://example.com").unwrap();
    let crawler = HtmlDomCrawler::default();

    let forms = crawler.extract_forms(html, &base_url).await.unwrap();

    assert_eq!(forms.len(), 2);

    // Check search form
    let search_form = &forms[0];
    assert_eq!(search_form.method, "GET");
    assert_eq!(search_form.action.as_ref().unwrap().path(), "/search");
    assert_eq!(search_form.fields.len(), 2);

    // Check login form
    let login_form = &forms[1];
    assert_eq!(login_form.method, "POST");
    assert_eq!(login_form.action.as_ref().unwrap().path(), "/login");
    assert!(login_form.fields.iter().any(|f| f.field_type == "email"));
    assert!(login_form.fields.iter().any(|f| f.field_type == "password"));
}

#[tokio::test]
async fn test_metadata_extraction() {
    let html = r#"
        <html>
            <head>
                <title>Test Page Title</title>
                <meta name="description" content="This is a test page description">
                <meta name="keywords" content="test, page, html">
                <meta name="author" content="Test Author">
                <meta property="og:title" content="OG Title">
                <meta property="og:description" content="OG Description">
                <meta name="twitter:card" content="summary">
                <link rel="canonical" href="https://example.com/canonical">
            </head>
            <body>
                <h1>Test Content</h1>
            </body>
        </html>
    "#;

    let crawler = HtmlDomCrawler::default();
    let metadata = crawler.extract_metadata(html).await.unwrap();

    assert_eq!(metadata.title, Some("Test Page Title".to_string()));
    assert_eq!(metadata.description, Some("This is a test page description".to_string()));
    assert_eq!(metadata.author, Some("Test Author".to_string()));
    assert_eq!(metadata.keywords, vec!["test", "page", "html"]);

    // Check Open Graph data
    assert_eq!(metadata.og_data.get("title"), Some(&"OG Title".to_string()));
    assert_eq!(metadata.og_data.get("description"), Some(&"OG Description".to_string()));

    // Check Twitter data
    assert_eq!(metadata.twitter_data.get("card"), Some(&"summary".to_string()));

    // Check canonical URL
    assert_eq!(metadata.canonical, Some(Url::parse("https://example.com/canonical").unwrap()));
}

#[tokio::test]
async fn test_text_content_extraction() {
    let html = r#"
        <html>
            <head>
                <title>Test Page</title>
                <script>console.log("test");</script>
                <style>body { color: red; }</style>
            </head>
            <body>
                <h1>Main Heading</h1>
                <p>This is a paragraph with <strong>bold text</strong>.</p>
                <div>
                    <p>Nested content</p>
                    <ul>
                        <li>List item 1</li>
                        <li>List item 2</li>
                    </ul>
                </div>
                <script>alert("This should be removed");</script>
            </body>
        </html>
    "#;

    let crawler = HtmlDomCrawler::default();
    let text_content = crawler.extract_text_content(html).await.unwrap().unwrap();

    // Should extract text but not script/style content
    assert!(text_content.contains("Main Heading"));
    assert!(text_content.contains("This is a paragraph"));
    assert!(text_content.contains("bold text"));
    assert!(text_content.contains("Nested content"));
    assert!(text_content.contains("List item 1"));

    // Should not contain script or style content
    assert!(!text_content.contains("console.log"));
    assert!(!text_content.contains("color: red"));
    assert!(!text_content.contains("alert"));
}

#[tokio::test]
async fn test_content_analysis() {
    let html = r#"
        <html>
            <head><title>Article Page</title></head>
            <body>
                <nav>
                    <a href="/home">Home</a>
                    <a href="/about">About</a>
                </nav>
                <article>
                    <h1>Article Title</h1>
                    <p>This is article content with <a href="/related">related link</a>.</p>
                    <p>More content here...</p>
                </article>
                <aside>
                    <a href="/sidebar1">Sidebar Link 1</a>
                    <a href="/sidebar2">Sidebar Link 2</a>
                </aside>
            </body>
        </html>
    "#;

    let base_url = Url::parse("https://example.com").unwrap();
    let crawler = HtmlDomCrawler::default();

    let analysis = crawler.analyze_content(html).await.unwrap();

    // Should detect as article content
    assert_eq!(analysis.content_type, crate::spider::traits::ContentType::Article);

    // Should have reasonable link density
    assert!(analysis.link_density > 0.0);

    // Should have quality score
    assert!(analysis.quality_score > 0.0 && analysis.quality_score <= 1.0);

    // Should count unique characters
    assert!(analysis.unique_text_chars > 0);
}

#[tokio::test]
async fn test_navigation_vs_content_links() {
    let html = r#"
        <html>
            <body>
                <nav class="navigation">
                    <a href="/home">Home</a>
                    <a href="/products">Products</a>
                    <a href="/contact">Contact</a>
                </nav>
                <main>
                    <h1>Product Page</h1>
                    <p>Check out our <a href="/product/1">featured product</a>.</p>
                    <p>Also see <a href="/product/2">related item</a>.</p>
                </main>
                <footer>
                    <a href="/privacy">Privacy</a>
                    <a href="/terms">Terms</a>
                </footer>
            </body>
        </html>
    "#;

    let base_url = Url::parse("https://example.com").unwrap();
    let link_extractor = crate::spider::link_extractor::HtmlLinkExtractor::default();

    let all_links = link_extractor.extract_links(html, &base_url).await.unwrap();
    let nav_links = link_extractor.extract_navigation_links(html, &base_url).await.unwrap();
    let content_links = link_extractor.extract_content_links(html, &base_url).await.unwrap();

    // Should have all links
    assert_eq!(all_links.len(), 7);

    // Should identify navigation links
    assert!(nav_links.len() >= 3); // nav + footer links

    // Content links should be fewer than total
    assert!(content_links.len() < all_links.len());

    // Content links should include product links
    assert!(content_links.iter().any(|url| url.path().contains("/product/")));
}

#[tokio::test]
async fn test_comprehensive_dom_crawling() {
    let html = r#"
        <html>
            <head>
                <title>Comprehensive Test</title>
                <meta name="description" content="Test description">
            </head>
            <body>
                <nav>
                    <a href="/nav1">Nav Link 1</a>
                </nav>
                <main>
                    <h1>Main Content</h1>
                    <p>Content with <a href="/content1">content link</a>.</p>
                </main>
                <form action="/test" method="POST">
                    <input type="text" name="field1">
                    <input type="submit" value="Submit">
                </form>
            </body>
        </html>
    "#;

    let base_url = Url::parse("https://example.com").unwrap();
    let crawler = HtmlDomCrawler::default();

    let result = crawler.crawl_dom(html, &base_url).await.unwrap();

    // Should extract all components
    assert!(!result.links.is_empty());
    assert!(!result.forms.is_empty());
    assert!(result.metadata.title.is_some());
    assert!(result.text_content.is_some());

    // Should have meaningful analysis
    assert!(result.analysis.quality_score > 0.0);
    assert!(result.analysis.unique_text_chars > 0);
}

#[tokio::test]
async fn test_spider_config() {
    let mut config = DomSpiderConfig::default();
    config.max_links_per_page = Some(2);
    config.extract_external_links = false;

    let html = r#"
        <html>
            <body>
                <a href="/link1">Link 1</a>
                <a href="/link2">Link 2</a>
                <a href="/link3">Link 3</a>
                <a href="https://external.com">External</a>
            </body>
        </html>
    "#;

    let base_url = Url::parse("https://example.com").unwrap();
    let crawler = HtmlDomCrawler::new(config);

    let links = crawler.extract_links(html, &base_url).await.unwrap();

    // Should respect max_links_per_page
    assert!(links.len() <= 2);

    // Should not include external links
    assert!(!links.iter().any(|url| url.host_str() == Some("external.com")));
}

#[tokio::test]
async fn test_form_type_detection() {
    let search_html = r#"
        <form action="/search" method="GET">
            <input type="text" name="q">
            <input type="submit" value="Search">
        </form>
    "#;

    let login_html = r#"
        <form action="/login" method="POST">
            <input type="email" name="email">
            <input type="password" name="password">
            <input type="submit" value="Login">
        </form>
    "#;

    let base_url = Url::parse("https://example.com").unwrap();
    let form_parser = crate::spider::form_parser::HtmlFormParser::default();

    let search_forms = form_parser.extract_search_forms(search_html, &base_url).await.unwrap();
    let login_forms = form_parser.extract_login_forms(login_html, &base_url).await.unwrap();

    assert_eq!(search_forms.len(), 1);
    assert_eq!(login_forms.len(), 1);
}