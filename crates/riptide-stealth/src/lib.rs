//! # Riptide Stealth
//!
//! A comprehensive stealth module for anti-detection measures in browser automation.
//! This crate provides tools to evade detection by anti-bot systems through various
//! techniques including user agent rotation, fingerprinting countermeasures, JavaScript
//! injection, request randomization, and proxy configuration.
//!
//! ## Features
//!
//! - **User Agent Rotation**: Intelligent rotation strategies (Random, Sequential, Sticky, Domain-based)
//! - **Fingerprinting Countermeasures**: WebGL, Canvas, Audio, Hardware, and Plugin spoofing
//! - **JavaScript Evasion**: Comprehensive browser API overrides and automation cleanup
//! - **Request Randomization**: Header, timing, viewport, and locale randomization
//! - **Proxy Configuration**: Support for various proxy types with rotation strategies
//! - **Stealth Presets**: None, Low, Medium, High - easy configuration for different scenarios
//!
//! ## Usage
//!
//! ```rust
//! use riptide_stealth::{StealthController, StealthPreset};
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
//!
//! // Calculate request delay
//! let delay = controller.calculate_delay();
//! ```
//!
//! ## Stealth Presets
//!
//! - **None**: No stealth measures applied
//! - **Low**: Basic stealth with minimal fingerprint changes
//! - **Medium**: Balanced detection vs performance (default)
//! - **High**: Maximum stealth with all countermeasures enabled

// Core modules
pub mod config;
pub mod evasion;
pub mod fingerprint;
pub mod javascript;
pub mod user_agent;

// Enhanced stealth features for crawl4ai parity
pub mod enhancements;

// Advanced anti-detection features
pub mod behavior;
pub mod rate_limiter;

// Detection evasion and CAPTCHA detection
pub mod detection;

// P1-B6: Enhanced stealth integration (Phase 1 Week 6)
pub mod cdp_integration;
pub mod fingerprint_enhanced;
pub mod stealth_level;

// Re-export main types for easy access
pub use config::{
    load_user_agents_from_file, DomainTiming, HeaderRandomization, LocaleRandomization,
    LocaleStrategy, ProxyAuth, ProxyConfig, ProxyEndpoint, ProxyRotation, ProxyType, RateLimit,
    RequestRandomization, StealthConfig, StealthPreset, TimingConfig, TimingJitter,
    ViewportRandomization,
};

pub use evasion::StealthController;

pub use fingerprint::{
    AudioConfig, BrowserFingerprint, CanvasConfig, CdpStealthConfig, FingerprintGenerator,
    FingerprintingConfig, FontConfig, HardwareConfig, PluginConfig, WebGlConfig, WebRtcConfig,
};

pub use javascript::JavaScriptInjector;

pub use user_agent::{BrowserType, RotationStrategy, UserAgentConfig, UserAgentManager};

// Re-export enhanced features
pub use enhancements::{
    HeaderConsistencyManager, ScreenResolution, ScreenResolutionManager, TimezoneInfo,
    TimezoneManager, WebRtcEnhanced,
};

// Re-export advanced anti-detection features
pub use behavior::{BehaviorSimulator, MousePath, Point, ScrollAction};
pub use rate_limiter::{DomainStats, RateLimiter};

// Re-export detection evasion features
pub use detection::{
    CaptchaDetection, CaptchaDetector, CaptchaType, DetectionEvasion, DetectionScore, RiskLevel,
};

// Re-export P1-B6 enhanced features
pub use cdp_integration::{BatchHeadersResult, CdpCommand, CdpStealthIntegrator};
pub use fingerprint_enhanced::{
    CdpStealthParams, EnhancedFingerprintGenerator, FingerprintConfig as EnhancedFingerprintConfig,
};
pub use stealth_level::{
    AudioLevelConfig, CanvasLevelConfig, HardwareLevelConfig, StealthLevel, StealthLevelConfig,
    WebGlLevelConfig, WebRtcLevelConfig,
};

// Tests module
#[cfg(test)]
mod tests;

// Version and crate information
/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// A convenience function to create a stealth controller with high stealth preset
pub fn create_high_stealth_controller() -> StealthController {
    StealthController::from_preset(StealthPreset::High)
}

/// A convenience function to create a stealth controller with medium stealth preset
pub fn create_medium_stealth_controller() -> StealthController {
    StealthController::from_preset(StealthPreset::Medium)
}

/// A convenience function to create a stealth controller with low stealth preset
pub fn create_low_stealth_controller() -> StealthController {
    StealthController::from_preset(StealthPreset::Low)
}

/// A convenience function to create a stealth controller with no stealth measures
pub fn create_no_stealth_controller() -> StealthController {
    StealthController::from_preset(StealthPreset::None)
}
