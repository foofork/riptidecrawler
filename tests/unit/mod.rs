//! Unit tests module
//!
//! Contains fast, isolated component-level tests following London School TDD principles.
//! Tests focus on single components with mocked dependencies.
//!
//! Total: 28 unit test files
//! Coverage Target: ≥85%

// Core system components
pub mod circuit_breaker_test;
pub mod rate_limiter_tests;
pub mod memory_manager_tests;
pub mod performance_monitor_tests;
pub mod health_system_tests;

// Event system tests
pub mod event_system_test;
pub mod event_system_comprehensive_tests;

// Resource management
pub mod resource_manager_unit_tests;
pub mod resource_manager_edge_cases;

// Chunking and strategies
pub mod chunking_strategies_tests;
pub mod strategies_pipeline_tests;

// Buffer and backpressure
pub mod buffer_backpressure_tests;

// Component model and validation
pub mod component_model_tests;
pub mod component_model_validation;
pub mod lifetime_validation;

// WASM components
pub mod wasm_manager_tests;
pub mod wasm_component_tests;
pub mod wasm_component_guard_test;

// Singleton pattern tests
pub mod singleton_integration_tests;
pub mod singleton_thread_safety_tests;

// Spider handler tests
pub mod spider_handler_tests;

// Telemetry and observability
pub mod telemetry_opentelemetry_test;
pub mod opentelemetry_test;

// Performance tests
pub mod ttfb_performance_tests;

// NDJSON format compliance
pub mod ndjson_format_compliance_tests;

// Circuit breaker quick test
pub mod quick_circuit_test;

// Utility and fix tests
pub mod fix_topic_chunker;
pub mod tdd_demo_test;
