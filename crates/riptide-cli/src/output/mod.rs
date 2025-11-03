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

/// Print error message with ✗ symbol
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print info message with ℹ symbol
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Print warning message with ⚠ symbol
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
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

/// Format duration in seconds to human-readable string
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

/// Format bytes to human-readable string (B, KB, MB, GB, etc.)
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

/// Format confidence score (0.0-1.0) as colored percentage
pub fn format_confidence(score: f64) -> String {
    let percentage = (score * 100.0) as u32;
    let color = if percentage >= 90 {
        "green"
    } else if percentage >= 70 {
        "yellow"
    } else {
        "red"
    };

    format!("{}%", percentage.to_string().color(color))
}

/// Print key-value pair with colored key
pub fn print_key_value(key: &str, value: &str) {
    println!("{}: {}", key.cyan().bold(), value);
}

/// Print section title with formatting
pub fn print_section(title: &str) {
    println!("\n{}", title.bold().underline());
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
