//! Integration tests module
//!
//! Contains comprehensive integration tests for the RipTide system,
//! including cross-component interactions, session persistence, and contract tests.
//!
//! Total: 38 integration test files
//! Coverage Target: ≥75%

// Core integration tests
pub mod contract_tests;
pub mod gap_fixes_integration;
pub mod session_persistence_tests;
pub mod wireup_tests;
pub mod health_tests;
pub mod full_pipeline_tests;
pub mod worker_integration_tests;
pub mod resource_management_tests;
pub mod resource_manager_integration_tests;

// Browser pool and CDP tests
pub mod browser_pool_tests;
pub mod browser_pool_manager_tests;
pub mod browser_pool_scaling_tests;
pub mod cdp_pool_tests;
pub mod memory_pressure_tests;
pub mod engine_selection_tests;

// Spider integration tests
pub mod spider_integration_tests;
pub mod spider_chrome_tests;
pub mod spider_chrome_benchmarks;
pub mod spider_multi_level_tests;
pub mod spider_query_aware_integration_test;

// Strategies integration tests
pub mod strategies_integration_test;
pub mod strategies_integration_tests;

// WASM integration tests
pub mod wasm_caching_tests;

// Streaming tests
pub mod streaming_integration_tests;

// Singleton tests
pub mod singleton_integration_tests;

// Phase-specific integration tests
pub mod phase3_integration_tests;
pub mod phase4_integration_tests;

// Root-level integration tests moved here (reorganization 2025-10-23)
pub mod integration_fetch_reliability;
pub mod integration_pipeline_orchestration;
pub mod integration_headless_cdp;
pub mod integration_dynamic_rendering;
pub mod integration_test;
pub mod integration_tests;

// CLI comprehensive tests
pub mod cli_comprehensive_test;

// Phase 10: Engine Optimization Quick Wins (3 optimizations, ~290 LOC, 60-80% cost savings)
pub mod phase10_engine_optimization;
pub mod probe_first_escalation_tests;

// Phase 10.4: Domain Warm-Start Caching (23 integration tests)
pub mod domain_warm_start_tests;
