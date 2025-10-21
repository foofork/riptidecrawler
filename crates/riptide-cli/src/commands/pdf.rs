use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::client::RipTideClient;
use crate::output;

// Use crate's pdf_impl when feature is enabled, or local stub when disabled
#[cfg(feature = "pdf")]
use crate::pdf_impl;

#[cfg(not(feature = "pdf"))]
mod pdf_impl {
    use anyhow::Result;

    pub async fn load_pdf(_input: &str) -> Result<Vec<u8>> {
        anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
    }

    pub fn extract_metadata(_data: &[u8]) -> Result<serde_json::Value> {
        anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
    }

    pub fn extract_full_content(_data: &[u8]) -> Result<serde_json::Value> {
        anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
    }

    pub fn convert_to_markdown(_data: &[u8]) -> Result<String> {
        anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
    }

    pub fn write_output(_content: &str, _path: Option<&str>) -> Result<()> {
        anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
    }

    pub fn parse_page_range(_range: &str) -> Result<Vec<u32>> {
        anyhow::bail!("PDF support not enabled. Rebuild with --features pdf")
    }
}

#[derive(Subcommand)]
pub enum PdfCommands {
    /// Extract text, tables, and images from PDF files
    Extract {
        /// Path to PDF file or URL
        #[arg(long)]
        input: String,

        /// Output format (text, json, markdown)
        #[arg(long, default_value = "text")]
        format: String,

        /// Extract tables as structured data
        #[arg(long)]
        tables: bool,

        /// Extract images with optional OCR
        #[arg(long)]
        images: bool,

        /// Enable OCR for image text extraction
        #[arg(long)]
        ocr: bool,

        /// Page range to extract (e.g., 1-5, 10-15)
        #[arg(long)]
        pages: Option<String>,

        /// Output directory for extracted content
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Extract metadata only
        #[arg(long)]
        metadata_only: bool,
    },

    /// Convert PDF to clean markdown format
    ToMd {
        /// Path to PDF file or URL
        #[arg(long)]
        input: String,

        /// Output markdown file path
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Preserve formatting and structure
        #[arg(long)]
        preserve_format: bool,

        /// Include image references
        #[arg(long)]
        include_images: bool,

        /// Convert tables to markdown tables
        #[arg(long)]
        convert_tables: bool,

        /// Page range to convert (e.g., 1-5, 10-15)
        #[arg(long)]
        pages: Option<String>,

        /// Extract images to directory
        #[arg(long)]
        image_dir: Option<String>,
    },

    /// Show PDF metadata and document information
    Info {
        /// Path to PDF file or URL
        #[arg(long)]
        input: String,

        /// Show detailed metadata
        #[arg(long)]
        detailed: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Stream PDF content page by page as NDJSON
    Stream {
        /// Path to PDF file or URL
        #[arg(long)]
        input: String,

        /// Include page metadata in stream
        #[arg(long)]
        include_metadata: bool,

        /// Extract tables in stream
        #[arg(long)]
        include_tables: bool,

        /// Extract images in stream
        #[arg(long)]
        include_images: bool,

        /// Page range to stream (e.g., 1-5, 10-15)
        #[arg(long)]
        pages: Option<String>,

        /// Batch size for streaming (pages per batch)
        #[arg(long, default_value = "1")]
        batch_size: u32,
    },
}

// Use the PdfDocMetadata type from riptide-pdf instead of defining our own
#[cfg(feature = "pdf")]
type PdfMetadata = riptide_pdf::PdfDocMetadata;

#[cfg(not(feature = "pdf"))]
#[derive(Serialize, Deserialize, Debug)]
struct PdfMetadata {
    title: Option<String>,
    author: Option<String>,
    subject: Option<String>,
    creator: Option<String>,
    producer: Option<String>,
    creation_date: Option<String>,
    modification_date: Option<String>,
    page_count: u32,
    file_size: u64,
    pdf_version: Option<String>,
    encrypted: bool,
}

// Permissions are not part of the core metadata for now
// This struct is kept for future use but not currently used
#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
struct PdfPermissions {
    print: bool,
    copy: bool,
    modify: bool,
    annotate: bool,
}

// Re-export types from riptide-pdf when available
#[cfg(feature = "pdf")]
use riptide_pdf::ExtractedTable as Table;

#[cfg(not(feature = "pdf"))]
#[derive(Serialize, Deserialize, Debug)]
struct Table {
    page: u32,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)] // Will be used when PDF library integration is complete
