//! Basic extraction example for riptide-html crate

use riptide_html::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 RipTide HTML Extraction Demo");
    println!("================================");

    let sample_html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Sample News Article</title>
                <meta name="description" content="This is a sample news article for testing HTML extraction">
                <meta property="og:title" content="Breaking News: HTML Extraction Works!">
            </head>
            <body>
                <article>
                    <h1>Breaking: Advanced HTML Extraction Now Available</h1>
                    <div class="author">By Jane Smith</div>
                    <time datetime="2023-10-01">October 1, 2023</time>

                    <div class="content">
                        <p>RipTide's new HTML extraction capabilities are now available, offering advanced CSS selector-based extraction and regex pattern matching.</p>
                        <p>The new features include support for table extraction, DOM traversal utilities, and comprehensive content chunking.</p>
                        <p>Contact us at support@riptide.com or call (555) 123-4567 for more information.</p>
                    </div>

                    <div class="tags">
                        <span class="tag">technology</span>
                        <span class="tag">web-scraping</span>
                        <span class="tag">rust</span>
                    </div>
                </article>

                <table id="features-table">
                    <caption>RipTide HTML Features</caption>
                    <thead>
                        <tr>
                            <th>Feature</th>
                            <th>Status</th>
                            <th>Description</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>CSS Extraction</td>
                            <td>✅ Complete</td>
                            <td>Extract content using CSS selectors</td>
                        </tr>
                        <tr>
                            <td>Regex Patterns</td>
                            <td>✅ Complete</td>
                            <td>Pattern-based content extraction</td>
                        </tr>
                        <tr>
                            <td>Table Processing</td>
                            <td>✅ Complete</td>
                            <td>Structured table data extraction</td>
                        </tr>
                    </tbody>
                </table>
            </body>
        </html>
    "#;

    // 1. CSS Extraction Demo
    println!("\n📋 CSS Extraction (Default Selectors):");
    println!("--------------------------------------");

    let css_result = css_extract_default(sample_html, "https://example.com/article").await?;
    println!("Title: {}", css_result.title);
    println!("Summary: {:?}", css_result.summary);
    println!(
        "Content (first 100 chars): {}...",
        css_result.content.chars().take(100).collect::<String>()
    );
    println!("Confidence: {:.2}", css_result.extraction_confidence);

    // 2. Custom CSS Selectors Demo
    println!("\n🎯 CSS Extraction (Custom Selectors):");
    println!("-------------------------------------");

    let mut custom_selectors = HashMap::new();
    custom_selectors.insert("title".to_string(), "h1".to_string());
    custom_selectors.insert("author".to_string(), ".author".to_string());
    custom_selectors.insert("date".to_string(), "time".to_string());
    custom_selectors.insert("content".to_string(), ".content p".to_string());
    custom_selectors.insert("tags".to_string(), ".tag".to_string());

    let custom_result = css_extract(
        sample_html,
        "https://example.com/article",
        &custom_selectors,
    )
    .await?;
    println!("H1 Title: {}", custom_result.title);
    println!("Content: {}", custom_result.content);

    // 3. Regex Extraction Demo
    println!("\n🔍 Regex Pattern Extraction:");
    println!("----------------------------");

    let regex_result =
        regex_extraction::extract_contacts(sample_html, "https://example.com").await?;
    println!("Extracted contacts: {}", regex_result.content);

    // 4. Table Extraction Demo
    println!("\n📊 Table Extraction:");
    println!("-------------------");

    let tables =
        dom_utils::extract_tables(sample_html, processor::TableExtractionMode::All).await?;
    for (i, table) in tables.iter().enumerate() {
        println!(
            "Table {}: {} ({}x{})",
            i + 1,
            table.caption.as_deref().unwrap_or("No caption"),
            table.rows.len(),
            table.headers.len()
        );

        println!("Headers: {:?}", table.headers);
        for (row_idx, row) in table.rows.iter().take(2).enumerate() {
            println!("Row {}: {:?}", row_idx + 1, row);
        }
    }

    // 5. DOM Utilities Demo
    println!("\n🌳 DOM Utilities:");
    println!("----------------");

    let images = dom_utils::extract_images(sample_html)?;
    println!("Images found: {}", images.len());

    let links = dom_utils::extract_links(sample_html)?;
    println!("Links found: {}", links.len());

    let outline = dom_utils::get_document_outline(sample_html)?;
    println!("Document outline:");
    for heading in outline {
        let indent = "  ".repeat((heading.level - 1) as usize);
        println!("{}H{}: {}", indent, heading.level, heading.text);
    }

    // 6. Content Chunking Demo
    println!("\n✂️ Content Chunking:");
    println!("-------------------");

    let processor = processor::DefaultHtmlProcessor::default();
    let chunks = processor
        .chunk_content(
            &css_result.content,
            processor::ChunkingMode::Sentence { max_sentences: 2 },
        )
        .await?;

    println!("Content split into {} chunks:", chunks.len());
    for (i, chunk) in chunks.iter().take(3).enumerate() {
        println!("Chunk {}: {} chars", i + 1, chunk.content.len());
        println!(
            "  \"{}...\"",
            chunk.content.chars().take(50).collect::<String>()
        );
    }

    // 7. HtmlProcessor Trait Demo
    println!("\n🏭 HtmlProcessor Interface:");
    println!("--------------------------");

    let confidence = processor.confidence_score(sample_html);
    println!("Processing confidence: {:.2}", confidence);
    println!("Processor name: {}", processor.processor_name());

    println!("\n✅ Demo completed successfully!");
    Ok(())
}
