//! DOM spider tests for link extraction and form detection
//!
//! Tests for Week 3 HTML completion features including:
//! - Link extraction accuracy
//! - Form detection and analysis
//! - Metadata extraction
//! - Malformed HTML handling

use std::collections::HashMap;
use tokio_test;

// Import HTML processing functionality
use riptide_html::{
    dom_utils::{
        extract_links, extract_images, DomTraverser, LinkInfo, ImageInfo,
        get_document_outline, HeadingInfo, DocumentStats
    },
    processor::{HtmlProcessor, DefaultHtmlProcessor, TableExtractionMode},
};

#[tokio::test]
async fn test_link_extraction_accuracy() {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Link Test Page</title>
    </head>
    <body>
        <nav>
            <a href="https://example.com" title="Example Site">External Link</a>
            <a href="/internal-page" class="internal">Internal Link</a>
            <a href="mailto:contact@example.com">Email Link</a>
            <a href="tel:+1234567890">Phone Link</a>
            <a href="#section1" class="anchor">Anchor Link</a>
            <a href="javascript:void(0)" onclick="doSomething()">JavaScript Link</a>
            <a href="https://example.com/page?param=value&another=123">Link with Parameters</a>
            <a href="ftp://files.example.com/document.pdf">FTP Link</a>
        </nav>
        <main>
            <p>This paragraph contains an <a href="https://inline.example.com">inline link</a> within text.</p>
            <div>
                <a href="https://nested.example.com" target="_blank" rel="noopener">
                    Nested Link with Multiple Attributes
                </a>
            </div>
        </main>
        <footer>
            <a href="/privacy" rel="privacy">Privacy Policy</a>
            <a href="/terms" rel="terms">Terms of Service</a>
        </footer>
    </body>
    </html>
    "#;

    let links = extract_links(html).unwrap();

    // Should extract all 11 links
    assert_eq!(links.len(), 11);

    // Test specific link properties
    let external_link = links.iter().find(|l| l.href == "https://example.com").unwrap();
    assert_eq!(external_link.text, "External Link");
    assert_eq!(external_link.title, Some("Example Site".to_string()));

    let internal_link = links.iter().find(|l| l.href == "/internal-page").unwrap();
    assert_eq!(internal_link.text, "Internal Link");
    assert!(internal_link.attributes.contains_key("class"));
    assert_eq!(internal_link.attributes["class"], "internal");

    let email_link = links.iter().find(|l| l.href == "mailto:contact@example.com").unwrap();
    assert_eq!(email_link.text, "Email Link");

    let anchor_link = links.iter().find(|l| l.href == "#section1").unwrap();
    assert_eq!(anchor_link.text, "Anchor Link");

    let target_blank_link = links.iter().find(|l| l.href == "https://nested.example.com").unwrap();
    assert_eq!(target_blank_link.target, Some("_blank".to_string()));
    assert_eq!(target_blank_link.rel, Some("noopener".to_string()));

    // Test parameter handling
    let param_link = links.iter().find(|l| l.href.contains("param=value")).unwrap();
    assert!(param_link.href.contains("param=value&another=123"));

    // Test protocol handling
    let ftp_link = links.iter().find(|l| l.href.starts_with("ftp://")).unwrap();
    assert_eq!(ftp_link.href, "ftp://files.example.com/document.pdf");
}

