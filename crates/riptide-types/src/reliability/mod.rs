//! Reliability patterns for fault tolerance

pub mod circuit;

// Re-export commonly used types
pub use circuit::{guarded_call, CircuitBreaker, Clock, Config, RealClock, State};
