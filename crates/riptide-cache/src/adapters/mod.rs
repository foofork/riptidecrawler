//! Infrastructure adapters implementing port traits
//!
//! This module contains concrete implementations of port traits defined in riptide-types.
//! Adapters provide the anti-corruption layer between domain logic and infrastructure.
//!
//! # Available Adapters
//!
//! - `redis_idempotency`: Redis implementation of `IdempotencyStore`

#[cfg(feature = "idempotency")]
pub mod redis_idempotency;

// Re-export adapters when features are enabled
#[cfg(feature = "idempotency")]
pub use redis_idempotency::RedisIdempotencyStore;