#[tokio::test]
async fn test_form_detection_and_analysis() {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Form Test Page</title>
    </head>
    <body>
        <form id="login-form" method="post" action="/login" class="auth-form">
            <label for="username">Username:</label>
            <input type="text" id="username" name="username" required>

            <label for="password">Password:</label>
            <input type="password" id="password" name="password" required>

            <input type="submit" value="Login">
        </form>

        <form id="contact-form" method="get" action="/contact" enctype="multipart/form-data">
            <fieldset>
                <legend>Contact Information</legend>

                <label for="name">Full Name:</label>
                <input type="text" id="name" name="name" placeholder="Enter your name">

                <label for="email">Email:</label>
                <input type="email" id="email" name="email" required>

                <label for="phone">Phone:</label>
                <input type="tel" id="phone" name="phone">

                <label for="message">Message:</label>
                <textarea id="message" name="message" rows="4" cols="50"></textarea>

                <label for="file">Attachment:</label>
                <input type="file" id="file" name="attachment" accept=".pdf,.doc,.docx">

                <label for="category">Category:</label>
                <select id="category" name="category">
                    <option value="">Select Category</option>
                    <option value="sales">Sales</option>
                    <option value="support">Support</option>
                    <option value="billing">Billing</option>
                </select>

                <fieldset>
                    <legend>Contact Preference</legend>
                    <input type="radio" id="email-contact" name="contact-pref" value="email">
                    <label for="email-contact">Email</label>

                    <input type="radio" id="phone-contact" name="contact-pref" value="phone">
                    <label for="phone-contact">Phone</label>
                </fieldset>

                <input type="checkbox" id="newsletter" name="newsletter" value="yes">
                <label for="newsletter">Subscribe to newsletter</label>

                <button type="submit">Send Message</button>
                <button type="reset">Reset Form</button>
            </fieldset>
        </form>

        <form id="search-form" method="get" action="/search">
            <input type="search" name="q" placeholder="Search...">
            <input type="hidden" name="source" value="website">
            <button type="submit">Search</button>
        </form>
    </body>
    </html>
    "#;

    let traverser = DomTraverser::new(html);
    let forms = traverser.get_elements_info("form").unwrap();

    // Should detect 3 forms
    assert_eq!(forms.len(), 3);

    // Test login form
    let login_form = forms.iter().find(|f| f.id == Some("login-form".to_string())).unwrap();
    assert_eq!(login_form.tag, "form");
    assert_eq!(login_form.attributes.get("method"), Some(&"post".to_string()));
    assert_eq!(login_form.attributes.get("action"), Some(&"/login".to_string()));
    assert!(login_form.classes.contains(&"auth-form".to_string()));

    // Test contact form
    let contact_form = forms.iter().find(|f| f.id == Some("contact-form".to_string())).unwrap();
    assert_eq!(contact_form.attributes.get("method"), Some(&"get".to_string()));
    assert_eq!(contact_form.attributes.get("action"), Some(&"/contact".to_string()));
    assert_eq!(contact_form.attributes.get("enctype"), Some(&"multipart/form-data".to_string()));

    // Test form inputs
    let all_inputs = traverser.get_elements_info("input").unwrap();
    assert!(!all_inputs.is_empty());

    // Count different input types
    let text_inputs = all_inputs.iter().filter(|i| i.attributes.get("type") == Some(&"text".to_string())).count();
    let password_inputs = all_inputs.iter().filter(|i| i.attributes.get("type") == Some(&"password".to_string())).count();
    let email_inputs = all_inputs.iter().filter(|i| i.attributes.get("type") == Some(&"email".to_string())).count();
    let file_inputs = all_inputs.iter().filter(|i| i.attributes.get("type") == Some(&"file".to_string())).count();
    let radio_inputs = all_inputs.iter().filter(|i| i.attributes.get("type") == Some(&"radio".to_string())).count();
    let checkbox_inputs = all_inputs.iter().filter(|i| i.attributes.get("type") == Some(&"checkbox".to_string())).count();

    assert!(text_inputs >= 2);
    assert_eq!(password_inputs, 1);
    assert_eq!(email_inputs, 1);
    assert_eq!(file_inputs, 1);
    assert_eq!(radio_inputs, 2);
    assert_eq!(checkbox_inputs, 1);

    // Test textareas and selects
    let textareas = traverser.get_elements_info("textarea").unwrap();
    assert_eq!(textareas.len(), 1);

    let selects = traverser.get_elements_info("select").unwrap();
    assert_eq!(selects.len(), 1);

    let select = &selects[0];
    assert_eq!(select.id, Some("category".to_string()));

    // Test options within select
    let options = traverser.get_elements_info("option").unwrap();
    assert!(options.len() >= 4); // At least 4 options (including empty)
}

