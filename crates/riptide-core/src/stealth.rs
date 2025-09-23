//! Stealth module for anti-detection measures in browser automation
//!
//! This module has been refactored into smaller, maintainable components.
//! All original functionality is preserved through re-exports for backward compatibility.
//!
//! ## Module Structure
//!
//! - `config` - Configuration structures and presets
//! - `fingerprint` - Browser fingerprinting countermeasures
//! - `user_agent` - User agent management and rotation
//! - `javascript` - JavaScript injection for evasion
//! - `evasion` - Main stealth controller and coordination
//!
//! ## Usage
//!
//! ```rust
//! use riptide_core::stealth::{StealthController, StealthPreset};
//!
//! // Create controller with preset
//! let mut controller = StealthController::from_preset(StealthPreset::High);
//!
//! // Get user agent
//! let user_agent = controller.next_user_agent();
//!
//! // Generate headers
//! let headers = controller.generate_headers();
//!
//! // Get JavaScript injection
//! let js_code = controller.get_stealth_js();
//! ```

// Re-export all types for backward compatibility
pub use self::config::*;
pub use self::evasion::StealthController;
pub use self::fingerprint::*;
pub use self::javascript::JavaScriptInjector;
pub use self::user_agent::*;

// Internal module declarations
mod config;
mod evasion;
mod fingerprint;
mod javascript;
mod user_agent;

#[cfg(test)]
mod tests;