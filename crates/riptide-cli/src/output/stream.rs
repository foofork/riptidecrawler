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

    /// Parse NDJSON string into array of values
    pub fn parse(ndjson: &str) -> Result<Vec<Value>> {
        let mut values = Vec::new();

        for line in ndjson.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let value: Value = serde_json::from_str(trimmed)?;
                values.push(value);
            }
        }

        Ok(values)
    }

    /// Process NDJSON stream line by line with a callback
    pub fn process_stream<F>(ndjson: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(&Value) -> Result<()>,
    {
        for line in ndjson.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let value: Value = serde_json::from_str(trimmed)?;
                callback(&value)?;
            }
        }
        Ok(())
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
    fn test_parse_ndjson() {
        let ndjson = r#"{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
{"id":3,"name":"Charlie"}"#;

        let values = StreamFormatter::parse(ndjson).unwrap();

        assert_eq!(values.len(), 3);
        assert_eq!(values[0]["name"], "Alice");
        assert_eq!(values[1]["name"], "Bob");
        assert_eq!(values[2]["name"], "Charlie");
    }

    #[test]
    fn test_parse_empty_lines() {
        let ndjson = r#"{"id":1}

{"id":2}

{"id":3}"#;

        let values = StreamFormatter::parse(ndjson).unwrap();

        // Should skip empty lines
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_process_stream() {
        let ndjson = r#"{"count":1}
{"count":2}
{"count":3}"#;

        let mut total = 0;
        StreamFormatter::process_stream(ndjson, |value| {
            if let Some(count) = value["count"].as_i64() {
                total += count;
            }
            Ok(())
        })
        .unwrap();

        assert_eq!(total, 6);
    }

    #[test]
    fn test_process_stream_error() {
        let invalid_ndjson = r#"{"valid":1}
not json
{"valid":2}"#;

        let result = StreamFormatter::process_stream(invalid_ndjson, |_| Ok(()));

        // Should fail on invalid JSON line
        assert!(result.is_err());
    }

    #[test]
    fn test_format_empty_array() {
        let data = json!([]);
        let result = StreamFormatter::format(&data).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_roundtrip() {
        let original = json!([
            {"id": 1, "active": true},
            {"id": 2, "active": false}
        ]);

        // Format to NDJSON
        let ndjson = StreamFormatter::format(&original).unwrap();

        // Parse back
        let parsed = StreamFormatter::parse(&ndjson).unwrap();

        // Should match original
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[1]["active"], false);
    }
}
