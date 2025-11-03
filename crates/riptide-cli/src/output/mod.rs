//! Output formatting module
//!
//! Provides multiple output formats for CLI commands:
//! - JSON: Machine-readable structured output
//! - Table: Terminal tables using comfy-table
//! - Text: Human-readable colored text
//! - Stream: NDJSON streaming for real-time processing

mod json;
mod stream;
mod table;
mod text;

pub use json::JsonFormatter;
pub use stream::StreamFormatter;
pub use table::TableFormatter;
pub use text::TextFormatter;

use anyhow::Result;
use colored::*;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Table};
use serde::Serialize;

/// Output format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Table,
    Text,
    Stream,
}

impl OutputFormat {
    /// Parse format from string (not implementing FromStr trait due to error type differences)
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "table" => Ok(Self::Table),
            "text" => Ok(Self::Text),
            "stream" | "ndjson" => Ok(Self::Stream),
            _ => Err(anyhow::anyhow!("Invalid output format: {}", s)),
        }
    }
}

/// Format data for output
pub fn format<T: Serialize>(data: &T, output_format: OutputFormat) -> Result<String> {
    match output_format {
        OutputFormat::Json => JsonFormatter::format(data),
        OutputFormat::Table => TableFormatter::format(data),
        OutputFormat::Text => TextFormatter::format(data),
        OutputFormat::Stream => StreamFormatter::format(data),
    }
}

/// Print formatted data to stdout
#[allow(dead_code)]
pub fn print<T: Serialize>(data: &T, output_format: OutputFormat) -> Result<()> {
    let output = format(data, output_format)?;
    println!("{}", output);
    Ok(())
}

// ===== Legacy Helper Functions =====
// Maintained for backward compatibility with existing commands

/// Print JSON with pretty formatting
pub fn print_json<T: Serialize>(data: &T) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("{} Failed to serialize JSON: {}", "✗".red(), e),
    }
}

/// Print success message with ✓ symbol
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print info message with ℹ symbol
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Create a new table with headers
pub fn create_table(headers: Vec<&str>) -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(
        headers
            .into_iter()
            .map(|h| Cell::new(h).set_alignment(CellAlignment::Center)),
    );
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::parse("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("JSON").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::parse("table").unwrap(), OutputFormat::Table);
        assert_eq!(OutputFormat::parse("text").unwrap(), OutputFormat::Text);
        assert_eq!(OutputFormat::parse("stream").unwrap(), OutputFormat::Stream);
        assert_eq!(OutputFormat::parse("ndjson").unwrap(), OutputFormat::Stream);
        assert!(OutputFormat::parse("invalid").is_err());
    }

    #[test]
    fn test_format_json() {
        let data = json!({"status": "success", "count": 42});
        let result = format(&data, OutputFormat::Json).unwrap();
        assert!(result.contains("\"status\""));
        assert!(result.contains("\"success\""));
        assert!(result.contains("42"));
    }
}
