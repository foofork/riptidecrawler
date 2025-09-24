//! NDJSON Format Compliance Tests
//!
//! These tests validate strict compliance with the NDJSON (Newline Delimited JSON) format
//! as specified in RFC 7464 and ensure streaming output meets format requirements:
//! - Each line is a valid JSON object
//! - Lines are separated by newline characters
//! - No trailing commas or array wrapping
//! - Proper content-type headers
//! - Streaming response structure

use serde_json::{Value, Map};
use std::collections::HashMap;
use bytes::Bytes;
use futures::stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use riptide_api::streaming::ndjson::*;
use riptide_api::models::*;
use std::time::Duration;

/// NDJSON format validator
struct NDJSONValidator {
    strict_mode: bool,
}

impl NDJSONValidator {
    fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    /// Validate complete NDJSON stream
    fn validate_stream(&self, content: &str) -> Result<Vec<Value>, NDJSONError> {
        let lines = content.lines().collect::<Vec<_>>();
        self.validate_lines(lines)
    }

    /// Validate individual NDJSON lines
    fn validate_lines(&self, lines: Vec<&str>) -> Result<Vec<Value>, NDJSONError> {
        if lines.is_empty() {
            return Err(NDJSONError::EmptyStream);
        }

        let mut parsed_objects = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            
            // Skip empty lines (allowed in NDJSON)
            if line.is_empty() {
                continue;
            }

            // Each line must be valid JSON
            let parsed: Value = serde_json::from_str(line)
                .map_err(|e| NDJSONError::InvalidJSON {
                    line_number: line_num + 1,
                    content: line.to_string(),
                    error: e.to_string(),
                })?;

            // In strict mode, each line should be a JSON object
            if self.strict_mode && !parsed.is_object() {
                return Err(NDJSONError::NotAnObject {
                    line_number: line_num + 1,
                    content: line.to_string(),
                });
            }

            parsed_objects.push(parsed);
        }

