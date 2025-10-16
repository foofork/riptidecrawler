//! PDF implementation helpers for CLI
//!
//! Provides utility functions for PDF loading, parsing, and output handling.

use anyhow::{Context, Result};

#[cfg(feature = "pdf")]
use riptide_pdf::PdfExtractor;

/// Load PDF from file path or URL
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
#[cfg(feature = "pdf")]
pub fn extract_metadata(pdf_data: &[u8]) -> Result<riptide_pdf::PdfDocMetadata> {
    let extractor = PdfExtractor::from_bytes(pdf_data)?;
    extractor.extract_metadata()
}

#[cfg(not(feature = "pdf"))]
pub fn extract_metadata(_pdf_data: &[u8]) -> Result<crate::commands::pdf::PdfMetadata> {
    anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
}

/// Extract full content from PDF
#[cfg(feature = "pdf")]
pub fn extract_full_content(pdf_data: &[u8]) -> Result<riptide_pdf::PdfContent> {
    let extractor = PdfExtractor::from_bytes(pdf_data)?;
    extractor.extract_all()
}

#[cfg(not(feature = "pdf"))]
pub fn extract_full_content(_pdf_data: &[u8]) -> Result<crate::commands::pdf::PdfExtractResult> {
    anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
}

/// Convert PDF to markdown
#[cfg(feature = "pdf")]
pub fn convert_to_markdown(pdf_data: &[u8]) -> Result<String> {
    let extractor = PdfExtractor::from_bytes(pdf_data)?;
    let content = extractor.extract_all()?;
    Ok(extractor.to_markdown(&content))
}

#[cfg(not(feature = "pdf"))]
pub fn convert_to_markdown(_pdf_data: &[u8]) -> Result<String> {
    anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
}

/// Write output to file or stdout
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
    fn test_parse_page_range() {
        assert_eq!(parse_page_range("1").unwrap(), vec![1]);
        assert_eq!(parse_page_range("1-3").unwrap(), vec![1, 2, 3]);
        assert_eq!(parse_page_range("1,3,5").unwrap(), vec![1, 3, 5]);
        assert_eq!(
            parse_page_range("1-3,5,7-9").unwrap(),
            vec![1, 2, 3, 5, 7, 8, 9]
        );
        assert!(parse_page_range("5-1").is_err()); // Invalid range
    }
}
