//! Infrastructure adapters implementing port traits
//!
//! This module contains concrete implementations of port traits defined in riptide-types.
//! Adapters provide the anti-corruption layer between domain logic and infrastructure.
//!
//! # Available Adapters
//!
//! - `redis_idempotency`: Redis implementation of `IdempotencyStore`
//! - `redis_session_storage`: Redis implementation of `SessionStorage`
//! - `redis_rate_limiter`: Redis implementation of rate limiting
//! - `redis_coordination`: Redis implementation of `DistributedCoordination`
//! - `memory_coordination`: In-memory implementation of `DistributedCoordination` (single-process only)
//! - `standard_circuit_breaker`: Standard lock-free circuit breaker adapter
//! - `llm_circuit_breaker`: LLM-specific circuit breaker adapter

#[cfg(feature = "idempotency")]
pub mod redis_idempotency;

#[cfg(feature = "idempotency")]
pub mod redis_session_storage;

// Sprint 4.4: Rate limiting adapter
pub mod redis_rate_limiter;

// Distributed coordination adapters
pub mod memory_coordination;
pub mod redis_coordination;

// Phase 2: Circuit breaker adapters
pub mod llm_circuit_breaker;
pub mod standard_circuit_breaker;

// Re-export adapters when features are enabled
#[cfg(feature = "idempotency")]
pub use redis_idempotency::RedisIdempotencyStore;

#[cfg(feature = "idempotency")]
pub use redis_session_storage::RedisSessionStorage;

pub use redis_rate_limiter::{RedisPerHostRateLimiter, RedisRateLimiter};

// Distributed coordination exports
pub use memory_coordination::MemoryCoordination;
pub use redis_coordination::RedisCoordination;

// Phase 2: Circuit breaker exports
pub use llm_circuit_breaker::LlmCircuitBreakerAdapter;
pub use standard_circuit_breaker::StandardCircuitBreakerAdapter;
