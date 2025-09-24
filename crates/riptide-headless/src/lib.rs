//! RipTide Headless Browser Management Library
//!
//! This library provides browser pool management for headless browser operations.
//! It includes connection pooling, health checking, and automatic recovery.

pub mod cdp;
pub mod launcher;
pub mod models;
pub mod pool;

pub use pool::{BrowserCheckout, BrowserPool, BrowserPoolConfig, PoolStats};