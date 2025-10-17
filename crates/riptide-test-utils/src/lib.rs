//! Common test utilities for EventMesh
//!
//! This crate provides shared test utilities, fixtures, and helpers
//! used across the EventMesh test suite.

pub mod fixtures;
pub mod assertions;
pub mod factories;

#[cfg(feature = "http-mock")]
pub mod mock_server;

/// Re-export commonly used test dependencies
pub use anyhow::{anyhow, Result};
pub use tokio;
pub use tempfile;
