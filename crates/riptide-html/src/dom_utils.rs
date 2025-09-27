//! DOM traversal and manipulation utilities
//!
//! This module provides utilities for traversing and manipulating HTML DOM structures,
//! including table extraction, text content extraction, and element information gathering.

use anyhow::Result;
use scraper::{Html, Selector, ElementRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::processor::{TableData, TableExtractionMode};

/// DOM traverser for navigating HTML structures
pub struct DomTraverser {
    document: Html,
}

/// Information about an HTML element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    /// Element tag name
    pub tag: String,
    /// Element ID attribute
    pub id: Option<String>,
    /// Element classes
    pub classes: Vec<String>,
    /// Element attributes
    pub attributes: HashMap<String, String>,
    /// Element text content
    pub text: String,
    /// Child elements count
    pub children_count: usize,
    /// Element depth in DOM tree
    pub depth: usize,
}

impl DomTraverser {
    /// Create a new DOM traverser from HTML
    pub fn new(html: &str) -> Self {
        Self {
            document: Html::parse_document(html),
        }
    }

    /// Get information about all elements matching a selector
    pub fn get_elements_info(&self, selector_str: &str) -> Result<Vec<ElementInfo>> {
        let selector = Selector::parse(selector_str)
            .map_err(|e| anyhow::anyhow!("Invalid selector: {}", e))?;

        let mut elements = Vec::new();
        for element in self.document.select(&selector) {
            elements.push(self.element_to_info(element, 0));
        }

        Ok(elements)
    }

    /// Convert an element reference to ElementInfo
    fn element_to_info(&self, element: ElementRef, depth: usize) -> ElementInfo {
        let value = element.value();

        ElementInfo {
            tag: value.name().to_string(),
            id: value.id().map(|s| s.to_string()),
            classes: value.classes().map(|s| s.to_string()).collect(),
            attributes: value.attrs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            text: element.text().collect::<Vec<_>>().join(" ").trim().to_string(),
            children_count: element.children().count(),
            depth,
        }
    }

    /// Find all tables in the document
    pub fn find_tables(&self) -> Result<Vec<ElementInfo>> {
        self.get_elements_info("table")
    }

    /// Extract text content from elements matching a selector
    pub fn extract_text(&self, selector_str: &str) -> Result<Vec<String>> {
        let selector = Selector::parse(selector_str)
            .map_err(|e| anyhow::anyhow!("Invalid selector: {}", e))?;

        let texts = self.document
            .select(&selector)
            .map(|element| element.text().collect::<Vec<_>>().join(" ").trim().to_string())
            .filter(|text| !text.is_empty())
            .collect();

        Ok(texts)
    }

    /// Get document statistics
    pub fn get_stats(&self) -> DocumentStats {
        let selector = Selector::parse("*").unwrap();
        let all_elements = self.document.select(&selector);
        let mut tag_counts = HashMap::new();
        let mut total_text_length = 0;

        for element in all_elements {
            let tag = element.value().name();
            *tag_counts.entry(tag.to_string()).or_insert(0) += 1;

            let text = element.text().collect::<String>();
            total_text_length += text.len();
        }

        let has_tables = tag_counts.contains_key("table");
        let has_forms = tag_counts.contains_key("form");
        let has_images = tag_counts.contains_key("img");
        let total_elements = tag_counts.values().sum();

        DocumentStats {
            total_elements,
            tag_counts,
            total_text_length,
            has_tables,
            has_forms,
            has_images,
        }
    }
}

/// Document statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStats {
    pub total_elements: usize,
    pub tag_counts: HashMap<String, usize>,
    pub total_text_length: usize,
    pub has_tables: bool,
    pub has_forms: bool,
    pub has_images: bool,
}

/// Extract elements matching a selector and return their information
pub fn traverse_elements(html: &str, selector_str: &str) -> Result<Vec<ElementInfo>> {
    let traverser = DomTraverser::new(html);
    traverser.get_elements_info(selector_str)
}

/// Extract text content from elements matching a selector
pub fn extract_text_content(html: &str, selector_str: &str) -> Result<Vec<String>> {
    let traverser = DomTraverser::new(html);
    traverser.extract_text(selector_str)
}

/// Find all tables in the document
pub fn find_tables(html: &str) -> Result<Vec<ElementInfo>> {
    let traverser = DomTraverser::new(html);
    traverser.find_tables()
}

