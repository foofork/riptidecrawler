use crate::client::RipTideClient;
use crate::commands::TablesArgs;
use crate::output;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};

#[derive(Serialize)]
struct TableExtractRequest {
    html_content: String,
}

/// Summary information about an extracted table from API
#[derive(Deserialize, Serialize, Debug)]
struct TableSummary {
    /// Unique table identifier for export operations
    id: String,
    /// Number of rows (count, not data)
    rows: usize,
    /// Number of columns (count, not data)
    columns: usize,
    /// Table headers (if present) - sample data only
    #[serde(default)]
    headers: Vec<String>,
    /// Sample data (first few rows) - not full data
    #[serde(default)]
    data: Vec<Vec<String>>,
    /// Table metadata
    #[serde(default)]
    metadata: TableMetadata,
}

/// Table metadata from API
#[derive(Deserialize, Serialize, Debug, Default)]
struct TableMetadata {
    /// Whether table has headers
    #[serde(default)]
    has_headers: bool,
    /// Detected data types for columns
    #[serde(default)]
    data_types: Vec<String>,
    /// Whether table has complex structure (spans)
    #[serde(default)]
    has_complex_structure: bool,
    /// Table caption (if present)
    #[serde(default)]
    caption: Option<String>,
    /// CSS classes from original HTML
    #[serde(default)]
    css_classes: Vec<String>,
    /// Table ID from HTML (if present)
    #[serde(default)]
    html_id: Option<String>,
}

/// Response from table extraction API
#[derive(Deserialize, Serialize, Debug)]
struct TableExtractResponse {
    /// Extracted table summaries with IDs
    tables: Vec<TableSummary>,
    /// Total extraction time in milliseconds
    #[serde(default)]
    extraction_time_ms: u64,
    /// Total number of tables found
    #[serde(default)]
    total_tables: usize,
}

/// Full table data structure (used after export)
#[derive(Deserialize, Serialize, Debug)]
struct TableData {
    id: String,
    rows: usize,
    columns: usize,
    headers: Vec<String>,
    data: Vec<Vec<String>>,
    #[serde(default)]
    caption: Option<String>,
}

pub async fn execute(client: RipTideClient, args: TablesArgs, output_format: &str) -> Result<()> {
    // Get HTML content from one of the sources
    let html_content = if args.stdin {
        output::print_info("Reading HTML from stdin...");
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else if let Some(url) = &args.url {
        output::print_info(&format!("Fetching HTML from: {}", url));
        fetch_html_from_url(url).await?
    } else if let Some(file_path) = &args.file {
        output::print_info(&format!("Reading HTML from file: {}", file_path));
        fs::read_to_string(file_path).context(format!("Failed to read file: {}", file_path))?
    } else {
        anyhow::bail!("Must provide --url, --file, or --stdin");
    };

    output::print_info("Extracting tables from HTML...");

    // Call the API to extract tables
    let request = TableExtractRequest { html_content };
    let response = client.post("/api/v1/tables/extract", &request).await?;
    let extract_result: TableExtractResponse = response.json().await?;

    output::print_success(&format!("Found {} table(s)", extract_result.tables.len()));

    // Fetch full table data from export endpoint for each table
    output::print_info("Fetching full table data...");
    let mut full_tables: Vec<TableData> = Vec::new();

    for (idx, table_summary) in extract_result.tables.iter().enumerate() {
        output::print_info(&format!(
            "Exporting table {}/{} (ID: {})...",
            idx + 1,
            extract_result.tables.len(),
            table_summary.id
        ));

        // Fetch full data from export endpoint
        match fetch_full_table_data(&client, &table_summary, &args.format).await {
            Ok(table_data) => {
                full_tables.push(table_data);
            }
            Err(e) => {
                output::print_warning(&format!(
                    "Failed to export table {}: {}",
                    table_summary.id, e
                ));
                // Fallback to summary data if export fails
                full_tables.push(TableData {
                    id: table_summary.id.clone(),
                    rows: table_summary.rows,
                    columns: table_summary.columns,
                    headers: table_summary.headers.clone(),
                    data: table_summary.data.clone(),
                    caption: table_summary.metadata.caption.clone(),
                });
            }
        }
    }

    // Format and output the tables with full data
    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&full_tables)?;
            output_result(&json_output, &args.output)?;
        }
        "csv" => {
            let csv_output = format_as_csv(&full_tables)?;
            output_result(&csv_output, &args.output)?;
        }
        "markdown" => {
            let markdown_output = format_as_markdown(&full_tables);
            output_result(&markdown_output, &args.output)?;
        }
        _ => {
            output::print_warning(&format!("Unknown format: {}, using markdown", args.format));
            let markdown_output = format_as_markdown(&full_tables);
            output_result(&markdown_output, &args.output)?;
        }
    }

    // If output_format is set to table, show a summary
    if output_format == "table" && args.output.is_none() {
        println!();
        let mut summary_table = output::create_table(vec!["Table #", "Rows", "Columns", "Caption"]);

        for (idx, table) in full_tables.iter().enumerate() {
            summary_table.add_row(vec![
                &(idx + 1).to_string(),
                &table.rows.to_string(),
                &table.columns.to_string(),
                table.caption.as_deref().unwrap_or("N/A"),
            ]);
        }

        println!("{}", summary_table);
    }

    Ok(())
}

async fn fetch_html_from_url(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("RipTide/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to fetch URL")?;

    let html = response.text().await.context("Failed to read response")?;
    Ok(html)
}

