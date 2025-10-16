//! Comprehensive test suite for metrics collection systems
//!
//! This module contains extensive tests for both PDF and Intelligence metrics,
//! covering:
//! - Unit tests for all metric types
//! - Integration tests
//! - Concurrency and thread-safety tests
//! - Performance benchmarks
//! - Edge cases and error conditions
//! - Memory leak detection
//! - Export format validation

mod pdf_metrics_comprehensive_test;
mod intelligence_metrics_comprehensive_test;
// Performance benchmarks are run separately via `cargo bench`
// mod performance_benchmarks;
