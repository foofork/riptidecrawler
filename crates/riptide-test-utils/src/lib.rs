//! Common test utilities for EventMesh
//!
//! This crate provides shared test utilities, fixtures, and helpers
//! used across the EventMesh test suite.

pub mod assertions;
pub mod factories;
pub mod fixtures;

// TODO: Add mock_server module when needed
// #[cfg(feature = "http-mock")]
// pub mod mock_server;

/// Re-export commonly used test dependencies
pub use anyhow::{anyhow, Result};
pub use tempfile;
pub use tokio;
