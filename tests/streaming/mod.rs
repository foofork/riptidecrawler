//! Streaming Tests Module
//!
//! Comprehensive test suite for NDJSON streaming endpoints:
//! - /crawl/stream
//! - /deepsearch/stream
//!
//! Test Categories:
//! - Performance (TTFB < 500ms)
//! - Error handling (zero unwrap/expect)
//! - Resource controls (backpressure, buffer limits)
//! - Streaming behavior (real-time results)
//! - Integration (end-to-end workflows)

pub mod ndjson_stream_tests;
pub mod deepsearch_stream_tests;

// Re-export test utilities for use in other test modules
pub use ndjson_stream_tests::*;
pub use deepsearch_stream_tests::*;