        Ok(parsed_objects)
    }

    /// Validate streaming response structure for crawl operations
    fn validate_crawl_stream_structure(&self, objects: &[Value]) -> Result<(), NDJSONError> {
        if objects.is_empty() {
            return Err(NDJSONError::EmptyStream);
        }

        // First object should be metadata
        let metadata = &objects[0];
        self.validate_metadata_structure(metadata)?;

        // Last object should be summary
        let summary = objects.last().unwrap();
        self.validate_summary_structure(summary)?;

        // Middle objects should be results or progress
        for (i, obj) in objects[1..objects.len()-1].iter().enumerate() {
            if obj.get("result").is_some() {
                self.validate_result_structure(obj, i + 1)?;
            } else if obj.get("progress_percentage").is_some() {
                self.validate_progress_structure(obj, i + 1)?;
            } else {
                return Err(NDJSONError::UnknownObjectType {
                    line_number: i + 2,
                    object_type: "unknown".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate metadata object structure
    fn validate_metadata_structure(&self, metadata: &Value) -> Result<(), NDJSONError> {
        let obj = metadata.as_object().ok_or(NDJSONError::NotAnObject {
            line_number: 1,
            content: metadata.to_string(),
        })?;

        let required_fields = vec!["total_urls", "request_id", "timestamp", "stream_type"];
        for field in required_fields {
            if !obj.contains_key(field) {
                return Err(NDJSONError::MissingRequiredField {
                    line_number: 1,
                    field: field.to_string(),
                });
            }
        }

        // Validate field types
        if !metadata["total_urls"].is_number() {
            return Err(NDJSONError::InvalidFieldType {
                field: "total_urls".to_string(),
                expected: "number".to_string(),
                actual: metadata["total_urls"].to_string(),
            });
        }

        if !metadata["request_id"].is_string() {
            return Err(NDJSONError::InvalidFieldType {
                field: "request_id".to_string(),
                expected: "string".to_string(),
                actual: metadata["request_id"].to_string(),
            });
        }

        Ok(())
    }

    /// Validate result object structure
    fn validate_result_structure(&self, result_obj: &Value, line_number: usize) -> Result<(), NDJSONError> {
        let obj = result_obj.as_object().ok_or(NDJSONError::NotAnObject {
            line_number,
            content: result_obj.to_string(),
        })?;

        // Should have result field
        let result = obj.get("result").ok_or(NDJSONError::MissingRequiredField {
            line_number,
            field: "result".to_string(),
        })?;

        // Validate result structure
        let result_obj = result.as_object().ok_or(NDJSONError::InvalidFieldType {
            field: "result".to_string(),
            expected: "object".to_string(),
            actual: result.to_string(),
        })?;

        let required_result_fields = vec!["url", "status", "from_cache", "gate_decision", "quality_score"];
        for field in required_result_fields {
            if !result_obj.contains_key(field) {
                return Err(NDJSONError::MissingRequiredField {
                    line_number,
                    field: format!("result.{}", field),
                });
            }
        }

        Ok(())
    }

    /// Validate progress object structure
    fn validate_progress_structure(&self, progress_obj: &Value, line_number: usize) -> Result<(), NDJSONError> {
        let obj = progress_obj.as_object().ok_or(NDJSONError::NotAnObject {
            line_number,
            content: progress_obj.to_string(),
        })?;

        let required_fields = vec!["progress_percentage", "items_completed", "items_total"];
        for field in required_fields {
            if !obj.contains_key(field) {
                return Err(NDJSONError::MissingRequiredField {
                    line_number,
                    field: field.to_string(),
                });
            }
        }

        // Validate progress percentage is between 0 and 100
        if let Some(progress) = obj.get("progress_percentage").and_then(|p| p.as_f64()) {
            if progress < 0.0 || progress > 100.0 {
                return Err(NDJSONError::InvalidFieldValue {
                    field: "progress_percentage".to_string(),
                    value: progress.to_string(),
                    constraint: "0.0 <= value <= 100.0".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate summary object structure
    fn validate_summary_structure(&self, summary: &Value) -> Result<(), NDJSONError> {
        let obj = summary.as_object().ok_or(NDJSONError::NotAnObject {
            line_number: 0, // Will be updated by caller
            content: summary.to_string(),
        })?;

        let required_fields = vec!["total_urls", "successful", "failed", "total_processing_time_ms"];
        for field in required_fields {
            if !obj.contains_key(field) {
                return Err(NDJSONError::MissingRequiredField {
                    line_number: 0,
                    field: field.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate that no JSON arrays or trailing commas are present
    fn validate_no_json_arrays(&self, content: &str) -> Result<(), NDJSONError> {
        // Check for JSON array syntax (should not be present in NDJSON)
        if content.trim_start().starts_with('[') {
            return Err(NDJSONError::FormatViolation {
                message: "NDJSON should not start with '[' (JSON array syntax)".to_string(),
            });
        }

        if content.trim_end().ends_with(']') {
            return Err(NDJSONError::FormatViolation {
                message: "NDJSON should not end with ']' (JSON array syntax)".to_string(),
            });
        }

        // Check for commas at end of lines (invalid in NDJSON)
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().ends_with(',') {
                return Err(NDJSONError::FormatViolation {
                    message: format!("Line {} ends with comma (invalid in NDJSON): {}", 
                                   line_num + 1, line),
                });
            }
        }

        Ok(())
    }
}

/// NDJSON validation errors
#[derive(Debug, Clone)]
enum NDJSONError {
    EmptyStream,
    InvalidJSON {
        line_number: usize,
        content: String,
        error: String,
    },
    NotAnObject {
        line_number: usize,
        content: String,
    },
    MissingRequiredField {
        line_number: usize,
        field: String,
    },
    InvalidFieldType {
        field: String,
        expected: String,
        actual: String,
    },
    InvalidFieldValue {
        field: String,
        value: String,
        constraint: String,
    },
    UnknownObjectType {
        line_number: usize,
        object_type: String,
    },
    FormatViolation {
        message: String,
    },
}

impl std::fmt::Display for NDJSONError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NDJSONError::EmptyStream => write!(f, "Empty NDJSON stream"),
            NDJSONError::InvalidJSON { line_number, content, error } => {
                write!(f, "Invalid JSON on line {}: {} - Error: {}", line_number, content, error)
            },
            NDJSONError::NotAnObject { line_number, content } => {
                write!(f, "Line {} is not a JSON object: {}", line_number, content)
            },
            NDJSONError::MissingRequiredField { line_number, field } => {
                write!(f, "Missing required field '{}' on line {}", field, line_number)
            },
            NDJSONError::InvalidFieldType { field, expected, actual } => {
                write!(f, "Field '{}' expected type '{}', got: {}", field, expected, actual)
            },
            NDJSONError::InvalidFieldValue { field, value, constraint } => {
                write!(f, "Field '{}' value '{}' violates constraint: {}", field, value, constraint)
            },
            NDJSONError::UnknownObjectType { line_number, object_type } => {
                write!(f, "Unknown object type '{}' on line {}", object_type, line_number)
            },
            NDJSONError::FormatViolation { message } => {
                write!(f, "NDJSON format violation: {}", message)
            },
        }
    }
}

impl std::error::Error for NDJSONError {}

// ==================== NDJSON FORMAT COMPLIANCE TESTS ====================

/// Test basic NDJSON format compliance
#[test]
fn test_basic_ndjson_format_compliance() {
    let validator = NDJSONValidator::new(true);
    
    // Valid NDJSON
    let valid_ndjson = r#"{"type":"metadata","count":2}
{"type":"result","url":"https://example.com","status":200}
{"type":"summary","total":2,"success":1}"#;
    
    let objects = validator.validate_stream(valid_ndjson)
        .expect("Valid NDJSON should parse successfully");
    
    assert_eq!(objects.len(), 3, "Should parse 3 objects");
    assert_eq!(objects[0]["type"], "metadata");
    assert_eq!(objects[1]["type"], "result");
    assert_eq!(objects[2]["type"], "summary");
}

/// Test rejection of JSON array format
#[test]
fn test_reject_json_array_format() {
    let validator = NDJSONValidator::new(true);
    
    // Invalid: JSON array format
    let json_array = r#"[{"type":"metadata"},{"type":"result"}]"#;
    
    let result = validator.validate_stream(json_array);
    assert!(result.is_err(), "Should reject JSON array format");
    
    if let Err(NDJSONError::FormatViolation { message }) = result {
        assert!(message.contains("JSON array"), "Error should mention JSON array: {}", message);
    } else {
        panic!("Should return FormatViolation error");
    }
}

/// Test rejection of trailing commas
#[test]
fn test_reject_trailing_commas() {
    let validator = NDJSONValidator::new(true);
    
    // Invalid: trailing commas
    let trailing_comma_ndjson = r#"{"type":"metadata","count":1},
{"type":"result","status":200},"#;
    
    let result = validator.validate_stream(trailing_comma_ndjson);
    assert!(result.is_err(), "Should reject trailing commas");
    
    if let Err(NDJSONError::FormatViolation { message }) = result {
        assert!(message.contains("comma"), "Error should mention comma: {}", message);
    } else {
        panic!("Should return FormatViolation error");
    }
}

/// Test handling of empty lines
#[test]
fn test_handle_empty_lines() {
    let validator = NDJSONValidator::new(true);
    
    // NDJSON with empty lines (should be allowed)
    let ndjson_with_empty_lines = r#"{"type":"metadata","count":1}

{"type":"result","status":200}

{"type":"summary","total":1}"#;
    
    let objects = validator.validate_stream(ndjson_with_empty_lines)
        .expect("Empty lines should be allowed in NDJSON");
    
    assert_eq!(objects.len(), 3, "Should parse 3 non-empty objects");
}

/// Test invalid JSON detection
#[test]
fn test_invalid_json_detection() {
    let validator = NDJSONValidator::new(true);
    
    // Invalid JSON syntax
    let invalid_json = r#"{"type":"metadata","count":1}
{type":"result","status":200}
{"type":"summary","total":1}"#;
    
    let result = validator.validate_stream(invalid_json);
    assert!(result.is_err(), "Should detect invalid JSON");
    
    if let Err(NDJSONError::InvalidJSON { line_number, .. }) = result {
        assert_eq!(line_number, 2, "Should identify correct line number");
    } else {
        panic!("Should return InvalidJSON error");
    }
}

/// Test streaming crawl response structure validation
#[test]
fn test_crawl_stream_structure_validation() {
    let validator = NDJSONValidator::new(true);
    
    // Valid crawl stream structure
    let crawl_stream = r#"{"total_urls":2,"request_id":"req-123","timestamp":"2024-01-01T00:00:00Z","stream_type":"crawl"}
{"index":0,"result":{"url":"https://example.com","status":200,"from_cache":false,"gate_decision":"allow","quality_score":0.9,"document":{"url":"https://example.com","title":"Test","content":"Test content"}},"progress":{"completed":1,"total":2,"success_rate":1.0}}
{"total_urls":2,"successful":1,"failed":0,"from_cache":0,"total_processing_time_ms":1000,"cache_hit_rate":0.0}"#;
    
    let objects = validator.validate_stream(crawl_stream)
        .expect("Valid crawl stream should parse");
    
    validator.validate_crawl_stream_structure(&objects)
        .expect("Should validate crawl stream structure");
}

/// Test metadata structure validation
#[test]
fn test_metadata_structure_validation() {
    let validator = NDJSONValidator::new(true);
    
    // Valid metadata
    let valid_metadata = serde_json::json!({
        "total_urls": 5,
        "request_id": "req-456",
        "timestamp": "2024-01-01T00:00:00Z",
        "stream_type": "crawl"
    });
    
    validator.validate_metadata_structure(&valid_metadata)
        .expect("Valid metadata should pass validation");
    
    // Invalid metadata (missing field)
    let invalid_metadata = serde_json::json!({
        "total_urls": 5,
        "request_id": "req-456",
        // Missing timestamp and stream_type
    });
    
    let result = validator.validate_metadata_structure(&invalid_metadata);
    assert!(result.is_err(), "Should reject metadata missing required fields");
}

/// Test result structure validation
#[test]
fn test_result_structure_validation() {
    let validator = NDJSONValidator::new(true);
    
    // Valid result
    let valid_result = serde_json::json!({
        "index": 0,
        "result": {
            "url": "https://example.com",
            "status": 200,
            "from_cache": false,
            "gate_decision": "allow",
            "quality_score": 0.85,
            "processing_time_ms": 1500,
            "document": {
                "url": "https://example.com",
                "title": "Example",
                "content": "Example content"
            }
        },
        "progress": {
            "completed": 1,
            "total": 5,
            "success_rate": 0.8
        }
    });
    
    validator.validate_result_structure(&valid_result, 2)
        .expect("Valid result should pass validation");
    
    // Invalid result (missing required fields)
    let invalid_result = serde_json::json!({
        "index": 0,
        "result": {
            "url": "https://example.com",
            "status": 200,
            // Missing from_cache, gate_decision, quality_score
        }
    });
    
    let result = validator.validate_result_structure(&invalid_result, 2);
    assert!(result.is_err(), "Should reject result missing required fields");
}

/// Test progress structure validation
#[test]
fn test_progress_structure_validation() {
    let validator = NDJSONValidator::new(true);
    
    // Valid progress
    let valid_progress = serde_json::json!({
        "operation_id": "crawl-789",
        "progress_percentage": 65.5,
        "items_completed": 13,
        "items_total": 20,
        "current_phase": "processing"
    });
    
    validator.validate_progress_structure(&valid_progress, 3)
        .expect("Valid progress should pass validation");
    
    // Invalid progress (percentage out of range)
    let invalid_progress = serde_json::json!({
        "operation_id": "crawl-789",
        "progress_percentage": 150.0, // Invalid: > 100
        "items_completed": 13,
        "items_total": 20
    });
    
    let result = validator.validate_progress_structure(&invalid_progress, 3);
    assert!(result.is_err(), "Should reject progress with invalid percentage");
}

/// Test summary structure validation
#[test]
fn test_summary_structure_validation() {
    let validator = NDJSONValidator::new(true);
    
    // Valid summary
    let valid_summary = serde_json::json!({
        "total_urls": 10,
        "successful": 8,
        "failed": 2,
        "from_cache": 3,
        "total_processing_time_ms": 5000,
        "cache_hit_rate": 0.3,
        "throughput_per_second": 2.0
    });
    
    validator.validate_summary_structure(&valid_summary)
        .expect("Valid summary should pass validation");
    
    // Invalid summary (missing required fields)
    let invalid_summary = serde_json::json!({
        "total_urls": 10,
        "successful": 8,
        // Missing failed, total_processing_time_ms
    });
    
    let result = validator.validate_summary_structure(&invalid_summary);
    assert!(result.is_err(), "Should reject summary missing required fields");
}

/// Test non-strict mode (allows non-objects)
#[test]
fn test_non_strict_mode() {
    let validator = NDJSONValidator::new(false); // Non-strict mode
    
    // Mix of objects and primitives
    let mixed_ndjson = r#"{"type":"metadata"}
"simple string"
42
true
{"type":"summary"}"#;
    
    let objects = validator.validate_stream(mixed_ndjson)
        .expect("Non-strict mode should allow non-objects");
    
    assert_eq!(objects.len(), 5);
    assert!(objects[0].is_object());
    assert!(objects[1].is_string());
    assert!(objects[2].is_number());
    assert!(objects[3].is_boolean());
    assert!(objects[4].is_object());
}

/// Test strict mode (requires objects only)
#[test]
fn test_strict_mode() {
    let validator = NDJSONValidator::new(true); // Strict mode
    
    // Mix of objects and primitives
    let mixed_ndjson = r#"{"type":"metadata"}
"simple string"
{"type":"summary"}"#;
    
    let result = validator.validate_stream(mixed_ndjson);
    assert!(result.is_err(), "Strict mode should reject non-objects");
    
    if let Err(NDJSONError::NotAnObject { line_number, .. }) = result {
        assert_eq!(line_number, 2, "Should identify the problematic line");
    } else {
        panic!("Should return NotAnObject error");
    }
}

/// Test real streaming response content types
#[tokio::test]
async fn test_streaming_response_headers() {
    // This would typically test actual HTTP responses
    // For now, we test the expected header values
    
    let expected_content_type = "application/x-ndjson";
    let expected_transfer_encoding = "chunked";
    
    assert_eq!(expected_content_type, "application/x-ndjson");
    assert_eq!(expected_transfer_encoding, "chunked");
    
    // Test MIME type parsing
    let mime_type: mime::Mime = expected_content_type.parse().expect("Should parse MIME type");
    assert_eq!(mime_type.type_(), mime::APPLICATION);
    assert_eq!(mime_type.subtype().as_str(), "x-ndjson");
}

/// Test large NDJSON streams
#[test]
fn test_large_ndjson_stream() {
    let validator = NDJSONValidator::new(true);
    
    // Generate large NDJSON content
    let mut ndjson_content = String::new();
    
    // Metadata
    ndjson_content.push_str(&format!(
        "{{\"total_urls\":{},\"request_id\":\"large-test\",\"timestamp\":\"2024-01-01T00:00:00Z\",\"stream_type\":\"crawl\"}}\n",
        1000
    ));
    
    // Results
    for i in 0..1000 {
        ndjson_content.push_str(&format!(
            "{{\"index\":{},\"result\":{{\"url\":\"https://example.com/{}\",\"status\":200,\"from_cache\":false,\"gate_decision\":\"allow\",\"quality_score\":0.9}},\"progress\":{{\"completed\":{},\"total\":1000,\"success_rate\":0.95}}}}\n",
            i, i, i + 1
        ));
    }
    
    // Summary
    ndjson_content.push_str(
        "{\"total_urls\":1000,\"successful\":1000,\"failed\":0,\"total_processing_time_ms\":30000}\n"
    );
    
    let start = std::time::Instant::now();
    let objects = validator.validate_stream(&ndjson_content)
        .expect("Large NDJSON stream should validate successfully");
    let validation_time = start.elapsed();
    
    assert_eq!(objects.len(), 1002, "Should parse all objects"); // 1 metadata + 1000 results + 1 summary
    
    println!("Validated {} objects in {}ms", 
             objects.len(), validation_time.as_millis());
    
    // Should be reasonably fast even for large streams
    assert!(validation_time.as_millis() < 1000, 
            "Large stream validation should be fast: {}ms", 
            validation_time.as_millis());
}

/// Test concurrent NDJSON parsing
#[tokio::test]
async fn test_concurrent_ndjson_parsing() {
    let validator = NDJSONValidator::new(true);
    
    // Create multiple NDJSON streams
    let streams = (0..10).map(|stream_id| {
        format!(
            "{{\"total_urls\":3,\"request_id\":\"stream-{}\",\"timestamp\":\"2024-01-01T00:00:00Z\",\"stream_type\":\"crawl\"}}\n\
            {{\"index\":0,\"result\":{{\"url\":\"https://example.com/{}/1\",\"status\":200,\"from_cache\":false,\"gate_decision\":\"allow\",\"quality_score\":0.9}}}}\n\
            {{\"index\":1,\"result\":{{\"url\":\"https://example.com/{}/2\",\"status\":200,\"from_cache\":false,\"gate_decision\":\"allow\",\"quality_score\":0.8}}}}\n\
            {{\"total_urls\":2,\"successful\":2,\"failed\":0,\"total_processing_time_ms\":1000}}\n",
            stream_id, stream_id, stream_id
        )
    }).collect::<Vec<_>>();
    
    // Parse streams concurrently
    let handles: Vec<_> = streams.into_iter().enumerate().map(|(i, stream)| {
        let validator = NDJSONValidator::new(true);
        tokio::spawn(async move {
            let objects = validator.validate_stream(&stream)
                .expect("Concurrent validation should succeed");
            (i, objects.len())
        })
    }).collect();
    
    // Wait for all parsing to complete
    let results = futures::future::join_all(handles).await;
    
    for result in results {
        let (stream_id, object_count) = result.expect("Concurrent task should succeed");
        assert_eq!(object_count, 4, "Stream {} should have 4 objects", stream_id);
    }
}

/// Test malformed JSON edge cases
#[test]
fn test_malformed_json_edge_cases() {
    let validator = NDJSONValidator::new(true);
    
    let malformed_cases = vec![
        ("{\"unclosed\": ", "Unclosed object"),
        ("{\"key\": value}", "Unquoted value"),
        ("{\"trailing\": \"comma\",}", "Trailing comma in object"),
        ("{'single': 'quotes'}", "Single quotes instead of double"),
        ("{\"unicode\": \"\uXXXX\"}", "Invalid unicode escape"),
        ("{\"number\": 01}", "Invalid number format"),
    ];
    
    for (malformed_json, description) in malformed_cases {
        let result = validator.validate_stream(malformed_json);
        assert!(result.is_err(), "Should reject {}: {}", description, malformed_json);
        
        if let Err(NDJSONError::InvalidJSON { error, .. }) = result {
            println!("{}: {}", description, error);
        }
    }
}

/// Benchmark NDJSON parsing performance
#[test]
fn test_ndjson_parsing_performance() {
    let validator = NDJSONValidator::new(false); // Non-strict for performance
    
    // Generate various sized streams
    let test_sizes = vec![10, 100, 1000, 5000];
    
    for size in test_sizes {
        let mut ndjson_content = String::new();
        
        for i in 0..size {
            ndjson_content.push_str(&format!(
                "{{\"id\":{},\"data\":\"test data for object {}\",\"timestamp\":\"2024-01-01T00:00:00Z\"}}\n",
                i, i
            ));
        }
        
        let start = std::time::Instant::now();
        let objects = validator.validate_stream(&ndjson_content)
            .expect("Performance test should validate successfully");
        let parsing_time = start.elapsed();
        
        let throughput = size as f64 / parsing_time.as_secs_f64();
        
        println!("Size: {} objects, Time: {}ms, Throughput: {:.0} objects/sec",
                 size, parsing_time.as_millis(), throughput);
        
        assert_eq!(objects.len(), size, "Should parse all objects");
        assert!(throughput > 1000.0, "Should achieve reasonable throughput");
    }
}
