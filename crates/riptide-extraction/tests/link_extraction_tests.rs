//! Comprehensive tests for enhanced link extraction with context

use riptide_extraction::enhanced_link_extraction::{
    EnhancedLinkExtractor, LinkExtractionConfig, LinkType,
};

#[test]
fn test_extract_anchor_text() {
    let html = r##"
        <html>
        <body>
            <a href="https://example.com">Click here to visit</a>
            <a href="/page">
                <span>Nested</span>
                <strong>Text</strong>
            </a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].anchor_text, "Click here to visit");
    assert_eq!(links[1].anchor_text, "Nested Text");
}

#[test]
fn test_extract_context_simple() {
    let html = r##"
        <html>
        <body>
            <p>This is some text before the link. Click <a href="/page">here</a> to continue reading.</p>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 1);
    let context = &links[0].surrounding_context;

    assert!(context.contains("before the link"));
    assert!(context.contains("here"));
    assert!(context.contains("continue reading"));
}

#[test]
fn test_extract_context_with_config() {
    let html = r##"
        <html>
        <body>
            <p>ABCDEFGHIJKLMNOPQRSTUVWXYZ <a href="/link">LINK</a> 0123456789</p>
        </body>
        </html>
    "##;

    let config = LinkExtractionConfig {
        context_chars_before: 10,
        context_chars_after: 10,
        ..Default::default()
    };

    let extractor = EnhancedLinkExtractor::with_config(config);
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 1);
    assert!(links[0].surrounding_context.len() < 50);
}

#[test]
fn test_classify_internal_links() {
    let html = r##"
        <html>
        <body>
            <a href="https://example.com/page1">Same Domain</a>
            <a href="https://example.com/page2">Also Same</a>
            <a href="/relative">Relative Link</a>
            <a href="page.html">Relative No Slash</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor
        .extract_links(html, Some("https://example.com"))
        .unwrap();

    assert_eq!(links.len(), 4);

    for link in &links {
        assert_eq!(link.link_type, LinkType::Internal);
    }
}

#[test]
fn test_classify_external_links() {
    let html = r##"
        <html>
        <body>
            <a href="https://google.com">Google</a>
            <a href="https://github.com">GitHub</a>
            <a href="https://example.org">Example Org</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor
        .extract_links(html, Some("https://example.com"))
        .unwrap();

    assert_eq!(links.len(), 3);

    for link in &links {
        assert_eq!(link.link_type, LinkType::External);
    }
}

#[test]
fn test_classify_download_links() {
    let html = r##"
        <html>
        <body>
            <a href="/document.pdf">PDF Document</a>
            <a href="/archive.zip">ZIP Archive</a>
            <a href="/spreadsheet.xlsx">Excel File</a>
            <a href="/presentation.pptx">PowerPoint</a>
            <a href="/installer.exe">Windows Installer</a>
            <a href="/package.deb">Debian Package</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 6);

    for link in &links {
        assert_eq!(link.link_type, LinkType::Download);
    }
}

#[test]
fn test_classify_anchor_links() {
    let html = r##"
        <html>
        <body>
            <a href="#section1">Section 1</a>
            <a href="#top">Back to Top</a>
            <a href="#footer">Footer</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 3);

    for link in &links {
        assert_eq!(link.link_type, LinkType::Anchor);
    }
}

#[test]
fn test_classify_email_links() {
    let html = r##"
        <html>
        <body>
            <a href="mailto:info@example.com">Email Us</a>
            <a href="mailto:support@example.com?subject=Help">Support</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 2);

    for link in &links {
        assert_eq!(link.link_type, LinkType::Email);
    }
}

#[test]
fn test_classify_phone_links() {
    let html = r##"
        <html>
        <body>
            <a href="tel:+1234567890">Call Us</a>
            <a href="tel:555-0123">Support Line</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 2);

    for link in &links {
        assert_eq!(link.link_type, LinkType::Phone);
    }
}

#[test]
fn test_extract_rel_attribute() {
    let html = r##"
        <html>
        <body>
            <a href="https://example.com" rel="nofollow">No Follow</a>
            <a href="https://example.com" rel="noopener">No Opener</a>
            <a href="https://example.com" rel="nofollow noopener">Both</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 3);
    assert_eq!(
        links[0].attributes.get("rel"),
        Some(&"nofollow".to_string())
    );
    assert_eq!(
        links[1].attributes.get("rel"),
        Some(&"noopener".to_string())
    );
    assert_eq!(
        links[2].attributes.get("rel"),
        Some(&"nofollow noopener".to_string())
    );
}

