//! Business metrics module for facade layer
//!
//! This module provides domain-level metrics (not transport metrics) for business operations.
//! It focuses on measuring business outcomes and domain events, distinct from infrastructure
//! metrics like HTTP requests or database queries.

pub mod business;

pub use business::BusinessMetrics;
