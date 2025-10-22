//! CLI Integration Tests Module
//!
//! Comprehensive test suite for CLI→API architecture

pub mod cli_api_integration;
pub mod config_validation;
pub mod e2e_workflow;
pub mod performance_tests;

// New CLI-API integration tests
pub mod api_client_tests;
pub mod fallback_tests;
pub mod integration_api_tests;
pub mod test_utils;

// Additional CLI tests
pub mod integration_tests;
pub mod e2e_tests;
pub mod real_api_tests;
pub mod real_world_integration;
pub mod real_world_tests;
pub mod cli_tables_test;
