//! JSON output formatter
//!
//! Provides pretty-printed JSON output for machine-readable data.

use anyhow::Result;
use serde::Serialize;

pub struct JsonFormatter;

impl JsonFormatter {
    /// Format data as pretty-printed JSON
    pub fn format<T: Serialize>(data: &T) -> Result<String> {
        let json = serde_json::to_string_pretty(data)?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_pretty() {
        let data = json!({
            "status": "success",
            "results": [1, 2, 3],
            "metadata": {
                "count": 3,
                "cached": false
            }
        });

        let result = JsonFormatter::format(&data).unwrap();

        // Should be multi-line and indented
        assert!(result.contains('\n'));
        assert!(result.contains("  \"status\""));
        assert!(result.contains("  \"results\""));
        assert!(result.contains("    \"count\""));
    }

    #[test]
    fn test_format_empty_object() {
        let data = json!({});
        let result = JsonFormatter::format(&data).unwrap();
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_format_array() {
        let data = json!([1, 2, 3, 4, 5]);
        let result = JsonFormatter::format(&data).unwrap();
        assert!(result.contains('['));
        assert!(result.contains(']'));
        assert!(result.contains("1"));
        assert!(result.contains("5"));
    }
}
