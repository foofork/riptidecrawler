//! Common test utilities for RipTide integration tests
//!
//! This module provides shared test infrastructure including:
//! - Mock servers for network-independent testing
//! - Timeout helpers for CI/CD environments
//! - Common fixtures and test data
//! - CLI test harness for real-world testing
//! - Content validation framework
//! - Baseline management for regression detection

pub mod mock_server;
pub mod timeouts;
pub mod test_harness;
pub mod content_validator;
pub mod baseline_manager;

// Re-export commonly used types
pub use test_harness::{TestHarness, TestUrl, TestUrls, ExtractionResult, TestSession};
pub use content_validator::{ContentValidator, ValidationRule, ValidationResult};
pub use baseline_manager::{BaselineManager, Baseline, ComparisonResult};
