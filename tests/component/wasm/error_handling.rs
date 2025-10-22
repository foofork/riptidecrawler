/// Error Handling and Propagation Tests
///
/// Tests WIT error conversion to host errors, error propagation through the
/// component model, and graceful degradation under error conditions.

use std::fmt;

/// WIT error variants (matching extractor.wit)
#[derive(Debug, Clone, PartialEq)]
enum ExtractionError {
    InvalidHtml(String),
    NetworkError(String),
    ParseError(String),
    ResourceLimit(String),
    ExtractorError(String),
    InternalError(String),
    UnsupportedMode(String),
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractionError::InvalidHtml(msg) => write!(f, "Invalid HTML: {}", msg),
            ExtractionError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ExtractionError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ExtractionError::ResourceLimit(msg) => write!(f, "Resource limit: {}", msg),
            ExtractionError::ExtractorError(msg) => write!(f, "Extractor error: {}", msg),
            ExtractionError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ExtractionError::UnsupportedMode(msg) => write!(f, "Unsupported mode: {}", msg),
        }
    }
}

impl std::error::Error for ExtractionError {}

/// Convert WIT errors to anyhow errors
fn wit_error_to_anyhow(error: ExtractionError) -> anyhow::Error {
    anyhow::anyhow!("{}", error)
}

/// Simulated extraction function that can fail
fn attempt_extraction(html: &str, url: &str, mode: &str) -> Result<String, ExtractionError> {
    // Validation
    if html.is_empty() {
        return Err(ExtractionError::InvalidHtml("Empty HTML content".to_string()));
    }

    if html.len() > 10 * 1024 * 1024 {
        return Err(ExtractionError::ResourceLimit("HTML exceeds 10MB limit".to_string()));
    }

    if !html.contains("<html") && !html.contains("<HTML") {
        return Err(ExtractionError::ParseError("Missing <html> tag".to_string()));
    }

    if url.is_empty() {
        return Err(ExtractionError::InvalidHtml("Empty URL".to_string()));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ExtractionError::NetworkError("Invalid URL scheme".to_string()));
    }

    match mode {
        "article" | "full" | "metadata" => {},
        "custom" => {},
        _ => return Err(ExtractionError::UnsupportedMode(format!("Unknown mode: {}", mode))),
    }

    // Simulate extraction
    if html.contains("<!--ERROR-->") {
        return Err(ExtractionError::ExtractorError("Trek-rs extraction failed".to_string()));
    }

    if html.contains("<!--PANIC-->") {
        return Err(ExtractionError::InternalError("Component panic recovered".to_string()));
    }

    Ok(format!("Extracted content from {}", url))
}

#[tokio::test]
async fn test_invalid_html_error_propagation() {
    let result = attempt_extraction("", "https://example.com", "article");

    assert!(result.is_err());
    let err = result.unwrap_err();

    match err {
        ExtractionError::InvalidHtml(msg) => {
            assert!(msg.contains("Empty HTML"));
        }
        _ => panic!("Expected InvalidHtml error"),
    }
}

#[tokio::test]
async fn test_parse_error_propagation() {
    let invalid_html = "Not HTML at all";
    let result = attempt_extraction(invalid_html, "https://example.com", "article");

    assert!(result.is_err());
    let err = result.unwrap_err();

    match err {
        ExtractionError::ParseError(msg) => {
            assert!(msg.contains("html"));
        }
        _ => panic!("Expected ParseError"),
    }
}

#[tokio::test]
async fn test_resource_limit_error() {
    let huge_html = "x".repeat(11 * 1024 * 1024); // 11MB
    let result = attempt_extraction(&huge_html, "https://example.com", "article");

    assert!(result.is_err());
    let err = result.unwrap_err();

    match err {
        ExtractionError::ResourceLimit(msg) => {
            assert!(msg.contains("10MB"));
        }
        _ => panic!("Expected ResourceLimit error"),
    }
}

#[tokio::test]
async fn test_network_error_invalid_url() {
    let html = "<html><body>Test</body></html>";
    let result = attempt_extraction(html, "ftp://invalid.com", "article");

    assert!(result.is_err());
    let err = result.unwrap_err();

    match err {
        ExtractionError::NetworkError(msg) => {
            assert!(msg.contains("Invalid URL"));
        }
        _ => panic!("Expected NetworkError"),
    }
}

#[tokio::test]
async fn test_extractor_error_propagation() {
    let html = "<html><body><!--ERROR-->Content</body></html>";
    let result = attempt_extraction(html, "https://example.com", "article");

    assert!(result.is_err());
    let err = result.unwrap_err();

    match err {
        ExtractionError::ExtractorError(msg) => {
            assert!(msg.contains("Trek-rs"));
        }
        _ => panic!("Expected ExtractorError"),
    }
}

#[tokio::test]
async fn test_internal_error_propagation() {
    let html = "<html><body><!--PANIC-->Content</body></html>";
    let result = attempt_extraction(html, "https://example.com", "article");

    assert!(result.is_err());
    let err = result.unwrap_err();

    match err {
        ExtractionError::InternalError(msg) => {
            assert!(msg.contains("panic"));
        }
        _ => panic!("Expected InternalError"),
    }
}

