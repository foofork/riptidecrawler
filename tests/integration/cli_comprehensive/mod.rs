//! Comprehensive CLI Integration Tests Module
//!
//! This module provides extensive integration testing for all CLI commands,
//! including real-world URL testing, output storage, and regression detection.

pub mod real_world_tests;

pub use real_world_tests::{
    CliTestHarness, ExpectedResult, TestResult, TestSession, TestUrl, load_test_urls,
};
