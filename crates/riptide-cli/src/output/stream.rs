//! Stream output formatter
//!
//! Provides NDJSON (newline-delimited JSON) streaming output for real-time processing.

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

pub struct StreamFormatter;

impl StreamFormatter {
    /// Format data as NDJSON (each item on its own line)
    pub fn format<T: Serialize>(data: &T) -> Result<String> {
        let value = serde_json::to_value(data)?;

        match value {
            Value::Array(arr) => Self::format_array(&arr),
            other => Self::format_single(&other),
        }
    }

    /// Format array as NDJSON (one JSON object per line)
    fn format_array(arr: &[Value]) -> Result<String> {
        let mut lines = Vec::new();

        for item in arr {
            let json = serde_json::to_string(item)?;
            lines.push(json);
        }

        Ok(lines.join("\n"))
    }

    /// Format single value as compact JSON
    fn format_single(value: &Value) -> Result<String> {
        let json = serde_json::to_string(value)?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_array() {
        let data = json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"},
            {"id": 3, "name": "Charlie"}
        ]);

        let result = StreamFormatter::format(&data).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);

        // Each line should be valid JSON
        for line in lines {
            let parsed: Value = serde_json::from_str(line).unwrap();
            assert!(parsed.is_object());
        }
    }

    #[test]
    fn test_format_single() {
        let data = json!({"status": "success", "count": 42});
        let result = StreamFormatter::format(&data).unwrap();

        // Should be single line compact JSON
        assert!(!result.contains('\n'));

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["status"], "success");
        assert_eq!(parsed["count"], 42);
    }

    #[test]
    fn test_format_empty_array() {
        let data = json!([]);
        let result = StreamFormatter::format(&data).unwrap();
        assert_eq!(result, "");
    }
}
