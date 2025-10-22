//! Integration tests module
//!
//! Contains comprehensive integration tests for the RipTide system,
//! including wireup tests, session persistence, contract tests, and gap fixes.

pub mod contract_tests;
pub mod gap_fixes_integration;
pub mod session_persistence_tests;
pub mod wireup_tests;
pub mod health_tests;
pub mod full_pipeline_tests;
pub mod worker_integration_tests;
pub mod spider_integration_tests;

// Browser and engine tests
pub mod browser_pool_tests;
pub mod engine_selection_tests;

// WASM integration tests
pub mod wasm_caching_tests;

// Streaming tests
pub mod streaming_integration_tests;

// Root-level integration tests moved here
pub mod integration_fetch_reliability;
pub mod integration_pipeline_orchestration;
pub mod integration_headless_cdp;
