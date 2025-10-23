//! Helper functions for PDF operations
//!
//! This module provides utility functions for PDF loading, parsing, and output handling.
//! These functions are used by both the CLI and library consumers.

use anyhow::{Context, Result};

#[cfg(feature = "pdf")]
use crate::{PdfContent, PdfDocMetadata, PdfExtractor};

/// Load PDF from file path or URL
///
/// # Arguments
/// * `input` - File path or HTTP(S) URL to PDF
///
/// # Returns
/// * `Result<Vec<u8>>` - PDF file contents as bytes
///
/// # Examples
/// ```no_run
/// use riptide_pdf::helpers::load_pdf;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let pdf_bytes = load_pdf("document.pdf").await?;
///     Ok(())
/// }
/// ```
pub async fn load_pdf(input: &str) -> Result<Vec<u8>> {
    if input.starts_with("http://") || input.starts_with("https://") {
        // Download PDF from URL
        let response = reqwest::get(input)
            .await
            .context("Failed to download PDF from URL")?;

        let bytes = response
            .bytes()
            .await
            .context("Failed to read PDF bytes from response")?;

        Ok(bytes.to_vec())
    } else {
        // Read from local file
        std::fs::read(input).context("Failed to read PDF file")
    }
}

/// Extract metadata from PDF bytes
///
/// # Arguments
/// * `pdf_data` - PDF file contents as bytes
///
/// # Returns
/// * `Result<PdfDocMetadata>` - Extracted PDF metadata
///
/// # Examples
/// ```no_run
/// use riptide_pdf::helpers::extract_metadata;
///
/// fn main() -> anyhow::Result<()> {
///     let pdf_bytes = std::fs::read("document.pdf")?;
///     let metadata = extract_metadata(&pdf_bytes)?;
///     println!("Pages: {}", metadata.page_count);
///     Ok(())
/// }
/// ```
#[cfg(feature = "pdf")]
pub fn extract_metadata(pdf_data: &[u8]) -> Result<PdfDocMetadata> {
    let extractor = PdfExtractor::from_bytes(pdf_data)?;
    extractor.extract_metadata()
}

/// Extract metadata from PDF bytes (stub for non-pdf feature)
#[cfg(not(feature = "pdf"))]
pub fn extract_metadata(_pdf_data: &[u8]) -> Result<serde_json::Value> {
    anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
}

/// Extract full content from PDF
///
/// # Arguments
/// * `pdf_data` - PDF file contents as bytes
///
/// # Returns
/// * `Result<PdfContent>` - Extracted PDF content including text, tables, and metadata
///
/// # Examples
/// ```no_run
/// use riptide_pdf::helpers::extract_full_content;
///
/// fn main() -> anyhow::Result<()> {
///     let pdf_bytes = std::fs::read("document.pdf")?;
///     let content = extract_full_content(&pdf_bytes)?;
///     println!("Text: {}", content.text);
///     Ok(())
/// }
/// ```
#[cfg(feature = "pdf")]
pub fn extract_full_content(pdf_data: &[u8]) -> Result<PdfContent> {
    let extractor = PdfExtractor::from_bytes(pdf_data)?;
    extractor.extract_all()
}

/// Extract full content from PDF (stub for non-pdf feature)
#[cfg(not(feature = "pdf"))]
pub fn extract_full_content(_pdf_data: &[u8]) -> Result<serde_json::Value> {
    anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
}

/// Convert PDF to markdown
///
/// # Arguments
/// * `pdf_data` - PDF file contents as bytes
///
/// # Returns
/// * `Result<String>` - PDF content converted to markdown format
///
/// # Examples
/// ```no_run
/// use riptide_pdf::helpers::convert_to_markdown;
///
/// fn main() -> anyhow::Result<()> {
///     let pdf_bytes = std::fs::read("document.pdf")?;
///     let markdown = convert_to_markdown(&pdf_bytes)?;
///     println!("{}", markdown);
///     Ok(())
/// }
/// ```
#[cfg(feature = "pdf")]
pub fn convert_to_markdown(pdf_data: &[u8]) -> Result<String> {
    let extractor = PdfExtractor::from_bytes(pdf_data)?;
    let content = extractor.extract_all()?;
    Ok(extractor.to_markdown(&content))
}

/// Convert PDF to markdown (stub for non-pdf feature)
#[cfg(not(feature = "pdf"))]
pub fn convert_to_markdown(_pdf_data: &[u8]) -> Result<String> {
    anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
}

/// Write output to file or stdout
///
/// # Arguments
/// * `content` - Content to write
/// * `output_path` - Optional file path. If None, writes to stdout
///
/// # Examples
/// ```no_run
/// use riptide_pdf::helpers::write_output;
///
/// fn main() -> anyhow::Result<()> {
///     write_output("Hello, world!", Some("output.txt"))?;
///     write_output("To stdout", None)?;
///     Ok(())
/// }
/// ```
pub fn write_output(content: &str, output_path: Option<&str>) -> Result<()> {
    if let Some(path) = output_path {
        std::fs::write(path, content).context("Failed to write output file")?;
        println!("Output written to: {}", path);
    } else {
        println!("{}", content);
    }
    Ok(())
}

/// Parse page range string (e.g., "1-5,10,15-20") into a vector of page numbers
///
/// # Arguments
/// * `range` - Page range string (e.g., "1-5,10,15-20")
///
/// # Returns
/// * `Result<Vec<u32>>` - Sorted, deduplicated vector of page numbers
///
/// # Examples
/// ```
/// use riptide_pdf::helpers::parse_page_range;
///
/// let pages = parse_page_range("1-3,5,7-9").unwrap();
/// assert_eq!(pages, vec![1, 2, 3, 5, 7, 8, 9]);
/// ```
pub fn parse_page_range(range: &str) -> Result<Vec<u32>> {
    let mut pages = Vec::new();

    for part in range.split(',') {
        let part = part.trim();
        if part.contains('-') {
            // Range like "1-5"
            let parts: Vec<&str> = part.split('-').collect();
            if parts.len() != 2 {
                anyhow::bail!("Invalid page range: {}", part);
            }

            let start: u32 = parts[0]
                .trim()
                .parse()
                .context(format!("Invalid page number: {}", parts[0]))?;
            let end: u32 = parts[1]
                .trim()
                .parse()
                .context(format!("Invalid page number: {}", parts[1]))?;

            if start > end {
                anyhow::bail!("Invalid range: {} - start must be <= end", part);
            }

            pages.extend(start..=end);
        } else {
            // Single page
            let page: u32 = part
                .parse()
                .context(format!("Invalid page number: {}", part))?;
            pages.push(page);
        }
    }

    pages.sort();
    pages.dedup();
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_page_range_single() {
        assert_eq!(parse_page_range("1").unwrap(), vec![1]);
    }

    #[test]
    fn test_parse_page_range_simple() {
        assert_eq!(parse_page_range("1-3").unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_page_range_list() {
        assert_eq!(parse_page_range("1,3,5").unwrap(), vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_page_range_complex() {
        assert_eq!(
            parse_page_range("1-3,5,7-9").unwrap(),
            vec![1, 2, 3, 5, 7, 8, 9]
        );
    }

    #[test]
    fn test_parse_page_range_invalid() {
        assert!(parse_page_range("5-1").is_err()); // Invalid range
    }

    #[test]
    fn test_parse_page_range_deduplication() {
        assert_eq!(parse_page_range("1,1,2,2,3").unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_page_range_overlapping() {
        assert_eq!(
            parse_page_range("1-5,3-7").unwrap(),
            vec![1, 2, 3, 4, 5, 6, 7]
        );
    }
}