struct PdfExtractResult {
    text: String,
    tables: Option<Vec<Table>>,
    images: Option<Vec<Image>>,
    metadata: PdfMetadata,
    extraction_time_ms: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Image {
    page: u32,
    width: u32,
    height: u32,
    format: String,
    path: Option<String>,
    ocr_text: Option<String>,
}

// Future use: Streaming PDF processing
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct PdfStreamItem {
    page: u32,
    content: String,
    metadata: Option<serde_json::Value>,
    tables: Option<Vec<Table>>,
    images: Option<Vec<Image>>,
}

pub async fn execute(
    _client: RipTideClient,
    command: PdfCommands,
    output_format: &str,
) -> Result<()> {
    use crate::metrics::MetricsManager;

    // Start metrics tracking
    let metrics_manager = MetricsManager::global();
    let command_name = match &command {
        PdfCommands::Extract { .. } => "pdf_extract",
        PdfCommands::ToMd { .. } => "pdf_to_md",
        PdfCommands::Info { .. } => "pdf_info",
        PdfCommands::Stream { .. } => "pdf_stream",
    };
    let tracking_id = metrics_manager.start_command(command_name).await?;

    let result = match command {
        PdfCommands::Extract {
            input,
            format,
            tables,
            images,
            ocr,
            pages,
            output,
            metadata_only,
        } => {
            execute_extract(
                input,
                format,
                tables,
                images,
                ocr,
                pages,
                output,
                metadata_only,
                output_format,
            )
            .await
        }
        PdfCommands::ToMd {
            input,
            output,
            preserve_format,
            include_images,
            convert_tables,
            pages,
            image_dir,
        } => {
            execute_to_md(
                input,
                output,
                preserve_format,
                include_images,
                convert_tables,
                pages,
                image_dir,
                output_format,
            )
            .await
        }
        PdfCommands::Info {
            input,
            detailed,
            format,
        } => execute_info(input, detailed, format, output_format).await,
        PdfCommands::Stream {
            input,
            include_metadata,
            include_tables,
            include_images,
            pages,
            batch_size,
        } => {
            execute_stream(
                input,
                include_metadata,
                include_tables,
                include_images,
                pages,
                batch_size,
            )
            .await
        }
    };

    // Complete metrics tracking
    match &result {
        Ok(_) => {
            metrics_manager.complete_command(&tracking_id).await?;
        }
        Err(e) => {
            metrics_manager
                .fail_command(&tracking_id, e.to_string())
                .await?;
        }
    }

    result
}

#[allow(clippy::too_many_arguments)]
async fn execute_extract(
    input: String,
    format: String,
    tables: bool,
    images: bool,
    ocr: bool,
    _pages: Option<String>,
    output_path: Option<String>,
    metadata_only: bool,
    _output_format: &str,
) -> Result<()> {
    output::print_info(&format!("Extracting content from PDF: {}", input));

    // Load PDF
    let pdf_data: Vec<u8> = pdf_impl::load_pdf(&input).await?;

    if metadata_only {
        // Extract only metadata
        let metadata = pdf_impl::extract_metadata(&pdf_data)?;

        match format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&metadata)?;
                pdf_impl::write_output(&json, output_path.as_deref())?;
            }
            _ => {
                let mut table = output::create_table(vec!["Property", "Value"]);
                if let Some(title) = &metadata.title {
                    table.add_row(vec!["Title", title]);
                }
                if let Some(author) = &metadata.author {
                    table.add_row(vec!["Author", author]);
                }
                if let Some(subject) = &metadata.subject {
                    table.add_row(vec!["Subject", subject]);
                }
                table.add_row(vec!["Pages", &metadata.page_count.to_string()]);
                table.add_row(vec!["File Size", &format!("{} bytes", metadata.file_size)]);
                if let Some(version) = &metadata.pdf_version {
                    table.add_row(vec!["PDF Version", version]);
                }
                table.add_row(vec![
                    "Encrypted",
                    if metadata.encrypted { "Yes" } else { "No" },
                ]);

                println!("{table}");
            }
        }

        output::print_success("Metadata extraction complete");
        return Ok(());
    }

    // Extract full content
    let content = pdf_impl::extract_full_content(&pdf_data)?;

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&content)?;
            pdf_impl::write_output(&json, output_path.as_deref())?;
        }
        "text" => {
            pdf_impl::write_output(&content.text, output_path.as_deref())?;
        }
        "markdown" => {
            #[cfg(feature = "pdf")]
            {
                let extractor = riptide_pdf::PdfExtractor::from_bytes(&pdf_data)?;
                let md = extractor.to_markdown(&content);
                pdf_impl::write_output(&md, output_path.as_deref())?;
            }
            #[cfg(not(feature = "pdf"))]
            {
                anyhow::bail!("Markdown conversion requires PDF feature");
            }
        }
        _ => {
            pdf_impl::write_output(&content.text, output_path.as_deref())?;
        }
    }

    // Show statistics
    let mut stats_table = output::create_table(vec!["Metric", "Value"]);
    stats_table.add_row(vec![
        "Pages Processed",
        &content.metadata.page_count.to_string(),
    ]);
    stats_table.add_row(vec![
        "Text Length",
        &format!("{} characters", content.text.len()),
    ]);
    stats_table.add_row(vec!["Tables Found", &content.tables.len().to_string()]);

    if tables {
        stats_table.add_row(vec!["Table Extraction", "Enabled"]);
    }
    if images {
        stats_table.add_row(vec!["Image Extraction", "Requested (partial support)"]);
    }
    if ocr {
        stats_table.add_row(vec!["OCR", "Requested (not yet implemented)"]);
    }

    println!("\n{}", stats_table);
    output::print_success("PDF extraction complete");

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn execute_to_md(
    input: String,
    output_path: Option<String>,
    preserve_format: bool,
    include_images: bool,
    convert_tables: bool,
    pages: Option<String>,
    image_dir: Option<String>,
    _output_format: &str,
) -> Result<()> {
    output::print_info(&format!("Converting PDF to markdown: {}", input));

    // Load PDF
    let pdf_data: Vec<u8> = pdf_impl::load_pdf(&input).await?;

    // Convert to markdown
    let markdown = pdf_impl::convert_to_markdown(&pdf_data)?;

    // Write output
    pdf_impl::write_output(&markdown, output_path.as_deref())?;

    // Show conversion options
    let mut options_table = output::create_table(vec!["Option", "Status"]);
    options_table.add_row(vec![
        "Format Preservation",
        if preserve_format {
            "Enabled"
        } else {
            "Disabled"
        },
    ]);
    options_table.add_row(vec![
        "Include Images",
        if include_images {
            "Enabled"
        } else {
            "Disabled"
        },
    ]);
    options_table.add_row(vec![
        "Convert Tables",
        if convert_tables {
            "Enabled"
        } else {
            "Disabled"
        },
    ]);

    if let Some(page_range) = &pages {
        options_table.add_row(vec!["Page Range", page_range]);
    }

    if let Some(ref img_dir) = image_dir {
        options_table.add_row(vec!["Image Directory", img_dir]);
    }

    println!("\n{}", options_table);

    output::print_success("Markdown conversion complete");

    Ok(())
}