/// Extract table data from HTML
pub async fn extract_tables(html: &str, mode: TableExtractionMode) -> Result<Vec<TableData>> {
    let document = Html::parse_document(html);
    let mut tables = Vec::new();

    let table_selector = match &mode {
        TableExtractionMode::All => "table",
        TableExtractionMode::WithHeaders => "table:has(th), table:has(thead)",
        TableExtractionMode::BySelector(selector) => selector,
        TableExtractionMode::MinSize { .. } => "table",
    };

    let selector = Selector::parse(table_selector)
        .map_err(|e| anyhow::anyhow!("Invalid table selector: {}", e))?;

    for table_element in document.select(&selector) {
        let table_data = extract_single_table(table_element)?;

        // Apply size filtering if specified
        if let TableExtractionMode::MinSize { min_rows, min_cols } = &mode {
            if table_data.rows.len() < *min_rows {
                continue;
            }
            if table_data.rows.iter().any(|row| row.len() < *min_cols) {
                continue;
            }
        }

        tables.push(table_data);
    }

    Ok(tables)
}

/// Extract data from a single table element
fn extract_single_table(table_element: ElementRef) -> Result<TableData> {
    let mut headers = Vec::new();
    let mut rows = Vec::new();
    let mut caption = None;
    let mut metadata = HashMap::new();

    // Extract table attributes
    let value = table_element.value();
    for (attr, val) in value.attrs() {
        metadata.insert(attr.to_string(), val.to_string());
    }

    // Extract caption
    if let Ok(caption_selector) = Selector::parse("caption") {
        if let Some(caption_element) = table_element.select(&caption_selector).next() {
            caption = Some(caption_element.text().collect::<String>().trim().to_string());
        }
    }

    // Extract headers from thead or first row with th elements
    if let Ok(header_selector) = Selector::parse("thead th, tr:first-child th") {
        headers = table_element
            .select(&header_selector)
            .map(|th| th.text().collect::<String>().trim().to_string())
            .collect();
    }

    // Extract rows from tbody or all tr elements
    if let Ok(row_selector) = Selector::parse("tbody tr, tr") {
        for row_element in table_element.select(&row_selector) {
            // Skip header rows if we already extracted headers
            if !headers.is_empty() {
                if let Ok(th_selector) = Selector::parse("th") {
                    if row_element.select(&th_selector).next().is_some() {
                        continue;
                    }
                }
            }

            if let Ok(cell_selector) = Selector::parse("td, th") {
                let row_data: Vec<String> = row_element
                    .select(&cell_selector)
                    .map(|cell| cell.text().collect::<String>().trim().to_string())
                    .collect();

                if !row_data.is_empty() {
                    rows.push(row_data);
                }
            }
        }
    }

    Ok(TableData {
        headers,
        rows,
        caption,
        metadata,
    })
}

/// Extract all images from HTML
pub fn extract_images(html: &str) -> Result<Vec<ImageInfo>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("img")
        .map_err(|e| anyhow::anyhow!("Invalid img selector: {}", e))?;

    let images = document
        .select(&selector)
        .map(|img| {
            let value = img.value();
            ImageInfo {
                src: value.attr("src").unwrap_or("").to_string(),
                alt: value.attr("alt").unwrap_or("").to_string(),
                title: value.attr("title").map(|s| s.to_string()),
                width: value.attr("width").and_then(|w| w.parse().ok()),
                height: value.attr("height").and_then(|h| h.parse().ok()),
                attributes: value.attrs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        })
        .collect();

    Ok(images)
}

/// Image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub src: String,
    pub alt: String,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub attributes: HashMap<String, String>,
}

/// Extract all links from HTML
pub fn extract_links(html: &str) -> Result<Vec<LinkInfo>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[href]")
        .map_err(|e| anyhow::anyhow!("Invalid link selector: {}", e))?;

    let links = document
        .select(&selector)
        .map(|link| {
            let value = link.value();
            LinkInfo {
                href: value.attr("href").unwrap_or("").to_string(),
                text: link.text().collect::<String>().trim().to_string(),
                title: value.attr("title").map(|s| s.to_string()),
                target: value.attr("target").map(|s| s.to_string()),
                rel: value.attr("rel").map(|s| s.to_string()),
                attributes: value.attrs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        })
        .collect();

    Ok(links)
}

/// Link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
    pub target: Option<String>,
    pub rel: Option<String>,
    pub attributes: HashMap<String, String>,
}

/// Clean HTML by removing unwanted elements
pub fn clean_html(html: &str, remove_selectors: &[&str]) -> Result<String> {
    let mut document = Html::parse_document(html);

    // Note: scraper doesn't support mutation, so we'll use string replacement
    let mut cleaned = html.to_string();

    for selector_str in remove_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                // Remove the entire element including its content
                let element_html = element.html();
                cleaned = cleaned.replace(&element_html, "");
            }
        }
        // Re-parse after each removal
        document = Html::parse_document(&cleaned);
    }

    Ok(cleaned)
}

