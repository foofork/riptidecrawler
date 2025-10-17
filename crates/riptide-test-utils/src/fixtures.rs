//! Test fixtures for common test data

/// Sample HTML content for testing extraction
pub mod html {
    /// Simple HTML document with basic structure
    pub const SIMPLE_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
</head>
<body>
    <h1>Test Heading</h1>
    <p>Test paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
</body>
</html>
"#;

    /// HTML with nested structure
    pub const NESTED_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Nested Content</title>
</head>
<body>
    <div class="container">
        <article class="content">
            <h1>Article Title</h1>
            <div class="metadata">
                <span class="author">John Doe</span>
                <span class="date">2025-10-17</span>
            </div>
            <div class="body">
                <p>First paragraph.</p>
                <p>Second paragraph with <a href="/link">a link</a>.</p>
            </div>
        </article>
    </div>
</body>
</html>
"#;

    /// HTML with tables
    pub const TABLE_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Table Test</title>
</head>
<body>
    <table>
        <thead>
            <tr>
                <th>Name</th>
                <th>Value</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>Item 1</td>
                <td>100</td>
            </tr>
            <tr>
                <td>Item 2</td>
                <td>200</td>
            </tr>
        </tbody>
    </table>
</body>
</html>
"#;

    /// HTML with script tags (should be stripped)
    pub const SCRIPT_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Script Test</title>
    <script>
        alert('This should be stripped');
    </script>
</head>
<body>
    <p>Content</p>
    <script>
        console.log('Also should be stripped');
    </script>
</body>
</html>
"#;
}

/// Sample URLs for testing
pub mod urls {
    /// Example HTTP URL
    pub const HTTP_URL: &str = "http://example.com/page";

    /// Example HTTPS URL
    pub const HTTPS_URL: &str = "https://example.com/page";

    /// Example URL with query parameters
    pub const URL_WITH_PARAMS: &str = "https://example.com/page?param1=value1&param2=value2";

    /// Example URL with fragment
    pub const URL_WITH_FRAGMENT: &str = "https://example.com/page#section";
}

/// Sample JSON data for testing
pub mod json {
    use serde_json::json;
    use serde_json::Value;

    /// Simple JSON object
    pub fn simple_object() -> Value {
        json!({
            "name": "Test",
            "value": 42,
            "active": true
        })
    }

    /// JSON array
    pub fn simple_array() -> Value {
        json!([
            {"id": 1, "name": "Item 1"},
            {"id": 2, "name": "Item 2"},
            {"id": 3, "name": "Item 3"}
        ])
    }

    /// Nested JSON structure
    pub fn nested_object() -> Value {
        json!({
            "user": {
                "id": 123,
                "name": "John Doe",
                "email": "john@example.com",
                "address": {
                    "street": "123 Main St",
                    "city": "Anytown",
                    "country": "USA"
                }
            },
            "metadata": {
                "created_at": "2025-10-17T12:00:00Z",
                "updated_at": "2025-10-17T12:30:00Z"
            }
        })
    }
}

/// Create temporary test files
pub mod temp_files {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use anyhow::Result;

    /// Create a temporary file with HTML content
    pub fn html_file(content: &str) -> Result<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(file)
    }

    /// Create a temporary file with JSON content
    pub fn json_file(content: &serde_json::Value) -> Result<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        let json_str = serde_json::to_string_pretty(content)?;
        file.write_all(json_str.as_bytes())?;
        file.flush()?;
        Ok(file)
    }
}