#[tokio::test]
async fn test_metadata_extraction() {
    let html = r#"
    <!DOCTYPE html>
    <html lang="en" data-theme="light">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta name="description" content="This is a test page for metadata extraction">
        <meta name="keywords" content="test, metadata, html, extraction">
        <meta name="author" content="Test Author">
        <meta property="og:title" content="Test Page">
        <meta property="og:description" content="Open Graph description">
        <meta property="og:image" content="https://example.com/image.jpg">
        <meta name="twitter:card" content="summary">
        <meta name="twitter:site" content="@testsite">
        <title>Test Page - Metadata Extraction</title>
        <link rel="canonical" href="https://example.com/test-page">
        <link rel="alternate" hreflang="es" href="https://example.com/es/test-page">
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "WebPage",
            "name": "Test Page",
            "description": "A test page for metadata extraction"
        }
        </script>
    </head>
    <body>
        <article itemscope itemtype="https://schema.org/Article">
            <h1 itemprop="headline">Main Article Title</h1>
            <p itemprop="description">Article description with structured data.</p>
            <div itemprop="author" itemscope itemtype="https://schema.org/Person">
                <span itemprop="name">John Doe</span>
            </div>
        </article>
    </body>
    </html>
    "#;

    let traverser = DomTraverser::new(html);

    // Test meta tag extraction
    let meta_tags = traverser.get_elements_info("meta").unwrap();
    assert!(meta_tags.len() >= 9);

    // Check specific meta tags
    let description_meta = meta_tags.iter()
        .find(|m| m.attributes.get("name") == Some(&"description".to_string()))
        .unwrap();
    assert_eq!(
        description_meta.attributes.get("content"),
        Some(&"This is a test page for metadata extraction".to_string())
    );

    let og_title_meta = meta_tags.iter()
        .find(|m| m.attributes.get("property") == Some(&"og:title".to_string()))
        .unwrap();
    assert_eq!(
        og_title_meta.attributes.get("content"),
        Some(&"Test Page".to_string())
    );

    // Test title extraction
    let title_text = traverser.extract_text("title").unwrap();
    assert_eq!(title_text[0], "Test Page - Metadata Extraction");

    // Test link tags
    let link_tags = traverser.get_elements_info("link").unwrap();
    assert!(link_tags.len() >= 2);

    let canonical_link = link_tags.iter()
        .find(|l| l.attributes.get("rel") == Some(&"canonical".to_string()))
        .unwrap();
    assert_eq!(
        canonical_link.attributes.get("href"),
        Some(&"https://example.com/test-page".to_string())
    );

    // Test structured data
    let structured_data = traverser.get_elements_info("[itemscope]").unwrap();
    assert!(!structured_data.is_empty());

    let article = structured_data.iter()
        .find(|e| e.attributes.get("itemtype") == Some(&"https://schema.org/Article".to_string()))
        .unwrap();
    assert_eq!(article.tag, "article");

    // Test JSON-LD script
    let json_ld_scripts = traverser.get_elements_info(r#"script[type="application/ld+json"]"#).unwrap();
    assert_eq!(json_ld_scripts.len(), 1);
    assert!(json_ld_scripts[0].text.contains("@context"));
    assert!(json_ld_scripts[0].text.contains("schema.org"));
}

#[tokio::test]
async fn test_malformed_html_handling() {
    let malformed_html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Malformed HTML Test
        <meta charset="UTF-8">
        <!-- Unclosed comment
    </head>
    <body>
        <div class="container">
            <p>This paragraph is not closed
            <div>Nested div without closing tag
                <span>Unclosed span
                <a href="https://example.com">Link without closing
            </div>
            <ul>
                <li>First item
                <li>Second item without closing
                <li>Third item</li>
            </ul>
            <table>
                <tr>
                    <td>Cell 1
                    <td>Cell 2</td>
                <tr>
                    <td>Cell 3</td>
                    <td>Cell 4
            </table>
            <img src="image.jpg" alt="Image without closing slash">
            <br>
            <hr>
            <input type="text" name="unclosed-input">
        </div>
        <!-- Another unclosed comment
    </body>
    "#;

    // The DOM parser should handle malformed HTML gracefully
    let traverser = DomTraverser::new(malformed_html);
    let stats = traverser.get_stats();

    // Should still extract meaningful information
    assert!(stats.total_elements > 0);
    assert!(stats.total_text_length > 0);

    // Test that we can still extract elements despite malformed structure
    let links = extract_links(malformed_html).unwrap();
    assert!(!links.is_empty());

    let images = extract_images(malformed_html).unwrap();
    assert!(!images.is_empty());

    // Test table extraction with malformed table
    let processor = DefaultHtmlProcessor::default();
    let tables = processor.extract_tables(malformed_html, TableExtractionMode::All).await.unwrap();
    assert!(!tables.is_empty());

    // Verify the table has some structure despite being malformed
    let table = &tables[0];
    assert!(!table.rows.is_empty());

    // Test that text extraction still works
    let all_text = traverser.extract_text("body").unwrap();
    assert!(!all_text.is_empty());
    assert!(all_text[0].contains("This paragraph"));
}

