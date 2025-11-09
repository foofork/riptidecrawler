//! HTTP API module for headless browser operations
//!
//! **Sprint 4.6 Note**: This module is a placeholder for future HTTP API consolidation.
//! Currently, HTTP API remains in riptide-headless to avoid circular dependencies.
//!
//! **Future Plan**: After riptide-headless is deprecated, HTTP endpoints will be
//! moved here completely.

// For now, re-export models from parent
pub use crate::models::{Artifacts, ArtifactsOut, PageAction, RenderReq, RenderResp, Timeouts};
