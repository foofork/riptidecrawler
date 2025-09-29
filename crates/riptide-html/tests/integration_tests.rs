//! Integration tests for riptide-html crate

use riptide_html::*;
use riptide_html::css_extraction::default_selectors_simple;
use std::collections::HashMap;

#[tokio::test]
async fn test_css_extraction_integration() {
    let html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Test Article</title>
                <meta name="description" content="This is a test article about HTML extraction">
                <meta property="og:title" content="OG Title">
            </head>
            <body>
                <article>
                    <h1>Main Article Title</h1>
                    <div class="author">John Doe</div>
                    <time datetime="2023-10-01">October 1, 2023</time>
                    <div class="content">
                        <p>This is the first paragraph of the article.</p>
                        <p>This is the second paragraph with more content.</p>
                    </div>
                    <div class="tags">
                        <span class="tag">technology</span>
                        <span class="tag">web-scraping</span>
                    </div>
                </article>
            </body>
        </html>
    "#;

    // Test default selectors
    let result = css_extract_default(html, "https://example.com/article").await.unwrap();

    assert_eq!(result.title, "Test Article");
    assert!(result.content.contains("first paragraph"));
    assert!(result.content.contains("second paragraph"));
    assert_eq!(result.summary, Some("This is a test article about HTML extraction".to_string()));
    assert_eq!(result.strategy_used, "css_json");
    assert!(result.extraction_confidence > 0.0);

    // Test custom selectors
    let mut custom_selectors = HashMap::new();
    custom_selectors.insert("title".to_string(), "h1".to_string());
    custom_selectors.insert("author".to_string(), ".author".to_string());
    custom_selectors.insert("date".to_string(), "time".to_string());
    custom_selectors.insert("content".to_string(), ".content p".to_string());
    custom_selectors.insert("tags".to_string(), ".tag".to_string());

    let custom_result = css_extract(html, "https://example.com/article", &custom_selectors).await.unwrap();

    assert_eq!(custom_result.title, "Main Article Title");
    assert!(custom_result.content.contains("first paragraph"));
    assert!(custom_result.content.contains("second paragraph"));
}

#[tokio::test]
async fn test_regex_extraction_integration() {
    let html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Contact Information</title>
            </head>
            <body>
                <div class="contact">
                    <p>Email us at: john.doe@example.com or support@company.org</p>
                    <p>Call us at: (555) 123-4567 or 555-987-6543</p>
                    <p>Visit our website: https://www.example.com</p>
                    <p>Our prices start at $29.99 and go up to $199.99</p>
                    <p>Published on: 2023-10-01</p>
                </div>
            </body>
        </html>
    "#;

    // Test default patterns
    let result = regex_extract(html, "https://example.com", &default_patterns()).await.unwrap();

    assert_eq!(result.title, "Contact Information");
    assert!(result.content.contains("john.doe@example.com"));
    assert!(result.content.contains("(555) 123-4567"));
    assert!(result.content.contains("https://www.example.com"));
    assert!(result.content.contains("$29.99"));

    // Test contact patterns specifically
    let contact_result = regex_extraction::extract_contacts(html, "https://example.com").await.unwrap();
    assert!(contact_result.content.contains("john.doe@example.com"));

    // Test financial patterns specifically
    let financial_result = regex_extraction::extract_financial(html, "https://example.com").await.unwrap();
    assert!(financial_result.content.contains("$29.99"));
}

