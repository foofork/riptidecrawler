//! Port interfaces for dependency inversion
//!
//! This module provides backend-agnostic trait definitions that enable
//! dependency inversion and facilitate testing. Concrete implementations
//! are provided in their respective infrastructure crates.

pub mod cache;
pub mod memory_cache;

// Re-export for convenience
pub use cache::CacheStorage;
pub use memory_cache::InMemoryCache;
