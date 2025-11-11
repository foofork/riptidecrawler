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
//! - `standard_circuit_breaker`: Standard lock-free circuit breaker adapter
//! - `llm_circuit_breaker`: LLM-specific circuit breaker adapter

#[cfg(feature = "idempotency")]
pub mod redis_idempotency;

#[cfg(feature = "idempotency")]
pub mod redis_session_storage;

// Sprint 4.4: Rate limiting adapter
pub mod redis_rate_limiter;

// Phase 2: Circuit breaker adapters
pub mod llm_circuit_breaker;
pub mod standard_circuit_breaker;

// Re-export adapters when features are enabled
#[cfg(feature = "idempotency")]
pub use redis_idempotency::RedisIdempotencyStore;

#[cfg(feature = "idempotency")]
pub use redis_session_storage::RedisSessionStorage;

pub use redis_rate_limiter::{RedisPerHostRateLimiter, RedisRateLimiter};

// Phase 2: Circuit breaker exports
pub use llm_circuit_breaker::LlmCircuitBreakerAdapter;
pub use standard_circuit_breaker::StandardCircuitBreakerAdapter;
