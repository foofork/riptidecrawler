//! Domain types for facade layer - Replaces serde_json::Value usage
//!
//! This module provides strongly-typed domain models for facade operations,
//! eliminating the use of serde_json::Value and providing compile-time type safety.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

// ============================================================================
// Pipeline Facade Domain Types (Sprint 5.1)
// ============================================================================

/// Output from different pipeline stages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum PipelineOutput {
    /// Result from fetch stage
    Fetch(FetchData),
    /// Result from extraction stage
    Extract(ExtractedData),
    /// Result from transformation stage
    Transform(TransformedData),
    /// Result from validation stage
    Validate(ValidationResult),
    /// Result from storage stage
    Store(StorageConfirmation),
}

/// Data fetched from a URL
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FetchData {
    /// Source URL
    pub url: String,
    /// Raw content bytes (serde handles Vec<u8> as array of numbers in JSON)
    pub content: Vec<u8>,
    /// Timestamp of fetch
    #[serde(with = "systemtime_serde")]
    pub timestamp: SystemTime,
    /// HTTP metadata (headers, status, etc.)
    pub metadata: HashMap<String, String>,
    /// Content type from headers
    pub content_type: Option<String>,
    /// HTTP status code
    pub status_code: u16,
}

/// Extracted data from content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedData {
    /// Extracted title
    pub title: Option<String>,
    /// Main text content
    pub text: String,
    /// Extracted links
    pub links: Vec<String>,
    /// Extracted images
    pub images: Vec<String>,
    /// Metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// Extraction strategy used
    pub strategy: String,
}

/// Transformed data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformedData {
    /// Transformed content
    pub content: String,
    /// Transformation applied
    pub transformation: String,
    /// Additional metadata from transformation
    pub metadata: HashMap<String, String>,
}

/// Result of validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors if any
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Validated content
    pub content: String,
}

/// Confirmation of storage operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageConfirmation {
    /// Storage destination
    pub destination: String,
    /// Storage key/path
    pub key: String,
    /// Timestamp of storage
    #[serde(with = "systemtime_serde")]
    pub stored_at: SystemTime,
    /// Size in bytes
    pub size_bytes: usize,
}

// ============================================================================
// Browser Facade Domain Types (Sprint 5.2)
// ============================================================================

/// Result from JavaScript execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptResult {
    /// The actual value returned
    pub value: ScriptValue,
    /// Type of the value
    pub value_type: ScriptValueType,
}

/// JavaScript value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum ScriptValue {
    /// String value
    String(String),
    /// Number value (f64 covers both int and float in JS)
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Object with string keys and values
    Object(HashMap<String, String>),
    /// Array of strings (simplified for common use case)
    Array(Vec<String>),
    /// Null value
    Null,
    /// Undefined value
    Undefined,
}

/// Type classification for script values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScriptValueType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
    Undefined,
}

/// Browser local storage data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LocalStorage {
    /// Storage entries (key-value pairs)
    #[serde(default)]
    pub entries: HashMap<String, String>,
    /// Total size in bytes
    #[serde(default)]
    pub size_bytes: usize,
}

// ============================================================================
// Extractor Facade Domain Types (Sprint 5.3)
// ============================================================================

/// Result of schema-based extraction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SchemaExtractionResult {
    /// Successfully extracted fields
    #[serde(default)]
    pub fields: HashMap<String, FieldValue>,
    /// List of required fields that were missing
    #[serde(default)]
    pub missing_required: Vec<String>,
    /// Warnings encountered during extraction
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Value extracted for a schema field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum FieldValue {
    /// Text field
    Text(String),
    /// Numeric field
    Number(f64),
    /// URL field
    Url(String),
    /// Date field (ISO 8601 string)
    Date(String),
    /// Field was present but empty
    Empty,
    /// Field was not found
    Missing,
}

impl FieldValue {
    /// Check if field has a value
    pub fn has_value(&self) -> bool {
        !matches!(self, FieldValue::Empty | FieldValue::Missing)
    }

    /// Get the value as a string if possible
    pub fn as_string(&self) -> Option<String> {
        match self {
            FieldValue::Text(s) | FieldValue::Url(s) | FieldValue::Date(s) => Some(s.clone()),
            FieldValue::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }
}

// ============================================================================
// Helper modules for serialization
// ============================================================================

mod systemtime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

// ============================================================================
// Conversion implementations
// ============================================================================

impl From<ScriptValue> for ScriptValueType {
    fn from(value: ScriptValue) -> Self {
        match value {
            ScriptValue::String(_) => ScriptValueType::String,
            ScriptValue::Number(_) => ScriptValueType::Number,
            ScriptValue::Boolean(_) => ScriptValueType::Boolean,
            ScriptValue::Object(_) => ScriptValueType::Object,
            ScriptValue::Array(_) => ScriptValueType::Array,
            ScriptValue::Null => ScriptValueType::Null,
            ScriptValue::Undefined => ScriptValueType::Undefined,
        }
    }
}

impl Default for PipelineOutput {
    fn default() -> Self {
        PipelineOutput::Fetch(FetchData {
            url: String::new(),
            content: Vec::new(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            content_type: None,
            status_code: 0,
        })
    }
}

impl Default for ScriptResult {
    fn default() -> Self {
        ScriptResult {
            value: ScriptValue::Null,
            value_type: ScriptValueType::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_output_serialization() {
        let fetch_data = FetchData {
            url: "https://example.com".to_string(),
            content: b"test content".to_vec(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
            content_type: Some("text/html".to_string()),
            status_code: 200,
        };

        let output = PipelineOutput::Fetch(fetch_data);
        let json = serde_json::to_string(&output).unwrap();
        let deserialized: PipelineOutput = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, PipelineOutput::Fetch(_)));
    }

    #[test]
    fn test_script_result() {
        let result = ScriptResult {
            value: ScriptValue::String("test".to_string()),
            value_type: ScriptValueType::String,
        };

        assert_eq!(result.value_type, ScriptValueType::String);
    }

    #[test]
    fn test_field_value_has_value() {
        assert!(FieldValue::Text("test".to_string()).has_value());
        assert!(!FieldValue::Missing.has_value());
        assert!(!FieldValue::Empty.has_value());
    }

    #[test]
    fn test_local_storage_default() {
        let storage = LocalStorage::default();
        assert_eq!(storage.size_bytes, 0);
        assert!(storage.entries.is_empty());
    }

    #[test]
    fn test_schema_extraction_result() {
        let mut result = SchemaExtractionResult::default();
        result
            .fields
            .insert("title".to_string(), FieldValue::Text("Test".to_string()));
        result.missing_required.push("author".to_string());

        assert_eq!(result.fields.len(), 1);
        assert_eq!(result.missing_required.len(), 1);
    }
}