#[test]
fn test_extract_target_attribute() {
    let html = r##"
        <html>
        <body>
            <a href="https://example.com" target="_blank">New Tab</a>
            <a href="/page" target="_self">Same Tab</a>
            <a href="/page" target="_parent">Parent</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 3);
    assert_eq!(
        links[0].attributes.get("target"),
        Some(&"_blank".to_string())
    );
    assert_eq!(
        links[1].attributes.get("target"),
        Some(&"_self".to_string())
    );
    assert_eq!(
        links[2].attributes.get("target"),
        Some(&"_parent".to_string())
    );
}

#[test]
fn test_extract_data_attributes() {
    let html = r##"
        <html>
        <body>
            <a href="/page"
               data-tracking="abc123"
               data-category="products"
               data-position="header">Link</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 1);
    assert_eq!(
        links[0].attributes.get("data-tracking"),
        Some(&"abc123".to_string())
    );
    assert_eq!(
        links[0].attributes.get("data-category"),
        Some(&"products".to_string())
    );
    assert_eq!(
        links[0].attributes.get("data-position"),
        Some(&"header".to_string())
    );
}

#[test]
fn test_extract_aria_attributes() {
    let html = r##"
        <html>
        <body>
            <a href="/page"
               aria-label="Read more about products"
               aria-describedby="desc1">Link</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 1);
    assert_eq!(
        links[0].attributes.get("aria-label"),
        Some(&"Read more about products".to_string())
    );
    assert_eq!(
        links[0].attributes.get("aria-describedby"),
        Some(&"desc1".to_string())
    );
}

#[test]
fn test_extract_all_common_attributes() {
    let html = r##"
        <html>
        <body>
            <a href="https://example.com"
               rel="nofollow"
               target="_blank"
               title="Example Site"
               class="external-link highlight"
               id="main-link"
               download>Download</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 1);
    let attrs = &links[0].attributes;

    assert!(attrs.contains_key("rel"));
    assert!(attrs.contains_key("target"));
    assert!(attrs.contains_key("title"));
    assert!(attrs.contains_key("class"));
    assert!(attrs.contains_key("id"));
    assert!(attrs.contains_key("download"));
}

#[test]
fn test_position_in_document() {
    let html = r##"
        <html>
        <body>
            <p>First paragraph.</p>
            <p>Second paragraph with <a href="/link1">first link</a> here.</p>
            <p>Third paragraph with <a href="/link2">second link</a> here.</p>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 2);
    // Second link should have higher position than first
    assert!(links[1].position > links[0].position);
}

#[test]
fn test_max_links_limitation() {
    let html = r##"
        <html>
        <body>
            <a href="/link1">Link 1</a>
            <a href="/link2">Link 2</a>
            <a href="/link3">Link 3</a>
            <a href="/link4">Link 4</a>
            <a href="/link5">Link 5</a>
            <a href="/link6">Link 6</a>
            <a href="/link7">Link 7</a>
            <a href="/link8">Link 8</a>
        </body>
        </html>
    "##;

    let config = LinkExtractionConfig {
        max_links: Some(5),
        ..Default::default()
    };

    let extractor = EnhancedLinkExtractor::with_config(config);
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 5);
}

#[test]
fn test_filter_by_link_type() {
    let html = r##"
        <html>
        <body>
            <a href="https://other.com/page">External</a>
            <a href="/internal">Internal</a>
            <a href="mailto:test@example.com">Email</a>
        </body>
        </html>
    "##;

    let config = LinkExtractionConfig {
        extract_internal: false,
        extract_external: true,
        extract_special: true,
        ..Default::default()
    };

    let extractor = EnhancedLinkExtractor::with_config(config);
    let links = extractor
        .extract_links(html, Some("https://example.com"))
        .unwrap();

    // Should only get external and email, not internal
    assert_eq!(links.len(), 2);
    assert!(links
        .iter()
        .any(|l| l.link_type == LinkType::External || l.link_type == LinkType::Email));
    assert!(!links.iter().any(|l| l.link_type == LinkType::Internal));
}

#[test]
fn test_extract_links_by_type() {
    let html = r##"
        <html>
        <body>
            <a href="https://google.com">External 1</a>
            <a href="https://github.com">External 2</a>
            <a href="/page1">Internal 1</a>
            <a href="/page2">Internal 2</a>
            <a href="/page3">Internal 3</a>
            <a href="mailto:test@example.com">Email</a>
            <a href="/file.pdf">Download</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let grouped = extractor
        .extract_links_by_type(html, Some("https://example.com"))
        .unwrap();

    assert_eq!(grouped.get(&LinkType::External).unwrap().len(), 2);
    assert_eq!(grouped.get(&LinkType::Internal).unwrap().len(), 3);
    assert_eq!(grouped.get(&LinkType::Email).unwrap().len(), 1);
    assert_eq!(grouped.get(&LinkType::Download).unwrap().len(), 1);
}

