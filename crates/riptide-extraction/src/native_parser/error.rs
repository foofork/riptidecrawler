//! Error types for native HTML parser

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NativeParserError {
    #[error("HTML parsing failed: {0}")]
    ParseError(String),

    #[error("HTML exceeds maximum size: {size} bytes (max: {max})")]
    OversizedHtml { size: usize, max: usize },

    #[error("Invalid UTF-8 encoding: {0}")]
    EncodingError(String),

    #[error("Extraction timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Document structure invalid: {0}")]
    InvalidStructure(String),

    #[error("No extractable content found")]
    NoContentFound,

    #[error("Quality too low: {score} (threshold: {threshold})")]
    LowQuality { score: f32, threshold: f32 },

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, NativeParserError>;
