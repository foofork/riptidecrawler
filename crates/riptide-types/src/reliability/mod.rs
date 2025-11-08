//! Reliability patterns for fault tolerance
//!
//! **ARCHITECTURE NOTE**: The circuit breaker lives here in `riptide-types` to avoid circular
//! dependencies. While this may seem like a violation of "types should not have behavior",
//! it's a necessary architectural compromise:
//!
//! - `riptide-fetch` needs circuit breakers for HTTP reliability
//! - `riptide-reliability` depends on `riptide-fetch` for HTTP clients
//! - Therefore, circuit breaker must be in a crate that both can depend on
//! - `riptide-types` is the lowest-level shared crate in the dependency graph
//!
//! The circuit breaker is lock-free, has no external dependencies beyond tokio/tracing,
//! and can be considered a "smart type" rather than business logic.

pub mod circuit;

// Re-export commonly used types
pub use circuit::{guarded_call, CircuitBreaker, Clock, Config, RealClock, State};
