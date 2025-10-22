/// WASM Integration Test Suite
///
/// Comprehensive testing of WebAssembly Component Model integration with
/// WIT bindings, resource management, instance pooling, and performance.

pub mod wit_bindings_integration;
pub mod resource_limits;
pub mod instance_pool;
pub mod e2e_integration;
pub mod error_handling;

// WASM extractor integration
pub mod wasm_extractor_integration;

// Component tests
pub mod wasm_component_tests;
pub mod wasm_component_guard_test;

// Re-export common test utilities
pub use wit_bindings_integration::*;
pub use resource_limits::*;
pub use instance_pool::*;
pub use e2e_integration::*;
pub use error_handling::*;