#[tokio::test]
async fn test_document_structure_analysis() {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Document Structure Test</title>
    </head>
    <body>
        <header>
            <h1>Main Title</h1>
            <nav>
                <ul>
                    <li><a href="#section1">Section 1</a></li>
                    <li><a href="#section2">Section 2</a></li>
                </ul>
            </nav>
        </header>

        <main>
            <section id="section1">
                <h2>Section 1 Title</h2>
                <article>
                    <h3>Article 1 in Section 1</h3>
                    <p>Article content...</p>
                </article>
                <article>
                    <h3>Article 2 in Section 1</h3>
                    <p>More article content...</p>
                </article>
            </section>

            <section id="section2">
                <h2>Section 2 Title</h2>
                <h3>Subsection A</h3>
                <p>Subsection content...</p>
                <h4>Sub-subsection</h4>
                <p>Deep nesting content...</p>
                <h3>Subsection B</h3>
                <p>More subsection content...</p>
            </section>

            <aside>
                <h2>Sidebar</h2>
                <p>Sidebar content...</p>
            </aside>
        </main>

        <footer>
            <p>Footer content</p>
        </footer>
    </body>
    </html>
    "#;

    let traverser = DomTraverser::new(html);
    let stats = traverser.get_stats();

    // Test document statistics
    assert!(stats.total_elements > 20);
    assert!(stats.tag_counts.contains_key("h1"));
    assert!(stats.tag_counts.contains_key("h2"));
    assert!(stats.tag_counts.contains_key("h3"));
    assert!(stats.tag_counts.contains_key("h4"));
    assert!(stats.tag_counts.contains_key("section"));
    assert!(stats.tag_counts.contains_key("article"));

    // Test heading outline extraction
    let outline = get_document_outline(html).unwrap();
    assert!(outline.len() >= 7); // h1, 2 h2s, 3 h3s, 1 h4

    // Verify heading hierarchy
    let h1_headings: Vec<_> = outline.iter().filter(|h| h.level == 1).collect();
    let h2_headings: Vec<_> = outline.iter().filter(|h| h.level == 2).collect();
    let h3_headings: Vec<_> = outline.iter().filter(|h| h.level == 3).collect();
    let h4_headings: Vec<_> = outline.iter().filter(|h| h.level == 4).collect();

    assert_eq!(h1_headings.len(), 1);
    assert!(h2_headings.len() >= 3); // Section 1, Section 2, Sidebar
    assert!(h3_headings.len() >= 3); // Articles and subsections
    assert!(h4_headings.len() >= 1); // Sub-subsection

    // Test semantic element detection
    let semantic_elements = traverser.get_elements_info("header, main, section, article, aside, footer").unwrap();
    assert!(semantic_elements.len() >= 6);

    // Test navigation structure
    let nav_elements = traverser.get_elements_info("nav").unwrap();
    assert_eq!(nav_elements.len(), 1);

    let nav_links = traverser.get_elements_info("nav a").unwrap();
    assert!(nav_links.len() >= 2);
}

#[tokio::test]
async fn test_nested_html_structures() {
    let html = r#"
    <div class="outer">
        <div class="level1">
            <div class="level2">
                <div class="level3">
                    <div class="level4">
                        <div class="level5">
                            <p>Deeply nested content</p>
                            <ul class="nested-list">
                                <li>Item 1
                                    <ul class="sub-list">
                                        <li>Sub-item 1.1
                                            <ul class="sub-sub-list">
                                                <li>Sub-sub-item 1.1.1</li>
                                                <li>Sub-sub-item 1.1.2</li>
                                            </ul>
                                        </li>
                                        <li>Sub-item 1.2</li>
                                    </ul>
                                </li>
                                <li>Item 2</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <table class="nested-table">
            <tr>
                <td>
                    <div class="cell-content">
                        <p>Table cell with nested content</p>
                        <form>
                            <input type="text" name="nested-input">
                            <div class="form-group">
                                <label>Nested label</label>
                                <select name="nested-select">
                                    <option value="1">Option 1</option>
                                    <option value="2">Option 2</option>
                                </select>
                            </div>
                        </form>
                    </div>
                </td>
            </tr>
        </table>
    </div>
    "#;

    let traverser = DomTraverser::new(html);

    // Test deep nesting handling
    let all_divs = traverser.get_elements_info("div").unwrap();
    assert!(all_divs.len() >= 7); // outer + level1-5 + cell-content + form-group

    // Test that we can find deeply nested elements
    let nested_p = traverser.get_elements_info("div div div div div p").unwrap();
    assert_eq!(nested_p.len(), 1);
    assert_eq!(nested_p[0].text, "Deeply nested content");

    // Test nested list structure
    let all_lists = traverser.get_elements_info("ul").unwrap();
    assert_eq!(all_lists.len(), 3); // nested-list, sub-list, sub-sub-list

    let all_list_items = traverser.get_elements_info("li").unwrap();
    assert!(all_list_items.len() >= 5);

    // Test nested form in table
    let nested_form = traverser.get_elements_info("table form").unwrap();
    assert_eq!(nested_form.len(), 1);

    let nested_inputs = traverser.get_elements_info("table input").unwrap();
    assert_eq!(nested_inputs.len(), 1);

    let nested_selects = traverser.get_elements_info("table select").unwrap();
    assert_eq!(nested_selects.len(), 1);

    // Test that we can extract text from deeply nested structures
    let deep_text = traverser.extract_text(".level5 p").unwrap();
    assert_eq!(deep_text[0], "Deeply nested content");
}

