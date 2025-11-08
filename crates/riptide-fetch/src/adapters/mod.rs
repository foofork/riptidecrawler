//! Infrastructure adapters for HTTP client implementations
//!
//! This module contains concrete implementations of HTTP client ports.
//! Adapters provide the anti-corruption layer between domain logic and HTTP libraries.
//!
//! # Available Adapters
//!
//! - `reqwest_http_client`: Production HTTP client using reqwest with connection pooling

pub mod reqwest_http_client;

// Re-export adapters
pub use reqwest_http_client::ReqwestHttpClient;