async fn execute_info(
    input: String,
    detailed: bool,
    format: String,
    _output_format: &str,
) -> Result<()> {
    output::print_info(&format!("Reading PDF metadata: {}", input));

    // Load PDF
    let pdf_data: Vec<u8> = pdf_impl::load_pdf(&input).await?;

    // Extract metadata
    let metadata = pdf_impl::extract_metadata(&pdf_data)?;

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&metadata)?;
            println!("{}", json);
        }
        _ => {
            output::print_section("PDF Document Information");

            let mut table = output::create_table(vec!["Property", "Value"]);
            table.add_row(vec!["File", &input]);

            if let Some(title) = &metadata.title {
                table.add_row(vec!["Title", title]);
            }
            if let Some(author) = &metadata.author {
                table.add_row(vec!["Author", author]);
            }
            if let Some(subject) = &metadata.subject {
                table.add_row(vec!["Subject", subject]);
            }
            if let Some(creator) = &metadata.creator {
                table.add_row(vec!["Creator", creator]);
            }
            if let Some(producer) = &metadata.producer {
                table.add_row(vec!["Producer", producer]);
            }

            table.add_row(vec!["Pages", &metadata.page_count.to_string()]);
            table.add_row(vec!["File Size", &format!("{} bytes", metadata.file_size)]);

            if let Some(version) = &metadata.pdf_version {
                table.add_row(vec!["PDF Version", version]);
            }

            table.add_row(vec![
                "Encrypted",
                if metadata.encrypted { "Yes" } else { "No" },
            ]);

            println!("{table}");

            if detailed {
                if let Some(created) = &metadata.creation_date {
                    output::print_section("Document Dates");
                    let mut date_table = output::create_table(vec!["Event", "Timestamp"]);
                    date_table.add_row(vec!["Created", created]);
                    if let Some(modified) = &metadata.modification_date {
                        date_table.add_row(vec!["Modified", modified]);
                    }
                    println!("{date_table}");
                }
            }
        }
    }

    output::print_success("Metadata extraction complete");

    Ok(())
}

async fn execute_stream(
    input: String,
    include_metadata: bool,
    include_tables: bool,
    include_images: bool,
    _pages: Option<String>,
    _batch_size: u32,
) -> Result<()> {
    output::print_info(&format!("Streaming PDF content: {}", input));

    // Load PDF
    let pdf_data: Vec<u8> = pdf_impl::load_pdf(&input).await?;

    // Extract full content
    let content = pdf_impl::extract_full_content(&pdf_data)?;

    // Determine which pages to stream
    let page_numbers: Vec<u32> = if let Some(range) = &_pages {
        pdf_impl::parse_page_range(range)?
    } else {
        (1..=content.metadata.page_count).collect()
    };

    // Stream pages as NDJSON
    for page_content in &content.pages {
        if !page_numbers.contains(&page_content.page_number) {
            continue;
        }

        let mut stream_item = serde_json::json!({
            "page": page_content.page_number,
            "content": page_content.text,
        });

        if include_metadata {
            stream_item["metadata"] = serde_json::json!({
                "width": page_content.width,
                "height": page_content.height,
            });
        }

        if include_tables {
            let page_tables: Vec<_> = content
                .tables
                .iter()
                .filter(|t| t.page == page_content.page_number)
                .collect();

            if !page_tables.is_empty() {
                stream_item["tables"] = serde_json::to_value(&page_tables)?;
            }
        }

        if include_images {
            stream_item["images_note"] =
                serde_json::json!("Image extraction not yet fully implemented");
        }

        // Output as NDJSON
        println!("{}", serde_json::to_string(&stream_item)?);
    }

    Ok(())
}
