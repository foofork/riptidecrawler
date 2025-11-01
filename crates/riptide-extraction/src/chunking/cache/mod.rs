//! Async token counting cache using tiktoken
//!
//! This module provides an efficient caching layer for exact token counts using tiktoken-rs.
//! The cache uses LRU eviction and is thread-safe for concurrent access.

pub mod tiktoken_cache;

pub use tiktoken_cache::{count_tokens_batch, count_tokens_exact, TiktokenCache};
