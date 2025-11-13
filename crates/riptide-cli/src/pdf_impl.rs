//! Implementation for PDF commands using riptide-pdf crate
//!
//! This module implements the actual PDF processing functionality,
//! delegating to the riptide-pdf crate for extraction and conversion.

use anyhow::{Context, Result};
use std::fs;

#[cfg(feature = "riptide-pdf")]
use riptide_pdf::PdfExtractor;

/// Download PDF from URL
pub async fn download_pdf(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to download PDF")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download PDF: HTTP {}", response.status());
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read PDF response")?;

    Ok(bytes.to_vec())
}

/// Load PDF from file or URL
pub async fn load_pdf(input: &str) -> Result<Vec<u8>> {
    if input.starts_with("http://") || input.starts_with("https://") {
        download_pdf(input).await
    } else {
        fs::read(input).context("Failed to read PDF file")
    }
}

/// Extract text from PDF
#[cfg(feature = "riptide-pdf")]
pub fn extract_text(data: &[u8]) -> Result<String> {
    let extractor = PdfExtractor::from_bytes(data).context("Failed to create PDF extractor")?;

    let content = extractor
        .extract_all()
        .context("Failed to extract PDF content")?;

    Ok(content.text)
}

#[cfg(not(feature = "riptide-pdf"))]
pub fn extract_text(_data: &[u8]) -> Result<String> {
    anyhow::bail!("PDF processing feature not enabled. Rebuild with --features riptide-pdf")
}

/// Extract tables from PDF
#[cfg(feature = "riptide-pdf")]
pub fn extract_tables(data: &[u8]) -> Result<Vec<riptide_pdf::ExtractedTable>> {
    let extractor = PdfExtractor::from_bytes(data).context("Failed to create PDF extractor")?;

    let content = extractor
        .extract_all()
        .context("Failed to extract PDF content")?;

    Ok(content.tables)
}

#[cfg(not(feature = "riptide-pdf"))]
pub fn extract_tables(_data: &[u8]) -> Result<Vec<serde_json::Value>> {
    anyhow::bail!("PDF processing feature not enabled. Rebuild with --features riptide-pdf")
}

/// Extract metadata from PDF
#[cfg(feature = "riptide-pdf")]
pub fn extract_metadata(data: &[u8]) -> Result<riptide_pdf::PdfDocMetadata> {
    let extractor = PdfExtractor::from_bytes(data).context("Failed to create PDF extractor")?;

    extractor
        .extract_metadata()
        .context("Failed to extract PDF metadata")
}

#[cfg(not(feature = "riptide-pdf"))]
pub fn extract_metadata(_data: &[u8]) -> Result<serde_json::Value> {
    anyhow::bail!("PDF processing feature not enabled. Rebuild with --features riptide-pdf")
}

/// Convert PDF to markdown
#[cfg(feature = "riptide-pdf")]
pub fn convert_to_markdown(data: &[u8]) -> Result<String> {
    let extractor = PdfExtractor::from_bytes(data).context("Failed to create PDF extractor")?;

    let content = extractor
        .extract_all()
        .context("Failed to extract PDF content")?;

    Ok(extractor.to_markdown(&content))
}

#[cfg(not(feature = "riptide-pdf"))]
pub fn convert_to_markdown(_data: &[u8]) -> Result<String> {
    anyhow::bail!("PDF processing feature not enabled. Rebuild with --features riptide-pdf")
}

/// Extract full PDF content
#[cfg(feature = "riptide-pdf")]
pub fn extract_full_content(data: &[u8]) -> Result<riptide_pdf::PdfContent> {
    let extractor = PdfExtractor::from_bytes(data).context("Failed to create PDF extractor")?;

    extractor
        .extract_all()
        .context("Failed to extract PDF content")
}

#[cfg(not(feature = "riptide-pdf"))]
pub fn extract_full_content(_data: &[u8]) -> Result<serde_json::Value> {
    anyhow::bail!("PDF processing feature not enabled. Rebuild with --features riptide-pdf")
}

/// Parse page range string (e.g., "1-5,10-15")
pub fn parse_page_range(range_str: &str) -> Result<Vec<u32>> {
    let mut pages = Vec::new();

    for part in range_str.split(',') {
        let part = part.trim();

        if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() != 2 {
                anyhow::bail!("Invalid page range format: {}", part);
            }

            let start: u32 = bounds[0]
                .parse()
                .context(format!("Invalid start page: {}", bounds[0]))?;
            let end: u32 = bounds[1]
                .parse()
                .context(format!("Invalid end page: {}", bounds[1]))?;

            if start > end {
                anyhow::bail!("Start page must be <= end page: {}", part);
            }

            for page in start..=end {
                pages.push(page);
            }
        } else {
            let page: u32 = part
                .parse()
                .context(format!("Invalid page number: {}", part))?;
            pages.push(page);
        }
    }

    Ok(pages)
}

/// Write output to file or stdout
pub fn write_output(content: &str, output_path: Option<&str>) -> Result<()> {
    if let Some(path) = output_path {
        fs::write(path, content).context(format!("Failed to write output to {}", path))?;
        println!("Output written to: {}", path);
    } else {
        println!("{}", content);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_page_range() {
        assert_eq!(parse_page_range("1").unwrap(), vec![1]);
        assert_eq!(parse_page_range("1-5").unwrap(), vec![1, 2, 3, 4, 5]);
        assert_eq!(parse_page_range("1-3,5-7").unwrap(), vec![1, 2, 3, 5, 6, 7]);
        assert_eq!(parse_page_range("10,15,20").unwrap(), vec![10, 15, 20]);
    }

    #[test]
    fn test_parse_page_range_errors() {
        assert!(parse_page_range("5-2").is_err()); // Start > end
        assert!(parse_page_range("abc").is_err()); // Invalid number
        assert!(parse_page_range("1-2-3").is_err()); // Invalid format
    }
}
