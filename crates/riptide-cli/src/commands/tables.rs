//! Table extraction CLI command (minimal layer)

use crate::client::RipTideClient;
use crate::commands::TablesArgs;
use crate::output;
use anyhow::Result;
use riptide_extraction::tables::{
    ApiClient, TableConverter, TableData, TableExtractResponse, TableExtractor, TableSource,
    TableSummary,
};

/// Execute table extraction command
pub async fn execute(client: RipTideClient, args: TablesArgs, output_format: &str) -> Result<()> {
    // Determine source of HTML content
    let source = if args.stdin {
        output::print_info("Reading HTML from stdin...");
        TableSource::Stdin
    } else if let Some(url) = &args.url {
        output::print_info(&format!("Fetching HTML from: {}", url));
        TableSource::Url(url)
    } else if let Some(file_path) = &args.file {
        output::print_info(&format!("Reading HTML from file: {}", file_path));
        TableSource::File(file_path)
    } else {
        anyhow::bail!("Must provide --url, --file, or --stdin");
    };

    // Extract HTML content
    let html_content = TableExtractor::extract_from_source(source).await?;

    output::print_info("Extracting tables from HTML...");

    // Call API to extract tables
    let request = TableExtractor::create_request(html_content);
    let api_adapter = RipTideApiAdapter { client: &client };
    let extract_result = api_adapter
        .post_json("/api/v1/tables/extract", &request)
        .await?;

    output::print_success(&format!("Found {} table(s)", extract_result.tables.len()));

    // Fetch full table data
    output::print_info("Fetching full table data...");
    let full_tables = fetch_all_tables(&api_adapter, &extract_result.tables, &args.format).await?;

    // Format and output
    output_tables(&full_tables, &args.format, &args.output)?;

    // Show summary if in table mode
    if output_format == "table" && args.output.is_none() {
        show_summary(&full_tables);
    }

    Ok(())
}

/// Fetch full data for all tables
async fn fetch_all_tables(
    client: &RipTideApiAdapter<'_>,
    summaries: &[TableSummary],
    format: &str,
) -> Result<Vec<TableData>> {
    let mut tables = Vec::new();

    for (idx, summary) in summaries.iter().enumerate() {
        output::print_info(&format!(
            "Exporting table {}/{} (ID: {})...",
            idx + 1,
            summaries.len(),
            summary.id
        ));

        match TableExtractor::fetch_full_table_data(client, summary, format).await {
            Ok(table) => tables.push(table),
            Err(e) => {
                output::print_warning(&format!("Failed to export table {}: {}", summary.id, e));
                // Fallback to summary data
                tables.push(TableData {
                    id: summary.id.clone(),
                    rows: summary.rows,
                    columns: summary.columns,
                    headers: summary.headers.clone(),
                    data: summary.data.clone(),
                    caption: summary.metadata.caption.clone(),
                });
            }
        }
    }

    Ok(tables)
}

/// Output tables in requested format
fn output_tables(tables: &[TableData], format: &str, output_path: &Option<String>) -> Result<()> {
    let content = match format {
        "json" => TableConverter::to_json(tables)?,
        "csv" => TableConverter::to_csv(tables)?,
        "markdown" => TableConverter::to_markdown(tables),
        _ => {
            output::print_warning(&format!("Unknown format: {}, using markdown", format));
            TableConverter::to_markdown(tables)
        }
    };

    if let Some(path) = output_path {
        std::fs::write(path, content)?;
        output::print_success(&format!("Output saved to: {}", path));
    } else {
        println!("{}", content);
    }

    Ok(())
}

/// Show summary table of extracted tables
fn show_summary(tables: &[TableData]) {
    println!();
    let mut summary_table = output::create_table(vec!["Table #", "Rows", "Columns", "Caption"]);

    for (idx, table) in tables.iter().enumerate() {
        summary_table.add_row(vec![
            &(idx + 1).to_string(),
            &table.rows.to_string(),
            &table.columns.to_string(),
            table.caption.as_deref().unwrap_or("N/A"),
        ]);
    }

    println!("{}", summary_table);
}

/// Adapter to make RipTideClient work with ApiClient trait
struct RipTideApiAdapter<'a> {
    client: &'a RipTideClient,
}

#[async_trait::async_trait]
impl<'a> ApiClient for RipTideApiAdapter<'a> {
    async fn post_json<T: serde::Serialize + Send + Sync>(
        &self,
        endpoint: &str,
        request: &T,
    ) -> Result<TableExtractResponse> {
        let response = self.client.post(endpoint, request).await?;
        response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))
    }

    async fn get_text(&self, endpoint: &str) -> Result<String> {
        let response = self.client.get(endpoint).await?;
        response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))
    }
}
