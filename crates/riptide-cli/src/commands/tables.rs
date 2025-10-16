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

#[derive(Deserialize, Serialize)]
struct Table {
    id: String,
    rows: usize,
    columns: usize,
    headers: Vec<String>,
    data: Vec<Vec<String>>,
    #[serde(default)]
    caption: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct TableExtractResponse {
    tables: Vec<Table>,
    #[serde(default)]
    count: usize,
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

    // Format and output the tables
    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&extract_result)?;
            output_result(&json_output, &args.output)?;
        }
        "csv" => {
            let csv_output = format_as_csv(&extract_result.tables)?;
            output_result(&csv_output, &args.output)?;
        }
        "markdown" => {
            let markdown_output = format_as_markdown(&extract_result.tables);
            output_result(&markdown_output, &args.output)?;
        }
        _ => {
            output::print_warning(&format!("Unknown format: {}, using markdown", args.format));
            let markdown_output = format_as_markdown(&extract_result.tables);
            output_result(&markdown_output, &args.output)?;
        }
    }

    // If output_format is set to table, show a summary
    if output_format == "table" && args.output.is_none() {
        println!();
        let mut summary_table = output::create_table(vec!["Table #", "Rows", "Columns", "Caption"]);

        for (idx, table) in extract_result.tables.iter().enumerate() {
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

fn format_as_markdown(tables: &[Table]) -> String {
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

fn format_as_csv(tables: &[Table]) -> Result<String> {
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
