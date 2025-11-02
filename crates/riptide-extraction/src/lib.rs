//! # RipTide Extraction Crate
//!
//! This crate provides HTML processing and content extraction capabilities for the RipTide project.
//! It includes CSS selector-based extraction, regex pattern extraction, DOM traversal utilities,
//! and table extraction interfaces.
//!
//! ## Features
//!
//! - **CSS Extraction**: Extract content using CSS selectors with JSON mapping
//! - **Regex Extraction**: Pattern-based content extraction with configurable rules
//! - **DOM Utils**: Utilities for DOM traversal and manipulation
//! - **Table Extraction**: Interface for extracting structured data from HTML tables
//! - **Chunking**: Content chunking interface for processing large documents
//!
//! ## Usage
//!
//! ```rust
//! use riptide_extraction::{HtmlProcessor, css_extraction, regex_extraction};
//! use std::collections::HashMap;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;
//!
//! // CSS extraction with default selectors
//! let result = css_extraction::extract_default(html, "https://example.com").await?;
//! println!("Title: {}", result.title);
//!
//! // Regex extraction with custom patterns
//! let patterns = regex_extraction::default_patterns();
//! let result = regex_extraction::extract(html, "https://example.com", &patterns).await?;
//! println!("Content: {}", result.content);
//! # Ok(())
//! # }
//! ```

// P1-A3 Phase 2D: Extraction domain modules moved from riptide-core
pub mod composition;
pub mod confidence;
pub mod confidence_integration;

pub mod css_extraction;
pub mod dom_utils;
pub mod processor;
pub mod regex_extraction;
// Strategy implementations module disabled due to circular dependency with riptide-core
// Re-enable once types are moved to a shared crate or dependency cycle is resolved
// pub mod strategy_implementations;
pub mod extraction_strategies;

// WASM extraction module (only with wasm-extractor feature)
#[cfg(feature = "wasm-extractor")]
pub mod wasm_extraction;

// Unified extractor with three-tier fallback
pub mod unified_extractor;

// pub mod spider;  // Temporarily disabled due to compilation errors
pub mod chunking;
pub mod enhanced_extractor;
pub mod table_extraction;
pub mod tables;

// P1-C2: HTML parser and extraction strategies moved from riptide-core
pub mod html_parser;
pub mod strategies;

// P2-F1 Day 3: WASM validation moved from riptide-core (only with wasm-extractor feature)
#[cfg(feature = "wasm-extractor")]
pub mod validation;

#[cfg(feature = "wasm-extractor")]
pub use validation::{
    validate_before_instantiation, ComponentMetadata, TypeMismatch, TypeSignature,
    ValidationReport, WitValidator,
};

// Re-export main interfaces
pub use css_extraction::{
    default_selectors, extract as css_extract, extract_default as css_extract_default,
    CssJsonExtractor,
};
pub use dom_utils::{
    extract_text_content, find_tables, traverse_elements, DomTraverser, ElementInfo,
};
pub use enhanced_extractor::StructuredExtractor;
pub use extraction_strategies::{
    extract_links_basic, fallback_extract, ContentExtractor, CssExtractorStrategy,
};

// Conditional exports for WASM extraction
#[cfg(feature = "wasm-extractor")]
pub use extraction_strategies::WasmExtractor as StrategyWasmExtractor;

#[cfg(feature = "wasm-extractor")]
pub use wasm_extraction::{
    CmExtractor, ExtractorConfig, HostExtractionMode, WasmExtractor, WasmResourceTracker,
};

// Always export unified extractor and native extractor
pub use unified_extractor::{NativeExtractor, UnifiedExtractor};

pub use processor::{
    ChunkingMode, HtmlProcessor, ProcessingError, ProcessingResult, TableExtractionMode,
};
pub use regex_extraction::{default_patterns, extract as regex_extract, RegexExtractor};
// Re-export ExtractedDoc from riptide-types
pub use riptide_types::ExtractedDoc;

// Re-export HTML parser types (moved from riptide-core)
pub use html_parser::{Link, Media, MediaType, Metadata};

// Enhanced link extraction with context and classification
pub mod enhanced_link_extraction;
pub use enhanced_link_extraction::{
    EnhancedLinkExtractor, ExtractedLink, LinkExtractionConfig, LinkType,
};

// Re-export strategies (moved from riptide-core)
// Note: ExtractionStrategy is a TRAIT defined in strategies::traits
// ExtractionStrategyType is an ENUM for configuration
pub use strategies::{
    ExtractionStrategy,     // Trait for implementing custom extractors
    ExtractionStrategyType, // Enum for selecting built-in strategies
    PerformanceMetrics,
    StrategyManager,
};

// Re-export confidence (Phase 2D)
pub use composition::{CompositionMode, StrategyComposer};
pub use confidence::{AggregationStrategy, ConfidenceScore, ConfidenceScorer};
pub use confidence_integration::{CssConfidenceScorer, WasmConfidenceScorer};

// // Re-export spider functionality
// pub use spider::{
//     DomSpider, DomCrawlerResult, FormData, MetaData,
//     HtmlDomCrawler, HtmlLinkExtractor, HtmlFormParser, HtmlMetaExtractor
// };

// // Re-export spider traits and types
// pub use spider::traits::{
//     FormField, ContentAnalysis, ContentType, NavigationHint, DomSpiderConfig
// };

// Strategy trait implementations (only available with strategy-traits feature)
#[cfg(feature = "strategy-traits")]
// Disabled due to circular dependency
// pub use strategy_implementations::{
//     HtmlCssExtractionStrategy, HtmlRegexExtractionStrategy, HtmlProcessorStrategy
// };
// Re-export chunking functionality
pub use chunking::{
    create_strategy, utils as chunking_utils, Chunk, ChunkMetadata, ChunkingConfig,
    ChunkingMode as ChunkingStrategyMode, ChunkingStrategy,
};

// Re-export table extraction functionality
pub use table_extraction::{
    extract_and_export_tables, extract_tables_advanced, AdvancedTableData, CellPosition, CellType,
    ColumnGroup, RowType, TableArtifact, TableCell, TableExtractionConfig, TableExtractionError,
    TableExtractor, TableHeaders, TableMetadata, TableRow, TableStructure,
};

// Re-export table extraction and conversion (CLI support)
pub use tables::{
    parse_content_to_table_data, parse_csv_to_data, parse_markdown_to_data, ApiClient,
    TableConverter, TableData, TableExtractRequest, TableExtractResponse,
    TableExtractor as TablesExtractor, TableSource, TableSummary,
};

// Re-export schema functionality
pub mod schema;
pub use schema::{
    ExtractionSchema, FieldSchema, SchemaAnalysis, SchemaComparator, SchemaExtractor,
    SchemaGenerator, SchemaLearnRequest, SchemaLearnResponse, SchemaMetadata, SchemaRegistry,
    SchemaTestRequest, SchemaTestResponse, SchemaValidator, SelectorRule, TestResult, TestSummary,
    ValidationRules,
};

// Native HTML parser module (for headless-rendered content)
pub mod native_parser;
pub use native_parser::{NativeHtmlParser, ParserConfig};

// Common types for extraction
use serde::{Deserialize, Serialize};

// Re-export shared types from riptide-types
pub use riptide_types::{ContentChunk, ExtractedContent, ExtractionQuality, ExtractionResult};

/// Regex pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern {
    pub name: String,
    pub pattern: String,
    pub field: String,
    pub required: bool,
}
