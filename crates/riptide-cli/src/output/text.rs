//! Text output formatter
//!
//! Provides human-readable colored text output with symbols.

use anyhow::Result;
use colored::*;
use serde::Serialize;
use serde_json::Value;

pub struct TextFormatter;

impl TextFormatter {
    /// Format data as human-readable text with colors
    pub fn format<T: Serialize>(data: &T) -> Result<String> {
        // Check NO_COLOR environment variable
        if std::env::var("NO_COLOR").is_ok() {
            colored::control::set_override(false);
        }

        let value = serde_json::to_value(data)?;
        Ok(Self::format_value(&value, 0))
    }

    /// Format a JSON value recursively
    fn format_value(value: &Value, indent: usize) -> String {
        match value {
            Value::Object(obj) => Self::format_object(obj, indent),
            Value::Array(arr) => Self::format_array(arr, indent),
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => Self::format_bool(*b),
            Value::Null => "null".dimmed().to_string(),
        }
    }

    /// Format object as key-value pairs
    fn format_object(obj: &serde_json::Map<String, Value>, indent: usize) -> String {
        let mut lines = Vec::new();
        let indent_str = "  ".repeat(indent);

        for (key, value) in obj {
            let formatted_key = format!("{}:", key).bold();

            match value {
                Value::Object(_) | Value::Array(_) => {
                    lines.push(format!("{}{}", indent_str, formatted_key));
                    lines.push(Self::format_value(value, indent + 1));
                }
                _ => {
                    let formatted_value = Self::format_value(value, indent);
                    lines.push(format!(
                        "{}{} {}",
                        indent_str, formatted_key, formatted_value
                    ));
                }
            }
        }

        lines.join("\n")
    }

    /// Format array items
    fn format_array(arr: &[Value], indent: usize) -> String {
        let mut lines = Vec::new();
        let indent_str = "  ".repeat(indent);

        for (i, item) in arr.iter().enumerate() {
            let prefix = format!("{}{}.", indent_str, i + 1).dimmed();

            match item {
                Value::Object(_) | Value::Array(_) => {
                    lines.push(prefix.to_string());
                    lines.push(Self::format_value(item, indent + 1));
                }
                _ => {
                    let formatted = Self::format_value(item, indent);
                    lines.push(format!("{} {}", prefix, formatted));
                }
            }
        }

        lines.join("\n")
    }

    /// Format boolean with symbols
    fn format_bool(b: bool) -> String {
        if b {
            "✓".green().to_string()
        } else {
            "✗".red().to_string()
        }
    }

    /// Format success message
    pub fn success(message: &str) -> String {
        if std::env::var("NO_COLOR").is_ok() {
            format!("✓ {}", message)
        } else {
            format!("{} {}", "✓".green(), message)
        }
    }

    /// Format error message
    pub fn error(message: &str) -> String {
        if std::env::var("NO_COLOR").is_ok() {
            format!("✗ {}", message)
        } else {
            format!("{} {}", "✗".red(), message)
        }
    }

    /// Format info message
    pub fn info(message: &str) -> String {
        if std::env::var("NO_COLOR").is_ok() {
            format!("ℹ {}", message)
        } else {
            format!("{} {}", "ℹ".blue(), message)
        }
    }

    /// Format warning message
    pub fn warning(message: &str) -> String {
        if std::env::var("NO_COLOR").is_ok() {
            format!("⚠ {}", message)
        } else {
            format!("{} {}", "⚠".yellow(), message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_object() {
        let data = json!({
            "title": "Example Domain",
            "url": "https://example.com",
            "cached": false,
            "count": 42
        });

        let result = TextFormatter::format(&data).unwrap();

        assert!(result.contains("title:"));
        assert!(result.contains("Example Domain"));
        assert!(result.contains("url:"));
        assert!(result.contains("https://example.com"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_format_array() {
        let data = json!(["item1", "item2", "item3"]);
        let result = TextFormatter::format(&data).unwrap();

        assert!(result.contains("item1"));
        assert!(result.contains("item2"));
        assert!(result.contains("item3"));
    }

    #[test]
    fn test_format_nested() {
        let data = json!({
            "summary": {
                "successful": 10,
                "failed": 2
            },
            "items": ["a", "b"]
        });

        let result = TextFormatter::format(&data).unwrap();

        assert!(result.contains("summary:"));
        assert!(result.contains("successful:"));
        assert!(result.contains("10"));
        assert!(result.contains("items:"));
    }

    #[test]
    fn test_format_bool() {
        assert!(TextFormatter::format_bool(true).contains('✓'));
        assert!(TextFormatter::format_bool(false).contains('✗'));
    }

    #[test]
    fn test_success_message() {
        let msg = TextFormatter::success("Operation completed");
        assert!(msg.contains('✓'));
        assert!(msg.contains("Operation completed"));
    }

    #[test]
    fn test_error_message() {
        let msg = TextFormatter::error("Operation failed");
        assert!(msg.contains('✗'));
        assert!(msg.contains("Operation failed"));
    }

    #[test]
    fn test_info_message() {
        let msg = TextFormatter::info("Processing...");
        assert!(msg.contains('ℹ'));
        assert!(msg.contains("Processing..."));
    }

    #[test]
    fn test_warning_message() {
        let msg = TextFormatter::warning("Deprecated feature");
        assert!(msg.contains('⚠'));
        assert!(msg.contains("Deprecated feature"));
    }

    #[test]
    fn test_no_color_env() {
        std::env::set_var("NO_COLOR", "1");
        colored::control::set_override(false);

        let msg = TextFormatter::success("test");
        // Should not contain ANSI codes when NO_COLOR is set
        assert!(!msg.contains("\x1b["));

        std::env::remove_var("NO_COLOR");
        colored::control::unset_override();
    }
}
