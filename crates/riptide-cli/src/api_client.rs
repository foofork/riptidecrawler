//! DEPRECATED: This file is no longer used.
//!
//! The functionality from this file has been consolidated into the `client` module.
//!
//! ## Migration Guide
//!
//! Old code using `RiptideApiClient`:
//! ```rust,ignore
//! use crate::api_client::RiptideApiClient;
//! let client = RiptideApiClient::new(base_url, api_key)?;
//! ```
//!
//! New code should use `client::ApiClient`:
//! ```rust,ignore
//! use crate::client::ApiClient;
//! let client = ApiClient::new(base_url, api_key)?;
//! ```
//!
//! ## Render Types
//!
//! The render-specific types (`RenderRequest`, `RenderResponse`, `ViewportConfig`,
//! `RenderMetadata`) that were defined here are NOT used by the CLI.
//!
//! The `commands::render` module defines its own types that match the actual API contract.
//!
//! This file can be safely deleted once all references are confirmed to be removed.

#![allow(dead_code)]

// This module is intentionally empty - all functionality has moved to client::ApiClient
