//! Table extraction and conversion module
//!
//! This module provides table extraction from HTML and conversion to various formats.

mod converter;
mod extractor;
mod parser;
mod types;

#[cfg(test)]
mod tests;

pub use converter::TableConverter;
pub use extractor::{ApiClient, TableExtractor, TableSource};
pub use parser::{parse_content_to_table_data, parse_csv_to_data, parse_markdown_to_data};
pub use types::{
    TableData, TableExtractRequest, TableExtractResponse, TableMetadata, TableSummary,
};
