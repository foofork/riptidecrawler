//! Domain Profiling Module
//!
//! This module provides domain-specific web scraping intelligence, including:
//! - Domain profile initialization and management
//! - Site structure analysis and baseline tracking
//! - Configuration management for domain-specific extraction
//! - Drift detection for website structure changes
//! - Profile versioning and history tracking

pub mod analyzer;
pub mod profiler;

// Re-export core types
pub use analyzer::{
    ContentPattern, DomainAnalyzer, DriftAnalyzer, DriftChange, DriftReport, DriftSummary,
    SiteAnalysisResult, SiteBaseline, SiteStructure, UrlPattern,
};
pub use profiler::{
    DomainConfig, DomainMetadata, DomainPatterns, DomainProfile, ProfileManager, ProfileRegistry,
};

// Re-export commonly used types
// Note: chrono, serde types are re-exported through child modules

/// Domain registry directory path
pub const DOMAIN_REGISTRY_DIR: &str = ".riptide/domains";

/// Result type for domain profiling operations
pub type Result<T> = anyhow::Result<T>;
