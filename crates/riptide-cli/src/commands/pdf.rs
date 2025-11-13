//! PDF command - Extract content from PDF files
//!
//! This command provides PDF processing capabilities including:
//! - Text extraction from local files or URLs
//! - Table extraction
//! - Metadata extraction
//! - Markdown conversion
//! - Page range selection
//!
//! The command delegates to the pdf_impl module for actual processing,
//! following the thin CLI pattern.

#[cfg(feature = "riptide-pdf")]
use crate::pdf_impl;
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};

/// Arguments for the PDF command
#[derive(Args, Clone, Debug)]
pub struct PdfArgs {
    /// PDF file path or URL
    #[arg(required = true)]
    pub input: String,

    /// Output format (text, markdown, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Extract only specific pages (e.g., "1-5,10-15")
    #[arg(short, long)]
    pub pages: Option<String>,

    /// Extract tables
    #[arg(long)]
    pub tables: bool,

    /// Extract metadata only
    #[arg(long)]
    pub metadata: bool,
}

/// Output structure for JSON format
#[cfg(feature = "riptide-pdf")]
#[derive(Debug, Serialize, Deserialize)]
struct PdfJsonOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tables: Option<Vec<TableOutput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<MetadataOutput>,
}

#[cfg(feature = "riptide-pdf")]
#[derive(Debug, Serialize, Deserialize)]
struct TableOutput {
    page: u32,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<PositionOutput>,
}

#[cfg(feature = "riptide-pdf")]
#[derive(Debug, Serialize, Deserialize)]
struct PositionOutput {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[cfg(feature = "riptide-pdf")]
#[derive(Debug, Serialize, Deserialize)]
struct MetadataOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    creator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    producer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    creation_date: Option<String>,
    page_count: u32,
}

/// Execute the PDF command
#[cfg(feature = "riptide-pdf")]
pub async fn execute(args: &PdfArgs) -> Result<()> {
    // Validate format
    match args.format.to_lowercase().as_str() {
        "text" | "markdown" | "json" => {}
        _ => anyhow::bail!(
            "Invalid format: {}. Supported formats: text, markdown, json",
            args.format
        ),
    }

    // Validate page range if specified
    if let Some(ref pages) = args.pages {
        pdf_impl::parse_page_range(pages)
            .context("Invalid page range format. Use format like '1-5,10-15'")?;
    }

    // Load PDF from file or URL
    eprintln!("Loading PDF from: {}", args.input);
    let pdf_data = pdf_impl::load_pdf(&args.input)
        .await
        .context("Failed to load PDF")?;

    eprintln!("PDF loaded successfully ({} bytes)", pdf_data.len());

    // Handle metadata-only extraction
    if args.metadata {
        let metadata =
            pdf_impl::extract_metadata(&pdf_data).context("Failed to extract PDF metadata")?;

        let output = format_metadata_output(&metadata, &args.format)?;
        pdf_impl::write_output(&output, args.output.as_deref())
            .context("Failed to write output")?;
        return Ok(());
    }

    // Extract content based on format
    let output = match args.format.to_lowercase().as_str() {
        "text" => {
            eprintln!("Extracting text...");
            let text =
                pdf_impl::extract_text(&pdf_data).context("Failed to extract text from PDF")?;

            if args.tables {
                let tables = pdf_impl::extract_tables(&pdf_data)
                    .context("Failed to extract tables from PDF")?;
                format_text_with_tables(&text, &tables)
            } else {
                text
            }
        }
        "markdown" => {
            eprintln!("Converting to markdown...");
            let markdown = pdf_impl::convert_to_markdown(&pdf_data)
                .context("Failed to convert PDF to markdown")?;

            if args.tables {
                // Markdown conversion already includes tables
                markdown
            } else {
                markdown
            }
        }
        "json" => {
            eprintln!("Extracting content as JSON...");
            let text = if !args.tables {
                Some(pdf_impl::extract_text(&pdf_data).context("Failed to extract text from PDF")?)
            } else {
                None
            };

            let tables = if args.tables {
                Some(
                    pdf_impl::extract_tables(&pdf_data)
                        .context("Failed to extract tables from PDF")?,
                )
            } else {
                None
            };

            let metadata = pdf_impl::extract_metadata(&pdf_data)
                .context("Failed to extract metadata from PDF")?;

            format_json_output(text, tables, Some(metadata))?
        }
        _ => unreachable!(), // Already validated above
    };

    // Write output
    pdf_impl::write_output(&output, args.output.as_deref()).context("Failed to write output")?;

    eprintln!("PDF processing completed successfully");
    Ok(())
}

#[cfg(not(feature = "riptide-pdf"))]
pub async fn execute(_args: &PdfArgs) -> Result<()> {
    anyhow::bail!(
        "PDF processing feature not enabled. Please rebuild with --features riptide-pdf\n\
         Example: cargo build --release --features riptide-pdf"
    )
}