#[tokio::test]
async fn test_unsupported_mode_error() {
    let html = "<html><body>Test</body></html>";
    let result = attempt_extraction(html, "https://example.com", "invalid_mode");

    assert!(result.is_err());
    let err = result.unwrap_err();

    match err {
        ExtractionError::UnsupportedMode(msg) => {
            assert!(msg.contains("invalid_mode"));
        }
        _ => panic!("Expected UnsupportedMode error"),
    }
}

#[tokio::test]
async fn test_error_display_formatting() {
    let errors = vec![
        ExtractionError::InvalidHtml("test".to_string()),
        ExtractionError::NetworkError("test".to_string()),
        ExtractionError::ParseError("test".to_string()),
        ExtractionError::ResourceLimit("test".to_string()),
        ExtractionError::ExtractorError("test".to_string()),
        ExtractionError::InternalError("test".to_string()),
        ExtractionError::UnsupportedMode("test".to_string()),
    ];

    for error in errors {
        let display = format!("{}", error);
        assert!(!display.is_empty());
        assert!(display.contains("test"));
    }
}

#[tokio::test]
async fn test_error_to_anyhow_conversion() {
    let error = ExtractionError::InvalidHtml("Test error".to_string());
    let anyhow_error = wit_error_to_anyhow(error);

    let error_string = format!("{}", anyhow_error);
    assert!(error_string.contains("Invalid HTML"));
    assert!(error_string.contains("Test error"));
}

#[tokio::test]
async fn test_error_chain_propagation() {
    // Simulate nested error handling
    fn level_1() -> Result<(), ExtractionError> {
        level_2()
    }

    fn level_2() -> Result<(), ExtractionError> {
        level_3()
    }

    fn level_3() -> Result<(), ExtractionError> {
        Err(ExtractionError::ParseError("Deep error".to_string()))
    }

    let result = level_1();
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, ExtractionError::ParseError(_)));
}

#[tokio::test]
async fn test_graceful_degradation_on_error() {
    // Test that partial results can be returned even on error
    fn extract_with_fallback(html: &str) -> String {
        match attempt_extraction(html, "https://example.com", "article") {
            Ok(result) => result,
            Err(_) => {
                // Graceful degradation: return minimal content
                html.chars().take(100).collect()
            }
        }
    }

    let invalid_html = "Invalid content";
    let result = extract_with_fallback(invalid_html);

    assert!(!result.is_empty());
    assert_eq!(result, "Invalid content");
}

#[tokio::test]
async fn test_error_recovery_retry_logic() {
    let mut attempts = 0;

    let result = loop {
        attempts += 1;

        let html = if attempts == 1 {
            "invalid" // First attempt fails
        } else {
            "<html><body>Valid</body></html>" // Second attempt succeeds
        };

        match attempt_extraction(html, "https://example.com", "article") {
            Ok(result) => break Ok(result),
            Err(e) if attempts < 3 => {
                // Retry
                continue;
            }
            Err(e) => break Err(e),
        }
    };

    assert!(result.is_ok());
    assert_eq!(attempts, 2);
}

#[tokio::test]
async fn test_error_context_preservation() {
    // Test that error context is preserved through conversions
    let original_message = "Specific error details";
    let error = ExtractionError::ParseError(original_message.to_string());

    let error_string = format!("{}", error);
    assert!(error_string.contains(original_message));
}

#[tokio::test]
async fn test_concurrent_error_handling() {
    let mut handles = vec![];

    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let html = if i % 2 == 0 {
                "<html><body>Valid</body></html>"
            } else {
                "invalid"
            };

            attempt_extraction(html, "https://example.com", "article")
        });

        handles.push(handle);
    }

    let mut successes = 0;
    let mut failures = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }
    }

    assert_eq!(successes, 5);
    assert_eq!(failures, 5);
}

#[tokio::test]
async fn test_error_logging_and_metrics() {
    // Test that errors can be logged and counted
    let mut error_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    let test_cases = vec![
        ("", "https://example.com", "article"),
        ("invalid", "https://example.com", "article"),
        ("<html><body>Test</body></html>", "", "article"),
        ("<html><body>Test</body></html>", "https://example.com", "invalid"),
    ];

    for (html, url, mode) in test_cases {
        if let Err(error) = attempt_extraction(html, url, mode) {
            let error_type = format!("{:?}", error).split('(').next().unwrap().to_string();
            *error_counts.entry(error_type).or_insert(0) += 1;
        }
    }

    assert!(!error_counts.is_empty());
    assert!(error_counts.values().sum::<u32>() == 4);
}

#[tokio::test]
async fn test_error_equals_and_clone() {
    let error1 = ExtractionError::ParseError("test".to_string());
    let error2 = error1.clone();

    assert_eq!(error1, error2);
}

#[tokio::test]
async fn test_successful_extraction_no_error() {
    let html = "<html><head><title>Test</title></head><body>Content</body></html>";
    let result = attempt_extraction(html, "https://example.com", "article");

    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("https://example.com"));
}
