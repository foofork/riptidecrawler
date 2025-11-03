//! Table output formatter
//!
//! Provides formatted terminal tables using comfy-table.

use anyhow::Result;
use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};
use serde::Serialize;
use serde_json::Value;

pub struct TableFormatter;

impl TableFormatter {
    /// Format data as a terminal table
    pub fn format<T: Serialize>(data: &T) -> Result<String> {
        let value = serde_json::to_value(data)?;

        match value {
            Value::Array(arr) => Self::format_array(&arr),
            Value::Object(obj) => Self::format_object(&obj),
            _ => Ok(format!("{}", value)),
        }
    }

    /// Format array of objects as table
    fn format_array(arr: &[Value]) -> Result<String> {
        if arr.is_empty() {
            return Ok(String::from("(empty)"));
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        // Extract headers from first object
        if let Some(Value::Object(first)) = arr.first() {
            let headers: Vec<String> = first.keys().cloned().collect();
            table.set_header(&headers);

            // Add rows
            for item in arr {
                if let Value::Object(obj) = item {
                    let row: Vec<String> = headers
                        .iter()
                        .map(|h| Self::value_to_string(obj.get(h)))
                        .collect();
                    table.add_row(row);
                }
            }
        } else {
            // Array of primitives - single column
            table.set_header(vec!["Value"]);
            for item in arr {
                table.add_row(vec![Self::value_to_string(Some(item))]);
            }
        }

        Ok(table.to_string())
    }

    /// Format object as two-column key-value table
    fn format_object(obj: &serde_json::Map<String, Value>) -> Result<String> {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["Key", "Value"]);

        for (key, value) in obj {
            let value_str = match value {
                Value::Object(_) | Value::Array(_) => {
                    serde_json::to_string_pretty(value).unwrap_or_else(|_| String::from("(error)"))
                }
                _ => Self::value_to_string(Some(value)),
            };
            table.add_row(vec![key.clone(), value_str]);
        }

        Ok(table.to_string())
    }

    /// Convert JSON value to display string
    fn value_to_string(value: Option<&Value>) -> String {
        match value {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Null) => String::from("null"),
            Some(Value::Array(arr)) => format!("[{} items]", arr.len()),
            Some(Value::Object(obj)) => format!("{{{} fields}}", obj.len()),
            None => String::from(""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_array_of_objects() {
        let data = json!([
            {"id": 1, "name": "Alice", "active": true},
            {"id": 2, "name": "Bob", "active": false}
        ]);

        let result = TableFormatter::format(&data).unwrap();

        assert!(result.contains("id"));
        assert!(result.contains("name"));
        assert!(result.contains("active"));
        assert!(result.contains("Alice"));
        assert!(result.contains("Bob"));
    }

    #[test]
    fn test_format_object() {
        let data = json!({
            "status": "success",
            "count": 42,
            "cached": false
        });

        let result = TableFormatter::format(&data).unwrap();

        assert!(result.contains("Key"));
        assert!(result.contains("Value"));
        assert!(result.contains("status"));
        assert!(result.contains("success"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_format_empty_array() {
        let data = json!([]);
        let result = TableFormatter::format(&data).unwrap();
        assert_eq!(result, "(empty)");
    }

    #[test]
    fn test_format_array_of_primitives() {
        let data = json!([1, 2, 3, 4, 5]);
        let result = TableFormatter::format(&data).unwrap();

        assert!(result.contains("Value"));
        assert!(result.contains('1'));
        assert!(result.contains('5'));
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(
            TableFormatter::value_to_string(Some(&json!("test"))),
            "test"
        );
        assert_eq!(TableFormatter::value_to_string(Some(&json!(42))), "42");
        assert_eq!(TableFormatter::value_to_string(Some(&json!(true))), "true");
        assert_eq!(TableFormatter::value_to_string(Some(&json!(null))), "null");
        assert_eq!(
            TableFormatter::value_to_string(Some(&json!([1, 2, 3]))),
            "[3 items]"
        );
        assert_eq!(
            TableFormatter::value_to_string(Some(&json!({"a": 1}))),
            "{1 fields}"
        );
        assert_eq!(TableFormatter::value_to_string(None), "");
    }
}
