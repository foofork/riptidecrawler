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

#[cfg(feature = "idempotency")]
pub mod redis_session_storage;

// Re-export adapters when features are enabled
#[cfg(feature = "idempotency")]
pub use redis_idempotency::RedisIdempotencyStore;

#[cfg(feature = "idempotency")]
pub use redis_session_storage::RedisSessionStorage;