#[cfg(feature = "riptide-pdf")]
fn format_text_with_tables(text: &str, tables: &[riptide_pdf::ExtractedTable]) -> String {
    let mut output = text.to_string();

    if !tables.is_empty() {
        output.push_str("\n\n=== EXTRACTED TABLES ===\n\n");

        for (idx, table) in tables.iter().enumerate() {
            output.push_str(&format!("Table {} (Page {}):\n", idx + 1, table.page));

            // Format headers
            if !table.headers.is_empty() {
                output.push_str(&table.headers.join(" | "));
                output.push('\n');
                output.push_str(&"-".repeat(table.headers.iter().map(|h| h.len() + 3).sum()));
                output.push('\n');
            }

            // Format rows
            for row in &table.rows {
                output.push_str(&row.join(" | "));
                output.push('\n');
            }

            output.push('\n');
        }
    }

    output
}

#[cfg(feature = "riptide-pdf")]
fn format_metadata_output(metadata: &riptide_pdf::PdfDocMetadata, format: &str) -> Result<String> {
    match format.to_lowercase().as_str() {
        "json" => {
            let output = MetadataOutput {
                title: metadata.title.clone(),
                author: metadata.author.clone(),
                subject: metadata.subject.clone(),
                creator: metadata.creator.clone(),
                producer: metadata.producer.clone(),
                creation_date: metadata.creation_date.clone(),
                page_count: metadata.page_count,
            };

            serde_json::to_string_pretty(&output).context("Failed to serialize metadata to JSON")
        }
        "text" | "markdown" => {
            let mut output = String::new();
            output.push_str("=== PDF METADATA ===\n\n");

            if let Some(ref title) = metadata.title {
                output.push_str(&format!("Title: {}\n", title));
            }
            if let Some(ref author) = metadata.author {
                output.push_str(&format!("Author: {}\n", author));
            }
            if let Some(ref subject) = metadata.subject {
                output.push_str(&format!("Subject: {}\n", subject));
            }
            if let Some(ref creator) = metadata.creator {
                output.push_str(&format!("Creator: {}\n", creator));
            }
            if let Some(ref producer) = metadata.producer {
                output.push_str(&format!("Producer: {}\n", producer));
            }
            if let Some(ref created) = metadata.creation_date {
                output.push_str(&format!("Created: {}\n", created));
            }
            if let Some(ref modified) = metadata.modification_date {
                output.push_str(&format!("Modified: {}\n", modified));
            }

            output.push_str(&format!("Pages: {}\n", metadata.page_count));

            Ok(output)
        }
        _ => anyhow::bail!("Unsupported metadata format: {}", format),
    }
}

#[cfg(feature = "riptide-pdf")]
fn format_json_output(
    text: Option<String>,
    tables: Option<Vec<riptide_pdf::ExtractedTable>>,
    metadata: Option<riptide_pdf::PdfDocMetadata>,
) -> Result<String> {
    let tables_output = tables.map(|tables| {
        tables
            .into_iter()
            .map(|table| TableOutput {
                page: table.page,
                headers: table.headers,
                rows: table.rows,
                position: table.position.map(|pos| PositionOutput {
                    x: pos.x,
                    y: pos.y,
                    width: pos.width,
                    height: pos.height,
                }),
            })
            .collect()
    });

    let metadata_output = metadata.map(|meta| MetadataOutput {
        title: meta.title,
        author: meta.author,
        subject: meta.subject,
        creator: meta.creator,
        producer: meta.producer,
        creation_date: meta.creation_date,
        page_count: meta.page_count,
    });

    let output = PdfJsonOutput {
        text,
        tables: tables_output,
        metadata: metadata_output,
    };

    serde_json::to_string_pretty(&output).context("Failed to serialize output to JSON")
}

#[cfg(test)]
#[cfg(feature = "riptide-pdf")]
mod tests {
    use super::*;

    #[test]
    fn test_args_validation() {
        let args = PdfArgs {
            input: "test.pdf".to_string(),
            format: "text".to_string(),
            output: None,
            pages: None,
            tables: false,
            metadata: false,
        };

        assert_eq!(args.input, "test.pdf");
        assert_eq!(args.format, "text");
        assert!(!args.tables);
        assert!(!args.metadata);
    }

    #[test]
    fn test_format_text_with_tables() {
        use riptide_pdf::ExtractedTable;

        let text = "Sample text content";
        let tables = vec![ExtractedTable {
            page: 1,
            headers: vec!["Header 1".to_string(), "Header 2".to_string()],
            rows: vec![
                vec!["Row 1 Col 1".to_string(), "Row 1 Col 2".to_string()],
                vec!["Row 2 Col 1".to_string(), "Row 2 Col 2".to_string()],
            ],
            position: None,
        }];

        let output = format_text_with_tables(text, &tables);

        assert!(output.contains("Sample text content"));
        assert!(output.contains("EXTRACTED TABLES"));
        assert!(output.contains("Header 1"));
        assert!(output.contains("Row 1 Col 1"));
    }

    #[test]
    fn test_json_output_format() {
        use riptide_pdf::PdfDocMetadata;

        let metadata = PdfDocMetadata {
            title: Some("Test Document".to_string()),
            author: Some("Test Author".to_string()),
            subject: None,
            creator: None,
            producer: None,
            creation_date: None,
            modification_date: None,
            page_count: 5,
            file_size: 0,
            pdf_version: None,
            encrypted: false,
        };

        let output = format_json_output(Some("Test text".to_string()), None, Some(metadata));

        assert!(output.is_ok());
        let json = output.unwrap();
        assert!(json.contains("Test Document"));
        assert!(json.contains("Test Author"));
        assert!(json.contains("Test text"));
    }
}