#[test]
fn test_extract_internal_links_only() {
    let html = r##"
        <html>
        <body>
            <a href="https://example.com/page1">Same Domain</a>
            <a href="https://other.com">Other Domain</a>
            <a href="/relative">Relative</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor
        .extract_internal_links(html, Some("https://example.com"))
        .unwrap();

    assert_eq!(links.len(), 2); // Same domain + relative
    for link in &links {
        assert_eq!(link.link_type, LinkType::Internal);
    }
}

#[test]
fn test_extract_external_links_only() {
    let html = r##"
        <html>
        <body>
            <a href="https://example.com/page1">Same Domain</a>
            <a href="https://google.com">Google</a>
            <a href="https://github.com">GitHub</a>
            <a href="/relative">Relative</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor
        .extract_external_links(html, Some("https://example.com"))
        .unwrap();

    assert_eq!(links.len(), 2); // Google + GitHub
    for link in &links {
        assert_eq!(link.link_type, LinkType::External);
    }
}

#[test]
fn test_handle_empty_href() {
    let html = r##"
        <html>
        <body>
            <a href="">Empty</a>
            <a href="   ">Whitespace</a>
            <a href="/valid">Valid</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    // Should only extract the valid link
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].anchor_text, "Valid");
}

#[test]
fn test_resolve_relative_urls() {
    let html = r##"
        <html>
        <body>
            <a href="/absolute">Absolute Path</a>
            <a href="relative.html">Relative Path</a>
            <a href="../parent.html">Parent Path</a>
            <a href="./current.html">Current Path</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor
        .extract_links(html, Some("https://example.com/dir/page.html"))
        .unwrap();

    assert_eq!(links.len(), 4);
    assert_eq!(links[0].url, "https://example.com/absolute");
    assert_eq!(links[1].url, "https://example.com/dir/relative.html");
    assert_eq!(links[2].url, "https://example.com/parent.html");
    assert_eq!(links[3].url, "https://example.com/dir/current.html");
}

#[test]
fn test_custom_download_extensions() {
    let html = r##"
        <html>
        <body>
            <a href="/file.custom">Custom Extension</a>
            <a href="/file.pdf">PDF</a>
        </body>
        </html>
    "##;

    let mut config = LinkExtractionConfig::default();
    config.download_extensions.push("custom".to_string());

    let extractor = EnhancedLinkExtractor::with_config(config);
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].link_type, LinkType::Download);
    assert_eq!(links[1].link_type, LinkType::Download);
}

#[test]
fn test_case_insensitive_extension_matching() {
    let html = r##"
        <html>
        <body>
            <a href="/file.PDF">Uppercase PDF</a>
            <a href="/file.Pdf">Mixed Case</a>
            <a href="/file.pdf">Lowercase</a>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, None).unwrap();

    assert_eq!(links.len(), 3);
    for link in &links {
        assert_eq!(link.link_type, LinkType::Download);
    }
}

#[test]
fn test_complex_real_world_example() {
    let html = r##"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Product Page</title>
        </head>
        <body>
            <nav>
                <a href="/" class="logo">Home</a>
                <a href="/products">Products</a>
                <a href="/about">About Us</a>
            </nav>
            <main>
                <article>
                    <h1>Amazing Product</h1>
                    <p>This is the best product ever. For more information,
                       <a href="/product/details" rel="nofollow" target="_self">click here</a>
                       to see full details.</p>
                    <p>You can also <a href="/product.pdf" download>download the manual</a>
                       or <a href="mailto:support@example.com">contact support</a>.</p>
                    <p>Visit our <a href="https://partner.com" rel="noopener" target="_blank">partner site</a>
                       for accessories.</p>
                </article>
            </main>
            <footer>
                <a href="#top">Back to Top</a>
                <a href="tel:+1234567890">Call Us</a>
            </footer>
        </body>
        </html>
    "##;

    let extractor = EnhancedLinkExtractor::new();
    let links = extractor
        .extract_links(html, Some("https://example.com"))
        .unwrap();

    // Should extract all 9 links
    assert_eq!(links.len(), 9);

    // Verify we have different types
    let types: std::collections::HashSet<_> = links.iter().map(|l| &l.link_type).collect();
    assert!(types.contains(&LinkType::Internal));
    assert!(types.contains(&LinkType::External));
    assert!(types.contains(&LinkType::Download));
    assert!(types.contains(&LinkType::Email));
    assert!(types.contains(&LinkType::Phone));
    assert!(types.contains(&LinkType::Anchor));

    // Verify attributes are captured
    let details_link = links
        .iter()
        .find(|l| l.anchor_text == "click here")
        .unwrap();
    assert_eq!(
        details_link.attributes.get("rel"),
        Some(&"nofollow".to_string())
    );
    assert_eq!(
        details_link.attributes.get("target"),
        Some(&"_self".to_string())
    );

    // Verify context is extracted
    assert!(details_link
        .surrounding_context
        .contains("more information"));
    assert!(details_link.surrounding_context.contains("full details"));
}