#[tokio::test]
async fn test_image_extraction_comprehensive() {
    let html = r#"
    <html>
    <body>
        <img src="image1.jpg" alt="First image" width="100" height="200" class="thumbnail">
        <img src="https://example.com/image2.png" alt="Second image" title="Image title" data-id="123">
        <img src="/relative/image3.gif" alt="" width="50">
        <img src="data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAYABgAAD..." alt="Base64 image">
        <picture>
            <source srcset="large.jpg" media="(min-width: 800px)">
            <source srcset="medium.jpg" media="(min-width: 400px)">
            <img src="small.jpg" alt="Responsive image">
        </picture>
        <figure>
            <img src="figure-image.jpg" alt="Figure image" id="fig1">
            <figcaption>Image caption</figcaption>
        </figure>
        <div style="background-image: url('background.jpg')">Background image div</div>
    </body>
    </html>
    "#;

    let images = extract_images(html).unwrap();

    // Should extract 5 img elements (not background images or source elements)
    assert_eq!(images.len(), 5);

    // Test first image properties
    let img1 = &images[0];
    assert_eq!(img1.src, "image1.jpg");
    assert_eq!(img1.alt, "First image");
    assert_eq!(img1.width, Some(100));
    assert_eq!(img1.height, Some(200));
    assert!(img1.attributes.contains_key("class"));

    // Test absolute URL image
    let img2 = images.iter().find(|img| img.src.starts_with("https://")).unwrap();
    assert_eq!(img2.src, "https://example.com/image2.png");
    assert_eq!(img2.title, Some("Image title".to_string()));

    // Test relative path image
    let img3 = images.iter().find(|img| img.src.starts_with("/relative/")).unwrap();
    assert_eq!(img3.src, "/relative/image3.gif");
    assert_eq!(img3.alt, ""); // Empty alt text

    // Test data URI image
    let data_img = images.iter().find(|img| img.src.starts_with("data:")).unwrap();
    assert!(data_img.src.starts_with("data:image/jpeg;base64,"));

    // Test responsive image (from picture element)
    let responsive_img = images.iter().find(|img| img.alt == "Responsive image").unwrap();
    assert_eq!(responsive_img.src, "small.jpg");

    // Test figure image
    let figure_img = images.iter().find(|img| img.src == "figure-image.jpg").unwrap();
    assert!(figure_img.attributes.contains_key("id"));
    assert_eq!(figure_img.attributes["id"], "fig1");
}

#[tokio::test]
async fn test_performance_large_dom() {
    // Generate a large HTML document with many nested elements
    let mut html = String::from("<html><body>");

    // Add 1000 nested divs with various attributes
    for i in 0..1000 {
        html.push_str(&format!(
            r#"<div id="div{}" class="item level{}" data-index="{}">
                <p>Content for item {}</p>
                <a href="/item/{}">Link {}</a>
                <img src="image{}.jpg" alt="Image {}">
            </div>"#,
            i, i % 10, i, i, i, i, i, i
        ));
    }

    html.push_str("</body></html>");

    let start = std::time::Instant::now();

    // Test DOM traversal performance
    let traverser = DomTraverser::new(&html);
    let stats = traverser.get_stats();

    let elapsed = start.elapsed();

    // Should handle large DOM efficiently (under 100ms for this size)
    assert!(elapsed.as_millis() < 100, "DOM traversal took too long: {:?}", elapsed);

    // Verify we processed all elements
    assert!(stats.total_elements >= 4000); // 1000 divs + 1000 p + 1000 a + 1000 img + html/body

    // Test specific element extraction
    let start = std::time::Instant::now();
    let all_links = extract_links(&html).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(all_links.len(), 1000);
    assert!(elapsed.as_millis() < 50, "Link extraction took too long: {:?}", elapsed);

    // Test image extraction
    let start = std::time::Instant::now();
    let all_images = extract_images(&html).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(all_images.len(), 1000);
    assert!(elapsed.as_millis() < 50, "Image extraction took too long: {:?}", elapsed);
}