#[tokio::test]
async fn test_table_extraction_integration() {
    let html = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <table id="employees" class="data-table">
                    <caption>Employee Information</caption>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Department</th>
                            <th>Salary</th>
                            <th>Start Date</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>John Smith</td>
                            <td>Engineering</td>
                            <td>$75,000</td>
                            <td>2020-01-15</td>
                        </tr>
                        <tr>
                            <td>Jane Johnson</td>
                            <td>Marketing</td>
                            <td>$65,000</td>
                            <td>2019-03-22</td>
                        </tr>
                        <tr>
                            <td>Bob Wilson</td>
                            <td>Sales</td>
                            <td>$70,000</td>
                            <td>2021-06-01</td>
                        </tr>
                    </tbody>
                </table>

                <table class="small-table">
                    <tr><td>Small</td></tr>
                    <tr><td>Table</td></tr>
                </table>
            </body>
        </html>
    "#;

    // Test extracting all tables
    let tables = dom_utils::extract_tables(html, processor::TableExtractionMode::All).await.unwrap();
    assert_eq!(tables.len(), 2);

    let main_table = &tables[0];
    assert_eq!(main_table.headers, vec!["Name", "Department", "Salary", "Start Date"]);
    assert_eq!(main_table.rows.len(), 3);
    assert_eq!(main_table.rows[0], vec!["John Smith", "Engineering", "$75,000", "2020-01-15"]);
    assert_eq!(main_table.caption, Some("Employee Information".to_string()));
    assert!(main_table.metadata.contains_key("id"));
    assert_eq!(main_table.metadata.get("id"), Some(&"employees".to_string()));

    // Test extracting only tables with headers
    let header_tables = dom_utils::extract_tables(html, processor::TableExtractionMode::WithHeaders).await.unwrap();
    assert_eq!(header_tables.len(), 1);

    // Test minimum size filtering
    let min_size_tables = dom_utils::extract_tables(
        html,
        processor::TableExtractionMode::MinSize { min_rows: 3, min_cols: 3 }
    ).await.unwrap();
    assert_eq!(min_size_tables.len(), 1);

    // Test CSS selector filtering
    let id_tables = dom_utils::extract_tables(
        html,
        processor::TableExtractionMode::BySelector("#employees".to_string())
    ).await.unwrap();
    assert_eq!(id_tables.len(), 1);
}

#[tokio::test]
async fn test_dom_utils_integration() {
    let html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>DOM Utils Test</title>
            </head>
            <body>
                <div id="main" class="container highlight">
                    <h1>Main Title</h1>
                    <h2>Section 1</h2>
                    <p>First paragraph content.</p>
                    <h3>Subsection 1.1</h3>
                    <p>Second paragraph content.</p>
                    <h2>Section 2</h2>
                    <p>Third paragraph content.</p>
                </div>

                <img src="image1.jpg" alt="First image" width="100" height="200">
                <img src="image2.png" alt="Second image" title="Image title">

                <a href="https://example.com" title="Example">External Link</a>
                <a href="/internal" target="_blank">Internal Link</a>
            </body>
        </html>
    "#;

    // Test element traversal
    let div_elements = dom_utils::traverse_elements(html, "div").unwrap();
    assert_eq!(div_elements.len(), 1);
    let main_div = &div_elements[0];
    assert_eq!(main_div.tag, "div");
    assert_eq!(main_div.id, Some("main".to_string()));
    assert!(main_div.classes.contains(&"container".to_string()));
    assert!(main_div.classes.contains(&"highlight".to_string()));

    // Test text extraction
    let paragraph_texts = dom_utils::extract_text_content(html, "p").unwrap();
    assert_eq!(paragraph_texts.len(), 3);
    assert!(paragraph_texts.contains(&"First paragraph content.".to_string()));
    assert!(paragraph_texts.contains(&"Second paragraph content.".to_string()));
    assert!(paragraph_texts.contains(&"Third paragraph content.".to_string()));

    // Test document outline
    let outline = dom_utils::get_document_outline(html).unwrap();
    assert_eq!(outline.len(), 4);
    assert_eq!(outline[0].level, 1);
    assert_eq!(outline[0].text, "Main Title");
    assert_eq!(outline[1].level, 2);
    assert_eq!(outline[1].text, "Section 1");
    assert_eq!(outline[2].level, 3);
    assert_eq!(outline[2].text, "Subsection 1.1");

    // Test image extraction
    let images = dom_utils::extract_images(html).unwrap();
    assert_eq!(images.len(), 2);
    assert_eq!(images[0].src, "image1.jpg");
    assert_eq!(images[0].alt, "First image");
    assert_eq!(images[0].width, Some(100));

    // Test link extraction
    let links = dom_utils::extract_links(html).unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].href, "https://example.com");
    assert_eq!(links[0].text, "External Link");
    assert_eq!(links[1].href, "/internal");
    assert_eq!(links[1].target, Some("_blank".to_string()));
}