/// Get document outline (headings structure)
pub fn get_document_outline(html: &str) -> Result<Vec<HeadingInfo>> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("h1, h2, h3, h4, h5, h6")
        .map_err(|e| anyhow::anyhow!("Invalid heading selector: {}", e))?;

    let headings = document
        .select(&selector)
        .map(|heading| {
            let value = heading.value();
            let level = match value.name() {
                "h1" => 1,
                "h2" => 2,
                "h3" => 3,
                "h4" => 4,
                "h5" => 5,
                "h6" => 6,
                _ => 1,
            };

            HeadingInfo {
                level,
                text: heading.text().collect::<String>().trim().to_string(),
                id: value.attr("id").map(|s| s.to_string()),
                attributes: value.attrs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        })
        .collect();

    Ok(headings)
}

/// Heading information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingInfo {
    pub level: u8,
    pub text: String,
    pub id: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_table_extraction() {
        let html = r#"
            <html>
                <body>
                    <table>
                        <caption>Test Table</caption>
                        <thead>
                            <tr><th>Name</th><th>Age</th><th>City</th></tr>
                        </thead>
                        <tbody>
                            <tr><td>John</td><td>30</td><td>New York</td></tr>
                            <tr><td>Jane</td><td>25</td><td>Los Angeles</td></tr>
                        </tbody>
                    </table>
                </body>
            </html>
        "#;

        let tables = extract_tables(html, TableExtractionMode::All).await.unwrap();

        assert_eq!(tables.len(), 1);
        let table = &tables[0];

        assert_eq!(table.headers, vec!["Name", "Age", "City"]);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["John", "30", "New York"]);
        assert_eq!(table.caption, Some("Test Table".to_string()));
    }

    #[test]
    fn test_dom_traversal() {
        let html = r#"
            <html>
                <body>
                    <div id="main" class="container">
                        <p>First paragraph</p>
                        <p>Second paragraph</p>
                    </div>
                </body>
            </html>
        "#;

        let elements = traverse_elements(html, "div").unwrap();

        assert_eq!(elements.len(), 1);
        let div = &elements[0];

        assert_eq!(div.tag, "div");
        assert_eq!(div.id, Some("main".to_string()));
        assert!(div.classes.contains(&"container".to_string()));
        assert_eq!(div.children_count, 5); // div contains: whitespace, p, whitespace, p, whitespace
    }

    #[test]
    fn test_text_extraction() {
        let html = r#"
            <html>
                <body>
                    <p>First paragraph</p>
                    <p>Second paragraph</p>
                    <div>Not a paragraph</div>
                </body>
            </html>
        "#;

        let texts = extract_text_content(html, "p").unwrap();

        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0], "First paragraph");
        assert_eq!(texts[1], "Second paragraph");
    }

    #[test]
    fn test_image_extraction() {
        let html = r#"
            <html>
                <body>
                    <img src="image1.jpg" alt="First image" width="100" height="200">
                    <img src="image2.png" alt="Second image" title="Image title">
                </body>
            </html>
        "#;

        let images = extract_images(html).unwrap();

        assert_eq!(images.len(), 2);
        assert_eq!(images[0].src, "image1.jpg");
        assert_eq!(images[0].alt, "First image");
        assert_eq!(images[0].width, Some(100));
        assert_eq!(images[0].height, Some(200));
    }

    #[test]
    fn test_link_extraction() {
        let html = r#"
            <html>
                <body>
                    <a href="https://example.com" title="Example">Example Link</a>
                    <a href="/internal" target="_blank">Internal Link</a>
                </body>
            </html>
        "#;

        let links = extract_links(html).unwrap();

        assert_eq!(links.len(), 2);
        assert_eq!(links[0].href, "https://example.com");
        assert_eq!(links[0].text, "Example Link");
        assert_eq!(links[0].title, Some("Example".to_string()));
    }

    #[test]
    fn test_document_outline() {
        let html = r#"
            <html>
                <body>
                    <h1>Main Title</h1>
                    <h2>Section 1</h2>
                    <h3>Subsection 1.1</h3>
                    <h2>Section 2</h2>
                </body>
            </html>
        "#;

        let outline = get_document_outline(html).unwrap();

        assert_eq!(outline.len(), 4);
        assert_eq!(outline[0].level, 1);
        assert_eq!(outline[0].text, "Main Title");
        assert_eq!(outline[1].level, 2);
        assert_eq!(outline[1].text, "Section 1");
        assert_eq!(outline[2].level, 3);
        assert_eq!(outline[2].text, "Subsection 1.1");
    }
}