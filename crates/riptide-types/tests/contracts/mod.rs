//! Contract tests for Riptide port traits
//!
//! This module provides comprehensive contract tests for all port traits in riptide-types.
//! These tests ensure that any implementation of a port trait adheres to the expected
//! behavior and invariants defined by the trait.
//!
//! # Purpose
//!
//! Contract tests serve as executable specifications for trait implementations.
//! They validate:
//! - **Correctness**: Operations behave as documented
//! - **Consistency**: State remains consistent across operations
//! - **Error Handling**: Errors are handled gracefully
//! - **Thread Safety**: Operations are safe under concurrent access
//! - **Performance**: Operations meet reasonable performance expectations
//!
//! # Usage
//!
//! ## Testing a New CacheStorage Implementation
//!
//! ```rust,ignore
//! use riptide_types::ports::CacheStorage;
//! use riptide_types::tests::contracts::cache_storage_contract;
//!
//! struct MyCache { /* ... */ }
//!
//! #[async_trait::async_trait]
//! impl CacheStorage for MyCache {
//!     // ... implementation
//! }
//!
//! #[tokio::test]
//! async fn test_my_cache_contract() {
//!     let cache = MyCache::new();
//!
//!     // Run all contract tests
//!     cache_storage_contract::run_all_tests(&cache).await.unwrap();
//!
//!     // Or run specific tests
//!     cache_storage_contract::test_basic_operations(&cache).await.unwrap();
//!     cache_storage_contract::test_ttl_expiration(&cache).await.unwrap();
//! }
//! ```
//!
//! ## Testing a New SessionStorage Implementation
//!
//! ```rust,ignore
//! use riptide_types::ports::SessionStorage;
//! use riptide_types::tests::contracts::session_storage_contract;
//!
//! struct MySessionStorage { /* ... */ }
//!
//! #[async_trait::async_trait]
//! impl SessionStorage for MySessionStorage {
//!     // ... implementation
//! }
//!
//! #[tokio::test]
//! async fn test_my_session_storage_contract() {
//!     let storage = MySessionStorage::new();
//!
//!     // Run all contract tests
//!     session_storage_contract::run_all_tests(&storage).await.unwrap();
//!
//!     // Or run specific tests
//!     session_storage_contract::test_crud_operations(&storage).await.unwrap();
//!     session_storage_contract::test_multi_tenancy(&storage).await.unwrap();
//! }
//! ```
//!
//! ## Testing a New CacheSync Implementation
//!
//! ```rust,ignore
//! use riptide_types::tests::contracts::coordination_contract::{self, CacheSync};
//!
//! struct MyCoordinator { /* ... */ }
//!
//! #[async_trait::async_trait]
//! impl CacheSync for MyCoordinator {
//!     // ... implementation
//! }
//!
//! #[tokio::test]
//! async fn test_my_coordinator_contract() {
//!     let coordinator = MyCoordinator::new();
//!
//!     // Run all contract tests
//!     coordination_contract::run_all_tests(&coordinator).await.unwrap();
//! }
//! ```
//!
//! # Test Organization
//!
//! Each contract test module provides:
//! - Individual test functions for specific behaviors
//! - A `run_all_tests()` convenience function
//! - Built-in tests using in-memory implementations
//!
//! # Integration with CI/CD
//!
//! Contract tests should be run:
//! - During development (with `cargo test -p riptide-types`)
//! - In CI for all implementations
//! - Before accepting new trait implementations
//!
//! # Adding New Contract Tests
//!
//! When adding a new port trait:
//! 1. Create a new module in `crates/riptide-types/tests/contracts/`
//! 2. Define test functions covering all trait methods
//! 3. Include edge cases, error handling, and concurrency tests
//! 4. Add a `run_all_tests()` convenience function
//! 5. Include self-tests with a simple in-memory implementation
//! 6. Export the module from this `mod.rs`
//! 7. Document usage patterns
//!
//! # Best Practices
//!
//! ## Test Independence
//! - Each test should be independent
//! - Clean up resources after each test
//! - Use unique keys/IDs to avoid conflicts
//!
//! ## Error Testing
//! - Test both success and failure paths
//! - Validate error messages are meaningful
//! - Ensure resources are cleaned up on errors
//!
//! ## Performance
//! - Keep tests fast (< 1 second each when possible)
//! - Use realistic data sizes
//! - Test performance-critical paths
//!
//! ## Coverage
//! - Test all trait methods
//! - Test default implementations
//! - Test boundary conditions
//! - Test concurrent access patterns
//!
//! # Example: Running All Contract Tests
//!
//! ```bash
//! # Run all contract tests in riptide-types
//! cargo test -p riptide-types --test '*'
//!
//! # Run specific contract test suite
//! cargo test -p riptide-types cache_storage_contract
//!
//! # Run with output for debugging
//! cargo test -p riptide-types -- --nocapture
//! ```

pub mod cache_storage_contract;
pub mod coordination_contract;
pub mod session_storage_contract;

// Re-export for convenience
#[allow(unused_imports)]
pub use cache_storage_contract as cache_storage;
#[allow(unused_imports)]
pub use coordination_contract as coordination;
#[allow(unused_imports)]
pub use session_storage_contract as session_storage;

#[cfg(test)]
mod tests {
    //! Self-tests to validate the contract test framework itself

    #[test]
    fn test_contract_modules_exist() {
        // Ensure all modules are properly exported
        // This test simply verifies that the module compiles and links correctly
    }
}