#[tokio::test]
async fn test_html_processor_trait() {
    let html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>Processor Test</title>
                <meta name="description" content="Testing the HTML processor">
            </head>
            <body>
                <article>
                    <h1>Article Title</h1>
                    <p>First paragraph of content.</p>
                    <p>Second paragraph of content.</p>
                </article>

                <table>
                    <tr><th>Header 1</th><th>Header 2</th></tr>
                    <tr><td>Data 1</td><td>Data 2</td></tr>
                </table>
            </body>
        </html>
    "#;

    let processor = processor::DefaultHtmlProcessor::default();

    // Test CSS extraction
    let css_result = processor.extract_with_css(
        html,
        "https://example.com",
        &default_selectors_simple()
    ).await.unwrap();

    assert_eq!(css_result.title, "Processor Test");
    assert!(css_result.content.contains("First paragraph"));

    // Test regex extraction
    let regex_result = processor.extract_with_regex(
        html,
        "https://example.com",
        &default_patterns()
    ).await.unwrap();

    assert_eq!(regex_result.title, "Processor Test");

    // Test table extraction
    let tables = processor.extract_tables(html, processor::TableExtractionMode::All).await.unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].headers, vec!["Header 1", "Header 2"]);

    // Test content chunking
    let content = "This is a test sentence. This is another sentence. And this is a third sentence.";
    let chunks = processor.chunk_content(
        content,
        processor::ChunkingMode::Sentence { max_sentences: 2 }
    ).await.unwrap();

    assert!(!chunks.is_empty());
    assert!(chunks[0].content.contains("This is a test sentence"));

    // Test confidence score
    let confidence = processor.confidence_score(html);
    assert!(confidence > 0.0 && confidence <= 1.0);

    // Test processor name
    assert_eq!(processor.processor_name(), "default_html_processor");
}

#[tokio::test]
async fn test_chunking_modes() {
    let content = "This is sentence one. This is sentence two! This is sentence three?\n\nThis is a new paragraph. Another sentence here.\n\nFinal paragraph with content.";
    let processor = processor::DefaultHtmlProcessor::default();

    // Test fixed size chunking
    let fixed_chunks = processor.chunk_content(
        content,
        processor::ChunkingMode::FixedSize { size: 50, overlap: 10 }
    ).await.unwrap();
    assert!(fixed_chunks.len() > 1);

    // Test sentence chunking
    let sentence_chunks = processor.chunk_content(
        content,
        processor::ChunkingMode::Sentence { max_sentences: 2 }
    ).await.unwrap();
    assert!(sentence_chunks.len() >= 2);

    // Test paragraph chunking
    let paragraph_chunks = processor.chunk_content(
        content,
        processor::ChunkingMode::Paragraph { max_paragraphs: 1 }
    ).await.unwrap();
    assert_eq!(paragraph_chunks.len(), 3);

    // Test token chunking
    let token_chunks = processor.chunk_content(
        content,
        processor::ChunkingMode::Token { max_tokens: 10, overlap: 2 }
    ).await.unwrap();
    assert!(token_chunks.len() > 1);

    // Test semantic chunking (basic implementation)
    let semantic_chunks = processor.chunk_content(
        content,
        processor::ChunkingMode::Semantic { similarity_threshold: 0.8 }
    ).await.unwrap();
    assert!(!semantic_chunks.is_empty());
}

#[test]
fn test_extraction_quality() {
    let quality = ExtractionQuality {
        content_length: 1000,
        title_quality: 0.9,
        content_quality: 0.8,
        structure_score: 0.7,
        metadata_completeness: 0.6,
    };

    let overall = quality.overall_score();
    assert!((overall - 0.75).abs() < 0.01); // (0.9 + 0.8 + 0.7 + 0.6) / 4 = 0.75
}