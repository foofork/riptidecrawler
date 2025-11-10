//! Example demonstrating enhanced link extraction with context
//!
//! This example shows how to extract links from HTML with:
//! - Anchor text
//! - Surrounding context
//! - Link type classification
//! - HTML attributes

use riptide_extraction::enhanced_link_extraction::{
    EnhancedLinkExtractor, LinkExtractionConfig, LinkType,
};

fn main() -> anyhow::Result<()> {
    // Sample HTML with various link types
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
                       <a href="/product/details" rel="nofollow" target="_self" data-tracking="details">click here</a>
                       to see full details and specifications.</p>
                    <p>You can also <a href="/product-manual.pdf" download>download the manual</a>
                       or <a href="mailto:support@example.com">contact support</a> for assistance.</p>
                    <p>Visit our <a href="https://partner.com" rel="noopener" target="_blank">partner site</a>
                       for compatible accessories.</p>
                    <p>Read our <a href="https://blog.example.com/reviews">latest reviews</a> and
                       check the <a href="#specifications">specifications section</a> below.</p>
                </article>
                <section id="specifications">
                    <h2>Specifications</h2>
                    <p>Need help? <a href="tel:+1-800-555-0123">Call us</a> anytime.</p>
                </section>
            </main>
            <footer>
                <a href="#top">Back to Top</a>
            </footer>
        </body>
        </html>
    "##;

    println!("=== Enhanced Link Extraction Demo ===\n");

    // Example 1: Basic extraction with default configuration
    println!("1. Basic Link Extraction:");
    println!("{}", "=".repeat(60));
    let extractor = EnhancedLinkExtractor::new();
    let links = extractor.extract_links(html, Some("https://example.com"))?;

    println!("Found {} links\n", links.len());

    for (i, link) in links.iter().enumerate() {
        println!("Link #{}", i + 1);
        println!("  URL: {}", link.url);
        println!("  Anchor Text: {}", link.anchor_text);
        println!("  Type: {:?}", link.link_type);
        println!(
            "  Context: {}...",
            &link.surrounding_context[..link.surrounding_context.len().min(80)]
        );

        if !link.attributes.is_empty() {
            println!("  Attributes:");
            for (key, value) in &link.attributes {
                println!("    - {}: {}", key, value);
            }
        }
        println!();
    }

    // Example 2: Extract links grouped by type
    println!("\n2. Links Grouped by Type:");
    println!("{}", "=".repeat(60));
    let grouped = extractor.extract_links_by_type(html, Some("https://example.com"))?;

    for (link_type, type_links) in &grouped {
        println!("{:?} links ({}):", link_type, type_links.len());
        for link in type_links {
            println!("  - {} ({})", link.anchor_text, link.url);
        }
        println!();
    }

    // Example 3: Extract only internal links
    println!("\n3. Internal Links Only:");
    println!("{}", "=".repeat(60));
    let internal_links = extractor.extract_internal_links(html, Some("https://example.com"))?;

    for link in &internal_links {
        println!("  - {} → {}", link.anchor_text, link.url);
    }

    // Example 4: Extract only external links
    println!("\n\n4. External Links Only:");
    println!("{}", "=".repeat(60));
    let external_links = extractor.extract_external_links(html, Some("https://example.com"))?;

    for link in &external_links {
        println!("  - {} → {}", link.anchor_text, link.url);
        if let Some(rel) = link.attributes.get("rel") {
            println!("    rel=\"{}\"", rel);
        }
        if let Some(target) = link.attributes.get("target") {
            println!("    target=\"{}\"", target);
        }
    }

    // Example 5: Custom configuration
    println!("\n\n5. Custom Configuration (limited context, max 5 links):");
    println!("{}", "=".repeat(60));
    let config = LinkExtractionConfig {
        context_chars_before: 30,
        context_chars_after: 30,
        max_links: Some(5),
        extract_anchors: false, // Skip anchor links
        ..Default::default()
    };

    let custom_extractor = EnhancedLinkExtractor::with_config(config);
    let custom_links = custom_extractor.extract_links(html, Some("https://example.com"))?;

    println!("Found {} links (max 5 configured)\n", custom_links.len());
    for link in &custom_links {
        println!("  {} → {}", link.anchor_text, link.url);
    }

    // Example 6: JSON output
    println!("\n\n6. JSON Output (first link):");
    println!("{}", "=".repeat(60));
    if let Some(first_link) = links.first() {
        let json = serde_json::to_string_pretty(first_link)?;
        println!("{}", json);
    }

    // Example 7: Statistics
    println!("\n\n7. Link Statistics:");
    println!("{}", "=".repeat(60));
    let total = links.len();
    let internal_count = links
        .iter()
        .filter(|l| l.link_type == LinkType::Internal)
        .count();
    let external_count = links
        .iter()
        .filter(|l| l.link_type == LinkType::External)
        .count();
    let download_count = links
        .iter()
        .filter(|l| l.link_type == LinkType::Download)
        .count();
    let email_count = links
        .iter()
        .filter(|l| l.link_type == LinkType::Email)
        .count();
    let phone_count = links
        .iter()
        .filter(|l| l.link_type == LinkType::Phone)
        .count();
    let anchor_count = links
        .iter()
        .filter(|l| l.link_type == LinkType::Anchor)
        .count();

    println!("Total links: {}", total);
    println!(
        "  Internal: {} ({:.1}%)",
        internal_count,
        internal_count as f64 / total as f64 * 100.0
    );
    println!(
        "  External: {} ({:.1}%)",
        external_count,
        external_count as f64 / total as f64 * 100.0
    );
    println!(
        "  Downloads: {} ({:.1}%)",
        download_count,
        download_count as f64 / total as f64 * 100.0
    );
    println!(
        "  Email: {} ({:.1}%)",
        email_count,
        email_count as f64 / total as f64 * 100.0
    );
    println!(
        "  Phone: {} ({:.1}%)",
        phone_count,
        phone_count as f64 / total as f64 * 100.0
    );
    println!(
        "  Anchor: {} ({:.1}%)",
        anchor_count,
        anchor_count as f64 / total as f64 * 100.0
    );

    Ok(())
}