/// Fetch full table data from the export endpoint
async fn fetch_full_table_data(
    client: &RipTideClient,
    table_summary: &TableSummary,
    format: &str,
) -> Result<TableData> {
    // For JSON format, we need to get structured data
    // For CSV and Markdown, the API returns formatted text, so we parse it back
    let export_format = match format {
        "json" => "json",
        "csv" => "csv",
        "markdown" => "markdown",
        _ => "markdown",
    };

    let export_url = format!(
        "/api/v1/tables/{}/export?format={}&include_headers=true",
        table_summary.id, export_format
    );

    let response = client.get(&export_url).await.context(format!(
        "Failed to fetch table export for ID: {}",
        table_summary.id
    ))?;

    match format {
        "json" => {
            // For JSON, parse the response directly
            let content = response.text().await?;

            // The API returns formatted JSON/CSV/Markdown, not structured data
            // So we need to parse the formatted output back into TableData
            // For now, use the summary data as the API returns formatted strings
            Ok(TableData {
                id: table_summary.id.clone(),
                rows: table_summary.rows,
                columns: table_summary.columns,
                headers: table_summary.headers.clone(),
                data: parse_content_to_table_data(&content, export_format, table_summary)?,
                caption: table_summary.metadata.caption.clone(),
            })
        }
        "csv" | "markdown" => {
            let content = response.text().await?;

            // Parse the formatted content back to structured data
            Ok(TableData {
                id: table_summary.id.clone(),
                rows: table_summary.rows,
                columns: table_summary.columns,
                headers: table_summary.headers.clone(),
                data: parse_content_to_table_data(&content, export_format, table_summary)?,
                caption: table_summary.metadata.caption.clone(),
            })
        }
        _ => {
            // Fallback to summary data
            Ok(TableData {
                id: table_summary.id.clone(),
                rows: table_summary.rows,
                columns: table_summary.columns,
                headers: table_summary.headers.clone(),
                data: table_summary.data.clone(),
                caption: table_summary.metadata.caption.clone(),
            })
        }
    }
}

/// Parse formatted content back to table data
fn parse_content_to_table_data(
    content: &str,
    format: &str,
    summary: &TableSummary,
) -> Result<Vec<Vec<String>>> {
    match format {
        "csv" => parse_csv_to_data(content),
        "markdown" => parse_markdown_to_data(content),
        "json" => {
            // Try to parse as JSON table data
            if let Ok(data) = serde_json::from_str::<Vec<Vec<String>>>(content) {
                Ok(data)
            } else {
                // Fallback to summary data
                Ok(summary.data.clone())
            }
        }
        _ => Ok(summary.data.clone()),
    }
}

/// Parse CSV content to table data
fn parse_csv_to_data(csv: &str) -> Result<Vec<Vec<String>>> {
    let mut data = Vec::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv.as_bytes());

    for result in reader.records() {
        let record = result.context("Failed to parse CSV record")?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();

        // Skip comment lines (table separators)
        if !row.is_empty() && !row[0].starts_with('#') {
            data.push(row);
        }
    }

    Ok(data)
}

/// Parse Markdown table content to table data
fn parse_markdown_to_data(markdown: &str) -> Result<Vec<Vec<String>>> {
    let mut data = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        // Skip headers, separators, and empty lines
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with('|') && trimmed.contains("---")
        {
            continue;
        }

        // Parse table rows (lines starting and ending with |)
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let row: Vec<String> = trimmed
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(|s| s.trim().to_string())
                .collect();

            if !row.is_empty() {
                data.push(row);
            }
        }
    }

    Ok(data)
}

fn format_as_markdown(tables: &[TableData]) -> String {
    let mut output = String::new();

    for (idx, table) in tables.iter().enumerate() {
        // Add caption if present
        if let Some(caption) = &table.caption {
            output.push_str(&format!("\n## Table {} - {}\n\n", idx + 1, caption));
        } else {
            output.push_str(&format!("\n## Table {}\n\n", idx + 1));
        }

        // Add headers
        if !table.headers.is_empty() {
            output.push_str("| ");
            output.push_str(&table.headers.join(" | "));
            output.push_str(" |\n");

            // Add separator
            output.push_str("|");
            for _ in &table.headers {
                output.push_str(" --- |");
            }
            output.push_str("\n");
        }

        // Add rows
        for row in &table.data {
            output.push_str("| ");
            output.push_str(&row.join(" | "));
            output.push_str(" |\n");
        }

        output.push('\n');
    }

    output
}

fn format_as_csv(tables: &[TableData]) -> Result<String> {
    let mut output = String::new();

    for (idx, table) in tables.iter().enumerate() {
        // Add table separator for multiple tables
        if idx > 0 {
            output.push_str("\n\n");
        }

        // Add caption as comment
        if let Some(caption) = &table.caption {
            output.push_str(&format!("# Table {} - {}\n", idx + 1, caption));
        } else {
            output.push_str(&format!("# Table {}\n", idx + 1));
        }

        // Add headers if present
        if !table.headers.is_empty() {
            output.push_str(&escape_csv_row(&table.headers));
            output.push('\n');
        }

        // Add rows
        for row in &table.data {
            output.push_str(&escape_csv_row(row));
            output.push('\n');
        }
    }

    Ok(output)
}

fn escape_csv_row(row: &[String]) -> String {
    row.iter()
        .map(|cell| {
            if cell.contains(',') || cell.contains('"') || cell.contains('\n') {
                format!("\"{}\"", cell.replace('"', "\"\""))
            } else {
                cell.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn output_result(content: &str, output_path: &Option<String>) -> Result<()> {
    if let Some(path) = output_path {
        fs::write(path, content)?;
        output::print_success(&format!("Output saved to: {}", path));
    } else {
        println!("{}", content);
    }
    Ok(())
}
