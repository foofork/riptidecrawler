//! Native HTML parser module for headless-rendered content
//!
//! This module provides a native Rust HTML parser that uses the `scraper` crate
//! to extract content from headless-rendered HTML, bypassing WASM entirely.

pub mod error;
pub mod extractors;
pub mod fallbacks;
pub mod parser;
pub mod quality;

#[cfg(test)]
mod tests;

// Re-export main types
pub use error::{NativeParserError, Result};
pub use parser::{NativeHtmlParser, ParserConfig};
