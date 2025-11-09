//! Data Transfer Objects (DTOs) for API handlers
//!
//! This module contains all request/response types for HTTP transport.
//! DTOs are separated from handlers to keep handlers <50 LOC.
//!
//! Phase 3 Sprint 3.1: Extracted from inline handler definitions

pub mod engine_selection;
pub mod pdf;
pub mod profiles;
pub mod sessions;
pub mod tables;
pub mod workers;

// Re-export commonly used types (commented out unused re-exports)
// pub use engine_selection::*;
// pub use pdf::*;
// pub use profiles::*;
// pub use sessions::*;
// pub use tables::*;
// pub use workers::*;
