//! Error types for the Riptide framework
//!
//! This module provides comprehensive error handling with:
//! - `RiptideError`: General framework errors
//! - `StrategyError`: Strategy-specific extraction errors with rich context

pub mod riptide_error;
pub mod strategy_error;

// Re-export for convenience
pub use riptide_error::{Result, RiptideError};
pub use strategy_error::StrategyError;
