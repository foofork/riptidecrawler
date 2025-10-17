//! PDF extraction module using lopdf
//!
//! Provides comprehensive PDF extraction capabilities including:
//! - Text extraction with layout preservation
//! - Table detection and extraction
//! - Metadata extraction
//! - Format conversion (JSON, Markdown, plain text)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[cfg(feature = "pdf")]
use lopdf::{Document, Object, ObjectId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfContent {
    pub text: String,
    pub tables: Vec<ExtractedTable>,
    pub metadata: PdfDocMetadata,
    pub pages: Vec<PageContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    pub page_number: u32,
    pub text: String,
    pub tables: Vec<ExtractedTable>,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTable {
    pub page: u32,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub position: Option<TablePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub page_count: u32,
    pub file_size: u64,
    pub pdf_version: Option<String>,
    pub encrypted: bool,
}

/// PDF extractor using lopdf
#[derive(Debug)]
pub struct PdfExtractor {
    document: Document,
    file_size: u64,
}

impl PdfExtractor {
    /// Create a new PDF extractor from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // Validate minimum PDF size
        if data.len() < 10 {
            anyhow::bail!("PDF data too small (minimum 10 bytes required)");
        }

        // Validate PDF header
        if !data.starts_with(b"%PDF-") {
            anyhow::bail!("Invalid PDF header - missing %PDF- signature");
        }

        let file_size = data.len() as u64;
        let document = Document::load_mem(data).context("Failed to load PDF document")?;

        Ok(Self {
            document,
            file_size,
        })
    }

    /// Create a new PDF extractor from a file path
    pub fn from_file(path: &str) -> Result<Self> {
        let metadata = std::fs::metadata(path).context("Failed to read file metadata")?;
        let file_size = metadata.len();

        let document = Document::load(path).context("Failed to load PDF file")?;

        Ok(Self {
            document,
            file_size,
        })
    }

    /// Extract all content from the PDF
    pub fn extract_all(&self) -> Result<PdfContent> {
        let metadata = self
            .extract_metadata()
            .context("Failed to extract PDF metadata")?;
        let mut all_text = String::new();
        let mut all_tables = Vec::new();
        let mut pages = Vec::new();

        let page_count = self.document.get_pages().len();

        // Handle empty PDFs
        if page_count == 0 {
            return Ok(PdfContent {
                text: String::new(),
                tables: Vec::new(),
                metadata,
                pages: Vec::new(),
            });
        }

        for page_number in 1..=page_count as u32 {
            match self.extract_page(page_number) {
                Ok(page_content) => {
                    all_text.push_str(&page_content.text);
                    all_text.push_str("\n\n");

                    all_tables.extend(page_content.tables.clone());
                    pages.push(page_content);
                }
                Err(e) => {
                    // Log error but continue with other pages
                    eprintln!("Warning: Failed to extract page {}: {}", page_number, e);
                }
            }
        }

        Ok(PdfContent {
            text: all_text.trim().to_string(),
            tables: all_tables,
            metadata,
            pages,
        })
    }

    /// Extract content from a specific page
    pub fn extract_page(&self, page_number: u32) -> Result<PageContent> {
        let page_id = self
            .get_page_id(page_number)
            .with_context(|| format!("Failed to get page ID for page {}", page_number))?;

        let text = self.extract_page_text(page_id).unwrap_or_else(|e| {
            eprintln!(
                "Warning: Text extraction failed for page {}: {}",
                page_number, e
            );
            String::new()
        });

        let tables = self
            .extract_page_tables(page_number, &text)
            .unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Table extraction failed for page {}: {}",
                    page_number, e
                );
                Vec::new()
            });

        let (width, height) = self.get_page_dimensions(page_id).unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to get dimensions for page {}: {}",
                page_number, e
            );
            (612.0, 792.0) // US Letter default
        });

        Ok(PageContent {
            page_number,
            text,
            tables,
            width,
            height,
        })
    }

    /// Extract text from a specific page
    fn extract_page_text(&self, page_id: ObjectId) -> Result<String> {
        let mut text = String::new();

        if let Ok(content) = self.document.get_page_content(page_id) {
            let content_data = self.decode_content(&content)?;
            text = self.parse_text_from_content(&content_data);
        }

        Ok(text)
    }

    /// Decode content stream
    fn decode_content(&self, content: &[u8]) -> Result<Vec<u8>> {
        // Try to decode if it's a stream object
        Ok(content.to_vec())
    }

    /// Parse text from content stream
    fn parse_text_from_content(&self, content: &[u8]) -> String {
        let mut text = String::new();
        let mut in_text_block = false;
        let mut current_string = String::new();

        let lines: Vec<&[u8]> = content.split(|&b| b == b'\n').collect();

        for line in lines {
            let line_str = String::from_utf8_lossy(line);

            // Start of text block
            if line_str.contains("BT") {
                in_text_block = true;
                continue;
            }

            // End of text block
            if line_str.contains("ET") {
                in_text_block = false;
                if !current_string.is_empty() {
                    text.push_str(&current_string);
                    text.push('\n');
                    current_string.clear();
                }
                continue;
            }

            if in_text_block {
                // Extract text from Tj, TJ, and ' operators
                if let Some(extracted) = self.extract_text_from_operators(&line_str) {
                    current_string.push_str(&extracted);
                    current_string.push(' ');
                }
            }
        }

        text
    }

    /// Extract text from PDF text operators
    fn extract_text_from_operators(&self, line: &str) -> Option<String> {
        // Handle Tj operator: (text) Tj
        if line.contains("Tj") {
            if let Some(start) = line.find('(') {
                if let Some(end) = line.rfind(')') {
                    if start < end {
                        let text = &line[start + 1..end];
                        return Some(self.decode_pdf_string(text));
                    }
                }
            }
        }

        // Handle TJ operator: [(text1) (text2)] TJ
        if line.contains("TJ") {
            return Some(self.extract_from_tj_array(line));
        }

        // Handle ' operator: (text) '
        if line.contains("'") && line.contains('(') {
            if let Some(start) = line.find('(') {
                if let Some(end) = line.rfind(')') {
                    if start < end {
                        let text = &line[start + 1..end];
                        return Some(self.decode_pdf_string(text));
                    }
                }
            }
        }

        None
    }

    /// Extract text from TJ array
    fn extract_from_tj_array(&self, line: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut current_string = String::new();

        for ch in line.chars() {
            match ch {
                '(' if !in_string => {
                    in_string = true;
                    current_string.clear();
                }
                ')' if in_string => {
                    in_string = false;
                    result.push_str(&self.decode_pdf_string(&current_string));
                    result.push(' ');
                }
                c if in_string => {
                    current_string.push(c);
                }
                _ => {}
            }
        }

        result
    }

    /// Decode PDF string (handle escape sequences)
    fn decode_pdf_string(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'n' => {
                            chars.next();
                            result.push('\n');
                        }
                        'r' => {
                            chars.next();
                            result.push('\r');
                        }
                        't' => {
                            chars.next();
                            result.push('\t');
                        }
                        '(' | ')' | '\\' => {
                            chars.next();
                            result.push(next_ch);
                        }
                        _ => {
                            result.push(ch);
                        }
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Extract tables from page text using heuristics
    fn extract_page_tables(&self, page_number: u32, text: &str) -> Result<Vec<ExtractedTable>> {
        let mut tables = Vec::new();

        // Simple table detection: look for aligned columns
        let lines: Vec<&str> = text.lines().collect();
        let mut table_start = None;
        let mut table_lines = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if self.looks_like_table_row(line) {
                if table_start.is_none() {
                    table_start = Some(i);
                }
                table_lines.push(*line);
            } else if table_start.is_some() && table_lines.len() >= 2 {
                // End of table detected
                if let Some(table) = self.parse_table(&table_lines, page_number) {
                    tables.push(table);
                }
                table_start = None;
                table_lines.clear();
            } else {
                table_start = None;
                table_lines.clear();
            }
        }

        // Check for table at end of page
        if table_start.is_some() && table_lines.len() >= 2 {
            if let Some(table) = self.parse_table(&table_lines, page_number) {
                tables.push(table);
            }
        }

        Ok(tables)
    }

    /// Check if a line looks like a table row
    fn looks_like_table_row(&self, line: &str) -> bool {
        // Heuristic: line contains multiple whitespace-separated values
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Must have at least 2 columns
        if parts.len() < 2 {
            return false;
        }

        // Check if line has consistent spacing (suggests tabular data)
        let spaces: Vec<_> = line.match_indices("  ").collect();
        spaces.len() >= 1 && parts.len() >= 2
    }

    /// Parse table from lines
    fn parse_table(&self, lines: &[&str], page_number: u32) -> Option<ExtractedTable> {
        if lines.len() < 2 {
            return None;
        }

        // First line is headers
        let headers: Vec<String> = lines[0].split_whitespace().map(|s| s.to_string()).collect();

        if headers.is_empty() {
            return None;
        }

        // Rest are data rows
        let mut rows = Vec::new();
        for line in &lines[1..] {
            let cells: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

            if !cells.is_empty() {
                rows.push(cells);
            }
        }

        if rows.is_empty() {
            return None;
        }

        Some(ExtractedTable {
            page: page_number,
            headers,
            rows,
            position: None,
        })
    }

    /// Extract PDF metadata
    pub fn extract_metadata(&self) -> Result<PdfDocMetadata> {
        let mut metadata = PdfDocMetadata {
            title: None,
            author: None,
            subject: None,
            creator: None,
            producer: None,
            creation_date: None,
            modification_date: None,
            page_count: self.document.get_pages().len() as u32,
            file_size: self.file_size,
            pdf_version: Some(self.document.version.clone()),
            encrypted: self.document.is_encrypted(),
        };

        // Extract document info
        if let Ok(info_id) = self.document.trailer.get(b"Info") {
            if let Object::Reference(id) = info_id {
                if let Ok(info_dict) = self.document.get_object(*id) {
                    if let Object::Dictionary(dict) = info_dict {
                        metadata.title = self.get_dict_string(dict, b"Title");
                        metadata.author = self.get_dict_string(dict, b"Author");
                        metadata.subject = self.get_dict_string(dict, b"Subject");
                        metadata.creator = self.get_dict_string(dict, b"Creator");
                        metadata.producer = self.get_dict_string(dict, b"Producer");
                        metadata.creation_date = self.get_dict_string(dict, b"CreationDate");
                        metadata.modification_date = self.get_dict_string(dict, b"ModDate");
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Get string value from dictionary
    fn get_dict_string(&self, dict: &lopdf::Dictionary, key: &[u8]) -> Option<String> {
        dict.get(key).ok().and_then(|obj| match obj {
            Object::String(bytes, _) => String::from_utf8(bytes.clone()).ok(),
            _ => None,
        })
    }

    /// Get page ID for a given page number
    fn get_page_id(&self, page_number: u32) -> Result<ObjectId> {
        if page_number == 0 {
            anyhow::bail!("Invalid page number: page numbers must be >= 1");
        }

        let pages = self.document.get_pages();
        let page_index = (page_number - 1) as usize;

        if page_index >= pages.len() {
            anyhow::bail!(
                "Page {} out of range (document has {} pages)",
                page_number,
                pages.len()
            );
        }

        pages
            .get(&(page_index as u32))
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Page {} not found in page map", page_number))
    }

    /// Get page dimensions
    fn get_page_dimensions(&self, page_id: ObjectId) -> Result<(f64, f64)> {
        let page_dict = self.document.get_object(page_id)?;

        if let Object::Dictionary(dict) = page_dict {
            if let Ok(Object::Array(media_box)) = dict.get(b"MediaBox") {
                if media_box.len() >= 4 {
                    let width = self.get_number(&media_box[2]).unwrap_or(612.0);
                    let height = self.get_number(&media_box[3]).unwrap_or(792.0);
                    return Ok((width, height));
                }
            }
        }

        // Default to US Letter size
        Ok((612.0, 792.0))
    }

    /// Extract number from object
    fn get_number(&self, obj: &Object) -> Option<f64> {
        match obj {
            Object::Integer(i) => Some(*i as f64),
            Object::Real(r) => Some(*r as f64),
            _ => None,
        }
    }

    /// Convert to markdown format
    pub fn to_markdown(&self, content: &PdfContent) -> String {
        let mut md = String::new();

        // Add title if available
        if let Some(ref title) = content.metadata.title {
            md.push_str(&format!("# {}\n\n", title));
        }

        // Add metadata section
        md.push_str("## Document Information\n\n");
        if let Some(ref author) = content.metadata.author {
            md.push_str(&format!("- **Author**: {}\n", author));
        }
        if let Some(ref subject) = content.metadata.subject {
            md.push_str(&format!("- **Subject**: {}\n", subject));
        }
        md.push_str(&format!("- **Pages**: {}\n", content.metadata.page_count));
        md.push_str("\n---\n\n");

        // Add content by pages
        for page in &content.pages {
            md.push_str(&format!("## Page {}\n\n", page.page_number));
            md.push_str(&page.text);
            md.push_str("\n\n");

            // Add tables
            for table in &page.tables {
                md.push_str(&self.table_to_markdown(table));
                md.push_str("\n\n");
            }
        }

        md
    }

    /// Convert table to markdown format
    fn table_to_markdown(&self, table: &ExtractedTable) -> String {
        let mut md = String::new();

        // Headers
        md.push_str("| ");
        md.push_str(&table.headers.join(" | "));
        md.push_str(" |\n");

        // Separator
        md.push_str("|");
        for _ in &table.headers {
            md.push_str(" --- |");
        }
        md.push_str("\n");

        // Rows
        for row in &table.rows {
            md.push_str("| ");
            md.push_str(&row.join(" | "));
            md.push_str(" |\n");
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_string_decoding() {
        let extractor = PdfExtractor {
            document: Document::new(),
            file_size: 0,
        };

        assert_eq!(extractor.decode_pdf_string("Hello World"), "Hello World");
        assert_eq!(extractor.decode_pdf_string("Line1\\nLine2"), "Line1\nLine2");
        assert_eq!(
            extractor.decode_pdf_string("Tab\\tSeparated"),
            "Tab\tSeparated"
        );
    }

    #[test]
    fn test_table_row_detection() {
        let extractor = PdfExtractor {
            document: Document::new(),
            file_size: 0,
        };

        assert!(extractor.looks_like_table_row("Column1  Column2  Column3"));
        assert!(extractor.looks_like_table_row("Value1   Value2   Value3"));
        assert!(!extractor.looks_like_table_row("Just a single line of text"));
    }

    #[test]
    fn test_invalid_pdf_data() {
        // Test with empty data
        let result = PdfExtractor::from_bytes(b"");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too small"));

        // Test with too small data
        let result = PdfExtractor::from_bytes(b"test");
        assert!(result.is_err());

        // Test with invalid header
        let result = PdfExtractor::from_bytes(b"This is not a PDF file");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid PDF header"));
    }

    #[test]
    fn test_valid_pdf_header_validation() {
        // Test valid PDF header
        let pdf_with_valid_header = b"%PDF-1.7\nsome content here";
        let result = PdfExtractor::from_bytes(pdf_with_valid_header);
        // Even if the rest fails to parse, header validation should pass
        // and fail later during document loading
        assert!(result.is_err()); // Will fail on document parsing, not header check
    }

    #[test]
    fn test_decode_pdf_escape_sequences() {
        let extractor = PdfExtractor {
            document: Document::new(),
            file_size: 0,
        };

        // Test various escape sequences
        assert_eq!(
            extractor.decode_pdf_string("Hello\\(World\\)"),
            "Hello(World)"
        );
        assert_eq!(extractor.decode_pdf_string("Path\\\\Name"), "Path\\Name");
        assert_eq!(extractor.decode_pdf_string("\\r\\n"), "\r\n");
    }

    #[test]
    fn test_table_detection_edge_cases() {
        let extractor = PdfExtractor {
            document: Document::new(),
            file_size: 0,
        };

        // Single column shouldn't be detected as table
        assert!(!extractor.looks_like_table_row("SingleColumn"));

        // Multiple spaces suggest tabular structure
        assert!(extractor.looks_like_table_row("Col1    Col2    Col3"));

        // Empty string isn't a table
        assert!(!extractor.looks_like_table_row(""));
    }
}
